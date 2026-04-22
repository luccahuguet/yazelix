// Test lane: default
//! Rust-owned launch-time terminal/ghostty materialization decisions for shell-owned launch paths.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::CoreError;
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, runtime_dir_from_env, state_dir_from_env,
};
use crate::ghostty_materialization::{
    GhosttyMaterializationRequest, generate_ghostty_materialization,
};
use crate::terminal_materialization::{
    TerminalGeneratedConfig, TerminalMaterializationRequest, generate_terminal_materialization,
};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::{Path, PathBuf};

const SUPPORTED_TERMINALS: &[&str] = &["ghostty", "wezterm", "kitty", "alacritty", "foot"];
const DEFAULT_TERMINAL_CONFIG_MODE: &str = "yazelix";
const DEFAULT_TERMINALS: &[&str] = &["ghostty"];

#[derive(Debug, Clone)]
pub struct LaunchMaterializationRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub selected_terminals: Vec<String>,
    pub desktop_fast_path: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LaunchMaterializationData {
    pub terminal_config_mode: String,
    pub selected_terminals: Vec<String>,
    pub generated_terminals: Vec<TerminalGeneratedConfig>,
    pub rerolled_ghostty_cursor: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchMaterializationPlan {
    terminal_config_mode: String,
    selected_terminals: Vec<String>,
    should_generate_terminal_configs: bool,
    should_reroll_ghostty_cursor: bool,
}

pub fn launch_materialization_request_from_env(
    selected_terminals: Vec<String>,
    desktop_fast_path: bool,
) -> Result<LaunchMaterializationRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = resolve_active_config_paths(
        &runtime_dir,
        &config_dir,
        config_override_from_env().as_deref(),
    )?;
    let state_dir = state_dir_from_env()?;

    Ok(LaunchMaterializationRequest {
        config_path: paths.config_file,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        runtime_dir,
        config_dir,
        state_dir,
        selected_terminals,
        desktop_fast_path,
    })
}

pub fn prepare_launch_materialization(
    request: &LaunchMaterializationRequest,
) -> Result<LaunchMaterializationData, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: false,
    })?
    .normalized_config;
    let plan = build_launch_materialization_plan(
        &normalized,
        &request.selected_terminals,
        request.desktop_fast_path,
        &request.state_dir,
    );

    let mut generated_terminals = Vec::new();
    if plan.should_generate_terminal_configs {
        generated_terminals = generate_terminal_materialization(&TerminalMaterializationRequest {
            config_path: request.config_path.clone(),
            default_config_path: request.default_config_path.clone(),
            contract_path: request.contract_path.clone(),
            runtime_dir: request.runtime_dir.clone(),
            state_dir: request.state_dir.clone(),
            terminals: plan.selected_terminals.clone(),
        })?
        .generated;
    } else if plan.should_reroll_ghostty_cursor {
        generate_ghostty_materialization(&GhosttyMaterializationRequest {
            runtime_dir: request.runtime_dir.clone(),
            config_dir: request.config_dir.clone(),
            state_dir: request.state_dir.clone(),
            transparency: string_config(&normalized, "transparency", "none"),
            ghostty_trail_color: optional_string_config(&normalized, "ghostty_trail_color"),
            ghostty_trail_effect: optional_string_config(&normalized, "ghostty_trail_effect"),
            ghostty_mode_effect: optional_string_config(&normalized, "ghostty_mode_effect"),
            ghostty_trail_glow: string_config(&normalized, "ghostty_trail_glow", "medium"),
        })?;
    }

    let rerolled_ghostty_cursor = plan.should_reroll_ghostty_cursor
        || (plan.should_generate_terminal_configs
            && plan
                .selected_terminals
                .iter()
                .any(|terminal| terminal == "ghostty")
            && ghostty_cursor_random_requested(&normalized));

    Ok(LaunchMaterializationData {
        terminal_config_mode: plan.terminal_config_mode,
        selected_terminals: plan.selected_terminals,
        generated_terminals,
        rerolled_ghostty_cursor,
    })
}

