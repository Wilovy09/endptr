use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::{PostmanCollection, Secrets};

pub fn endptr_dir() -> PathBuf {
    PathBuf::from(".endptr")
}

fn secrets_path() -> PathBuf {
    endptr_dir().join("secrets.json")
}

fn ensure_dir() -> std::io::Result<()> {
    fs::create_dir_all(endptr_dir())
}

// ── Collections ───────────────────────────────────────────────────────────────

/// Load every `*.json` file in `.endptr/` (excluding `secrets.json`).
/// Returns `(path, collection)` pairs sorted by filename.
pub fn load_all_collections() -> Vec<(PathBuf, PostmanCollection)> {
    let _ = ensure_dir();
    let dir = endptr_dir();

    let mut result: Vec<(PathBuf, PostmanCollection)> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "json" {
                return None;
            }
            if path.file_name()?.to_str()? == "secrets.json" {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let col: PostmanCollection = serde_json::from_str(&content).ok()?;
            Some((path, col))
        })
        .collect();

    result.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Bootstrap: if nothing exists, create default collection.json
    if result.is_empty() {
        let default_path = dir.join("collection.json");
        let col = PostmanCollection::default();
        let _ = save_collection(&default_path, &col);
        result.push((default_path, col));
    }

    result
}

pub fn save_collection(path: &Path, collection: &PostmanCollection) -> std::io::Result<()> {
    let _ = ensure_dir();
    let json = serde_json::to_string_pretty(collection)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, json)
}

/// Create a new collection file. Returns its path.
pub fn create_collection(name: &str) -> std::io::Result<(PathBuf, PostmanCollection)> {
    let _ = ensure_dir();
    // Sanitize filename
    let filename = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let path = endptr_dir().join(format!("{filename}.json"));
    let col = PostmanCollection::new(name);
    save_collection(&path, &col)?;
    Ok((path, col))
}

pub fn delete_collection(path: &Path) -> std::io::Result<()> {
    fs::remove_file(path)
}

// ── Workflows ─────────────────────────────────────────────────────────────────

pub fn workflows_dir() -> PathBuf {
    endptr_dir().join("workflows")
}

fn ensure_workflows_dir() -> std::io::Result<()> {
    fs::create_dir_all(workflows_dir())
}

/// Load every `*.yaml` file in `.endptr/workflows/`.
/// Returns `(path, workflow)` pairs sorted by filename.
pub fn load_all_workflows() -> Vec<(PathBuf, Workflow)> {
    let dir = workflows_dir();
    if !dir.exists() {
        return vec![];
    }

    let mut result: Vec<(PathBuf, Workflow)> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "yaml" {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let wf: Workflow = serde_yaml::from_str(&content).ok()?;
            Some((path, wf))
        })
        .collect();

    result.sort_by(|(a, _), (b, _)| a.cmp(b));
    result
}

pub fn save_workflow(path: &Path, workflow: &Workflow) -> std::io::Result<()> {
    let _ = ensure_workflows_dir();
    let yaml = serde_yaml::to_string(workflow)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, yaml)
}

pub fn delete_workflow(path: &Path) -> std::io::Result<()> {
    fs::remove_file(path)
}

// ── Secrets ───────────────────────────────────────────────────────────────────

pub fn load_secrets() -> Secrets {
    let _ = ensure_dir();
    let path = secrets_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Secrets::default()
    }
}

pub fn save_secrets(secrets: &Secrets) -> std::io::Result<()> {
    let _ = ensure_dir();
    let json = serde_json::to_string_pretty(secrets)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(secrets_path(), json)
}
