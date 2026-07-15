use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use ratconfig::string_list_values_from_json;
use ratconfig::toml_adapter::{
    get_toml_path, parse_toml_value, set_toml_value_text, unset_toml_value_text,
};
use serde_json::Value as JsonValue;

use crate::{catalog::*, common::*, custom_popups::custom_popups};

pub(crate) fn config_field(path: &str) -> Result<&'static ConfigFieldSpec> {
    CONFIG_FIELDS
        .iter()
        .find(|spec| spec.field.path == path)
        .ok_or_else(|| error(format!("unknown config path: {path}")))
}
pub(crate) fn validate_config_file_at(path: PathBuf) -> Result<PathBuf> {
    let value = read_optional_toml_file_value(&path, "invalid config.toml")?;
    validate_root_config(&value)?;
    Ok(path)
}
pub(crate) fn default_config() -> Result<JsonValue> {
    parse_toml_value(DEFAULT_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default config.toml", error))
}
pub(crate) fn default_config_path_value(
    defaults: &JsonValue,
    field_path: &str,
) -> Result<JsonValue> {
    get_toml_path(defaults, field_path)
        .cloned()
        .ok_or_else(|| error(format!("default config.toml is missing {field_path}")))
}
pub(crate) fn read_toml_file_value(path: &Path, label: &'static str) -> Result<JsonValue> {
    let raw = fs::read_to_string(path)?;
    parse_toml_value(&raw).map_err(|error| boxed_debug(label, error))
}
pub(crate) fn read_optional_toml_file_value(path: &Path, label: &'static str) -> Result<JsonValue> {
    if path_entry_exists(path)? {
        read_toml_file_value(path, label)
    } else {
        Ok(JsonValue::Object(Default::default()))
    }
}
fn effective_config_path_value(value: &JsonValue, field_path: &str) -> Result<JsonValue> {
    config_path_value(value, &default_config()?, field_path)
}
pub(crate) fn config_path_value(
    value: &JsonValue,
    defaults: &JsonValue,
    field_path: &str,
) -> Result<JsonValue> {
    get_toml_path(value, field_path)
        .cloned()
        .map_or_else(|| default_config_path_value(defaults, field_path), Ok)
}
pub(crate) fn read_config_field(path: &Path, spec: &ConfigFieldSpec) -> Result<String> {
    let value = read_optional_toml_file_value(path, "config.toml")?;
    validate_root_config(&value)?;
    let value = effective_config_path_value(&value, spec.field.path)?;
    validate_config_value(spec.field.path, &value)?;
    match spec.field.kind {
        "string" => Ok(spec.field.json_choice(&value)?.to_string()),
        "string_list" => Ok(serde_json::to_string(&json_string_list(
            spec.field.path,
            &value,
        )?)?),
        "boolean" => Ok(json_bool(spec.field.path, &value)?.to_string()),
        "integer" => Ok(json_i64(spec.field.path, &value)?.to_string()),
        _ => Err(error(format!(
            "{} must be {}",
            spec.field.path, spec.field.validation
        ))),
    }
}
pub(crate) fn read_bar_widgets_field(path: &Path) -> Result<String> {
    let value = read_optional_toml_file_value(path, "config.toml")?;
    validate_root_config(&value)?;
    Ok(serde_json::to_string(&bar_widgets(
        &effective_config_path_value(&value, BAR_WIDGETS_PATH)?,
    )?)?)
}
pub(crate) fn read_agent_popup_kdl(path: &Path) -> Result<String> {
    let value = read_optional_toml_file_value(path, "config.toml")?;
    validate_root_config(&value)?;
    let command = agent_command(&value)?;
    if command == AGENT_AUTO_COMMAND {
        return Ok(String::new());
    }
    Ok(render_agent_popup_kdl(&command, &agent_args(&value)?))
}
pub(crate) fn bar_widgets(value: &JsonValue) -> Result<Vec<String>> {
    string_list_values_from_json(BAR_WIDGETS_PATH, value, &string_values(BAR_WIDGET_VALUES))
        .map_err(error)
}
pub(crate) fn agent_command(value: &JsonValue) -> Result<String> {
    let value = effective_config_path_value(value, AGENT_COMMAND_PATH)?;
    let command = config_field(AGENT_COMMAND_PATH)?
        .field
        .json_choice(&value)?;
    validate_agent_command(command)?;
    Ok(command.to_string())
}
pub(crate) fn agent_args(value: &JsonValue) -> Result<Vec<String>> {
    json_string_list(
        AGENT_ARGS_PATH,
        &effective_config_path_value(value, AGENT_ARGS_PATH)?,
    )
}
fn json_string_list(path: &str, value: &JsonValue) -> Result<Vec<String>> {
    string_list_values_from_json(path, value, &[]).map_err(error)
}
pub(crate) fn write_config_field(path: &Path, field_path: &str, value: &JsonValue) -> Result<()> {
    validate_config_value(field_path, value)?;
    let raw = if path_entry_exists(path)? {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update config.toml", error))?
        .text;
    if field_path == AGENT_COMMAND_PATH && value.as_str() == Some(AGENT_AUTO_COMMAND) {
        text = unset_toml_value_text(&text, AGENT_ARGS_PATH)
            .map_err(|error| boxed_debug("could not clear agent.args", error))?
            .text;
    }
    let value =
        parse_toml_value(&text).map_err(|error| boxed_debug("invalid config.toml", error))?;
    validate_root_config(&value)?;
    atomic_write(path, &text)
}
pub(crate) fn unset_config_field(path: &Path, field_path: &str) -> Result<()> {
    default_config_value(field_path)?;
    if !path_entry_exists(path)? {
        return Ok(());
    }
    let raw = fs::read_to_string(path)?;
    let text = unset_toml_value_text(&raw, field_path)
        .map_err(|error| boxed_debug("could not update config.toml", error))?
        .text;
    let value =
        parse_toml_value(&text).map_err(|error| boxed_debug("invalid config.toml", error))?;
    validate_root_config(&value)?;
    if config_is_empty(&value) {
        fs::remove_file(path)?;
        Ok(())
    } else {
        atomic_write(path, &text)
    }
}
fn config_is_empty(value: &JsonValue) -> bool {
    value
        .as_object()
        .is_some_and(|table| table.values().all(config_is_empty))
}
pub(crate) fn default_config_value(field_path: &str) -> Result<JsonValue> {
    if field_path != BAR_WIDGETS_PATH {
        config_field(field_path)?;
    }
    default_config_path_value(&default_config()?, field_path)
}
pub(crate) fn validate_config_value(field_path: &str, value: &JsonValue) -> Result<()> {
    if field_path == BAR_WIDGETS_PATH {
        return bar_widgets(value).map(|_| ());
    }

    let spec = &config_field(field_path)?.field;
    match spec.kind {
        "boolean" => json_bool(field_path, value).map(|_| ()),
        "string" => {
            let value = spec.json_choice(value)?;
            if field_path == EDITOR_COMMAND_PATH {
                validate_editor_command(value)?;
            } else if field_path == AGENT_COMMAND_PATH {
                validate_agent_command(value)?;
            } else if popup_keybinding_spec(field_path).is_some() {
                validate_managed_popup_keybinding(field_path, value)?;
            }
            Ok(())
        }
        "string_list" => json_string_list(field_path, value).map(|_| ()),
        "integer" => {
            let value = json_i64(field_path, value)?;
            if matches!(
                field_path,
                POPUP_SIDE_MARGIN_PATH | POPUP_VERTICAL_MARGIN_PATH
            ) && value < 0
            {
                return Err(error(format!("{field_path} must be zero or greater")));
            }
            if field_path == WELCOME_DURATION_SECONDS_PATH && !(1..=60).contains(&value) {
                return Err(error(format!("{field_path} must be between 1 and 60")));
            }
            Ok(())
        }
        _ => Err(error(format!("{field_path} must be {}", spec.validation))),
    }
}
pub(crate) fn validate_root_config(value: &JsonValue) -> Result<()> {
    let table = value
        .as_object()
        .ok_or_else(|| error("config.toml root must be a table"))?;
    validate_config_table(table, "")?;
    validate_popup_keybindings(value)?;
    validate_agent_config(value)
}
fn validate_config_table(table: &serde_json::Map<String, JsonValue>, parent: &str) -> Result<()> {
    for (key, value) in table {
        let path = if parent.is_empty() {
            key.clone()
        } else {
            format!("{parent}.{key}")
        };
        if key.contains('.') {
            return Err(error(format!(
                "{path} must use nested TOML tables, not a quoted dotted key"
            )));
        }
        if path == "popups" {
            continue;
        }
        if root_config_paths().any(|candidate| candidate == path) {
            validate_config_value(&path, value)?;
        } else if root_config_paths().any(|candidate| {
            candidate
                .strip_prefix(&path)
                .is_some_and(|suffix| suffix.starts_with('.'))
        }) {
            let child = value
                .as_object()
                .ok_or_else(|| error(format!("{path} must be a table")))?;
            validate_config_table(child, &path)?;
        } else {
            return Err(error(format!(
                "{path} is not supported; use a documented Nova config path"
            )));
        }
    }
    Ok(())
}
fn root_config_paths() -> impl Iterator<Item = &'static str> {
    CONFIG_FIELDS
        .iter()
        .map(|spec| spec.field.path)
        .chain([BAR_WIDGETS_PATH])
}
pub(crate) fn validate_agent_config(value: &JsonValue) -> Result<()> {
    let command_value = effective_config_path_value(value, AGENT_COMMAND_PATH)?;
    let command = config_field(AGENT_COMMAND_PATH)?
        .field
        .json_choice(&command_value)?;
    validate_agent_command(command)?;
    let args = agent_args(value)?;
    if command == AGENT_AUTO_COMMAND && !args.is_empty() {
        return Err(error(
            "agent.args requires agent.command to be a custom command",
        ));
    }
    Ok(())
}
fn validate_editor_command(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(error("editor.command must not be empty"));
    }
    if value.chars().any(char::is_whitespace) {
        return Err(error(
            "editor.command must be one executable command without arguments",
        ));
    }
    Ok(())
}
fn validate_agent_command(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(error("agent.command must not be empty"));
    }
    if value.chars().any(char::is_whitespace) {
        return Err(error(
            "agent.command must be auto or one executable command without arguments",
        ));
    }
    Ok(())
}
fn render_agent_popup_kdl(command: &str, args: &[String]) -> String {
    let mut text = format!(
        "            agent {{\n                command {}\n",
        kdl_string(command)
    );
    for (index, arg) in args.iter().enumerate() {
        text.push_str(&format!(
            "                arg_{} {}\n",
            index + 1,
            kdl_string(arg)
        ));
    }
    text.push_str(
        "                pane_title \"agent_popup\"\n                width_percent 100\n                height_percent 100\n                preserve_terminal_title true\n                toggle_close_behavior \"hide\"\n            }",
    );
    text
}
pub(crate) fn validate_popup_keybindings(value: &JsonValue) -> Result<()> {
    let mut used = BTreeMap::new();
    for spec in POPUP_KEYBINDINGS {
        let value = effective_config_path_value(value, spec.path)?;
        let chord = config_field(spec.path)?.field.json_choice(&value)?;
        validate_managed_popup_keybinding(spec.path, chord)?;
        if let Some(existing) = used.insert(chord.to_ascii_lowercase(), spec.path.to_string()) {
            return Err(error(format!(
                "{} conflicts with {existing}: {chord}",
                spec.path
            )));
        }
    }
    for popup in custom_popups(value)? {
        let path = format!("popups.{}.keybinding", popup.id);
        if let Some(existing) = used.insert(popup.keybinding.to_ascii_lowercase(), path.clone()) {
            return Err(error(format!(
                "{path} conflicts with {existing}: {}",
                popup.keybinding
            )));
        }
    }
    Ok(())
}
pub(crate) fn popup_keybinding_spec(field_path: &str) -> Option<&'static PopupKeybindingSpec> {
    POPUP_KEYBINDINGS
        .iter()
        .find(|spec| spec.path == field_path)
}
pub(crate) fn validate_managed_popup_keybinding(field_path: &str, value: &str) -> Result<()> {
    validate_key_chord(field_path, value)?;
    let conflicts = KEY_BINDINGS
        .iter()
        .any(|[_group, chord, _action, _owner, _source]| {
            packaged_chord_matches(chord, value) && !popup_default_chord_matches(value)
        });
    if conflicts {
        return Err(error(format!(
            "{field_path} conflicts with packaged key {value}"
        )));
    }
    Ok(())
}
fn packaged_chord_matches(pattern: &str, value: &str) -> bool {
    pattern.split(" / ").any(|chord| {
        chord.eq_ignore_ascii_case(value)
            || matches!(
                (chord, value.strip_prefix("Alt ")),
                (
                    "Alt 1-9",
                    Some("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                )
            )
    })
}
fn popup_default_chord_matches(value: &str) -> bool {
    POPUP_KEYBINDINGS
        .iter()
        .any(|spec| spec.default.eq_ignore_ascii_case(value))
}
fn validate_key_chord(field_path: &str, value: &str) -> Result<()> {
    value
        .rsplit_once(' ')
        .filter(|(modifiers, key)| {
            matches!(
                *modifiers,
                "Ctrl"
                    | "Alt"
                    | "Shift"
                    | "Ctrl Alt"
                    | "Ctrl Shift"
                    | "Alt Shift"
                    | "Ctrl Alt Shift"
            ) && valid_key_token(key)
        })
        .map(|_| ())
        .ok_or_else(|| keybinding_syntax_error(field_path))
}
fn valid_key_token(key: &str) -> bool {
    matches!(key.as_bytes(), [ch] if ch.is_ascii_alphanumeric())
        || matches!(
            key,
            "Left"
                | "Right"
                | "Up"
                | "Down"
                | "Enter"
                | "Esc"
                | "Tab"
                | "Backspace"
                | "Space"
                | "Home"
                | "End"
                | "PageUp"
                | "PageDown"
        )
}
fn keybinding_syntax_error(field_path: &str) -> Box<dyn std::error::Error> {
    error(format!("{field_path} must be a key chord like Alt Shift A"))
}
