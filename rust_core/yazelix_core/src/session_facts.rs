// Test lane: default
//! Cached current-session facts for panes that must survive config version skew.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, config_override_from_env, runtime_dir_from_env};
use crate::helix_external::HelixExternalPair;
use crate::session_config_snapshot::{
    load_session_facts_from_snapshot_path, session_config_snapshot_path_from_env,
};
use crate::settings_surface::read_config_table;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

const SESSION_FACTS_SCHEMA_VERSION: u64 = 1;
const SESSION_FACTS_PATH_ENV: &str = "YAZELIX_SESSION_FACTS_PATH";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionFactsData {
    pub hide_sidebar_on_file_open: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub helix_external: Option<HelixExternalPair>,
    pub yazi_command: String,
    pub ya_command: String,
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
    pub game_of_life_cell_style: String,
    pub default_shell: String,
    pub terminals: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionFactsCache {
    schema_version: u64,
    #[serde(rename = "source_config_file")]
    _source_config_file: String,
    #[serde(rename = "normalized_config")]
    _normalized_config: JsonMap<String, JsonValue>,
    facts: SessionFactsData,
}

impl Default for SessionFactsData {
    fn default() -> Self {
        Self {
            hide_sidebar_on_file_open: false,
            editor_command: None,
            helix_external: None,
            yazi_command: "yazi".to_string(),
            ya_command: "ya".to_string(),
            popup_width_percent: 90,
            popup_height_percent: 90,
            game_of_life_cell_style: "full_block".to_string(),
            default_shell: "nu".to_string(),
            terminals: vec!["ghostty".to_string()],
        }
    }
}

impl SessionFactsData {
    pub(crate) fn from_normalized_config(config: &JsonMap<String, JsonValue>) -> Self {
        let defaults = Self::default();
        Self {
            hide_sidebar_on_file_open: config
                .get("hide_sidebar_on_file_open")
                .and_then(JsonValue::as_bool)
                .unwrap_or(defaults.hide_sidebar_on_file_open),
            editor_command: normalized_string(config, "editor_command"),
            helix_external: normalized_helix_external(config),
            yazi_command: normalized_string(config, "yazi_command")
                .unwrap_or(defaults.yazi_command),
            ya_command: normalized_string(config, "yazi_ya_command").unwrap_or(defaults.ya_command),
            popup_width_percent: normalized_i64(config, "popup_width_percent")
                .filter(|value| (1..=100).contains(value))
                .unwrap_or(defaults.popup_width_percent),
            popup_height_percent: normalized_i64(config, "popup_height_percent")
                .filter(|value| (1..=100).contains(value))
                .unwrap_or(defaults.popup_height_percent),
            game_of_life_cell_style: normalized_string(config, "game_of_life_cell_style")
                .unwrap_or(defaults.game_of_life_cell_style),
            default_shell: normalized_string(config, "default_shell")
                .unwrap_or(defaults.default_shell),
            terminals: normalized_string_list(config, "terminals")
                .filter(|items| !items.is_empty())
                .unwrap_or(defaults.terminals),
        }
        .sanitized()
    }

    fn sanitized(mut self) -> Self {
        let defaults = Self::default();
        if self.yazi_command.trim().is_empty() {
            self.yazi_command = defaults.yazi_command;
        }
        if self.ya_command.trim().is_empty() {
            self.ya_command = defaults.ya_command;
        }
        if !(1..=100).contains(&self.popup_width_percent) {
            self.popup_width_percent = defaults.popup_width_percent;
        }
        if !(1..=100).contains(&self.popup_height_percent) {
            self.popup_height_percent = defaults.popup_height_percent;
        }
        if self.game_of_life_cell_style.trim().is_empty() {
            self.game_of_life_cell_style = defaults.game_of_life_cell_style;
        }
        if self.default_shell.trim().is_empty() {
            self.default_shell = defaults.default_shell;
        }
        self.terminals = non_empty_strings(self.terminals);
        if self.terminals.is_empty() {
            self.terminals = defaults.terminals;
        }
        self.editor_command = self
            .editor_command
            .and_then(|value| non_empty_string(value.as_str()));
        self.helix_external = self.helix_external.and_then(|external| {
            HelixExternalPair::normalized(&external.binary, &external.runtime_path)
        });
        self
    }
}

pub fn compute_session_facts_from_env() -> Result<SessionFactsData, CoreError> {
    if let Some(path) = session_config_snapshot_path_from_env() {
        return load_session_facts_from_snapshot_path(&path);
    }

    if let Some(path) = session_facts_cache_path_from_env() {
        return load_legacy_session_facts_cache_from_path(&path);
    }

    compute_lossy_session_facts_from_config()
}

fn session_facts_cache_path_from_env() -> Option<PathBuf> {
    std::env::var(SESSION_FACTS_PATH_ENV)
        .ok()
        .and_then(|value| non_empty_string(value.as_str()))
        .map(PathBuf::from)
}

fn load_legacy_session_facts_cache_from_path(path: &Path) -> Result<SessionFactsData, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "legacy_session_facts_read",
            "Could not read the legacy Yazelix session facts cache.",
            "Restart this Yazelix window so it inherits a launch-time config snapshot.",
            path.to_string_lossy(),
            source,
        )
    })?;
    let cache = serde_json::from_str::<SessionFactsCache>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "legacy_session_facts_parse",
            format!(
                "Could not parse the legacy Yazelix session facts cache {}: {source}",
                path.display()
            ),
            "Restart this Yazelix window so it inherits a launch-time config snapshot.",
            serde_json::json!({ "path": path.to_string_lossy() }),
        )
    })?;
    if cache.schema_version != SESSION_FACTS_SCHEMA_VERSION {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "legacy_session_facts_schema",
            format!(
                "Unsupported legacy Yazelix session facts schema {} at {}.",
                cache.schema_version,
                path.display()
            ),
            "Restart this Yazelix window so it inherits a launch-time config snapshot.",
            serde_json::json!({
                "path": path.to_string_lossy(),
                "expected_schema_version": SESSION_FACTS_SCHEMA_VERSION,
                "actual_schema_version": cache.schema_version,
            }),
        ));
    }
    Ok(cache.facts.sanitized())
}

