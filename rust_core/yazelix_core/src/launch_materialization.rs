// Test lane: default
//! Rust-owned launch-time terminal/ghostty materialization decisions for shell-owned launch paths.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::CoreError;
use crate::control_plane::{config_dir_from_env, runtime_dir_from_env, state_dir_from_env};
use crate::ghostty_cursor_registry::{CursorRegistry, YazelixCursorRegistryExt};
use crate::runtime_component_enabled;
use crate::startup_facts::DEFAULT_TERMINAL_CONFIG_MODE;
use crate::terminal_materialization::{
    MarsEmojiFont, MarsProfile, TerminalGeneratedConfig, TerminalMaterializationRequest,
    generate_terminal_materialization, mars_emoji_font_override_from_env, mars_profile_from_env,
    terminal_has_generated_config,
};
use crate::terminal_variant::active_terminal_from_runtime_dir;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static LAUNCH_SCOPED_TERMINAL_STATE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct LaunchMaterializationRequest {
    pub config_path: PathBuf,
    pub cursor_config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub state_dir: PathBuf,
    pub active_terminal: String,
    pub desktop_fast_path: bool,
    pub mars_profile: MarsProfile,
    pub mars_emoji_font: Option<MarsEmojiFont>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LaunchMaterializationData {
    pub terminal_config_mode: String,
    pub generated_terminals: Vec<TerminalGeneratedConfig>,
    pub rerolled_ghostty_cursor: bool,
}

pub fn launch_materialization_request_from_env(
    desktop_fast_path: bool,
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
        state_dir,
        active_terminal,
        desktop_fast_path,
        mars_emoji_font: mars_emoji_font_override_from_env()?,
        mars_profile: mars_profile_from_env()?,
    })
}

pub fn prepare_launch_materialization(
    request: &LaunchMaterializationRequest,
    normalized: &JsonMap<String, JsonValue>,
) -> Result<LaunchMaterializationData, CoreError> {
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
    let terminal_config_mode = string_config(
        normalized,
        "terminal_config_mode",
        DEFAULT_TERMINAL_CONFIG_MODE,
    );
    let generate_terminal_configs = should_materialize_terminal_config(
        &terminal_config_mode,
        request.desktop_fast_path,
        &request.active_terminal,
    );

    let mut generated_terminals = Vec::new();
    if generate_terminal_configs {
        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            &terminal_config_mode,
            &request.active_terminal,
            &request.state_dir,
            ghostty_random_requested,
            request.mars_profile,
        );
        let terminal_data = generate_terminal_materialization(&TerminalMaterializationRequest {
            config_path: request.config_path.clone(),
            cursor_config_path: request.cursor_config_path.clone(),
            default_config_path: request.default_config_path.clone(),
            contract_path: request.contract_path.clone(),
            runtime_dir: request.runtime_dir.clone(),
            state_dir: terminal_state_dir,
            terminals: vec![request.active_terminal.clone()],
            mars_emoji_font: request.mars_emoji_font,
            mars_profile: request.mars_profile,
        })?;
        generated_terminals = terminal_data.generated;
    }

    let rerolled_ghostty_cursor = launch_rerolled_yazelix_cursor(
        generate_terminal_configs,
        &request.active_terminal,
        ghostty_random_requested,
    );

    Ok(LaunchMaterializationData {
        terminal_config_mode,
        generated_terminals,
        rerolled_ghostty_cursor,
    })
}

fn uses_yazelix_ghostty_cursor(terminal_config_mode: &str, terminal: &str) -> bool {
    terminal_config_mode == "yazelix" && terminal_uses_yazelix_cursor(terminal)
}

fn terminal_uses_yazelix_cursor(terminal: &str) -> bool {
    yazelix_terminal_support::terminal_support().uses_yazelix_cursor(terminal)
}

fn launch_rerolled_yazelix_cursor(
    should_generate_terminal_configs: bool,
    terminal: &str,
    ghostty_random_requested: bool,
) -> bool {
    should_generate_terminal_configs
        && terminal_uses_yazelix_cursor(terminal)
        && ghostty_random_requested
}

fn terminal_materialization_state_dir_for_launch(
    terminal_config_mode: &str,
    terminal: &str,
    state_dir: &Path,
    ghostty_random_requested: bool,
    mars_profile: MarsProfile,
) -> PathBuf {
    if uses_yazelix_ghostty_cursor(terminal_config_mode, terminal)
        && (ghostty_random_requested || uses_mars_shader_profile(terminal, mars_profile))
    {
        return launch_scoped_terminal_state_dir(state_dir);
    }

    state_dir.to_path_buf()
}

