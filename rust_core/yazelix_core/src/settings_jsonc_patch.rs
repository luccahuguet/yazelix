//! Comment-preserving edits for the canonical Yazelix settings JSONC file.

use crate::bridge::{CoreError, ErrorClass};
use jsonc_parser::ParseOptions;
use jsonc_parser::cst::{CstInputValue, CstObject, CstRootNode};
use serde_json::{Value as JsonValue, json};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsJsoncPatchMutation {
    Inserted,
    Replaced,
    Removed,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsJsoncPatchOutcome {
    pub text: String,
    pub mutation: SettingsJsoncPatchMutation,
}

impl SettingsJsoncPatchOutcome {
    pub fn changed(&self) -> bool {
        self.mutation != SettingsJsoncPatchMutation::Unchanged
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsJsoncPatchError {
    InvalidJsonc { source: String },
    InvalidPath { path: String },
    RewriteRequired { path: String, detail: String },
    UnsupportedValue { path: String, detail: String },
}

pub fn set_jsonc_value_text(
    raw: &str,
    setting_path: &str,
    value: &JsonValue,
) -> Result<SettingsJsoncPatchOutcome, SettingsJsoncPatchError> {
    let parts = split_setting_path(setting_path)?;
    let replacement = cst_input_from_json_value(value, setting_path)?;
    let root = parse_cst(raw)?;
    let root_object = root.object_value_or_create().ok_or_else(|| {
        rewrite_required(
            setting_path,
            "The settings root is not a JSON object, so this patch cannot be applied without rewriting the file.",
        )
    })?;
    let parent = parent_object_or_create(root_object, &parts, setting_path)?;
    let leaf = parts.last().expect("split path guarantees a leaf");
    let mutation = match parent.get(leaf) {
        Some(prop) => {
            prop.set_value(replacement);
            SettingsJsoncPatchMutation::Replaced
        }
        None => {
            parent.append(leaf, replacement);
            SettingsJsoncPatchMutation::Inserted
        }
    };
    let text = root.to_string();
    let mutation = if text == raw {
        SettingsJsoncPatchMutation::Unchanged
    } else {
        mutation
    };
    validate_jsonc(&text)?;
    Ok(SettingsJsoncPatchOutcome { text, mutation })
}

pub fn unset_jsonc_value_text(
    raw: &str,
    setting_path: &str,
) -> Result<SettingsJsoncPatchOutcome, SettingsJsoncPatchError> {
    let parts = split_setting_path(setting_path)?;
    let root = parse_cst(raw)?;
    let Some(root_object) = root.object_value() else {
        return Ok(SettingsJsoncPatchOutcome {
            text: raw.to_string(),
            mutation: SettingsJsoncPatchMutation::Unchanged,
        });
    };
    let Some(parent) = parent_object_if_present(root_object, &parts, setting_path)? else {
        return Ok(SettingsJsoncPatchOutcome {
            text: raw.to_string(),
            mutation: SettingsJsoncPatchMutation::Unchanged,
        });
    };
    let leaf = parts.last().expect("split path guarantees a leaf");
    let Some(prop) = parent.get(leaf) else {
        return Ok(SettingsJsoncPatchOutcome {
            text: raw.to_string(),
            mutation: SettingsJsoncPatchMutation::Unchanged,
        });
    };
    prop.remove();
    let text = root.to_string();
    validate_jsonc(&text)?;
    Ok(SettingsJsoncPatchOutcome {
        text,
        mutation: SettingsJsoncPatchMutation::Removed,
    })
}

pub fn set_settings_jsonc_value_text(
    source_path: &Path,
    raw: &str,
    setting_path: &str,
    value: &JsonValue,
) -> Result<SettingsJsoncPatchOutcome, CoreError> {
    set_jsonc_value_text(raw, setting_path, value)
        .map_err(|error| patch_error_to_core_error(source_path, error))
}

pub fn unset_settings_jsonc_value_text(
    source_path: &Path,
    raw: &str,
    setting_path: &str,
) -> Result<SettingsJsoncPatchOutcome, CoreError> {
    unset_jsonc_value_text(raw, setting_path)
        .map_err(|error| patch_error_to_core_error(source_path, error))
}

pub(crate) fn jsonc_parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

fn parse_cst(raw: &str) -> Result<CstRootNode, SettingsJsoncPatchError> {
    CstRootNode::parse(raw, &jsonc_parse_options()).map_err(|source| {
        SettingsJsoncPatchError::InvalidJsonc {
            source: source.to_string(),
        }
    })
}

fn validate_jsonc(raw: &str) -> Result<(), SettingsJsoncPatchError> {
    jsonc_parser::parse_to_serde_value::<JsonValue>(raw, &jsonc_parse_options())
        .map(|_| ())
        .map_err(|source| SettingsJsoncPatchError::InvalidJsonc {
            source: source.to_string(),
        })
}

fn split_setting_path(path: &str) -> Result<Vec<String>, SettingsJsoncPatchError> {
    let parts = path
        .split('.')
        .map(str::trim)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if parts.is_empty()
        || parts.iter().any(|part| {
            part.is_empty()
                || part.contains('[')
                || part.contains(']')
                || !part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
    {
        return Err(SettingsJsoncPatchError::InvalidPath {
            path: path.to_string(),
        });
    }
    Ok(parts)
}

fn parent_object_or_create(
    root_object: CstObject,
    parts: &[String],
    setting_path: &str,
) -> Result<CstObject, SettingsJsoncPatchError> {
    let mut current = root_object;
    for part in &parts[..parts.len().saturating_sub(1)] {
        current = current.object_value_or_create(part).ok_or_else(|| {
            rewrite_required(
                setting_path,
                "A parent settings path exists but is not a JSON object, so Yazelix cannot patch through it safely.",
            )
        })?;
    }
    Ok(current)
}

fn parent_object_if_present(
    root_object: CstObject,
    parts: &[String],
    setting_path: &str,
) -> Result<Option<CstObject>, SettingsJsoncPatchError> {
    let mut current = root_object;
    for part in &parts[..parts.len().saturating_sub(1)] {
        let Some(prop) = current.get(part) else {
            return Ok(None);
        };
        let Some(value) = prop.value() else {
            return Err(rewrite_required(
                setting_path,
                "A parent settings path has no value, so Yazelix cannot remove through it safely.",
            ));
        };
        let Some(object) = value.as_object() else {
            return Err(rewrite_required(
                setting_path,
                "A parent settings path exists but is not a JSON object, so Yazelix cannot remove through it safely.",
            ));
        };
        current = object;
    }
    Ok(Some(current))
}

fn cst_input_from_json_value(
    value: &JsonValue,
    setting_path: &str,
) -> Result<CstInputValue, SettingsJsoncPatchError> {
    match value {
        JsonValue::Null => Ok(CstInputValue::Null),
        JsonValue::Bool(value) => Ok(CstInputValue::Bool(*value)),
        JsonValue::Number(value) => Ok(CstInputValue::Number(value.to_string())),
        JsonValue::String(value) => Ok(CstInputValue::String(value.clone())),
        JsonValue::Array(values) => {
            let mut items = Vec::new();
            for value in values {
                let Some(value) = value.as_str() else {
                    return Err(unsupported_value(
                        setting_path,
                        "Only arrays of strings are supported by the safe JSONC patcher.",
                    ));
                };
                items.push(CstInputValue::String(value.to_string()));
            }
            Ok(CstInputValue::Array(items))
        }
        JsonValue::Object(object) => {
            let mut properties = Vec::new();
            for (key, value) in object {
                properties.push((key.clone(), cst_input_from_json_value(value, setting_path)?));
            }
            Ok(CstInputValue::Object(properties))
        }
    }
}

fn unsupported_value(setting_path: &str, detail: &str) -> SettingsJsoncPatchError {
    SettingsJsoncPatchError::UnsupportedValue {
        path: setting_path.to_string(),
        detail: detail.to_string(),
    }
}

fn rewrite_required(setting_path: &str, detail: &str) -> SettingsJsoncPatchError {
    SettingsJsoncPatchError::RewriteRequired {
        path: setting_path.to_string(),
        detail: detail.to_string(),
    }
}

fn patch_error_to_core_error(source_path: &Path, error: SettingsJsoncPatchError) -> CoreError {
    match error {
        SettingsJsoncPatchError::InvalidJsonc { source } => CoreError::classified(
            ErrorClass::Config,
            "invalid_settings_jsonc",
            format!(
                "Could not parse Yazelix settings JSONC at {}: {source}.",
                source_path.display(),
            ),
            "Fix the JSONC syntax in settings.jsonc and retry. Comments must use `//` or `/* ... */`, not `#`.",
            json!({
                "path": source_path.display().to_string(),
                "error": source,
            }),
        ),
        SettingsJsoncPatchError::InvalidPath { path } => CoreError::classified(
            ErrorClass::Usage,
            "invalid_settings_path",
            format!("Invalid Yazelix settings path: {path}."),
            "Use a dotted settings path such as editor.hide_sidebar_on_file_open.",
            json!({ "path": path }),
        ),
        SettingsJsoncPatchError::RewriteRequired { path, detail } => CoreError::classified(
            ErrorClass::Config,
            "settings_jsonc_rewrite_required",
            format!("Yazelix cannot safely patch {path} without rewriting settings.jsonc."),
            detail,
            json!({ "path": path }),
        ),
        SettingsJsoncPatchError::UnsupportedValue { path, detail } => CoreError::classified(
            ErrorClass::Config,
            "unsupported_settings_jsonc_patch_value",
            format!("Yazelix cannot safely patch {path}."),
            detail,
            json!({ "path": path }),
        ),
    }
}
