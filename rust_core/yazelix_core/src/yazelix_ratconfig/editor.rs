// Test lane: default

use super::{
    ConfigUiEditBehavior, ConfigUiField, ConfigUiModel, UiRowRef, visible_rows_for_tab_search,
};
use serde_json::Value as JsonValue;
use std::collections::BTreeSet;

pub(crate) struct ConfigUiApp {
    pub(crate) model: ConfigUiModel,
    pub(crate) selected_tab: usize,
    pub(crate) selected_row: usize,
    pub(crate) search: String,
    pub(crate) search_active: bool,
    pub(crate) edit: Option<ConfigUiEditState>,
    pub(crate) notice: Option<ConfigUiNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigUiNotice {
    pub(crate) text: String,
    pub(crate) is_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigUiEditState {
    pub(crate) field_index: usize,
    pub(crate) input: String,
    pub(crate) mode: ConfigUiEditMode,
    pub(crate) choice_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigUiEditMode {
    Text,
    Choice,
    MultiChoice,
}

impl ConfigUiApp {
    pub(crate) fn new(model: ConfigUiModel) -> Self {
        Self {
            model,
            selected_tab: 0,
            selected_row: 0,
            search: String::new(),
            search_active: false,
            edit: None,
            notice: None,
        }
    }

    pub(crate) fn visible_rows(&self) -> Vec<UiRowRef> {
        visible_rows_for_tab_search(&self.model, self.selected_tab, &self.search)
    }

    pub(crate) fn next_tab(&mut self) {
        let len = self.model.tabs.len();
        if len > 0 {
            self.selected_tab = (self.selected_tab + 1) % len;
            self.selected_row = 0;
        }
    }

    pub(crate) fn previous_tab(&mut self) {
        let len = self.model.tabs.len();
        if len > 0 {
            self.selected_tab = (self.selected_tab + len - 1) % len;
            self.selected_row = 0;
        }
    }

    pub(crate) fn move_down(&mut self) {
        let len = self.visible_rows().len();
        if len > 0 {
            self.selected_row = (self.selected_row + 1).min(len - 1);
        }
    }

    pub(crate) fn move_up(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(1);
    }

    pub(crate) fn clamp_selection(&mut self) {
        if self.selected_tab >= self.model.tabs.len() {
            self.selected_tab = 0;
        }
        self.clamp_selection_for_len(self.visible_rows().len());
    }

    pub(crate) fn clamp_selection_for_len(&mut self, len: usize) {
        self.selected_row = if len == 0 {
            0
        } else {
            self.selected_row.min(len - 1)
        };
    }

    pub(crate) fn selected_field_index(&self) -> Option<usize> {
        let row = self.visible_rows().get(self.selected_row).copied()?;
        match row {
            UiRowRef::Field(index) => Some(index),
            _ => None,
        }
    }

    pub(crate) fn selected_field(&self) -> Option<&ConfigUiField> {
        self.selected_field_index()
            .and_then(|index| self.model.fields.get(index))
    }

    pub(crate) fn notice_info(&mut self, text: impl Into<String>) {
        self.notice = Some(ConfigUiNotice {
            text: text.into(),
            is_error: false,
        });
    }

    pub(crate) fn notice_error(&mut self, text: impl Into<String>) {
        self.notice = Some(ConfigUiNotice {
            text: text.into(),
            is_error: true,
        });
    }
}

pub(crate) fn edit_input_for_field(field: &ConfigUiField) -> String {
    if field.current_value == "not set" {
        if is_bool_field(field) {
            return "false".to_string();
        }
        if is_scalar_enum_field(field) && !field.allowed_values.is_empty() {
            return field.allowed_values[0].clone();
        }
        return String::new();
    }
    if field.edit_behavior == ConfigUiEditBehavior::FriendlyStringList {
        return friendly_string_list_edit_input(field);
    }
    if is_string_field(field) || is_scalar_enum_field(field) {
        return parse_rendered_json_string(&field.current_value)
            .unwrap_or_else(|| field.current_value.clone());
    }
    if field.edit_value.is_empty() {
        field.current_value.clone()
    } else {
        field.edit_value.clone()
    }
}

pub(crate) fn edit_mode_for_field(field: &ConfigUiField) -> ConfigUiEditMode {
    if is_enum_string_list_field(field) {
        ConfigUiEditMode::MultiChoice
    } else if is_direct_choice_field(field) {
        ConfigUiEditMode::Choice
    } else {
        ConfigUiEditMode::Text
    }
}

pub(crate) fn initial_edit_choice_index(field: &ConfigUiField, input: &str) -> usize {
    if is_scalar_enum_field(field)
        && let Some(index) = field
            .allowed_values
            .iter()
            .position(|allowed| allowed == input)
    {
        return index;
    }
    if is_enum_string_list_field(field)
        && let Ok(values) = parse_string_list_values(field, input)
        && let Some(index) = values.first().and_then(|value| {
            field
                .allowed_values
                .iter()
                .position(|allowed| allowed == value)
        })
    {
        return index;
    }
    0
}

pub(crate) fn parse_edit_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let trimmed = input.trim();
    match field.kind.as_str() {
        "bool" | "boolean" => parse_bool_input(field, trimmed),
        "int" | "integer" => parse_i64_input(field, trimmed),
        "float" | "number" => parse_f64_input(field, trimmed),
        "string" => parse_string_field_input(field, input),
        "string_list" if field.edit_behavior == ConfigUiEditBehavior::FriendlyStringList => {
            parse_friendly_string_list_input(field, trimmed)
        }
        "string_list" => parse_string_list_input(field, trimmed),
        "array" => parse_json_input(field, trimmed, "JSON array").and_then(|value| {
            if value.is_array() {
                Ok(value)
            } else {
                Err(format!("{} must be a JSON array.", field.path))
            }
        }),
        "object" => parse_json_input(field, trimmed, "JSON object").and_then(|value| {
            if value.is_object() {
                Ok(value)
            } else {
                Err(format!("{} must be a JSON object.", field.path))
            }
        }),
        _ => parse_json_input(field, trimmed, "JSON value"),
    }
}

fn parse_bool_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    match input {
        "true" => Ok(JsonValue::Bool(true)),
        "false" => Ok(JsonValue::Bool(false)),
        _ => Err(format!("{} must be true or false.", field.path)),
    }
}

fn parse_i64_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let value = input
        .parse::<i64>()
        .map_err(|_| format!("{} must be an integer.", field.path))?;
    Ok(JsonValue::Number(value.into()))
}

