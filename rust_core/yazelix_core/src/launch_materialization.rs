// Test lane: default
//! Rust-owned launch-time terminal/ghostty materialization decisions for shell-owned launch paths.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::CoreError;
use crate::config_normalize::{normalize_config, NormalizeConfigRequest};
use crate::control_plane::{config_dir_from_env, runtime_dir_from_env, state_dir_from_env};
use crate::ghostty_cursor_registry::{
    CursorRegistry, ResolvedCursorRegistryState, YazelixCursorRegistryExt,
};
use crate::ghostty_materialization::{
    generate_ghostty_materialization, GhosttyMaterializationRequest,
};
use crate::runtime_component_enabled;
use crate::terminal_materialization::{
    generate_terminal_materialization, yzxterm_profile_from_env, TerminalGeneratedConfig,
    TerminalMaterializationRequest, YzxtermProfile,
};
use crate::terminal_variant::active_terminal_from_runtime_dir;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_TERMINAL_CONFIG_MODE: &str = "yazelix";
static LAUNCH_SCOPED_TERMINAL_STATE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct LaunchMaterializationRequest {
    pub config_path: PathBuf,
    pub cursor_config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub active_terminal: String,
    pub desktop_fast_path: bool,
    pub force_terminal_config_generation: bool,
    pub yzxterm_profile: YzxtermProfile,
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
    desktop_fast_path: bool,
    force_terminal_config_generation: bool,
    config_override: Option<&str>,
) -> Result<LaunchMaterializationRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = resolve_active_config_paths(&runtime_dir, &config_dir, config_override)?;
    let state_dir = state_dir_from_env()?;
    let active_terminal = active_terminal_from_runtime_dir(&runtime_dir)?;

    Ok(LaunchMaterializationRequest {
        config_path: paths.config_file,
        cursor_config_path: paths.user_cursor_config,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        runtime_dir,
        config_dir,
        state_dir,
        active_terminal,
        desktop_fast_path,
        force_terminal_config_generation,
        yzxterm_profile: yzxterm_profile_from_env()?,
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
        &request.active_terminal,
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
        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            &plan,
            &request.state_dir,
            ghostty_random_requested,
            request.yzxterm_profile,
        );
        let terminal_data = generate_terminal_materialization(&TerminalMaterializationRequest {
            config_path: request.config_path.clone(),
            cursor_config_path: request.cursor_config_path.clone(),
            default_config_path: request.default_config_path.clone(),
            contract_path: request.contract_path.clone(),
            runtime_dir: request.runtime_dir.clone(),
            state_dir: terminal_state_dir,
            terminals: plan.selected_terminals.clone(),
            yzxterm_profile: request.yzxterm_profile,
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

    let rerolled_ghostty_cursor = launch_rerolled_yazelix_cursor(&plan, ghostty_random_requested);

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

fn launch_rerolled_yazelix_cursor(
    plan: &LaunchMaterializationPlan,
    ghostty_random_requested: bool,
) -> bool {
    plan.should_reroll_ghostty_cursor
        || (plan.should_generate_terminal_configs
            && plan
                .selected_terminals
                .iter()
                .any(|terminal| terminal_uses_yazelix_cursor(terminal))
            && ghostty_random_requested)
}

fn terminal_materialization_state_dir_for_launch(
    plan: &LaunchMaterializationPlan,
    state_dir: &Path,
    ghostty_random_requested: bool,
    yzxterm_profile: YzxtermProfile,
) -> PathBuf {
    if plan.should_generate_terminal_configs
        && plan_uses_yazelix_ghostty_cursor(plan)
        && (ghostty_random_requested || plan_uses_yzxterm_shader_profile(plan, yzxterm_profile))
    {
        return launch_scoped_terminal_state_dir(state_dir);
    }

    state_dir.to_path_buf()
}

fn plan_uses_yzxterm_shader_profile(
    plan: &LaunchMaterializationPlan,
    yzxterm_profile: YzxtermProfile,
) -> bool {
    yzxterm_profile == YzxtermProfile::Shaders
        && plan
            .selected_terminals
            .iter()
            .any(|terminal| terminal == "yzxterm")
}

fn launch_scoped_terminal_state_dir(state_dir: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let sequence = LAUNCH_SCOPED_TERMINAL_STATE_COUNTER.fetch_add(1, Ordering::Relaxed);

    state_dir
        .join("terminal_launches")
        .join(format!("{}-{nanos}-{sequence}", std::process::id()))
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
    active_terminal: &str,
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
    let selected_terminals = vec![active_terminal.to_string()];

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn config_with_mode(terminal_config_mode: &str) -> JsonMap<String, JsonValue> {
        let mut config = JsonMap::new();
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
        std::fs::create_dir_all(&ghostty_dir).unwrap();
        std::fs::write(ghostty_dir.join("config"), "existing").unwrap();
        let config = config_with_mode("yazelix");

        let plan =
            build_launch_materialization_plan(&config, "ghostty", true, false, temp.path(), true);

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
        let config = config_with_mode("yazelix");

        let plan =
            build_launch_materialization_plan(&config, "ghostty", true, true, temp.path(), true);

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

    // Defends: non-desktop launch materialization regenerates only the caller-provided active terminal.
    #[test]
    fn full_launch_materialization_uses_active_terminal_from_request() {
        let temp = tempdir().unwrap();
        let config = config_with_mode("user");

        let plan =
            build_launch_materialization_plan(&config, "ghostty", false, false, temp.path(), false);

        assert_eq!(
            plan,
            LaunchMaterializationPlan {
                terminal_config_mode: "user".to_string(),
                selected_terminals: vec!["ghostty".to_string()],
                should_generate_terminal_configs: true,
                should_reroll_ghostty_cursor: false,
            }
        );
        assert!(!plan_uses_yazelix_ghostty_cursor(&plan));
    }

    // Regression: yzxterm consumes the same Yazelix cursor materialization facts as Ghostty.
    #[test]
    fn yzxterm_launch_materialization_uses_yazelix_cursor() {
        let temp = tempdir().unwrap();
        let config = config_with_mode("yazelix");

        let plan =
            build_launch_materialization_plan(&config, "yzxterm", false, false, temp.path(), false);

        assert_eq!(plan.selected_terminals, vec!["yzxterm".to_string()]);
        assert!(plan_uses_yazelix_ghostty_cursor(&plan));
    }

    // Regression: yzxterm random cursor launches reroll the same Yazelix cursor state as Ghostty, so launch facts and verbose reporting stay accurate.
    #[test]
    fn yzxterm_random_cursor_launch_reports_reroll() {
        let temp = tempdir().unwrap();
        let config = config_with_mode("yazelix");
        let plan =
            build_launch_materialization_plan(&config, "yzxterm", false, false, temp.path(), true);

        assert!(launch_rerolled_yazelix_cursor(&plan, true));
    }

    // Regression: random cursor launches use a launch-scoped terminal config root so opening a new window cannot rewrite config watched by existing Ghostty/yzxterm windows.
    #[test]
    fn random_cursor_launch_uses_scoped_terminal_state_dir() {
        let temp = tempdir().unwrap();
        let config = config_with_mode("yazelix");
        let plan =
            build_launch_materialization_plan(&config, "ghostty", false, false, temp.path(), true);

        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            &plan,
            temp.path(),
            true,
            YzxtermProfile::Full,
        );

        assert_ne!(terminal_state_dir, temp.path());
        assert!(terminal_state_dir.starts_with(temp.path().join("terminal_launches")));
    }

    // Defends: named cursor launches keep using the stable generated state root, avoiding unnecessary per-window config churn.
    #[test]
    fn named_cursor_launch_uses_stable_terminal_state_dir() {
        let temp = tempdir().unwrap();
        let config = config_with_mode("yazelix");
        let plan =
            build_launch_materialization_plan(&config, "ghostty", false, false, temp.path(), false);

        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            &plan,
            temp.path(),
            false,
            YzxtermProfile::Full,
        );

        assert_eq!(terminal_state_dir, temp.path());
    }

    // Regression: yzxterm shader profile uses launch-scoped shader/config snapshots even with a named cursor, so opening another yzxterm window cannot rewrite GLSL files used by an existing one.
    #[test]
    fn yzxterm_shader_profile_uses_scoped_terminal_state_dir() {
        let temp = tempdir().unwrap();
        let config = config_with_mode("yazelix");
        let plan =
            build_launch_materialization_plan(&config, "yzxterm", false, false, temp.path(), false);

        let first = terminal_materialization_state_dir_for_launch(
            &plan,
            temp.path(),
            false,
            YzxtermProfile::Shaders,
        );
        let second = terminal_materialization_state_dir_for_launch(
            &plan,
            temp.path(),
            false,
            YzxtermProfile::Shaders,
        );

        assert_ne!(first, temp.path());
        assert_ne!(second, temp.path());
        assert_ne!(first, second);
        assert!(first.starts_with(temp.path().join("terminal_launches")));
        assert!(second.starts_with(temp.path().join("terminal_launches")));
    }

    // Defends: yzxterm profiles that do not load custom cursor shaders keep using the stable generated config root.
    #[test]
    fn yzxterm_without_shader_profile_uses_stable_terminal_state_dir() {
        let temp = tempdir().unwrap();
        let config = config_with_mode("yazelix");
        let plan =
            build_launch_materialization_plan(&config, "yzxterm", false, false, temp.path(), false);

        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            &plan,
            temp.path(),
            false,
            YzxtermProfile::Full,
        );

        assert_eq!(terminal_state_dir, temp.path());
    }
}
