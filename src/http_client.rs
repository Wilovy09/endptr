use srtemplate::SrTemplate;

use crate::models::{AuthType, CurrentRequest, HttpMethod, HttpResponse, Secrets};

pub fn resolve_template(text: &str, secrets: &Secrets) -> String {
    let tpl = SrTemplate::default();
    for (k, v) in secrets {
        tpl.add_variable(k, v.as_str());
    }
    tpl.render(text).unwrap_or_else(|_| text.to_string())
}

/// Append enabled query params to a URL, respecting existing `?`.
fn append_params(base_url: &str, params: &[(&str, &str)]) -> String {
    if params.is_empty() {
        return base_url.to_string();
    }
    let sep = if base_url.contains('?') { '&' } else { '?' };
    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{base_url}{sep}{query}")
}

/// Percent-encode a query param key or value (minimal, covers common chars).
fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn send_request(request: CurrentRequest, secrets: Secrets) -> Result<HttpResponse, String> {
    // ── Resolve URL + params ──────────────────────────────────────────────────
    let base_url = resolve_template(&request.url, &secrets);

    let resolved_params: Vec<(String, String)> = request
        .params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| {
            (
                resolve_template(&p.key, &secrets),
                resolve_template(&p.value, &secrets),
            )
        })
        .collect();

    let param_refs: Vec<(&str, &str)> = resolved_params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let url = append_params(&base_url, &param_refs);

    // ── Build request ─────────────────────────────────────────────────────────
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let method = match request.method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::PATCH => reqwest::Method::PATCH,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::HEAD => reqwest::Method::HEAD,
        HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
    };

    let mut builder = client.request(method, &url);

    // Auth header (before user headers so user can override)
    if let Some((k, v)) = request.auth.to_auth_header() {
        let resolved_v = resolve_template(&v, &secrets);
        builder = builder.header(k, resolved_v);
    }

    // User-defined headers
    for (key, value) in &request.headers {
        let resolved = resolve_template(value, &secrets);
        builder = builder.header(key.as_str(), resolved);
    }

    // Body
    if !request.body.is_empty() {
        let resolved_body = resolve_template(&request.body, &secrets);
        builder = builder.body(resolved_body);
    }

    // ── Send ──────────────────────────────────────────────────────────────────
    let start = std::time::Instant::now();
    let response = builder.send().map_err(|e| e.to_string())?;
    let time_ms = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();
    let response_headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = response.text().map_err(|e| e.to_string())?;

    Ok(HttpResponse {
        status,
        status_text,
        response_headers,
        body,
        time_ms,
    })
}
