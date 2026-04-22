// Test lane: default
//! Rust-owned transient pane facts for popup and menu callers.

use crate::bridge::CoreError;
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, load_normalized_config_for_control,
    runtime_dir_from_env,
};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TransientPaneFactsData {
    pub popup_program: Vec<String>,
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
}

pub fn compute_transient_pane_facts_from_env() -> Result<TransientPaneFactsData, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let normalized =
        load_normalized_config_for_control(&runtime_dir, &config_dir, config_override.as_deref())?;

    Ok(TransientPaneFactsData {
        popup_program: string_list_config(&normalized, "popup_program", &["lazygit"]),
        popup_width_percent: int_config(&normalized, "popup_width_percent", 90),
        popup_height_percent: int_config(&normalized, "popup_height_percent", 90),
    })
}

fn int_config(config: &JsonMap<String, JsonValue>, key: &str, default: i64) -> i64 {
    config
        .get(key)
        .and_then(|value| match value {
            JsonValue::Number(number) => number.as_i64(),
            JsonValue::String(raw) => raw.trim().parse::<i64>().ok(),
            _ => None,
        })
        .unwrap_or(default)
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

    // Defends: transient pane facts preserve popup defaults and geometry without Nushell-side config parsing.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
    #[test]
    fn transient_pane_facts_defaults_and_filters_popup_program() {
        let mut config = JsonMap::new();
        assert_eq!(
            string_list_config(&config, "popup_program", &["lazygit"]),
            vec!["lazygit"]
        );
        assert_eq!(int_config(&config, "popup_width_percent", 90), 90);

        config.insert(
            "popup_program".into(),
            json!([" lazygit ", "", "gitui", 5, null]),
        );
        config.insert("popup_width_percent".into(), json!("82"));
        config.insert("popup_height_percent".into(), json!(76));

        assert_eq!(
            string_list_config(&config, "popup_program", &["lazygit"]),
            vec!["lazygit", "gitui"]
        );
        assert_eq!(int_config(&config, "popup_width_percent", 90), 82);
        assert_eq!(int_config(&config, "popup_height_percent", 90), 76);
    }
}
