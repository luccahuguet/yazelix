// Test lane: default
//! Cached current-session facts for panes that must survive config version skew.

use crate::active_config_surface::primary_config_paths;
use crate::bridge::CoreError;
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, runtime_dir_from_env, state_dir_from_env,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

const SESSION_FACTS_SCHEMA_VERSION: u64 = 1;
const SESSION_FACTS_FILE_NAME: &str = "session_facts.json";
const SESSION_FACTS_PATH_ENV: &str = "YAZELIX_SESSION_FACTS_PATH";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionFactsData {
    pub enable_sidebar: bool,
    pub initial_sidebar_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helix_runtime_path: Option<String>,
    pub yazi_command: String,
    pub ya_command: String,
    pub popup_program: Vec<String>,
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
    pub game_of_life_cell_style: String,
    pub default_shell: String,
    pub terminals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionFactsCache {
    schema_version: u64,
    source_config_file: String,
    normalized_config: JsonMap<String, JsonValue>,
    facts: SessionFactsData,
}

impl Default for SessionFactsData {
    fn default() -> Self {
        Self {
            enable_sidebar: true,
            initial_sidebar_state: "open".to_string(),
            editor_command: None,
            helix_runtime_path: None,
            yazi_command: "yazi".to_string(),
            ya_command: "ya".to_string(),
            popup_program: vec!["lazygit".to_string()],
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
            enable_sidebar: config
                .get("enable_sidebar")
                .and_then(JsonValue::as_bool)
                .unwrap_or(defaults.enable_sidebar),
            initial_sidebar_state: normalized_string(config, "initial_sidebar_state")
                .unwrap_or(defaults.initial_sidebar_state),
            editor_command: normalized_string(config, "editor_command"),
            helix_runtime_path: normalized_string(config, "helix_runtime_path"),
            yazi_command: normalized_string(config, "yazi_command")
                .unwrap_or(defaults.yazi_command),
            ya_command: normalized_string(config, "yazi_ya_command").unwrap_or(defaults.ya_command),
            popup_program: normalized_string_list(config, "popup_program")
                .filter(|items| !items.is_empty())
                .unwrap_or(defaults.popup_program),
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
        if self.initial_sidebar_state != "open" && self.initial_sidebar_state != "closed" {
            self.initial_sidebar_state = defaults.initial_sidebar_state;
        }
        if self.yazi_command.trim().is_empty() {
            self.yazi_command = defaults.yazi_command;
        }
        if self.ya_command.trim().is_empty() {
            self.ya_command = defaults.ya_command;
        }
        self.popup_program = non_empty_strings(self.popup_program);
        if self.popup_program.is_empty() {
            self.popup_program = defaults.popup_program;
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
        self.helix_runtime_path = self
            .helix_runtime_path
            .and_then(|value| non_empty_string(value.as_str()));
        self
    }
}

pub fn compute_session_facts_from_env() -> Result<SessionFactsData, CoreError> {
    if let Some(path) = session_facts_cache_path_from_env() {
        if let Some(facts) = read_session_facts_cache(&path) {
            return Ok(facts);
        }
    }

    let state_dir = state_dir_from_env()?;
    if let Some(facts) = read_session_facts_cache(&session_facts_cache_path(&state_dir)) {
        return Ok(facts);
    }

    compute_lossy_session_facts_from_config()
}

pub fn session_facts_cache_path(state_dir: &Path) -> PathBuf {
    state_dir.join("state").join(SESSION_FACTS_FILE_NAME)
}

pub fn session_facts_cache_path_from_state_path(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .map(|state_dir| state_dir.join(SESSION_FACTS_FILE_NAME))
        .unwrap_or_else(|| state_path.with_file_name(SESSION_FACTS_FILE_NAME))
}

pub fn write_session_facts_cache_from_normalized_config(
    state_path: &Path,
    source_config_file: &str,
    config: &JsonMap<String, JsonValue>,
) -> Result<(), CoreError> {
    let path = session_facts_cache_path_from_state_path(state_path);
    let cache = SessionFactsCache {
        schema_version: SESSION_FACTS_SCHEMA_VERSION,
        source_config_file: source_config_file.to_string(),
        normalized_config: config.clone(),
        facts: SessionFactsData::from_normalized_config(config),
    };
    write_json_atomic(&path, &cache)
}

fn session_facts_cache_path_from_env() -> Option<PathBuf> {
    std::env::var(SESSION_FACTS_PATH_ENV)
        .ok()
        .and_then(|value| non_empty_string(value.as_str()))
        .map(PathBuf::from)
}

fn read_session_facts_cache(path: &Path) -> Option<SessionFactsData> {
    let raw = fs::read_to_string(path).ok()?;
    let cache = serde_json::from_str::<SessionFactsCache>(&raw).ok()?;
    (cache.schema_version == SESSION_FACTS_SCHEMA_VERSION).then(|| cache.facts.sanitized())
}

fn compute_lossy_session_facts_from_config() -> Result<SessionFactsData, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let paths = primary_config_paths(&runtime_dir, &config_dir);

    let mut facts = SessionFactsData::default();
    facts.apply_lossy_table(read_toml_table_lossy(&paths.default_config_path).as_ref());

    let active_config_path = config_override
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if paths.user_config.exists() {
                paths.user_config.clone()
            } else {
                paths.default_config_path.clone()
            }
        });

    if active_config_path != paths.default_config_path {
        facts.apply_lossy_table(read_toml_table_lossy(&active_config_path).as_ref());
    }

    Ok(facts.sanitized())
}

