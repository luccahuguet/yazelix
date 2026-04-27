// Test lane: default
//! Rust-owned transient pane facts for popup and menu callers.

use crate::active_config_surface::primary_config_paths;
use crate::bridge::CoreError;
use crate::control_plane::{config_dir_from_env, config_override_from_env, runtime_dir_from_env};
use serde::Serialize;
use std::fs;

const DEFAULT_POPUP_PROGRAM: &[&str] = &["lazygit"];
const DEFAULT_POPUP_WIDTH_PERCENT: i64 = 90;
const DEFAULT_POPUP_HEIGHT_PERCENT: i64 = 90;

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
    let paths = primary_config_paths(&runtime_dir, &config_dir);

    let mut facts = read_transient_pane_facts_lossy(&paths.default_config_path);
    let active_config_path = config_override
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            if paths.user_config.exists() {
                paths.user_config.clone()
            } else {
                paths.default_config_path.clone()
            }
        });

    if active_config_path != paths.default_config_path {
        facts.apply(read_transient_pane_fact_overrides_lossy(
            &active_config_path,
        ));
    }

    Ok(facts)
}

impl Default for TransientPaneFactsData {
    fn default() -> Self {
        Self {
            popup_program: DEFAULT_POPUP_PROGRAM
                .iter()
                .map(|item| item.to_string())
                .collect(),
            popup_width_percent: DEFAULT_POPUP_WIDTH_PERCENT,
            popup_height_percent: DEFAULT_POPUP_HEIGHT_PERCENT,
        }
    }
}

impl TransientPaneFactsData {
    fn apply(&mut self, overrides: TransientPaneFactOverrides) {
        if let Some(popup_program) = overrides.popup_program {
            self.popup_program = popup_program;
        }
        if let Some(width) = overrides.popup_width_percent {
            self.popup_width_percent = width;
        }
        if let Some(height) = overrides.popup_height_percent {
            self.popup_height_percent = height;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TransientPaneFactOverrides {
    popup_program: Option<Vec<String>>,
    popup_width_percent: Option<i64>,
    popup_height_percent: Option<i64>,
}

fn read_transient_pane_facts_lossy(path: &std::path::Path) -> TransientPaneFactsData {
    let mut facts = TransientPaneFactsData::default();
    facts.apply(read_transient_pane_fact_overrides_lossy(path));
    facts
}

fn read_transient_pane_fact_overrides_lossy(path: &std::path::Path) -> TransientPaneFactOverrides {
    let Ok(raw) = fs::read_to_string(path) else {
        return TransientPaneFactOverrides::default();
    };
    let Ok(config) = toml::from_str::<toml::Table>(&raw) else {
        return TransientPaneFactOverrides::default();
    };
    transient_pane_fact_overrides_from_config(&config)
}

fn transient_pane_fact_overrides_from_config(config: &toml::Table) -> TransientPaneFactOverrides {
    let Some(zellij) = config.get("zellij").and_then(toml::Value::as_table) else {
        return TransientPaneFactOverrides::default();
    };

    TransientPaneFactOverrides {
        popup_program: zellij.get("popup_program").and_then(toml_string_list),
        popup_width_percent: zellij.get("popup_width_percent").and_then(toml_percent),
        popup_height_percent: zellij.get("popup_height_percent").and_then(toml_percent),
    }
}

fn toml_string_list(value: &toml::Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items
            .iter()
            .filter_map(toml::Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    })
}

fn toml_percent(value: &toml::Value) -> Option<i64> {
    let parsed = match value {
        toml::Value::Integer(number) => Some(*number),
        toml::Value::String(raw) => raw.trim().parse::<i64>().ok(),
        _ => None,
    }?;
    (1..=100).contains(&parsed).then_some(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: transient pane facts preserve popup defaults and geometry without Nushell-side config parsing.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn transient_pane_facts_defaults_and_filters_popup_program() {
        let mut facts = TransientPaneFactsData::default();
        assert_eq!(facts.popup_program, vec!["lazygit"]);
        assert_eq!(facts.popup_width_percent, 90);

        let config: toml::Table = toml::from_str(
            r#"[zellij]
popup_program = [" lazygit ", "", "gitui", 5]
popup_width_percent = "82"
popup_height_percent = 76
"#,
        )
        .unwrap();
        facts.apply(transient_pane_fact_overrides_from_config(&config));

        assert_eq!(facts.popup_program, vec!["lazygit", "gitui"]);
        assert_eq!(facts.popup_width_percent, 82);
        assert_eq!(facts.popup_height_percent, 76);
    }
}
