use std::{
    fs, io,
    path::Path,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use ratconfig::ConfigUiField;

use crate::catalog::FieldSpec;

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
impl FieldSpec {
    pub(crate) fn json_choice<'a>(&self, value: &'a JsonValue) -> Result<&'a str> {
        let Some(value) = value.as_str() else {
            return Err(error(format!("{} must be a string", self.path)));
        };
        if !self.allowed_values.is_empty() && !self.allowed_values.contains(&value) {
            return Err(error(format!(
                "{} must be one of: {}",
                self.path,
                self.allowed_values.join(", ")
            )));
        }
        Ok(value)
    }
}
pub(crate) fn kdl_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
pub(crate) fn kdl_string(value: &str) -> String {
    format!("{value:?}")
}
pub(crate) fn json_bool(path: &str, value: &JsonValue) -> Result<bool> {
    value
        .as_bool()
        .ok_or_else(|| error(format!("{path} must be true or false")))
}
pub(crate) fn json_i64(path: &str, value: &JsonValue) -> Result<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .ok_or_else(|| error(format!("{path} must be an integer")))
}
pub(crate) fn json_positive_i64(path: &str, value: &JsonValue) -> Result<i64> {
    let value = json_i64(path, value)?;
    if value <= 0 {
        return Err(error(format!("{path} must be a positive integer")));
    }
    Ok(value)
}
pub(crate) fn atomic_write(path: &Path, text: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| error(format!("{} has no parent directory", path.display())))?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(
        ".{}.tmp-{}-{nonce}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config.toml"),
        process::id()
    ));
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
pub(crate) fn deep_merge_toml(base: &mut TomlValue, overrides: &TomlValue) {
    match (base, overrides) {
        (TomlValue::Table(base), TomlValue::Table(overrides)) => {
            for (key, value) in overrides {
                match base.get_mut(key) {
                    Some(base) => deep_merge_toml(base, value),
                    None => {
                        base.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (base, overrides) => *base = overrides.clone(),
    }
}
pub(crate) fn error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(io::Error::other(message.into()))
}
pub(crate) fn boxed_debug(
    message: &'static str,
    error: impl std::fmt::Debug,
) -> Box<dyn std::error::Error> {
    Box::new(io::Error::other(format!("{message}: {error:?}")))
}

pub(crate) fn reject_read_only_source(path: &Path, source_id: &str) -> Result<()> {
    if path_read_only(path) {
        return Err(error(format!(
            "config source `{source_id}` is read-only: {}",
            path.display()
        )));
    }
    Ok(())
}
pub(crate) fn path_read_only(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().readonly())
        .unwrap_or(false)
}
pub(crate) fn path_entry_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
pub(crate) fn read_optional_text(path: &Path) -> Result<String> {
    if path_entry_exists(path)? {
        Ok(fs::read_to_string(path)?)
    } else {
        Ok(String::new())
    }
}
pub(crate) fn string_values(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

pub(crate) fn retain_toml_leaf_fields(fields: &mut Vec<ConfigUiField>) {
    fields.retain(|field| field.type_label.as_deref() != Some("table"));
}
