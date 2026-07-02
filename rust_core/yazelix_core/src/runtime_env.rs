use crate::bridge::{CoreError, ErrorClass};
use crate::helix_external::{HelixExternalPair, is_custom_helix_binary_command, is_helix_command};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use yazelix_zellij_config_pack::MANAGED_SIDEBAR_LAYOUT_NAME;

#[derive(Debug, Deserialize)]
pub struct RuntimeEnvComputeRequest {
    pub runtime_dir: PathBuf,
    pub home_dir: PathBuf,
    #[serde(default)]
    pub xdg_config_home: Option<PathBuf>,
    #[serde(default)]
    pub current_path: RuntimePathInput,
    #[serde(default)]
    pub current_lazygit_config_file: Option<String>,
    #[serde(default)]
    pub editor_command: Option<String>,
    #[serde(default)]
    pub helix_external: Option<HelixExternalPair>,
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
    let host_user_path_entries = existing_host_user_path_entries(&request.home_dir);
    let path_entries = if runtime_path_entries.is_empty() {
        stable_dedupe(
            host_user_path_entries
                .into_iter()
                .chain(current_path_entries)
                .collect(),
        )
    } else {
        stable_dedupe(
            runtime_path_entries
                .into_iter()
                .chain(host_user_path_entries)
                .chain(current_path_entries)
                .collect(),
        )
    };

