use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    http_client,
    models::{CurrentRequest, OcCollection, OcHttp, OcItem, Secrets},
};

// ── Workflow definition ───────────────────────────────────────────────────────

/// A workflow: ordered sequence of named HTTP steps.
///
/// Stored in `.endptr/workflows/<name>.yaml`.
///
/// Template references between steps:
///   `{{step_name.response.status}}`
///   `{{step_name.response.body.field}}`
///   `{{step_name.response.body.nested.field}}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    #[serde(default)]
    pub steps: Vec<WorkflowStep>,
}

impl Workflow {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: vec![],
        }
    }
}

/// A single workflow step: either an inline HTTP definition or a reference to a
/// saved collection request.
///
/// Inline HTTP:
/// ```yaml
/// - name: create_user
///   http:
///     method: POST
///     url: https://api.example.com/users
///     body:
///       type: json
///       data: '{"name": "John"}'
/// ```
///
/// Collection request reference (`"<Collection Name>.<Request Name>"`):
/// ```yaml
/// - name: get_post
///   ref: "My API.Get 1 post"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    /// Inline HTTP definition. Mutually exclusive with `step_ref`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<OcHttp>,
    /// Reference to a saved request: `"<collection_name>.<request_name>"`.
    /// Mutually exclusive with `http`.
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub step_ref: Option<String>,
}

// ── Execution ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_name: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub body: String,
    pub time_ms: u64,
    pub error: Option<String>,
}

impl StepResult {
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }
}

/// Run all steps in order. Each step's response is injected into the template
/// context for subsequent steps using dot-notation keys:
///
///   `{{step_name.response.status}}`
///   `{{step_name.response.body.field}}`
///
/// `collections` is used to resolve `ref: "Collection.Request"` steps.
/// Stops on the first step that fails (network error, etc.).
pub fn run_workflow(
    workflow: &Workflow,
    secrets: &Secrets,
    collections: &[(std::path::PathBuf, OcCollection)],
) -> Vec<StepResult> {
    let mut results: Vec<StepResult> = vec![];
    let mut ctx: HashMap<String, String> = secrets.clone();

    for step in &workflow.steps {
        // Resolve the OcHttp for this step
        let base_http: OcHttp = match (&step.http, &step.step_ref) {
            (Some(http), _) => http.clone(),
            (None, Some(ref_str)) => {
                match resolve_ref(ref_str, collections) {
                    Some(http) => http,
                    None => {
                        results.push(StepResult {
                            step_name: step.name.clone(),
                            method: String::new(),
                            url: String::new(),
                            status: 0,
                            body: String::new(),
                            time_ms: 0,
                            error: Some(format!("ref not found: {ref_str}")),
                        });
                        break;
                    }
                }
            }
            (None, None) => {
                results.push(StepResult {
                    step_name: step.name.clone(),
                    method: String::new(),
                    url: String::new(),
                    status: 0,
                    body: String::new(),
                    time_ms: 0,
                    error: Some("step has neither `http` nor `ref`".to_string()),
                });
                break;
            }
        };

        let resolved_http = resolve_http(&base_http, &ctx);
        let method_str = resolved_http.method.clone();
        let url_str = resolved_http.url.clone();
        let request = CurrentRequest::from(&resolved_http);

        match http_client::send_request(request, ctx.clone()) {
            Ok(resp) => {
                ctx.insert(
                    format!("{}.response.status", step.name),
                    resp.status.to_string(),
                );
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp.body) {
                    flatten_json(&mut ctx, &format!("{}.response.body", step.name), &json);
                }
                results.push(StepResult {
                    step_name: step.name.clone(),
                    method: method_str,
                    url: url_str,
                    status: resp.status,
                    body: resp.body,
                    time_ms: resp.time_ms,
                    error: None,
                });
            }
            Err(e) => {
                results.push(StepResult {
                    step_name: step.name.clone(),
                    method: method_str,
                    url: url_str,
                    status: 0,
                    body: String::new(),
                    time_ms: 0,
                    error: Some(e),
                });
                break;
            }
        }
    }

    results
}

/// Resolve a `"Collection Name.Request Name"` reference to its OcHttp.
fn resolve_ref(
    ref_str: &str,
    collections: &[(std::path::PathBuf, OcCollection)],
) -> Option<OcHttp> {
    let (col_name, req_name) = ref_str.split_once('.')?;
    let col_name = col_name.trim();
    let req_name = req_name.trim();

    for (_, col) in collections {
        if col.name == col_name {
            return find_request_in_items(&col.items, req_name);
        }
    }
    None
}

fn find_request_in_items(items: &[OcItem], name: &str) -> Option<OcHttp> {
    for item in items {
        match item {
            OcItem::Request(req) if req.info.name == name => return Some(req.http.clone()),
            OcItem::Folder(folder) => {
                if let Some(http) = find_request_in_items(&folder.items, name) {
                    return Some(http);
                }
            }
            _ => {}
        }
    }
    None
}

/// Resolve all `{{key}}` placeholders in an OcHttp using the given context.
fn resolve_http(http: &OcHttp, ctx: &HashMap<String, String>) -> OcHttp {
    let mut out = http.clone();
    out.url = resolve_str(&http.url, ctx);
    if let Some(ref headers) = http.headers {
        out.headers = Some(
            headers
                .iter()
                .map(|h| crate::models::OcHeader {
                    key: h.key.clone(),
                    value: resolve_str(&h.value, ctx),
                })
                .collect(),
        );
    }
    if let Some(ref params) = http.params {
        out.params = Some(
            params
                .iter()
                .map(|p| crate::models::OcParam {
                    key: resolve_str(&p.key, ctx),
                    value: resolve_str(&p.value, ctx),
                    enabled: p.enabled,
                })
                .collect(),
        );
    }
    if let Some(ref body) = http.body {
        out.body = Some(crate::models::OcBody {
            body_type: body.body_type.clone(),
            data: resolve_str(&body.data, ctx),
        });
    }
    out
}

/// Replace every `{{key}}` in `text` with its value from `ctx`.
/// Handles dot-notation keys like `step.response.body.token`.
fn resolve_str(text: &str, ctx: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (k, v) in ctx {
        result = result.replace(&format!("{{{{{}}}}}", k), v);
    }
    result
}

/// Flatten a JSON value into dot-notation template keys.
///
/// `{"token": "abc", "user": {"id": 1}}` with prefix `step.response.body` produces:
///   `step.response.body.token = "abc"`
///   `step.response.body.user.id = "1"`
fn flatten_json(ctx: &mut HashMap<String, String>, prefix: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                flatten_json(ctx, &format!("{prefix}.{k}"), v);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                flatten_json(ctx, &format!("{prefix}.{i}"), v);
            }
        }
        serde_json::Value::String(s) => {
            ctx.insert(prefix.to_string(), s.clone());
        }
        serde_json::Value::Number(n) => {
            ctx.insert(prefix.to_string(), n.to_string());
        }
        serde_json::Value::Bool(b) => {
            ctx.insert(prefix.to_string(), b.to_string());
        }
        serde_json::Value::Null => {
            ctx.insert(prefix.to_string(), "null".to_string());
        }
    }
}