fn parse_f64_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let value = input
        .parse::<f64>()
        .map_err(|_| format!("{} must be a number.", field.path))?;
    let number = serde_json::Number::from_f64(value)
        .ok_or_else(|| format!("{} must be a finite number.", field.path))?;
    Ok(JsonValue::Number(number))
}

fn parse_string_field_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let value = parse_string_input(input)
        .map_err(|message| format!("{} must be a string: {message}.", field.path))?;
    ensure_allowed_value(field, &value)?;
    Ok(JsonValue::String(value))
}

fn parse_string_list_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let strings = parse_string_list_values(field, input)?;
    Ok(JsonValue::Array(
        strings.into_iter().map(JsonValue::String).collect(),
    ))
}

fn parse_friendly_string_list_input(
    field: &ConfigUiField,
    input: &str,
) -> Result<JsonValue, String> {
    if input.starts_with('[') {
        return parse_string_list_input(field, input);
    }
    if input.is_empty() || input.eq_ignore_ascii_case("disabled") {
        return Ok(JsonValue::Array(Vec::new()));
    }
    Ok(JsonValue::Array(
        input
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| JsonValue::String(value.to_string()))
            .collect(),
    ))
}

pub(crate) fn parse_string_list_values(
    field: &ConfigUiField,
    input: &str,
) -> Result<Vec<String>, String> {
    let value = parse_json_input(field, input, "JSON string array")?;
    let array = value
        .as_array()
        .ok_or_else(|| format!("{} must be a JSON string array.", field.path))?;
    let mut strings = Vec::with_capacity(array.len());
    for value in array {
        let Some(value) = value.as_str() else {
            return Err(format!("{} must contain only strings.", field.path));
        };
        ensure_allowed_value(field, value)?;
        strings.push(value.to_string());
    }
    Ok(strings)
}