impl SessionFactsData {
    fn apply_lossy_table(&mut self, config: Option<&toml::Table>) {
        let Some(config) = config else {
            return;
        };

        if let Some(core) = toml_section(config, "core") {
            if let Some(value) = toml_string(core.get("game_of_life_cell_style")) {
                self.game_of_life_cell_style = value;
            }
        }

        if let Some(shell) = toml_section(config, "shell") {
            if let Some(value) = toml_string(shell.get("default")) {
                self.default_shell = value;
            }
        }

        if let Some(helix) = toml_section(config, "helix") {
            self.helix_runtime_path = toml_optional_string(helix.get("runtime_path"));
        }

        if let Some(editor) = toml_section(config, "editor") {
            self.editor_command = toml_optional_string(editor.get("command"));
            if let Some(value) = toml_bool(editor.get("enable_sidebar")) {
                self.enable_sidebar = value;
            }
            if let Some(value) = toml_string(editor.get("initial_sidebar_state")) {
                self.initial_sidebar_state = value;
            }
        }

        if let Some(terminal) = toml_section(config, "terminal") {
            if let Some(values) = toml_string_list(terminal.get("terminals")) {
                self.terminals = values;
            }
        }

        if let Some(zellij) = toml_section(config, "zellij") {
            if let Some(values) = toml_string_list(zellij.get("popup_program")) {
                self.popup_program = values;
            }
            if let Some(value) = toml_percent(zellij.get("popup_width_percent")) {
                self.popup_width_percent = value;
            }
            if let Some(value) = toml_percent(zellij.get("popup_height_percent")) {
                self.popup_height_percent = value;
            }
        }

        if let Some(yazi) = toml_section(config, "yazi") {
            self.yazi_command = toml_optional_string(yazi.get("command"))
                .unwrap_or_else(|| self.yazi_command.clone());
            self.ya_command = toml_optional_string(yazi.get("ya_command"))
                .unwrap_or_else(|| self.ya_command.clone());
        }
    }
}

fn normalized_string(config: &JsonMap<String, JsonValue>, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(JsonValue::as_str)
        .and_then(non_empty_string)
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

fn read_toml_table_lossy(path: &Path) -> Option<toml::Table> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| toml::from_str::<toml::Table>(&raw).ok())
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

fn toml_bool(value: Option<&TomlValue>) -> Option<bool> {
    value.and_then(TomlValue::as_bool)
}

fn toml_string_list(value: Option<&TomlValue>) -> Option<Vec<String>> {
    value.and_then(TomlValue::as_array).map(|items| {
        non_empty_strings(
            items
                .iter()
                .filter_map(TomlValue::as_str)
                .map(ToOwned::to_owned)
                .collect(),
        )
    })
}

fn toml_percent(value: Option<&TomlValue>) -> Option<i64> {
    let parsed = match value? {
        TomlValue::Integer(number) => Some(*number),
        TomlValue::String(raw) => raw.trim().parse::<i64>().ok(),
        _ => None,
    }?;
    (1..=100).contains(&parsed).then_some(parsed)
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

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "session_facts_cache_mkdir",
                "Could not create the Yazelix session facts cache directory.",
                "Check permissions under the Yazelix state directory, then retry.",
                parent.to_string_lossy(),
                source,
            )
        })?;
    }

    let raw = serde_json::to_string_pretty(value).map_err(|source| {
        CoreError::classified(
            crate::bridge::ErrorClass::Internal,
            "session_facts_cache_serialize",
            format!("Could not serialize Yazelix session facts: {source}"),
            "Report this as a Yazelix internal error.",
            serde_json::json!({}),
        )
    })?;
    let temp_path = path.with_extension(format!("json.tmp.{}", std::process::id()));
    fs::write(&temp_path, raw).map_err(|source| {
        CoreError::io(
            "session_facts_cache_write",
            "Could not write the Yazelix session facts cache.",
            "Check permissions under the Yazelix state directory, then retry.",
            temp_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temp_path, path).map_err(|source| {
        CoreError::io(
            "session_facts_cache_replace",
            "Could not replace the Yazelix session facts cache.",
            "Check permissions under the Yazelix state directory, then retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Defends: generated session facts preserve the pane/runtime values used after the user config has moved on.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn session_facts_from_normalized_config_keeps_current_session_values() {
        let config = JsonMap::from_iter([
            ("editor_command".to_string(), json!("nvim")),
            ("initial_sidebar_state".to_string(), json!("closed")),
            ("yazi_command".to_string(), json!("yy")),
            ("yazi_ya_command".to_string(), json!("ya-test")),
            ("popup_program".to_string(), json!(["gitui", "status"])),
            ("popup_width_percent".to_string(), json!(82)),
            ("popup_height_percent".to_string(), json!("76")),
            ("game_of_life_cell_style".to_string(), json!("dotted")),
            ("default_shell".to_string(), json!("bash")),
            ("terminals".to_string(), json!(["ghostty", "wezterm"])),
        ]);

        let facts = SessionFactsData::from_normalized_config(&config);

        assert_eq!(facts.editor_command.as_deref(), Some("nvim"));
        assert_eq!(facts.initial_sidebar_state, "closed");
        assert_eq!(facts.yazi_command, "yy");
        assert_eq!(facts.ya_command, "ya-test");
        assert_eq!(facts.popup_program, vec!["gitui", "status"]);
        assert_eq!(facts.popup_width_percent, 82);
        assert_eq!(facts.popup_height_percent, 76);
        assert_eq!(facts.game_of_life_cell_style, "dotted");
        assert_eq!(facts.default_shell, "bash");
        assert_eq!(facts.terminals, vec!["ghostty", "wezterm"]);
    }
}
