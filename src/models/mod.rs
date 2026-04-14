use std::collections::HashMap;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

// ── HTTP Method ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }

    pub fn all() -> [HttpMethod; 7] {
        [
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
            HttpMethod::HEAD,
            HttpMethod::OPTIONS,
        ]
    }

    pub fn next(&self) -> HttpMethod {
        let all = Self::all();
        let idx = all.iter().position(|m| m == self).unwrap_or(0);
        all[(idx + 1) % all.len()].clone()
    }

    pub fn prev(&self) -> HttpMethod {
        let all = Self::all();
        let idx = all.iter().position(|m| m == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()].clone()
    }

    pub fn color(&self) -> Color {
        match self {
            HttpMethod::GET => Color::Green,
            HttpMethod::POST => Color::Blue,
            HttpMethod::PUT => Color::Yellow,
            HttpMethod::PATCH => Color::Cyan,
            HttpMethod::DELETE => Color::Red,
            HttpMethod::HEAD => Color::Magenta,
            HttpMethod::OPTIONS => Color::Gray,
        }
    }

    pub fn from_str(s: &str) -> HttpMethod {
        match s.to_uppercase().as_str() {
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ── Auth ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AuthType {
    #[default]
    None,
    BasicAuth,
    Bearer,
    OAuth2,
    JWT,
}

impl AuthType {
    pub fn label(&self) -> &str {
        match self {
            AuthType::None => "None",
            AuthType::BasicAuth => "Basic Auth",
            AuthType::Bearer => "Bearer",
            AuthType::OAuth2 => "OAuth 2.0",
            AuthType::JWT => "JWT",
        }
    }

    pub fn all() -> [AuthType; 5] {
        [
            AuthType::None,
            AuthType::BasicAuth,
            AuthType::Bearer,
            AuthType::OAuth2,
            AuthType::JWT,
        ]
    }

    pub fn next(&self) -> AuthType {
        let all = Self::all();
        let idx = all.iter().position(|a| a == self).unwrap_or(0);
        all[(idx + 1) % all.len()].clone()
    }

    pub fn prev(&self) -> AuthType {
        let all = Self::all();
        let idx = all.iter().position(|a| a == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()].clone()
    }

    /// Does this type need a username + password pair?
    pub fn needs_user_pass(&self) -> bool {
        matches!(self, AuthType::BasicAuth)
    }

    /// Does this type need a single token field?
    pub fn needs_token(&self) -> bool {
        matches!(self, AuthType::Bearer | AuthType::OAuth2 | AuthType::JWT)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub username: String,
    pub password: String,
    pub token: String,
}

impl AuthConfig {
    /// Build the `Authorization` header value if configured.
    pub fn to_auth_header(&self) -> Option<(String, String)> {
        match &self.auth_type {
            AuthType::None => None,
            AuthType::BasicAuth => {
                // Manual base64 — avoids extra dependency.
                let raw = format!("{}:{}", self.username, self.password);
                let encoded = base64_encode(raw.as_bytes());
                Some(("Authorization".into(), format!("Basic {encoded}")))
            }
            AuthType::Bearer | AuthType::OAuth2 | AuthType::JWT => {
                if self.token.is_empty() {
                    None
                } else {
                    Some(("Authorization".into(), format!("Bearer {}", self.token)))
                }
            }
        }
    }
}

/// Minimal base64 encoding (RFC 4648) — no dependency needed.
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 {
            chunk[1] as usize
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            chunk[2] as usize
        } else {
            0
        };
        out.push(CHARS[(b0 >> 2)] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[b2 & 0x3f] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ── Query Params ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl QueryParam {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            enabled: true,
        }
    }
}

// ── Postman Collection v2.1 ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    pub item: Vec<PostmanItem>,
}

impl PostmanCollection {
    pub fn new(name: &str) -> Self {
        Self {
            info: PostmanInfo {
                name: name.to_string(),
                schema: "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
                    .to_string(),
            },
            item: vec![],
        }
    }
}

impl Default for PostmanCollection {
    fn default() -> Self {
        Self::new("endptr")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanInfo {
    pub name: String,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanItem {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<PostmanRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<Vec<PostmanItem>>,
}

impl PostmanItem {
    pub fn is_folder(&self) -> bool {
        self.item.is_some()
    }

    pub fn is_request(&self) -> bool {
        self.request.is_some()
    }

    pub fn new_request(name: &str, request: PostmanRequest) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            request: Some(request),
            item: None,
        }
    }

    pub fn new_folder(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            request: None,
            item: Some(vec![]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanRequest {
    pub method: String,
    pub url: PostmanUrl,
    pub header: Vec<PostmanHeader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<PostmanBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<PostmanAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanUrl {
    pub raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<Vec<PostmanQueryParam>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanQueryParam {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanHeader {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

/// Postman auth object (Postman collection v2.1).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic: Option<Vec<PostmanAuthField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<Vec<PostmanAuthField>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanAuthField {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub field_type: String,
}

impl PostmanAuth {
    pub fn from_config(config: &AuthConfig) -> Option<PostmanAuth> {
        match &config.auth_type {
            AuthType::None => None,
            AuthType::BasicAuth => Some(PostmanAuth {
                auth_type: "basic".into(),
                basic: Some(vec![
                    PostmanAuthField {
                        key: "username".into(),
                        value: config.username.clone(),
                        field_type: "string".into(),
                    },
                    PostmanAuthField {
                        key: "password".into(),
                        value: config.password.clone(),
                        field_type: "string".into(),
                    },
                ]),
                bearer: None,
            }),
            AuthType::Bearer | AuthType::OAuth2 | AuthType::JWT => Some(PostmanAuth {
                auth_type: "bearer".into(),
                bearer: Some(vec![PostmanAuthField {
                    key: "token".into(),
                    value: config.token.clone(),
                    field_type: "string".into(),
                }]),
                basic: None,
            }),
        }
    }

    pub fn to_config(&self) -> AuthConfig {
        match self.auth_type.as_str() {
            "basic" => {
                let fields = self.basic.as_deref().unwrap_or(&[]);
                let username = fields
                    .iter()
                    .find(|f| f.key == "username")
                    .map(|f| f.value.clone())
                    .unwrap_or_default();
                let password = fields
                    .iter()
                    .find(|f| f.key == "password")
                    .map(|f| f.value.clone())
                    .unwrap_or_default();
                AuthConfig {
                    auth_type: AuthType::BasicAuth,
                    username,
                    password,
                    token: String::new(),
                }
            }
            "bearer" => {
                let token = self
                    .bearer
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .find(|f| f.key == "token")
                    .map(|f| f.value.clone())
                    .unwrap_or_default();
                AuthConfig {
                    auth_type: AuthType::Bearer,
                    token,
                    username: String::new(),
                    password: String::new(),
                }
            }
            _ => AuthConfig::default(),
        }
    }
}

// ── Working state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CurrentRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub auth: AuthConfig,
    pub params: Vec<QueryParam>,
}

impl CurrentRequest {
    pub fn to_postman_item(&self, name: &str) -> PostmanItem {
        let query: Option<Vec<PostmanQueryParam>> = if self.params.is_empty() {
            None
        } else {
            Some(
                self.params
                    .iter()
                    .map(|p| PostmanQueryParam {
                        key: p.key.clone(),
                        value: p.value.clone(),
                        disabled: if p.enabled { None } else { Some(true) },
                    })
                    .collect(),
            )
        };

        PostmanItem::new_request(
            name,
            PostmanRequest {
                method: self.method.as_str().to_string(),
                url: PostmanUrl {
                    raw: self.url.clone(),
                    query,
                },
                header: self
                    .headers
                    .iter()
                    .map(|(k, v)| PostmanHeader {
                        key: k.clone(),
                        value: v.clone(),
                        disabled: None,
                    })
                    .collect(),
                body: if self.body.is_empty() {
                    None
                } else {
                    Some(PostmanBody {
                        mode: Some("raw".to_string()),
                        raw: Some(self.body.clone()),
                    })
                },
                auth: PostmanAuth::from_config(&self.auth),
            },
        )
    }
}

impl From<&PostmanRequest> for CurrentRequest {
    fn from(req: &PostmanRequest) -> Self {
        let params: Vec<QueryParam> = req
            .url
            .query
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|q| QueryParam {
                key: q.key.clone(),
                value: q.value.clone(),
                enabled: q.disabled.map(|d| !d).unwrap_or(true),
            })
            .collect();

        let auth = req.auth.as_ref().map(|a| a.to_config()).unwrap_or_default();

        Self {
            method: HttpMethod::from_str(&req.method),
            url: req.url.raw.clone(),
            headers: req
                .header
                .iter()
                .map(|h| (h.key.clone(), h.value.clone()))
                .collect(),
            body: req
                .body
                .as_ref()
                .and_then(|b| b.raw.clone())
                .unwrap_or_default(),
            auth,
            params,
        }
    }
}

// ── Secrets ───────────────────────────────────────────────────────────────────

pub type Secrets = HashMap<String, String>;

// ── HTTP Response ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub response_headers: Vec<(String, String)>,
    pub body: String,
    pub time_ms: u64,
}

impl HttpResponse {
    pub fn status_color(&self) -> Color {
        match self.status {
            200..=299 => Color::Green,
            300..=399 => Color::Yellow,
            400..=499 => Color::Red,
            500..=599 => Color::LightRed,
            _ => Color::Gray,
        }
    }
}

// ── Tree helpers ──────────────────────────────────────────────────────────────

pub type ItemPath = Vec<usize>;

pub fn resolve_path<'a>(items: &'a [PostmanItem], path: &[usize]) -> Option<&'a PostmanItem> {
    let (head, tail) = path.split_first()?;
    let item = items.get(*head)?;
    if tail.is_empty() {
        Some(item)
    } else {
        resolve_path(item.item.as_deref()?, tail)
    }
}

pub fn resolve_path_mut<'a>(
    items: &'a mut Vec<PostmanItem>,
    path: &[usize],
) -> Option<&'a mut PostmanItem> {
    let (head, tail) = path.split_first()?;
    let item = items.get_mut(*head)?;
    if tail.is_empty() {
        Some(item)
    } else {
        resolve_path_mut(item.item.as_mut()?, tail)
    }
}