fn parse_json_input(
    field: &ConfigUiField,
    input: &str,
    expected: &str,
) -> Result<JsonValue, String> {
    serde_json::from_str::<JsonValue>(input)
        .map_err(|source| format!("{} must be a valid {expected}: {source}.", field.path))
}

fn parse_string_input(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.starts_with('"') {
        serde_json::from_str::<String>(trimmed).map_err(|source| source.to_string())
    } else {
        Ok(input.to_string())
    }
}

pub(crate) fn parse_rendered_json_string(value: &str) -> Option<String> {
    serde_json::from_str::<String>(value).ok()
}

fn ensure_allowed_value(field: &ConfigUiField, value: &str) -> Result<(), String> {
    if field.allowed_values.is_empty()
        || field.allowed_values.iter().any(|allowed| allowed == value)
    {
        return Ok(());
    }
    Err(format!(
        "{} must be one of: {}.",
        field.path,
        field.allowed_values.join(", ")
    ))
}

pub(crate) fn single_choice_status_value(
    field: &ConfigUiField,
    edit: &ConfigUiEditState,
) -> String {
    let highlighted = field
        .allowed_values
        .get(edit.choice_index)
        .map(String::as_str)
        .unwrap_or("none");
    if highlighted == edit.input {
        format!("selected {}", edit.input)
    } else {
        format!("selected {}, highlighted {highlighted}", edit.input)
    }
}

pub(crate) fn multi_choice_status_value(field: &ConfigUiField, edit: &ConfigUiEditState) -> String {
    let enabled = parse_string_list_values(field, &edit.input)
        .map(|values| values.len())
        .unwrap_or(0);
    let selected = field
        .allowed_values
        .get(edit.choice_index)
        .map(String::as_str)
        .unwrap_or("none");
    format!(
        "{enabled}/{} enabled, selected {selected}",
        field.allowed_values.len()
    )
}

pub(crate) fn toggled_string_list_input(
    field: &ConfigUiField,
    input: &str,
    choice_index: usize,
) -> Result<String, String> {
    let target = field
        .allowed_values
        .get(choice_index)
        .ok_or_else(|| format!("{} has no value selected.", field.path))?;
    let mut values = parse_string_list_values(field, input)?;
    if values.iter().any(|value| value == target) {
        values.retain(|value| value != target);
    } else {
        values.push(target.clone());
    }
    values = ordered_string_list_values(field, &values);
    serde_json::to_string(&values)
        .map_err(|source| format!("Could not render {} string list: {source}.", field.path))
}

fn ordered_string_list_values(field: &ConfigUiField, values: &[String]) -> Vec<String> {
    if field.allowed_values.is_empty() {
        return values.to_vec();
    }
    let selected = values.iter().cloned().collect::<BTreeSet<_>>();
    field
        .allowed_values
        .iter()
        .filter(|value| selected.contains(*value))
        .cloned()
        .collect()
}

pub(crate) fn is_bool_field(field: &ConfigUiField) -> bool {
    matches!(field.kind.as_str(), "bool" | "boolean")
}

fn is_direct_choice_field(field: &ConfigUiField) -> bool {
    is_bool_field(field) || is_scalar_enum_field(field)
}

fn is_string_field(field: &ConfigUiField) -> bool {
    field.kind == "string"
}

pub(crate) fn is_scalar_enum_field(field: &ConfigUiField) -> bool {
    is_string_field(field) && !field.allowed_values.is_empty()
}

pub(crate) fn is_enum_string_list_field(field: &ConfigUiField) -> bool {
    field.kind == "string_list" && !field.allowed_values.is_empty()
}

pub(crate) fn structured_only_edit_notice(field: &ConfigUiField) -> Option<&str> {
    if let ConfigUiEditBehavior::StructuredOnly { notice } = &field.edit_behavior {
        return Some(notice.as_str());
    }
    if matches!(field.kind.as_str(), "array" | "object" | "string_list_map") {
        return Some(
            "Structured editor unavailable for this complex field; edit the source config directly.",
        );
    }
    None
}

