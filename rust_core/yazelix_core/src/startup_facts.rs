// Test lane: default
//! Rust-owned startup/session config facts for shell-owned launch paths.

use crate::appearance_mode::APPEARANCE_MODE_DARK;
use crate::bridge::CoreError;
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, load_normalized_config_for_control,
    runtime_dir_from_env,
};
use crate::terminal_variant::active_terminal_from_runtime_dir;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};

const DEFAULT_SHELL: &str = "nu";
const DEFAULT_WELCOME_STYLE: &str = "random";
const DEFAULT_GAME_OF_LIFE_CELL_STYLE: &str = "full_block";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StartupFactsData {
    pub default_shell: String,
    pub debug_mode: bool,
    pub skip_welcome_screen: bool,
    pub welcome_style: String,
    pub game_of_life_cell_style: String,
    pub appearance_mode: String,
    pub welcome_duration_seconds: f64,
    pub show_macchina_on_welcome: bool,
    pub terminals: Vec<String>,
}

pub fn compute_startup_facts_from_env() -> Result<StartupFactsData, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let normalized =
        load_normalized_config_for_control(&runtime_dir, &config_dir, config_override.as_deref())?;

    compute_startup_facts_from_config(&runtime_dir, &normalized)
}

pub fn compute_startup_facts_from_config(
    runtime_dir: &std::path::Path,
    normalized: &JsonMap<String, JsonValue>,
) -> Result<StartupFactsData, CoreError> {
    Ok(StartupFactsData {
        default_shell: string_config(normalized, "default_shell", DEFAULT_SHELL),
        debug_mode: bool_config(normalized, "debug_mode", false),
        skip_welcome_screen: bool_config(normalized, "skip_welcome_screen", false),
        welcome_style: string_config(normalized, "welcome_style", DEFAULT_WELCOME_STYLE),
        game_of_life_cell_style: string_config(
            normalized,
            "game_of_life_cell_style",
            DEFAULT_GAME_OF_LIFE_CELL_STYLE,
        ),
        appearance_mode: string_config(normalized, "appearance_mode", APPEARANCE_MODE_DARK),
        welcome_duration_seconds: float_config(normalized, "welcome_duration_seconds", 4.0),
        show_macchina_on_welcome: bool_config(normalized, "show_macchina_on_welcome", false),
        terminals: vec![active_terminal_from_runtime_dir(runtime_dir)?],
    })
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

fn bool_config(config: &JsonMap<String, JsonValue>, key: &str, default: bool) -> bool {
    config
        .get(key)
        .and_then(|value| match value {
            JsonValue::Bool(value) => Some(*value),
            JsonValue::String(raw) => match raw.trim().to_ascii_lowercase().as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or(default)
}

fn float_config(config: &JsonMap<String, JsonValue>, key: &str, default: f64) -> f64 {
    config
        .get(key)
        .and_then(|value| match value {
            JsonValue::Number(number) => number.as_f64(),
            JsonValue::String(raw) => raw.trim().parse::<f64>().ok(),
            _ => None,
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Defends: startup facts coerce retained session and welcome settings out of normalized config shapes.
    #[test]
    fn startup_fact_helpers_coerce_strings_bools_numbers_and_lists() {
        let mut config = JsonMap::new();
        config.insert("default_shell".into(), json!("bash"));
        config.insert("debug_mode".into(), json!(true));
        config.insert("skip_welcome_screen".into(), json!("true"));
        config.insert("welcome_style".into(), json!("minimal"));
        config.insert("game_of_life_cell_style".into(), json!("dotted"));
        config.insert("appearance_mode".into(), json!("light"));
        config.insert("welcome_duration_seconds".into(), json!("2.5"));
        config.insert("show_macchina_on_welcome".into(), json!("false"));

        assert_eq!(
            string_config(&config, "default_shell", DEFAULT_SHELL),
            "bash"
        );
        assert!(bool_config(&config, "debug_mode", false));
        assert!(bool_config(&config, "skip_welcome_screen", false));
        assert_eq!(float_config(&config, "welcome_duration_seconds", 1.0), 2.5);
        assert!(!bool_config(&config, "show_macchina_on_welcome", true));
        assert_eq!(
            string_config(
                &config,
                "game_of_life_cell_style",
                DEFAULT_GAME_OF_LIFE_CELL_STYLE
            ),
            "dotted"
        );
        assert_eq!(
            string_config(&config, "appearance_mode", APPEARANCE_MODE_DARK),
            "light"
        );
    }
}