fn build_launch_materialization_plan(
    normalized: &JsonMap<String, JsonValue>,
    requested_terminals: &[String],
    desktop_fast_path: bool,
    state_dir: &Path,
) -> LaunchMaterializationPlan {
    let terminal_config_mode = string_config(
        normalized,
        "terminal_config_mode",
        DEFAULT_TERMINAL_CONFIG_MODE,
    );
    let selected_terminals = if requested_terminals.is_empty() {
        normalize_selected_terminals(string_list_config(
            normalized,
            "terminals",
            DEFAULT_TERMINALS,
        ))
    } else {
        normalize_selected_terminals(requested_terminals.to_vec())
    };

    if selected_terminals.is_empty() {
        return LaunchMaterializationPlan {
            terminal_config_mode,
            selected_terminals,
            should_generate_terminal_configs: false,
            should_reroll_ghostty_cursor: false,
        };
    }

    if !desktop_fast_path {
        return LaunchMaterializationPlan {
            terminal_config_mode,
            selected_terminals,
            should_generate_terminal_configs: true,
            should_reroll_ghostty_cursor: false,
        };
    }

    if terminal_config_mode != "yazelix" {
        return LaunchMaterializationPlan {
            terminal_config_mode,
            selected_terminals,
            should_generate_terminal_configs: false,
            should_reroll_ghostty_cursor: false,
        };
    }

    let should_generate_terminal_configs = selected_terminals
        .iter()
        .any(|terminal| !generated_terminal_config_path(state_dir, terminal).is_file());
    let should_reroll_ghostty_cursor = !should_generate_terminal_configs
        && selected_terminals
            .iter()
            .any(|terminal| terminal == "ghostty")
        && ghostty_cursor_random_requested(normalized);

    LaunchMaterializationPlan {
        terminal_config_mode,
        selected_terminals,
        should_generate_terminal_configs,
        should_reroll_ghostty_cursor,
    }
}

fn normalize_selected_terminals(terminals: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for terminal in terminals {
        let trimmed = terminal.trim().to_ascii_lowercase();
        if trimmed.is_empty() || !SUPPORTED_TERMINALS.contains(&trimmed.as_str()) {
            continue;
        }
        if !normalized.contains(&trimmed) {
            normalized.push(trimmed);
        }
    }
    normalized
}

fn generated_terminal_config_path(state_dir: &Path, terminal: &str) -> PathBuf {
    let root = state_dir.join("configs").join("terminal_emulators");
    match terminal {
        "ghostty" => root.join("ghostty").join("config"),
        "wezterm" => root.join("wezterm").join(".wezterm.lua"),
        "kitty" => root.join("kitty").join("kitty.conf"),
        "alacritty" => root.join("alacritty").join("alacritty.toml"),
        "foot" => root.join("foot").join("foot.ini"),
        _ => root.join(terminal),
    }
}

fn ghostty_cursor_random_requested(config: &JsonMap<String, JsonValue>) -> bool {
    [
        "ghostty_trail_color",
        "ghostty_trail_effect",
        "ghostty_mode_effect",
    ]
    .iter()
    .any(|key| optional_string_config(config, key).as_deref() == Some("random"))
}

fn string_config(config: &JsonMap<String, JsonValue>, key: &str, default: &str) -> String {
    config
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .to_string()
}

fn optional_string_config(config: &JsonMap<String, JsonValue>, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn string_list_config(
    config: &JsonMap<String, JsonValue>,
    key: &str,
    default: &[&str],
) -> Vec<String> {
    match config.get(key) {
        Some(JsonValue::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str())
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        _ => default.iter().map(|item| item.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn config_with_terminals(
        terminals: &[&str],
        terminal_config_mode: &str,
        ghostty_trail_color: &str,
    ) -> JsonMap<String, JsonValue> {
        let mut config = JsonMap::new();
        config.insert("terminals".into(), json!(terminals));
        config.insert("terminal_config_mode".into(), json!(terminal_config_mode));
        config.insert("ghostty_trail_color".into(), json!(ghostty_trail_color));
        config.insert("ghostty_trail_effect".into(), json!("tail"));
        config.insert("ghostty_mode_effect".into(), json!("ripple"));
        config
    }

    // Defends: desktop fast-path launch keeps terminal generation minimal and rerolls Ghostty only when random cursor state survives on an existing generated config.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn desktop_fast_path_rerolls_existing_random_ghostty_config_without_full_regeneration() {
        let temp = tempdir().unwrap();
        let ghostty_dir = temp
            .path()
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty");
        std::fs::create_dir_all(&ghostty_dir).unwrap();
        std::fs::write(ghostty_dir.join("config"), "existing").unwrap();
        let config = config_with_terminals(&["ghostty"], "yazelix", "random");

        let plan =
            build_launch_materialization_plan(&config, &["ghostty".to_string()], true, temp.path());

        assert_eq!(
            plan,
            LaunchMaterializationPlan {
                terminal_config_mode: "yazelix".to_string(),
                selected_terminals: vec!["ghostty".to_string()],
                should_generate_terminal_configs: false,
                should_reroll_ghostty_cursor: true,
            }
        );
    }

    // Defends: non-desktop launch materialization still regenerates the configured terminal set even after the request-assembly cut.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn full_launch_materialization_uses_configured_terminals_when_callers_do_not_override_them() {
        let temp = tempdir().unwrap();
        let config = config_with_terminals(&["ghostty", "wezterm", ""], "user", "reef");

        let plan = build_launch_materialization_plan(&config, &[], false, temp.path());

        assert_eq!(
            plan,
            LaunchMaterializationPlan {
                terminal_config_mode: "user".to_string(),
                selected_terminals: vec!["ghostty".to_string(), "wezterm".to_string()],
                should_generate_terminal_configs: true,
                should_reroll_ghostty_cursor: false,
            }
        );
    }
}