fn uses_mars_shader_profile(terminal: &str, mars_profile: MarsProfile) -> bool {
    mars_profile == MarsProfile::Shaders && terminal == "mars"
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

fn should_generate_terminal_configs(terminal_config_mode: &str, desktop_fast_path: bool) -> bool {
    !desktop_fast_path || terminal_config_mode == "yazelix"
}

fn should_materialize_terminal_config(
    terminal_config_mode: &str,
    desktop_fast_path: bool,
    terminal: &str,
) -> bool {
    should_generate_terminal_configs(terminal_config_mode, desktop_fast_path)
        && terminal_has_generated_config(terminal)
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
        let config = config_with_mode("yazelix");
        let mode = string_config(
            &config,
            "terminal_config_mode",
            DEFAULT_TERMINAL_CONFIG_MODE,
        );

        assert!(should_generate_terminal_configs(&mode, true));
    }

    // Regression: the packaged Kitty default must not enter the retained
    // Mars-only terminal materializer before spawning the Kitty process.
    #[test]
    fn kitty_desktop_launch_skips_mars_only_terminal_materialization() {
        let config = config_with_mode("yazelix");
        let mode = string_config(
            &config,
            "terminal_config_mode",
            DEFAULT_TERMINAL_CONFIG_MODE,
        );

        assert!(!should_materialize_terminal_config(&mode, true, "kitty"));
        assert!(should_materialize_terminal_config(&mode, true, "mars"));
    }

    // Defends: non-desktop launch materialization regenerates only the caller-provided active terminal.
    #[test]
    fn full_launch_materialization_uses_active_terminal_from_request() {
        let config = config_with_mode("user");
        let mode = string_config(
            &config,
            "terminal_config_mode",
            DEFAULT_TERMINAL_CONFIG_MODE,
        );

        assert_eq!(mode, "user");
        assert!(should_generate_terminal_configs(&mode, false));
        assert!(!uses_yazelix_ghostty_cursor(&mode, "ghostty"));
    }

    // Regression: mars consumes the same Yazelix cursor materialization facts as Ghostty.
    #[test]
    fn mars_launch_materialization_uses_yazelix_cursor() {
        assert!(uses_yazelix_ghostty_cursor("yazelix", "mars"));
    }

    // Regression: mars random cursor launches reroll the same Yazelix cursor state as Ghostty, so launch facts and verbose reporting stay accurate.
    #[test]
    fn mars_random_cursor_launch_reports_reroll() {
        assert!(launch_rerolled_yazelix_cursor(true, "mars", true));
    }

    // Regression: random cursor launches use a launch-scoped terminal config root so opening a new window cannot rewrite config watched by existing Ghostty/mars windows.
    #[test]
    fn random_cursor_launch_uses_scoped_terminal_state_dir() {
        let temp = tempdir().unwrap();

        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            "yazelix",
            "ghostty",
            temp.path(),
            true,
            MarsProfile::Full,
        );

        assert_ne!(terminal_state_dir, temp.path());
        assert!(terminal_state_dir.starts_with(temp.path().join("terminal_launches")));
    }

    // Defends: named cursor launches keep using the stable generated state root, avoiding unnecessary per-window config churn.
    #[test]
    fn named_cursor_launch_uses_stable_terminal_state_dir() {
        let temp = tempdir().unwrap();

        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            "yazelix",
            "ghostty",
            temp.path(),
            false,
            MarsProfile::Full,
        );

        assert_eq!(terminal_state_dir, temp.path());
    }

    // Regression: mars shader profile uses launch-scoped shader/config snapshots even with a named cursor, so opening another mars window cannot rewrite GLSL files used by an existing one.
    #[test]
    fn mars_shader_profile_uses_scoped_terminal_state_dir() {
        let temp = tempdir().unwrap();

        let first = terminal_materialization_state_dir_for_launch(
            "yazelix",
            "mars",
            temp.path(),
            false,
            MarsProfile::Shaders,
        );
        let second = terminal_materialization_state_dir_for_launch(
            "yazelix",
            "mars",
            temp.path(),
            false,
            MarsProfile::Shaders,
        );

        assert_ne!(first, temp.path());
        assert_ne!(second, temp.path());
        assert_ne!(first, second);
        assert!(first.starts_with(temp.path().join("terminal_launches")));
        assert!(second.starts_with(temp.path().join("terminal_launches")));
    }

    // Defends: mars profiles that do not load custom cursor shaders keep using the stable generated config root.
    #[test]
    fn mars_without_shader_profile_uses_stable_terminal_state_dir() {
        let temp = tempdir().unwrap();

        let terminal_state_dir = terminal_materialization_state_dir_for_launch(
            "yazelix",
            "mars",
            temp.path(),
            false,
            MarsProfile::Full,
        );

        assert_eq!(terminal_state_dir, temp.path());
    }
}
