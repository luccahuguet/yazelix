// Test lane: default
//! Rust-owned launch-time terminal/ghostty materialization decisions for shell-owned launch paths.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::CoreError;
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::{config_dir_from_env, runtime_dir_from_env, state_dir_from_env};
use crate::ghostty_cursor_registry::{
    CursorRegistry, ResolvedCursorRegistryState, YazelixCursorRegistryExt,
};
use crate::ghostty_materialization::{
    GhosttyMaterializationRequest, generate_ghostty_materialization,
};
use crate::runtime_component_enabled;
use crate::terminal_materialization::{
    TerminalGeneratedConfig, TerminalMaterializationRequest, generate_terminal_materialization,
};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::{Path, PathBuf};

const SUPPORTED_TERMINALS: &[&str] = &["ghostty", "yzxterm", "wezterm", "ratty", "kitty"];
const DEFAULT_TERMINAL_CONFIG_MODE: &str = "yazelix";
const DEFAULT_TERMINALS: &[&str] = &["ghostty", "wezterm"];

#[derive(Debug, Clone)]
pub struct LaunchMaterializationRequest {
    pub config_path: PathBuf,
    pub cursor_config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub selected_terminals: Vec<String>,
    pub desktop_fast_path: bool,
    pub force_terminal_config_generation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LaunchMaterializationData {
    pub terminal_config_mode: String,
    pub selected_terminals: Vec<String>,
    pub generated_terminals: Vec<TerminalGeneratedConfig>,
    pub ghostty_cursor_name: Option<String>,
    pub ghostty_cursor_color_hex: Option<String>,
    pub ghostty_cursor_family: Option<String>,
    pub ghostty_cursor_divider: Option<String>,
    pub ghostty_cursor_primary_color_hex: Option<String>,
    pub ghostty_cursor_secondary_color_hex: Option<String>,
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
    force_terminal_config_generation: bool,
    config_override: Option<&str>,
) -> Result<LaunchMaterializationRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = resolve_active_config_paths(&runtime_dir, &config_dir, config_override)?;
    let state_dir = state_dir_from_env()?;

    Ok(LaunchMaterializationRequest {
        config_path: paths.config_file,
        cursor_config_path: paths.user_cursor_config,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        runtime_dir,
        config_dir,
        state_dir,
        selected_terminals,
        desktop_fast_path,
        force_terminal_config_generation,
    })
}

pub fn prepare_launch_materialization(
    request: &LaunchMaterializationRequest,
) -> Result<LaunchMaterializationData, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: true,
    })?
    .normalized_config;
    let cursor_config_path = request.cursor_config_path.clone();
    let cursors_enabled = runtime_component_enabled(&request.runtime_dir, "cursors")?;
    let cursor_registry = if cursors_enabled {
        Some(CursorRegistry::load(&cursor_config_path)?)
    } else {
        None
    };
    let ghostty_random_requested = cursor_registry
        .as_ref()
        .map(CursorRegistry::is_random_request)
        .unwrap_or(false);
    let plan = build_launch_materialization_plan(
        &normalized,
        &request.selected_terminals,
        request.desktop_fast_path,
        request.force_terminal_config_generation,
        &request.state_dir,
        ghostty_random_requested,
    );

    let mut generated_terminals = Vec::new();
    let mut ghostty_cursor_name = None;
    let mut ghostty_cursor_color_hex = None;
    let mut ghostty_cursor_family = None;
    let mut ghostty_cursor_divider = None;
    let mut ghostty_cursor_primary_color_hex = None;
    let mut ghostty_cursor_secondary_color_hex = None;
    if plan.should_generate_terminal_configs {
        let terminal_data = generate_terminal_materialization(&TerminalMaterializationRequest {
            config_path: request.config_path.clone(),
            cursor_config_path: request.cursor_config_path.clone(),
            default_config_path: request.default_config_path.clone(),
            contract_path: request.contract_path.clone(),
            runtime_dir: request.runtime_dir.clone(),
            state_dir: request.state_dir.clone(),
            terminals: plan.selected_terminals.clone(),
        })?;
        if plan_uses_yazelix_ghostty_cursor(&plan) {
            if let Some(cursor_data) = terminal_data.cursor.as_ref() {
                ghostty_cursor_name = cursor_data.cursor_state.selected_color.clone();
                ghostty_cursor_color_hex = cursor_data.cursor_state.selected_color_hex.clone();
                ghostty_cursor_family = cursor_data.cursor_state.selected_family.clone();
                ghostty_cursor_divider = cursor_data.cursor_state.selected_divider.clone();
                ghostty_cursor_primary_color_hex =
                    cursor_data.cursor_state.selected_primary_color_hex.clone();
                ghostty_cursor_secondary_color_hex = cursor_data
                    .cursor_state
                    .selected_secondary_color_hex
                    .clone();
            }
        }
        generated_terminals = terminal_data.generated;
    } else if plan.should_reroll_ghostty_cursor {
        let ghostty_data = generate_ghostty_materialization(&GhosttyMaterializationRequest {
            runtime_dir: request.runtime_dir.clone(),
            config_dir: request.config_dir.clone(),
            state_dir: request.state_dir.clone(),
            transparency: string_config(&normalized, "transparency", "none"),
            cursor_config_path,
        })?;
        ghostty_cursor_name = ghostty_data.cursor_state.selected_color;
        ghostty_cursor_color_hex = ghostty_data.cursor_state.selected_color_hex;
        ghostty_cursor_family = ghostty_data.cursor_state.selected_family;
        ghostty_cursor_divider = ghostty_data.cursor_state.selected_divider;
        ghostty_cursor_primary_color_hex = ghostty_data.cursor_state.selected_primary_color_hex;
        ghostty_cursor_secondary_color_hex = ghostty_data.cursor_state.selected_secondary_color_hex;
    } else if plan_uses_yazelix_ghostty_cursor(&plan) && cursors_enabled {
        let cursor_state = cursor_registry
            .as_ref()
            .expect("cursor registry is loaded when cursors are enabled")
            .resolve();
        ghostty_cursor_name = resolved_ghostty_cursor_name(&cursor_state);
        ghostty_cursor_color_hex = resolved_ghostty_cursor_color_hex(&cursor_state);
        ghostty_cursor_family = resolved_ghostty_cursor_family(&cursor_state);
        ghostty_cursor_divider = resolved_ghostty_cursor_divider(&cursor_state);
        ghostty_cursor_primary_color_hex = resolved_ghostty_cursor_primary_color_hex(&cursor_state);
        ghostty_cursor_secondary_color_hex =
            resolved_ghostty_cursor_secondary_color_hex(&cursor_state);
    }

    let rerolled_ghostty_cursor = plan.should_reroll_ghostty_cursor
        || (plan.should_generate_terminal_configs
            && plan
                .selected_terminals
                .iter()
                .any(|terminal| terminal == "ghostty")
            && ghostty_random_requested);

    Ok(LaunchMaterializationData {
        terminal_config_mode: plan.terminal_config_mode,
        selected_terminals: plan.selected_terminals,
        generated_terminals,
        ghostty_cursor_name,
        ghostty_cursor_color_hex,
        ghostty_cursor_family,
        ghostty_cursor_divider,
        ghostty_cursor_primary_color_hex,
        ghostty_cursor_secondary_color_hex,
        rerolled_ghostty_cursor,
    })
}

