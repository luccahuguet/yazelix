//! Yazelix adapter for reusable ratconfig JSONC edits.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::{Value as JsonValue, json};
use std::path::Path;

pub use yazelix_ratconfig::jsonc::{
    PatchError as SettingsJsoncPatchError, PatchMutation as SettingsJsoncPatchMutation,
    PatchOutcome as SettingsJsoncPatchOutcome, jsonc_parse_options, set_jsonc_value_text,
    unset_jsonc_value_text,
};

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
