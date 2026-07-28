use std::{fs, path::Path};

use ratconfig::toml_adapter::{set_toml_value_text, unset_toml_value_text};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;
use yazelix_cursors::{CursorRegistry, cursor_config_field_specs};

use crate::{catalog::*, common::*, mars_inventory::MarsInventory, root_config::config_field};

pub(crate) fn write_cursor_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    if !cursor_config_field_specs()
        .iter()
        .any(|spec| spec.path == field_path && spec.kind.is_writable())
    {
        return Err(error(format!("unknown cursor config path: {field_path}")));
    }
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update cursors.toml", error))?
        .text;
    CursorRegistry::parse_str(path, &text)?;
    atomic_write(path, &text)
}
pub(crate) fn unset_cursor_config_field(path: &Path, field_path: &str) -> Result<()> {
    if !cursor_config_field_specs()
        .iter()
        .any(|spec| spec.path == field_path && spec.kind.is_writable())
    {
        return Err(error(format!("unknown cursor config path: {field_path}")));
    }
    let raw = fs::read_to_string(path)?;
    let outcome = unset_toml_value_text(&raw, field_path)
        .map_err(|error| boxed_debug("could not update cursors.toml", error))?;
    CursorRegistry::parse_str(path, &outcome.text)?;
    if outcome.changed() {
        atomic_write(path, &outcome.text)?;
    }
    Ok(())
}
pub(crate) fn write_mars_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    if field_path == MARS_APPEARANCE_PRESET_PATH {
        let appearance = config_field(APPEARANCE_MODE_PATH)?;
        appearance.field.json_choice(value)?;
    } else {
        let inventory = MarsInventory::parse()?;
        let field = inventory
            .field(field_path)
            .ok_or_else(|| error(format!("unknown Mars config path: {field_path}")))?;
        if !field.is_editable() {
            return Err(error(format!(
                "Mars config path {field_path} has no validator-backed inline editor"
            )));
        }
        field.validate(value)?;
    }
    let raw = if path_entry_exists(path)? {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let outcome = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update mars/config.toml", error))?;
    if outcome.changed() {
        atomic_write(path, &outcome.text)?;
    }
    Ok(())
}
pub(crate) fn unset_mars_config_field(path: &Path, field_path: &str) -> Result<()> {
    let inventory = MarsInventory::parse()?;
    let field = inventory
        .field(field_path)
        .ok_or_else(|| error(format!("unknown Mars config path: {field_path}")))?;
    if !field.is_editable() {
        return Err(error(format!(
            "Mars config path {field_path} has no validator-backed inline editor"
        )));
    }
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(path)?;
    let text = unset_toml_value_text(&raw, field_path)
        .map_err(|error| boxed_debug("could not update mars/config.toml", error))?
        .text;
    if text.trim().is_empty() {
        fs::remove_file(path)?;
        Ok(())
    } else {
        atomic_write(path, &text)
    }
}

pub(crate) fn write_starship_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let spec = STARSHIP_FIELDS
        .iter()
        .find(|spec| spec.path == field_path)
        .ok_or_else(|| error(format!("unknown Starship config path: {field_path}")))?;
    spec.json_choice(value)?;
    let raw = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update starship.toml", error))?
        .text;
    atomic_write(path, &text)
}
pub(crate) fn unset_starship_config_field(path: &Path, field_path: &str) -> Result<()> {
    if !STARSHIP_FIELDS.iter().any(|spec| spec.path == field_path) {
        return Err(error(format!("unknown Starship config path: {field_path}")));
    }
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(path)?;
    let text = unset_toml_value_text(&raw, field_path)
        .map_err(|error| boxed_debug("could not update starship.toml", error))?
        .text;
    let value: TomlValue = toml::from_str(&text)
        .map_err(|error| boxed_debug("could not read updated starship.toml", error))?;
    if !toml_has_values(&value) {
        fs::remove_file(path)?;
        Ok(())
    } else {
        atomic_write(path, &text)
    }
}
fn toml_has_values(value: &TomlValue) -> bool {
    match value {
        TomlValue::Table(table) => table.values().any(toml_has_values),
        _ => true,
    }
}
pub(crate) fn write_effective_starship_config(user: &Path, output: &Path) -> Result<()> {
    let mut config: TomlValue = toml::from_str(DEFAULT_STARSHIP_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Starship config", error))?;
    if user.is_file() {
        let overrides = toml::from_str(&fs::read_to_string(user)?)
            .map_err(|error| boxed_debug("invalid user Starship config", error))?;
        deep_merge_toml(&mut config, &overrides);
    }
    let text = toml::to_string_pretty(&config)
        .map_err(|error| boxed_debug("could not serialize effective Starship config", error))?;
    atomic_write(output, &text)
}
