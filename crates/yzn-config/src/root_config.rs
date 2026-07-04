use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use ratconfig::toml_adapter::{get_toml_path, parse_toml_value, set_toml_value_text};
use ratconfig::{
    ConfigContract, join_toml_contract_text_from_version, reconcile_joined_toml_contract_text,
    string_list_values_from_json,
};
use serde_json::Value as JsonValue;

use crate::{catalog::*, common::*, custom_popups::custom_popups};

pub(crate) fn config_field(path: &str) -> Result<&'static ConfigFieldSpec> {
    CONFIG_FIELDS
        .iter()
        .find(|spec| spec.field.path == path)
        .ok_or_else(|| error(format!("unknown config path: {path}")))
}
pub(crate) fn root_config_field_paths() -> impl Iterator<Item = &'static str> {
    CONFIG_FIELDS
        .iter()
        .map(|spec| spec.field.path)
        .chain([BAR_WIDGETS_PATH])
}
pub(crate) fn ensure_config_file_at(path: PathBuf) -> Result<PathBuf> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let raw = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        DEFAULT_CONFIG_TOML.to_string()
    };
    let reconciled = reconcile_contract(&raw)?;
    let completed = fill_missing_defaults(&reconciled)?;
    if completed != raw || !path.exists() {
        if path.exists() {
            if path_read_only(&path) && toml_semantically_equal(&raw, &completed)? {
                return Ok(path);
            }
            reject_read_only_source(&path, SOURCE_CONFIG)?;
        }
        atomic_write(&path, &completed)?;
    }
    Ok(path)
}
fn toml_semantically_equal(left: &str, right: &str) -> Result<bool> {
    Ok(
        parse_toml_value(left).map_err(|error| boxed_debug("invalid TOML", error))?
            == parse_toml_value(right).map_err(|error| boxed_debug("invalid TOML", error))?,
    )
}
fn reconcile_contract(raw: &str) -> Result<String> {
    let contract = ConfigContract {
        id: CONTRACT_ID.to_string(),
        baseline_version: CONTRACT_VERSION,
        current_version: CONTRACT_VERSION,
        changes: Vec::new(),
    };
    let joined =
        join_toml_contract_text_from_version(raw, &contract, CONTRACT_STATE_PATH, CONTRACT_VERSION)
            .or_else(|_| reconcile_joined_toml_contract_text(raw, &contract, CONTRACT_STATE_PATH))
            .map_err(|error| boxed_debug("could not reconcile config contract", error))?;
    Ok(joined.text)
}
fn fill_missing_defaults(raw: &str) -> Result<String> {
    let mut text = raw.to_string();
    let defaults = default_config()?;
    for field_path in root_config_field_paths() {
        let default = default_config_path_value(&defaults, field_path)?;
        let value = parse_toml_value(&text).map_err(|error| boxed_debug("invalid TOML", error))?;
        if get_toml_path(&value, field_path).is_none() {
            text = set_toml_value_text(&text, field_path, &default)
                .map_err(|error| boxed_debug("could not write missing default", error))?
                .text;
        }
    }
    Ok(text)
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
pub(crate) fn read_config_field(path: &Path, spec: &ConfigFieldSpec) -> Result<String> {
    let value = read_toml_file_value(path, "config.toml")?;
    validate_popup_keybindings(&value)?;
    let Some(value) = get_toml_path(&value, spec.field.path) else {
        return Err(error(format!("unknown config path: {}", spec.field.path)));
    };
    validate_config_value(spec.field.path, value)?;
    match spec.field.kind {
        "string" => Ok(spec.field.json_choice(value)?.to_string()),
        "boolean" => Ok(json_bool(spec.field.path, value)?.to_string()),
        "integer" => Ok(json_i64(spec.field.path, value)?.to_string()),
        _ => Err(error(format!(
            "{} must be {}",
            spec.field.path, spec.field.validation
        ))),
    }
}
pub(crate) fn read_bar_widgets_field(path: &Path) -> Result<String> {
    let value = read_toml_file_value(path, "config.toml")?;
    validate_popup_keybindings(&value)?;
    let Some(value) = get_toml_path(&value, BAR_WIDGETS_PATH) else {
        return Err(error(format!("unknown config path: {BAR_WIDGETS_PATH}")));
    };
    Ok(serde_json::to_string(&bar_widgets(value)?)?)
}
pub(crate) fn bar_widgets(value: &JsonValue) -> Result<Vec<String>> {
    string_list_values_from_json(BAR_WIDGETS_PATH, value, &string_values(BAR_WIDGET_VALUES))
        .map_err(error)
}
pub(crate) fn write_config_field(path: &Path, field_path: &str, value: &JsonValue) -> Result<()> {
    validate_config_value(field_path, value)?;
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update config.toml", error))?
        .text;
    let text = fill_missing_defaults(&reconcile_contract(&text)?)?;
    let value =
        parse_toml_value(&text).map_err(|error| boxed_debug("invalid config.toml", error))?;
    validate_popup_keybindings(&value)?;
    atomic_write(path, &text)
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
            } else if popup_keybinding_spec(field_path).is_some() {
                validate_managed_popup_keybinding(field_path, value)?;
            }
            Ok(())
        }
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
pub(crate) fn validate_popup_keybindings(value: &JsonValue) -> Result<()> {
    let mut used = BTreeMap::new();
    for spec in POPUP_KEYBINDINGS {
        let Some(value) = get_toml_path(value, spec.path) else {
            continue;
        };
        let chord = config_field(spec.path)?.field.json_choice(value)?;
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