fn compute_lossy_session_facts_from_config() -> Result<SessionFactsData, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let paths = resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;

    let mut facts = SessionFactsData::default();
    facts.apply_lossy_table(read_config_table_lossy(&paths.default_config_path).as_ref());

    if paths.config_file != paths.default_config_path {
        facts.apply_lossy_table(read_config_table_lossy(&paths.config_file).as_ref());
    }

    Ok(facts.sanitized())
}

impl SessionFactsData {
    fn apply_lossy_table(&mut self, config: Option<&toml::Table>) {
        let Some(config) = config else {
            return;
        };

        if let Some(shell) = toml_section(config, "shell") {
            if let Some(value) = toml_string(shell.get("program")) {
                self.default_shell = value;
            }
        }

        if let Some(editor) = toml_section(config, "editor") {
            self.editor_command = toml_optional_string(editor.get("command"));
        }
    }
}

fn normalized_string(config: &JsonMap<String, JsonValue>, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(JsonValue::as_str)
        .and_then(non_empty_string)
}

fn normalized_helix_external(config: &JsonMap<String, JsonValue>) -> Option<HelixExternalPair> {
    config
        .get("helix_external")
        .and_then(HelixExternalPair::from_json)
}

fn normalized_string_list(config: &JsonMap<String, JsonValue>, key: &str) -> Option<Vec<String>> {
    config.get(key).and_then(JsonValue::as_array).map(|items| {
        non_empty_strings(
            items
                .iter()
                .filter_map(JsonValue::as_str)
                .map(ToOwned::to_owned)
                .collect(),
        )
    })
}

fn normalized_i64(config: &JsonMap<String, JsonValue>, key: &str) -> Option<i64> {
    match config.get(key)? {
        JsonValue::Number(number) => number.as_i64(),
        JsonValue::String(raw) => raw.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn read_config_table_lossy(path: &Path) -> Option<toml::Table> {
    read_config_table(path, "read_session_facts_config").ok()
}

fn toml_section<'a>(config: &'a toml::Table, section: &str) -> Option<&'a toml::Table> {
    config.get(section).and_then(TomlValue::as_table)
}

fn toml_string(value: Option<&TomlValue>) -> Option<String> {
    value.and_then(TomlValue::as_str).and_then(non_empty_string)
}

fn toml_optional_string(value: Option<&TomlValue>) -> Option<String> {
    value.and_then(TomlValue::as_str).and_then(non_empty_string)
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn non_empty_strings(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| non_empty_string(value.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Defends: generated session facts preserve the pane/runtime values used after the user config has moved on.
    #[test]
    fn session_facts_from_normalized_config_keeps_current_session_values() {
        let config = JsonMap::from_iter([
            ("editor_command".to_string(), json!("nvim")),
            ("hide_sidebar_on_file_open".to_string(), json!(true)),
            ("yazi_command".to_string(), json!("yy")),
            ("yazi_ya_command".to_string(), json!("ya-test")),
            ("popup_width_percent".to_string(), json!(82)),
            ("popup_height_percent".to_string(), json!("76")),
            ("game_of_life_cell_style".to_string(), json!("dotted")),
            ("default_shell".to_string(), json!("bash")),
            ("terminals".to_string(), json!(["ghostty", "wezterm"])),
        ]);

        let facts = SessionFactsData::from_normalized_config(&config);

        assert_eq!(facts.editor_command.as_deref(), Some("nvim"));
        assert!(facts.hide_sidebar_on_file_open);
        assert_eq!(facts.yazi_command, "yy");
        assert_eq!(facts.ya_command, "ya-test");
        assert_eq!(facts.popup_width_percent, 82);
        assert_eq!(facts.popup_height_percent, 76);
        assert_eq!(facts.game_of_life_cell_style, "dotted");
        assert_eq!(facts.default_shell, "bash");
        assert_eq!(facts.terminals, vec!["ghostty", "wezterm"]);
    }
}
