use crate::bridge::CoreError;
use crate::zellij_render_plan::managed_sidebar_layout_name;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct RuntimeEnvComputeRequest {
    pub runtime_dir: PathBuf,
    pub home_dir: PathBuf,
    #[serde(default)]
    pub current_path: RuntimePathInput,
    #[serde(default = "default_enable_sidebar")]
    pub enable_sidebar: bool,
    #[serde(default = "default_initial_sidebar_state")]
    pub initial_sidebar_state: String,
    #[serde(default)]
    pub editor_command: Option<String>,
    #[serde(default)]
    pub helix_runtime_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RuntimePathInput {
    List(Vec<String>),
    String(String),
}

impl Default for RuntimePathInput {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeEnvComputeData {
    pub runtime_env: JsonMap<String, JsonValue>,
    pub editor_kind: String,
    pub path_entries: Vec<String>,
}

pub fn compute_runtime_env(
    request: &RuntimeEnvComputeRequest,
) -> Result<RuntimeEnvComputeData, CoreError> {
    let normalized_path_entries = normalize_path_entries(&request.current_path);
    let current_path_entries =
        strip_runtime_owned_path_entries(normalized_path_entries, &request.runtime_dir);
    let runtime_path_entries = existing_runtime_path_entries(&request.runtime_dir);
    let path_entries = if runtime_path_entries.is_empty() {
        current_path_entries
    } else {
        stable_dedupe(
            runtime_path_entries
                .into_iter()
                .chain(current_path_entries)
                .collect(),
        )
    };

    let resolved_editor_command = resolve_editor_command(request);
    let editor_kind = resolve_editor_kind(&resolved_editor_command);
    let default_layout_name =
        managed_sidebar_layout_name(request.enable_sidebar, &request.initial_sidebar_state)?;
    let editor_command = if editor_kind == "helix" {
        path_to_string(
            &request
                .runtime_dir
                .join("shells")
                .join("posix")
                .join("yazelix_hx.sh"),
        )
    } else {
        resolved_editor_command.clone()
    };

    let mut runtime_env = JsonMap::new();
    runtime_env.insert(
        "PATH".to_string(),
        JsonValue::Array(
            path_entries
                .iter()
                .cloned()
                .map(JsonValue::String)
                .collect(),
        ),
    );
    runtime_env.insert(
        "YAZELIX_RUNTIME_DIR".to_string(),
        JsonValue::String(path_to_string(&request.runtime_dir)),
    );
    runtime_env.insert(
        "IN_YAZELIX_SHELL".to_string(),
        JsonValue::String("true".to_string()),
    );
    runtime_env.insert(
        "ZELLIJ_DEFAULT_LAYOUT".to_string(),
        JsonValue::String(default_layout_name.to_string()),
    );
    runtime_env.insert(
        "YAZI_CONFIG_HOME".to_string(),
        JsonValue::String(path_to_string(
            &request
                .home_dir
                .join(".local")
                .join("share")
                .join("yazelix")
                .join("configs")
                .join("yazi"),
        )),
    );
    runtime_env.insert(
        "EDITOR".to_string(),
        JsonValue::String(editor_command.clone()),
    );
    runtime_env.insert("VISUAL".to_string(), JsonValue::String(editor_command));

    if editor_kind == "helix" {
        runtime_env.insert(
            "YAZELIX_MANAGED_HELIX_BINARY".to_string(),
            JsonValue::String(resolved_editor_command),
        );
    }

    if let Some(helix_runtime) = resolve_helix_runtime(request) {
        runtime_env.insert(
            "HELIX_RUNTIME".to_string(),
            JsonValue::String(helix_runtime),
        );
    }

    Ok(RuntimeEnvComputeData {
        runtime_env,
        editor_kind,
        path_entries,
    })
}

fn default_enable_sidebar() -> bool {
    true
}

fn default_initial_sidebar_state() -> String {
    "open".into()
}

fn normalize_path_entries(value: &RuntimePathInput) -> Vec<String> {
    match value {
        RuntimePathInput::List(entries) => entries.clone(),
        RuntimePathInput::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                trimmed
                    .split(path_list_separator())
                    .map(ToOwned::to_owned)
                    .collect()
            }
        }
    }
}

fn path_list_separator() -> char {
    if cfg!(windows) { ';' } else { ':' }
}

fn runtime_owned_path_entries(runtime_dir: &Path) -> Vec<String> {
    [
        runtime_dir.join("toolbin"),
        runtime_dir.join("bin"),
        runtime_dir.join("libexec"),
    ]
    .into_iter()
    .map(|path| path_to_string(&path))
    .collect()
}

fn strip_runtime_owned_path_entries(entries: Vec<String>, runtime_dir: &Path) -> Vec<String> {
    let runtime_owned: HashSet<String> = runtime_owned_path_entries(runtime_dir)
        .into_iter()
        .collect();
    entries
        .into_iter()
        .filter(|entry| !runtime_owned.contains(entry))
        .collect()
}

fn existing_runtime_path_entries(runtime_dir: &Path) -> Vec<String> {
    [runtime_dir.join("toolbin"), runtime_dir.join("bin")]
        .into_iter()
        .filter(|path| path.exists())
        .map(|path| path_to_string(&path))
        .collect()
}

fn stable_dedupe(entries: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for entry in entries {
        if seen.insert(entry.clone()) {
            deduped.push(entry);
        }
    }

    deduped
}

fn resolve_editor_command(request: &RuntimeEnvComputeRequest) -> String {
    request
        .editor_command
        .as_deref()
        .map(str::trim)
        .filter(|editor| !editor.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "hx".to_string())
}

fn resolve_editor_kind(editor: &str) -> String {
    if is_helix_editor_command(editor) {
        "helix".to_string()
    } else if is_neovim_editor_command(editor) {
        "neovim".to_string()
    } else {
        String::new()
    }
}

fn resolve_helix_runtime(request: &RuntimeEnvComputeRequest) -> Option<String> {
    request
        .helix_runtime_path
        .as_deref()
        .map(str::trim)
        .filter(|runtime| !runtime.is_empty())
        .map(ToOwned::to_owned)
}

fn is_helix_editor_command(editor: &str) -> bool {
    let normalized = editor.trim();
    normalized.ends_with("/hx")
        || normalized == "hx"
        || normalized.ends_with("/helix")
        || normalized == "helix"
}

fn is_neovim_editor_command(editor: &str) -> bool {
    let normalized = editor.trim();
    normalized.ends_with("/nvim")
        || normalized == "nvim"
        || normalized.ends_with("/neovim")
        || normalized == "neovim"
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
