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
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    #[default]
    None,
    Basic,
    Bearer,
    OAuth2,
    JWT,
}

impl AuthType {
    pub fn label(&self) -> &str {
        match self {
            AuthType::None => "None",
            AuthType::Basic => "Basic Auth",
            AuthType::Bearer => "Bearer",
            AuthType::OAuth2 => "OAuth 2.0",
            AuthType::JWT => "JWT",
        }
    }

    pub fn all() -> [AuthType; 5] {
        [
            AuthType::None,
            AuthType::Basic,
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub username: String,
    pub password: String,
    pub token: String,
}

impl AuthConfig {
    pub fn to_auth_header(&self) -> Option<(String, String)> {
        match &self.auth_type {
            AuthType::None => None,
            AuthType::Basic => {
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
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(CHARS[b0 >> 2] as char);
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

// ── OpenCollection ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcCollection {
    pub name: String,
    #[serde(default)]
    pub items: Vec<OcItem>,
}

impl OcCollection {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            items: vec![],
        }
    }
}

impl Default for OcCollection {
    fn default() -> Self {
        Self::new("endptr")
    }
}

/// A collection item: either a request or a folder.
/// Uses untagged deserialization: request has `info` + `http`; folder has `name` + `items`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OcItem {
    Request(OcRequest),
    Folder(OcFolder),
}

impl OcItem {
    pub fn new_folder(name: &str) -> Self {
        OcItem::Folder(OcFolder {
            name: name.to_string(),
            description: None,
            items: vec![],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcRequest {
    pub info: OcInfo,
    pub http: OcHttp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcFolder {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub items: Vec<OcItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OcHttp {
    pub method: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<OcHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<OcParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<OcBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<OcAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcHeader {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcParam {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcBody {
    #[serde(rename = "type")]
    pub body_type: String,
    pub data: String,
}

/// Auth value: either the string `"inherit"` or a concrete auth config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OcAuth {
    Inherit(String),
    Config(OcAuthConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcAuthConfig {
    #[serde(rename = "type")]
    pub auth_type: AuthType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl OcAuthConfig {
    pub fn to_auth_config(&self) -> AuthConfig {
        AuthConfig {
            auth_type: self.auth_type.clone(),
            username: self.username.clone().unwrap_or_default(),
            password: self.password.clone().unwrap_or_default(),
            token: self.token.clone().unwrap_or_default(),
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
    pub fn to_oc_request(&self, name: &str, seq: u32) -> OcItem {
        let headers = if self.headers.is_empty() {
            None
        } else {
            Some(
                self.headers
                    .iter()
                    .map(|(k, v)| OcHeader {
                        key: k.clone(),
                        value: v.clone(),
                    })
                    .collect(),
            )
        };

        let params = if self.params.is_empty() {
            None
        } else {
            Some(
                self.params
                    .iter()
                    .map(|p| OcParam {
                        key: p.key.clone(),
                        value: p.value.clone(),
                        enabled: p.enabled,
                    })
                    .collect(),
            )
        };

        let body = if self.body.is_empty() {
            None
        } else {
            Some(OcBody {
                body_type: "json".to_string(),
                data: self.body.clone(),
            })
        };

        let auth = match self.auth.auth_type {
            AuthType::None => None,
            AuthType::Basic => Some(OcAuth::Config(OcAuthConfig {
                auth_type: AuthType::Basic,
                username: Some(self.auth.username.clone()),
                password: Some(self.auth.password.clone()),
                token: None,
            })),
            ref t => Some(OcAuth::Config(OcAuthConfig {
                auth_type: t.clone(),
                username: None,
                password: None,
                token: Some(self.auth.token.clone()),
            })),
        };

        OcItem::Request(OcRequest {
            info: OcInfo {
                name: name.to_string(),
                item_type: "http".to_string(),
                seq,
            },
            http: OcHttp {
                method: self.method.as_str().to_string(),
                url: self.url.clone(),
                headers,
                params,
                body,
                auth,
            },
            description: None,
        })
    }
}

impl From<&OcHttp> for CurrentRequest {
    fn from(http: &OcHttp) -> Self {
        let auth = match &http.auth {
            Some(OcAuth::Config(c)) => c.to_auth_config(),
            _ => AuthConfig::default(),
        };

        Self {
            method: HttpMethod::from_str(&http.method),
            url: http.url.clone(),
            headers: http
                .headers
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(|h| (h.key.clone(), h.value.clone()))
                .collect(),
            body: http
                .body
                .as_ref()
                .map(|b| b.data.clone())
                .unwrap_or_default(),
            auth,
            params: http
                .params
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .map(|p| QueryParam {
                    key: p.key.clone(),
                    value: p.value.clone(),
                    enabled: p.enabled,
                })
                .collect(),
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

pub fn resolve_path<'a>(items: &'a [OcItem], path: &[usize]) -> Option<&'a OcItem> {
    let (head, tail) = path.split_first()?;
    let item = items.get(*head)?;
    if tail.is_empty() {
        Some(item)
    } else {
        match item {
            OcItem::Folder(f) => resolve_path(&f.items, tail),
            OcItem::Request(_) => None,
        }
    }
}

pub fn resolve_path_mut<'a>(
    items: &'a mut Vec<OcItem>,
    path: &[usize],
) -> Option<&'a mut OcItem> {
    let (head, tail) = path.split_first()?;
    let item = items.get_mut(*head)?;
    if tail.is_empty() {
        Some(item)
    } else {
        match item {
            OcItem::Folder(f) => resolve_path_mut(&mut f.items, tail),
            OcItem::Request(_) => None,
        }
    }
}
