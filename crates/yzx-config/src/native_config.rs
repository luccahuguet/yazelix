use std::{fs, path::Path};

use ratconfig::toml_adapter::{get_toml_path, set_toml_value_text, unset_toml_value_text};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;
use yazelix_cursors::{CursorRegistry, DEFAULT_CURSOR_CONFIG_TEMPLATE};

use crate::{catalog::*, common::*};

pub(crate) fn cursor_defaults(active: &CursorRegistry) -> Result<CursorRegistry> {
    let mut defaults = CursorRegistry::parse_str(
        Path::new("default-cursors.toml"),
        DEFAULT_CURSOR_CONFIG_TEMPLATE,
    )?;
    defaults
        .enabled_cursors
        .retain(|name| active.definitions.contains_key(name));
    if defaults.enabled_cursors.is_empty() {
        defaults.enabled_cursors.clone_from(&active.enabled_cursors);
    }
    Ok(defaults)
}
pub(crate) fn write_cursor_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    if !CURSOR_FIELDS.iter().any(|spec| spec.path == field_path) {
        return Err(error(format!("unknown cursor config path: {field_path}")));
    }
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update cursors.toml", error))?
        .text;
    CursorRegistry::parse_str(path, &text)?;
    atomic_write(path, &text)
}
pub(crate) fn restore_cursor_config_field(path: &Path, field_path: &str) -> Result<()> {
    let active = yazelix_cursors::load_cursor_config(path)?;
    let defaults = cursor_defaults(&active)?;
    let defaults = serde_json::to_value(defaults)?;
    let value = get_toml_path(&defaults, field_path)
        .ok_or_else(|| error(format!("unknown cursor config path: {field_path}")))?;
    write_cursor_config_field(path, field_path, value)
}

pub(crate) fn write_mars_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let spec = MARS_FIELDS
        .iter()
        .find(|spec| spec.path == field_path)
        .ok_or_else(|| error(format!("unknown Mars config path: {field_path}")))?;
    validate_mars_field(spec, value)?;
    let raw = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update mars/config.toml", error))?
        .text;
    atomic_write(path, &text)
}
pub(crate) fn unset_mars_config_field(path: &Path, field_path: &str) -> Result<()> {
    if !MARS_FIELDS.iter().any(|spec| spec.path == field_path) {
        return Err(error(format!("unknown Mars config path: {field_path}")));
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
pub(crate) fn validate_mars_field(spec: &FieldSpec, value: &JsonValue) -> Result<()> {
    match spec.kind {
        "boolean" if value.is_boolean() => Ok(()),
        "integer" => {
            let value = json_i64(spec.path, value)?;
            if matches!(spec.path, "window.width" | "window.height") && value <= 0 {
                return Err(error(format!("{} must be positive", spec.path)));
            }
            Ok(())
        }
        "float" => {
            let value = value
                .as_f64()
                .ok_or_else(|| error(format!("{} must be {}", spec.path, spec.validation)))?;
            match spec.path {
                "window.opacity" if !(0.0..=1.0).contains(&value) => {
                    Err(error("window.opacity must be between 0.0 and 1.0"))
                }
                "fonts.size" | "line-height" if value <= 0.0 => {
                    Err(error(format!("{} must be positive", spec.path)))
                }
                _ => Ok(()),
            }
        }
        "string" => {
            spec.json_choice(value)?;
            Ok(())
        }
        _ => Err(error(format!("{} must be {}", spec.path, spec.validation))),
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