fn plan_uses_yazelix_ghostty_cursor(plan: &LaunchMaterializationPlan) -> bool {
    plan.terminal_config_mode == "yazelix"
        && plan
            .selected_terminals
            .iter()
            .any(|terminal| terminal_uses_yazelix_cursor(terminal))
}

fn terminal_uses_yazelix_cursor(terminal: &str) -> bool {
    matches!(terminal, "ghostty" | "yzxterm")
}

fn resolved_ghostty_cursor_name(state: &ResolvedCursorRegistryState) -> Option<String> {
    if state.trail_disabled {
        Some("none".to_string())
    } else {
        state
            .selected_cursor
            .as_ref()
            .map(|cursor| cursor.name.clone())
    }
}

fn resolved_ghostty_cursor_color_hex(state: &ResolvedCursorRegistryState) -> Option<String> {
    state
        .selected_cursor
        .as_ref()
        .map(|cursor| cursor.cursor_color_hex().to_string())
}

fn resolved_ghostty_cursor_family(state: &ResolvedCursorRegistryState) -> Option<String> {
    state
        .selected_cursor
        .as_ref()
        .map(|cursor| cursor.family_name().to_string())
}

fn resolved_ghostty_cursor_divider(state: &ResolvedCursorRegistryState) -> Option<String> {
    state
        .selected_cursor
        .as_ref()
        .and_then(|cursor| cursor.divider_name())
        .map(str::to_string)
}

fn resolved_ghostty_cursor_primary_color_hex(
    state: &ResolvedCursorRegistryState,
) -> Option<String> {
    state
        .selected_cursor
        .as_ref()
        .and_then(|cursor| cursor.split_primary_color_hex())
        .map(str::to_string)
}

fn resolved_ghostty_cursor_secondary_color_hex(
    state: &ResolvedCursorRegistryState,
) -> Option<String> {
    state
        .selected_cursor
        .as_ref()
        .and_then(|cursor| cursor.split_secondary_color_hex())
        .map(str::to_string)
}

