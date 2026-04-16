use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    models::{OcCollection, Secrets},
    workflow::Workflow,
};

pub fn endptr_dir() -> PathBuf {
    PathBuf::from(".endptr")
}

fn secrets_path() -> PathBuf {
    endptr_dir().join("secrets.yaml")
}

fn ensure_dir() -> std::io::Result<()> {
    fs::create_dir_all(endptr_dir())
}

// ── Collections ───────────────────────────────────────────────────────────────

/// Load every `*.yaml` file in `.endptr/` (excluding `secrets.yaml`).
/// Returns `(path, collection)` pairs sorted by filename.
pub fn load_all_collections() -> Vec<(PathBuf, OcCollection)> {
    let _ = ensure_dir();
    let dir = endptr_dir();

    let mut result: Vec<(PathBuf, OcCollection)> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "yaml" {
                return None;
            }
            if path.file_name()?.to_str()? == "secrets.yaml" {
                return None;
            }
            let content = fs::read_to_string(&path).ok()?;
            let col: OcCollection = serde_yaml::from_str(&content).ok()?;
            Some((path, col))
        })
        .collect();

    result.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Bootstrap: if nothing exists, create default collection.yaml
    if result.is_empty() {
        let default_path = dir.join("collection.yaml");
        let col = OcCollection::default();
        let _ = save_collection(&default_path, &col);
        result.push((default_path, col));
    }

    result
}

pub fn save_collection(path: &Path, collection: &OcCollection) -> std::io::Result<()> {
    let _ = ensure_dir();
    let yaml = serde_yaml::to_string(collection)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, yaml)
}

/// Create a new collection file. Returns its path.
pub fn create_collection(name: &str) -> std::io::Result<(PathBuf, OcCollection)> {
    let _ = ensure_dir();
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
    let path = endptr_dir().join(format!("{filename}.yaml"));
    let col = OcCollection::new(name);
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
            .and_then(|s| serde_yaml::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Secrets::default()
    }
}

pub fn save_secrets(secrets: &Secrets) -> std::io::Result<()> {
    let _ = ensure_dir();
    let yaml = serde_yaml::to_string(secrets)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(secrets_path(), yaml)
}