fn friendly_string_list_edit_input(field: &ConfigUiField) -> String {
    serde_json::from_str::<Vec<String>>(&field.edit_value)
        .map(|keys| keys.join(", "))
        .unwrap_or_else(|_| field.edit_value.clone())
}

pub(crate) fn field_bool_value(field: &ConfigUiField) -> Option<bool> {
    match field.current_value.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_jsonc_patch::{SettingsJsoncPatchMutation, set_jsonc_value_text};
    use crate::yazelix_ratconfig::{
        ConfigUiApplyStatus, ConfigUiPathOwner, ConfigUiValueState, row_line_for_model,
    };
    use serde_json::json;
    use std::path::PathBuf;

    fn field(path: &str, kind: &str, value: &str, allowed: &[&str]) -> ConfigUiField {
        ConfigUiField {
            path: path.to_string(),
            tab: "general".to_string(),
            kind: kind.to_string(),
            current_value: value.to_string(),
            edit_value: value.to_string(),
            default_value: value.to_string(),
            state: ConfigUiValueState::Explicit,
            description: String::new(),
            allowed_values: allowed.iter().map(|value| (*value).to_string()).collect(),
            validation: String::new(),
            rebuild_required: false,
            apply_status: ConfigUiApplyStatus {
                summary: "after save".to_string(),
                label: "after save".to_string(),
                detail: "The host application applies this field after saving.".to_string(),
                pending: true,
            },
            edit_behavior: ConfigUiEditBehavior::Default,
        }
    }

    // Defends: the reusable ratconfig layer can drive a non-Yazelix config fixture with bool, select, multiselect, rendering, and JSONC patching.
    #[test]
    fn non_yazelix_fixture_uses_generic_model_editor_render_and_jsonc_patch() {
        let model = ConfigUiModel {
            active_config_path: PathBuf::from("/tmp/acme/settings.jsonc"),
            cursor_config_path: PathBuf::from("/tmp/acme/cursors.jsonc"),
            default_cursor_config_path: PathBuf::from("/tmp/acme/default_cursors.jsonc"),
            active_config_exists: true,
            config_owner: ConfigUiPathOwner::User,
            config_read_only: false,
            tabs: vec!["general".to_string()],
            fields: vec![
                field("server.enabled", "bool", "false", &[]),
                field("ui.theme", "string", "\"light\"", &["light", "dark"]),
                field(
                    "plugins.enabled",
                    "string_list",
                    r#"["git"]"#,
                    &["git", "search"],
                ),
            ],
            sidecars: Vec::new(),
            native_config_statuses: Vec::new(),
            diagnostics: Vec::new(),
        };
        let app = ConfigUiApp::new(model);

        assert_eq!(app.visible_rows().len(), 3);
        assert_eq!(
            row_line_for_model(&app.model, app.visible_rows()[0])
                .spans
                .iter()
                .map(|span| span.content.trim().to_string())
                .collect::<Vec<_>>(),
            vec!["after save", "server.enabled", "false"]
        );
        assert_eq!(
            parse_edit_input(&app.model.fields[0], "true").expect("bool"),
            json!(true)
        );
        assert_eq!(
            parse_edit_input(&app.model.fields[1], "dark").expect("select"),
            json!("dark")
        );
        assert_eq!(
            toggled_string_list_input(&app.model.fields[2], r#"["git"]"#, 1).expect("toggle"),
            r#"["git","search"]"#
        );

        let raw = r#"{
  // host-owned config
  "ui": { "theme": "light" }
}
"#;
        let patched =
            set_jsonc_value_text(raw, "ui.theme", &json!("dark")).expect("generic jsonc patch");
        assert_eq!(patched.mutation, SettingsJsoncPatchMutation::Replaced);
        assert!(patched.text.contains("// host-owned config"));
        assert!(patched.text.contains(r#""theme": "dark""#));
    }
}