fn build_launch_materialization_plan(
    normalized: &JsonMap<String, JsonValue>,
    requested_terminals: &[String],
    desktop_fast_path: bool,
    force_terminal_config_generation: bool,
    state_dir: &Path,
    ghostty_random_requested: bool,
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

    let should_generate_terminal_configs = desktop_fast_path
        || force_terminal_config_generation
        || selected_terminals
            .iter()
            .any(|terminal| !generated_terminal_config_path(state_dir, terminal).is_file());
    let should_reroll_ghostty_cursor = !should_generate_terminal_configs
        && selected_terminals
            .iter()
            .any(|terminal| terminal_uses_yazelix_cursor(terminal))
        && ghostty_random_requested;

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
        "yzxterm" => root.join("yzxterm").join("config.toml"),
        "wezterm" => root.join("wezterm").join(".wezterm.lua"),
        "ratty" => root.join("ratty").join("ratty.toml"),
        "kitty" => root.join("kitty").join("kitty.conf"),
        _ => root.join(terminal),
    }
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
    ) -> JsonMap<String, JsonValue> {
        let mut config = JsonMap::new();
        config.insert("terminals".into(), json!(terminals));
        config.insert("terminal_config_mode".into(), json!(terminal_config_mode));
        config
    }

    // Regression: desktop fast-path launch regenerates existing Yazelix terminal configs before terminal handoff, because stale generated files can point at old runtime assets.
    #[test]
    fn desktop_fast_path_regenerates_existing_terminal_configs() {
        let temp = tempdir().unwrap();
        let ghostty_dir = temp
            .path()
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty");
        let wezterm_dir = temp
            .path()
            .join("configs")
            .join("terminal_emulators")
            .join("wezterm");
        std::fs::create_dir_all(&ghostty_dir).unwrap();
        std::fs::create_dir_all(&wezterm_dir).unwrap();
        std::fs::write(ghostty_dir.join("config"), "existing").unwrap();
        std::fs::write(wezterm_dir.join(".wezterm.lua"), "existing").unwrap();
        let config = config_with_terminals(&["ghostty", "wezterm"], "yazelix");

        let plan = build_launch_materialization_plan(
            &config,
            &["ghostty".to_string(), "wezterm".to_string()],
            true,
            false,
            temp.path(),
            true,
        );

        assert_eq!(
            plan,
            LaunchMaterializationPlan {
                terminal_config_mode: "yazelix".to_string(),
                selected_terminals: vec!["ghostty".to_string(), "wezterm".to_string()],
                should_generate_terminal_configs: true,
                should_reroll_ghostty_cursor: false,
            }
        );
    }

    // Regression: desktop first launch after a package/runtime update must refresh existing terminal configs before opening the terminal, because stale configs can still point at old store assets.
    #[test]
    fn desktop_fast_path_forces_terminal_generation_when_inputs_changed() {
        let temp = tempdir().unwrap();
        let ghostty_dir = temp
            .path()
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty");
        std::fs::create_dir_all(&ghostty_dir).unwrap();
        std::fs::write(ghostty_dir.join("config"), "stale existing config").unwrap();
        let config = config_with_terminals(&["ghostty"], "yazelix");

        let plan = build_launch_materialization_plan(
            &config,
            &["ghostty".to_string()],
            true,
            true,
            temp.path(),
            true,
        );

        assert_eq!(
            plan,
            LaunchMaterializationPlan {
                terminal_config_mode: "yazelix".to_string(),
                selected_terminals: vec!["ghostty".to_string()],
                should_generate_terminal_configs: true,
                should_reroll_ghostty_cursor: false,
            }
        );
    }

    // Defends: non-desktop launch materialization still regenerates the configured terminal set even after the request-assembly cut.
    #[test]
    fn full_launch_materialization_uses_configured_terminals_when_callers_do_not_override_them() {
        let temp = tempdir().unwrap();
        let config = config_with_terminals(&["ghostty", "wezterm", ""], "user");

        let plan =
            build_launch_materialization_plan(&config, &[], false, false, temp.path(), false);

        assert_eq!(
            plan,
            LaunchMaterializationPlan {
                terminal_config_mode: "user".to_string(),
                selected_terminals: vec!["ghostty".to_string(), "wezterm".to_string()],
                should_generate_terminal_configs: true,
                should_reroll_ghostty_cursor: false,
            }
        );
        assert!(!plan_uses_yazelix_ghostty_cursor(&plan));
    }

    // Defends: missing terminal config materializes Ghostty first so first-run launches use the cursor-trail runtime identity.
    #[test]
    fn full_launch_materialization_defaults_to_ghostty_then_wezterm() {
        let temp = tempdir().unwrap();
        let config = JsonMap::new();

        let plan =
            build_launch_materialization_plan(&config, &[], false, false, temp.path(), false);

        assert_eq!(
            plan.selected_terminals,
            vec!["ghostty".to_string(), "wezterm".to_string()]
        );
        assert!(plan_uses_yazelix_ghostty_cursor(&plan));
    }

    // Regression: yzxterm consumes the same Yazelix cursor materialization facts as Ghostty.
    #[test]
    fn yzxterm_launch_materialization_uses_yazelix_cursor() {
        let temp = tempdir().unwrap();
        let config = config_with_terminals(&["yzxterm"], "yazelix");

        let plan =
            build_launch_materialization_plan(&config, &[], false, false, temp.path(), false);

        assert_eq!(plan.selected_terminals, vec!["yzxterm".to_string()]);
        assert!(plan_uses_yazelix_ghostty_cursor(&plan));
    }
}
