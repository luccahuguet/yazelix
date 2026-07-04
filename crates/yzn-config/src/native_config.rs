use std::{fs, path::Path};

use ratconfig::toml_adapter::set_toml_value_text;
use serde_json::Value as JsonValue;

use crate::{catalog::*, common::*};

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
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update mars/config.toml", error))?
        .text;
    atomic_write(path, &text)
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
            let value = spec.json_choice(value)?;
            if mars_color_path(spec.path) {
                validate_hex_color(spec.path, value)
            } else {
                Ok(())
            }
        }
        _ => Err(error(format!("{} must be {}", spec.path, spec.validation))),
    }
}

fn mars_color_path(path: &str) -> bool {
    matches!(
        path,
        "colors.background" | "colors.foreground" | "colors.dim-foreground"
    )
}

fn validate_hex_color(path: &str, value: &str) -> Result<()> {
    if value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
    {
        Ok(())
    } else {
        Err(error(format!("{path} must be a hex color like #111416")))
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
    validate_starship_field(spec, value)?;
    let raw = fs::read_to_string(path)?;
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update starship.toml", error))?
        .text;
    atomic_write(path, &text)
}
pub(crate) fn validate_starship_field(spec: &FieldSpec, value: &JsonValue) -> Result<()> {
    match spec.kind {
        "boolean" => json_bool(spec.path, value).map(|_| ()),
        "string" => spec.json_choice(value).map(|_| ()),
        _ => Err(error(format!("{} must be {}", spec.path, spec.validation))),
    }
}