    validate_helix_editor_runtime_pair(request)?;
    let resolved_editor_command = resolve_editor_command(request);
    let editor_kind = resolve_editor_kind(&resolved_editor_command);
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
        "YAZELIX_RTK_REQUIRED".to_string(),
        JsonValue::String("true".to_string()),
    );
    runtime_env.insert(
        "YAZELIX_CODEX_COMMAND".to_string(),
        JsonValue::String("rtk codex".to_string()),
    );
    runtime_env.insert(
        "ZELLIJ_DEFAULT_LAYOUT".to_string(),
        JsonValue::String(MANAGED_SIDEBAR_LAYOUT_NAME.to_string()),
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
    if let Some(lazygit_config_file) = resolve_lazygit_config_file(request, &editor_kind) {
        runtime_env.insert(
            "LG_CONFIG_FILE".to_string(),
            JsonValue::String(lazygit_config_file),
        );
    }
    runtime_env.insert(
        "EDITOR".to_string(),
        JsonValue::String(editor_command.clone()),
    );
    runtime_env.insert("VISUAL".to_string(), JsonValue::String(editor_command));

    if editor_kind == "helix" {
        runtime_env.insert(
            "YAZELIX_MANAGED_HELIX_BINARY".to_string(),
            JsonValue::String(resolve_managed_helix_binary(
                request,
                &resolved_editor_command,
            )),
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

fn existing_host_user_path_entries(home_dir: &Path) -> Vec<String> {
    [
        home_dir.join(".local").join("bin"),
        home_dir
            .join(".local")
            .join("state")
            .join("nix")
            .join("profile")
            .join("bin"),
        home_dir.join(".nix-profile").join("bin"),
        PathBuf::from("/nix/var/nix/profiles/default/bin"),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .map(|path| path_to_string(&path))
    .collect()
}

fn resolve_lazygit_config_file(
    request: &RuntimeEnvComputeRequest,
    editor_kind: &str,
) -> Option<String> {
    if editor_kind != "helix" {
        return None;
    }

    let runtime_config = request
        .runtime_dir
        .join("configs")
        .join("lazygit")
        .join("yazelix_config.yml");
    if !runtime_config.is_file() {
        return None;
    }

    let mut files = vec![path_to_string(&runtime_config)];
    if let Some(config_file) = request
        .current_lazygit_config_file
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        files.extend(
            config_file
                .split(',')
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .filter(|path| !is_yazelix_lazygit_runtime_config(path))
                .map(ToOwned::to_owned),
        );
    } else {
        let user_config = request
            .xdg_config_home
            .clone()
            .unwrap_or_else(|| request.home_dir.join(".config"))
            .join("lazygit")
            .join("config.yml");
        if user_config.is_file() {
            files.push(path_to_string(&user_config));
        }
    }

    Some(files.join(","))
}

fn is_yazelix_lazygit_runtime_config(path: &str) -> bool {
    path.trim_end_matches('/')
        .ends_with("/configs/lazygit/yazelix_config.yml")
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
    if let Some(external) = &request.helix_external {
        return external.binary.clone();
    }
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
        .helix_external
        .as_ref()
        .map(|external| external.runtime_path.clone())
}

fn resolve_managed_helix_binary(
    request: &RuntimeEnvComputeRequest,
    resolved_editor_command: &str,
) -> String {
    if let Some(external) = &request.helix_external {
        return external.binary.clone();
    }
    if matches!(resolved_editor_command.trim(), "hx" | "helix") {
        return path_to_string(&request.runtime_dir.join("libexec").join("hx"));
    }
    resolved_editor_command.to_string()
}

fn is_helix_editor_command(editor: &str) -> bool {
    is_helix_command(editor)
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

fn validate_helix_editor_runtime_pair(request: &RuntimeEnvComputeRequest) -> Result<(), CoreError> {
    let Some(editor_command) = request
        .editor_command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    if request.helix_external.is_some() {
        if is_helix_command(editor_command) {
            return Ok(());
        }
        return Err(CoreError::classified(
            ErrorClass::Config,
            "helix_external_conflicts_with_editor",
            "helix.external is set while editor.command points at a non-Helix editor.",
            "Remove helix.external for non-Helix editors, or leave editor.command empty to use the external Helix pair.",
            serde_json::json!({
                "editor_command": editor_command,
            }),
        ));
    }
    if is_custom_helix_binary_command(editor_command) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "helix_external_required",
            "A custom Helix binary must be configured as a binary/runtime pair.",
            "Set helix.external = { binary = \"/path/to/hx\", runtime_path = \"/path/to/helix/runtime\" } instead of setting only editor.command.",
            serde_json::json!({
                "editor_command": editor_command,
            }),
        ));
    }
    Ok(())
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::{RuntimeEnvComputeRequest, RuntimePathInput, compute_runtime_env};
    use std::fs;

    fn request_with_path(
        runtime_dir: std::path::PathBuf,
        home_dir: std::path::PathBuf,
        current_path: &str,
    ) -> RuntimeEnvComputeRequest {
        RuntimeEnvComputeRequest {
            runtime_dir,
            home_dir,
            xdg_config_home: None,
            current_path: RuntimePathInput::String(current_path.to_string()),
            current_lazygit_config_file: None,
            editor_command: None,
            helix_external: None,
        }
    }

    fn index_of(entries: &[String], suffix: &str) -> usize {
        entries
            .iter()
            .position(|entry| entry.ends_with(suffix))
            .unwrap_or_else(|| panic!("missing PATH entry ending with {suffix}: {entries:?}"))
    }

    // Regression: GUI/desktop launchers often start with a sparse PATH, but Yazelix agent panes still need host-installed commands such as ~/.local/bin/codex.
    #[test]
    fn desktop_runtime_path_adds_standard_user_bins() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path().join("runtime");
        let home_dir = temp.path().join("home");
        for dir in [
            runtime_dir.join("toolbin"),
            runtime_dir.join("bin"),
            home_dir.join(".local").join("bin"),
            home_dir
                .join(".local")
                .join("state")
                .join("nix")
                .join("profile")
                .join("bin"),
            home_dir.join(".nix-profile").join("bin"),
        ] {
            fs::create_dir_all(dir).unwrap();
        }

        let data = compute_runtime_env(&request_with_path(runtime_dir, home_dir, "/usr/bin:/bin"))
            .unwrap();

        let runtime_bin = index_of(&data.path_entries, "/runtime/bin");
        let user_local_bin = index_of(&data.path_entries, "/home/.local/bin");
        let nix_profile_bin = index_of(&data.path_entries, "/home/.nix-profile/bin");
        let usr_bin = index_of(&data.path_entries, "/usr/bin");

        assert!(runtime_bin < user_local_bin);
        assert!(user_local_bin < nix_profile_bin);
        assert!(nix_profile_bin < usr_bin);
    }

    // Invariant: adding standard host user bins must not duplicate entries already inherited from the parent shell.
    #[test]
    fn standard_user_bins_are_deduped_against_current_path() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path().join("runtime");
        let home_dir = temp.path().join("home");
        let user_local_bin = home_dir.join(".local").join("bin");
        fs::create_dir_all(runtime_dir.join("bin")).unwrap();
        fs::create_dir_all(&user_local_bin).unwrap();

        let data = compute_runtime_env(&request_with_path(
            runtime_dir,
            home_dir,
            &format!("{}:/usr/bin", user_local_bin.display()),
        ))
        .unwrap();
        let user_local_bin_string = user_local_bin.to_string_lossy().to_string();

        assert_eq!(
            data.path_entries
                .iter()
                .filter(|entry| *entry == &user_local_bin_string)
                .count(),
            1
        );
    }

    // Defends: every Yazelix runtime session advertises the RTK requirement for Codex/agent use.
    #[test]
    fn runtime_env_marks_rtk_as_required_for_codex_sessions() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path().join("runtime");
        let home_dir = temp.path().join("home");
        fs::create_dir_all(runtime_dir.join("bin")).unwrap();

        let data = compute_runtime_env(&request_with_path(runtime_dir, home_dir, "/usr/bin:/bin"))
            .unwrap();

        assert_eq!(
            data.runtime_env["YAZELIX_RTK_REQUIRED"].as_str(),
            Some("true")
        );
        assert_eq!(
            data.runtime_env["YAZELIX_CODEX_COMMAND"].as_str(),
            Some("rtk codex")
        );
    }
}
