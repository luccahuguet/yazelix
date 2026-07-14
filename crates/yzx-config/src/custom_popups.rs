use std::{collections::BTreeMap, path::Path};

use ratconfig::toml_adapter::get_toml_path;
use serde_json::Value as JsonValue;

use crate::{
    common::*,
    root_config::{
        read_optional_toml_file_value, validate_managed_popup_keybinding,
        validate_popup_keybindings,
    },
};

pub(crate) struct CustomPopup {
    pub(crate) id: String,
    command: String,
    args: Vec<String>,
    title: String,
    pub(crate) keybinding: String,
    keep_alive: bool,
}
pub(crate) fn read_custom_popups_kdl(path: &Path) -> Result<String> {
    let value = read_optional_toml_file_value(path, "config.toml")?;
    validate_popup_keybindings(&value)?;
    let mut text = String::new();
    for popup in custom_popups(&value)? {
        text.push_str(&format!(
            "            {} {{\n                command {}\n",
            popup.id,
            kdl_string(&popup.command)
        ));
        for (index, arg) in popup.args.iter().enumerate() {
            text.push_str(&format!(
                "                arg_{} {}\n",
                index + 1,
                kdl_string(arg)
            ));
        }
        text.push_str(&format!(
            "                pane_title {}\n                command_marker {}\n                width_percent 100\n                height_percent 100\n",
            kdl_string(&popup.title),
            kdl_string(&popup.title),
        ));
        if popup.keep_alive {
            text.push_str("                toggle_close_behavior \"hide\"\n");
        }
        text.push_str("            }\n");
    }
    Ok(text)
}
pub(crate) fn read_custom_popup_keybindings_kdl(path: &Path) -> Result<String> {
    let value = read_optional_toml_file_value(path, "config.toml")?;
    validate_popup_keybindings(&value)?;
    let mut text = String::new();
    for popup in custom_popups(&value)? {
        text.push_str(&format!(
            "        bind {} {{\n            MessagePlugin \"yzpp\" {{\n                name \"toggle\"\n                payload {}\n            }}\n        }}\n",
            kdl_string(&popup.keybinding),
            kdl_string(&popup.id),
        ));
    }
    Ok(text)
}
pub(crate) fn custom_popups(value: &JsonValue) -> Result<Vec<CustomPopup>> {
    let Some(popups) = get_toml_path(value, "popups") else {
        return Ok(Vec::new());
    };
    let table = popups
        .as_object()
        .ok_or_else(|| error("popups must be a table"))?;
    let mut parsed = table
        .iter()
        .map(|(id, value)| custom_popup(id, value))
        .collect::<Result<Vec<_>>>()?;
    parsed.sort_by(|left, right| left.id.cmp(&right.id));
    validate_custom_popup_titles(&parsed)?;
    Ok(parsed)
}
fn custom_popup(id: &str, value: &JsonValue) -> Result<CustomPopup> {
    validate_custom_popup_id(id)?;
    let path = format!("popups.{id}");
    let table = value
        .as_object()
        .ok_or_else(|| error(format!("{path} must be a table")))?;
    for field in table.keys() {
        if !matches!(
            field.as_str(),
            "command" | "args" | "title" | "keybinding" | "keep_alive"
        ) {
            return Err(error(format!(
                "{path}.{field} is not supported; use command, args, title, keybinding, or keep_alive"
            )));
        }
    }

    let command_path = format!("{path}.command");
    let command = required_string(table, "command", &command_path)?.to_string();
    validate_popup_command(&command_path, &command)?;

    let args = table
        .get("args")
        .map(|value| string_array(&format!("{path}.args"), value))
        .transpose()?
        .unwrap_or_default();

    let title = table
        .get("title")
        .map(|value| nonempty_string(&format!("{path}.title"), value))
        .transpose()?
        .map(str::to_string)
        .unwrap_or_else(|| format!("{id}_popup"));

    let keybinding_path = format!("{path}.keybinding");
    let keybinding = required_string(table, "keybinding", &keybinding_path)?.to_string();
    validate_managed_popup_keybinding(&keybinding_path, &keybinding)?;

    let keep_alive = table
        .get("keep_alive")
        .map(|value| json_bool(&format!("{path}.keep_alive"), value))
        .transpose()?
        .unwrap_or(false);

    Ok(CustomPopup {
        id: id.to_string(),
        command,
        args,
        title,
        keybinding,
        keep_alive,
    })
}
fn validate_custom_popup_id(id: &str) -> Result<()> {
    const BUILTIN_POPUP_IDS: &[&str] = &["config", "agent", "git", "menu"];
    if BUILTIN_POPUP_IDS.contains(&id) {
        return Err(error(format!(
            "popups.{id} conflicts with packaged popup id"
        )));
    }
    let mut chars = id.chars();
    let valid = chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if valid {
        Ok(())
    } else {
        Err(error(format!(
            "popups.{id} id must start with an ASCII letter or _ and contain only ASCII letters, digits, _, or -"
        )))
    }
}
fn validate_custom_popup_titles(popups: &[CustomPopup]) -> Result<()> {
    const BUILTIN_POPUP_TITLES: &[&str] =
        &["config_popup", "agent_popup", "git_popup", "menu_popup"];
    let mut used = BTreeMap::new();
    for popup in popups {
        let title = popup.title.trim();
        let path = format!("popups.{}.title", popup.id);
        if BUILTIN_POPUP_TITLES.contains(&title) {
            return Err(error(format!(
                "{path} conflicts with packaged popup title {title}"
            )));
        }
        if let Some(existing) = used.insert(title.to_string(), popup.id.as_str()) {
            return Err(error(format!(
                "{path} conflicts with popups.{existing}.title: {title}"
            )));
        }
    }
    Ok(())
}
fn validate_popup_command(path: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(error(format!("{path} must not be empty")));
    }
    if value.chars().any(char::is_whitespace) {
        return Err(error(format!(
            "{path} must be one executable command without arguments; use args for arguments"
        )));
    }
    Ok(())
}
fn required_string<'a>(
    table: &'a serde_json::Map<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<&'a str> {
    table
        .get(field)
        .ok_or_else(|| error(format!("{path} is required")))
        .and_then(|value| nonempty_string(path, value))
}
fn nonempty_string<'a>(path: &str, value: &'a JsonValue) -> Result<&'a str> {
    let value = value
        .as_str()
        .ok_or_else(|| error(format!("{path} must be a string")))?;
    if value.trim().is_empty() {
        return Err(error(format!("{path} must not be empty")));
    }
    Ok(value)
}
fn string_array(path: &str, value: &JsonValue) -> Result<Vec<String>> {
    let values = value
        .as_array()
        .ok_or_else(|| error(format!("{path} must be an array of strings")))?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            nonempty_string(&format!("{path}[{index}]"), value).map(str::to_string)
        })
        .collect()
}
