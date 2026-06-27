use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    config_dir_from_env, home_dir_from_env, load_normalized_config_for_control,
    runtime_dir_from_env, state_dir_from_env,
};
use crate::settings_surface::{read_settings_jsonc_value, render_settings_jsonc_value};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn config_override_extra_env(
    config_override: Option<&str>,
) -> Vec<(String, Option<String>)> {
    config_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            vec![(
                "YAZELIX_CONFIG_OVERRIDE".to_string(),
                Some(value.to_string()),
            )]
        })
        .unwrap_or_default()
}

pub(super) fn resolve_cli_config_override(raw: &str) -> Result<String, CoreError> {
    resolve_config_override_path(
        raw,
        &std::env::current_dir().map_err(|source| {
            CoreError::io(
                "config_override_cwd",
                "Could not read the current working directory while resolving --config.",
                "cd into a valid directory, then retry.",
                ".",
                source,
            )
        })?,
        &home_dir_from_env()?,
    )
}

pub(super) fn resolve_config_override_path(
    raw: &str,
    cwd: &Path,
    home: &Path,
) -> Result<String, CoreError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::usage("Missing value for --config."));
    }

    let path = if trimmed == "~" {
        home.to_path_buf()
    } else if let Some(rest) = trimmed.strip_prefix("~/") {
        home.join(rest)
    } else {
        PathBuf::from(trimmed)
    };
    let path = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };
    Ok(path.to_string_lossy().to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SessionConfigOverrideKind {
    Bool,
    Float,
    Int,
    String,
    StringList,
    StringListMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SessionConfigOverrideField {
    pub(super) kind: SessionConfigOverrideKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SessionConfigPatch {
    pub(super) path: String,
    pub(super) value: JsonValue,
}

pub(super) fn prepare_session_config_override(
    base_config_override: Option<&str>,
    with_overrides: &[String],
) -> Result<Option<String>, CoreError> {
    if with_overrides.is_empty() {
        return Ok(base_config_override.map(ToOwned::to_owned));
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    materialize_session_config_override(
        &runtime_dir,
        &config_dir,
        &state_dir,
        base_config_override,
        with_overrides,
    )
    .map(Some)
}

pub(super) fn materialize_session_config_override(
    runtime_dir: &Path,
    config_dir: &Path,
    state_dir: &Path,
    base_config_override: Option<&str>,
    with_overrides: &[String],
) -> Result<String, CoreError> {
    let active_paths = crate::active_config_surface::resolve_active_config_paths(
        runtime_dir,
        config_dir,
        base_config_override,
    )?;
    let contract_fields = load_session_config_override_fields(&active_paths.contract_path)?;
    let mut root = read_settings_jsonc_value(&active_paths.config_file)?;
    for raw in with_overrides {
        let patch = parse_session_config_patch(raw, &contract_fields)?;
        apply_session_config_patch(&mut root, &patch)?;
    }

    let session_dir = state_dir.join("config_overrides").join(format!(
        "session_{}_{}",
        std::process::id(),
        epoch_millis()
    ));
    fs::create_dir_all(&session_dir).map_err(|source| {
        CoreError::io(
            "session_config_override_dir",
            "Could not create the Yazelix one-shot config override directory.",
            "Check permissions for the Yazelix state directory, then retry.",
            session_dir.display().to_string(),
            source,
        )
    })?;
    let session_config = session_dir.join(crate::user_config_paths::SETTINGS_CONFIG);
    let rendered = render_settings_jsonc_value(&root)?;
    fs::write(&session_config, rendered).map_err(|source| {
        CoreError::io(
            "session_config_override_write",
            "Could not write the Yazelix one-shot config override.",
            "Check permissions for the Yazelix state directory, then retry.",
            session_config.display().to_string(),
            source,
        )
    })?;

    let session_config_override = session_config.to_string_lossy().to_string();
    load_normalized_config_for_control(runtime_dir, config_dir, Some(&session_config_override))?;
    Ok(session_config_override)
}

pub(super) fn epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub(super) fn load_session_config_override_fields(
    contract_path: &Path,
) -> Result<HashMap<String, SessionConfigOverrideField>, CoreError> {
    let raw = fs::read_to_string(contract_path).map_err(|source| {
        CoreError::io(
            "read_config_contract",
            "Could not read the Yazelix config contract.",
            "Reinstall Yazelix so config_metadata/main_config_contract.toml is present.",
            contract_path.display().to_string(),
            source,
        )
    })?;
    let contract = raw.parse::<toml::Table>().map_err(|source| {
        CoreError::toml(
            "read_config_contract",
            "Could not parse the Yazelix config contract.",
            "Reinstall Yazelix so config_metadata/main_config_contract.toml is valid.",
            contract_path.display().to_string(),
            source,
        )
    })?;
    let fields = contract
        .get("fields")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Internal,
                "missing_config_contract_fields",
                "Yazelix config contract is missing its fields table.",
                "Report this as a Yazelix internal error.",
                serde_json::json!({ "path": contract_path.display().to_string() }),
            )
        })?;

    let mut parsed = HashMap::new();
    for (path, value) in fields {
        let field = value.as_table().ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Internal,
                "invalid_config_contract_field",
                format!("Yazelix config contract field {path} is not a table."),
                "Report this as a Yazelix internal error.",
                serde_json::json!({ "path": contract_path.display().to_string() }),
            )
        })?;
        let raw_kind = field
            .get("kind")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Internal,
                    "missing_config_contract_kind",
                    format!("Yazelix config contract field {path} has no kind."),
                    "Report this as a Yazelix internal error.",
                    serde_json::json!({ "path": contract_path.display().to_string() }),
                )
            })?;
        let kind = match raw_kind {
            "bool" => SessionConfigOverrideKind::Bool,
            "float" => SessionConfigOverrideKind::Float,
            "int" => SessionConfigOverrideKind::Int,
            "string" => SessionConfigOverrideKind::String,
            "string_list" => SessionConfigOverrideKind::StringList,
            "string_list_map" => SessionConfigOverrideKind::StringListMap,
            _ => continue,
        };
        parsed.insert(path.clone(), SessionConfigOverrideField { kind });
    }
    Ok(parsed)
}

pub(super) fn parse_session_config_patch(
    raw: &str,
    fields: &HashMap<String, SessionConfigOverrideField>,
) -> Result<SessionConfigPatch, CoreError> {
    let (raw_path, raw_value) = raw.split_once('=').ok_or_else(|| {
        CoreError::usage("yzx --with expects key=value, for example `--with editor.command=nvim`.")
    })?;
    let path = raw_path.trim();
    if path.is_empty() {
        return Err(CoreError::usage(
            "yzx --with requires a config path before `=`.",
        ));
    }
    let field = fields.get(path).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Config,
            "unknown_session_config_override",
            format!("Unknown Yazelix config setting for --with: {path}."),
            "Use a supported settings.jsonc path from the Yazelix config contract.",
            serde_json::json!({ "path": path }),
        )
    })?;
    Ok(SessionConfigPatch {
        path: path.to_string(),
        value: parse_session_config_patch_value(path, raw_value, field.kind)?,
    })
}

pub(super) fn parse_session_config_patch_value(
    path: &str,
    raw: &str,
    kind: SessionConfigOverrideKind,
) -> Result<JsonValue, CoreError> {
    match kind {
        SessionConfigOverrideKind::Bool => match raw.trim() {
            "true" => Ok(JsonValue::Bool(true)),
            "false" => Ok(JsonValue::Bool(false)),
            _ => Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_session_config_override_bool",
                format!("Invalid boolean value for --with {path}."),
                "Use `true` or `false`.",
                serde_json::json!({ "path": path, "value": raw }),
            )),
        },
        SessionConfigOverrideKind::Float => {
            let value = raw.trim().parse::<f64>().map_err(|_| {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_float",
                    format!("Invalid float value for --with {path}."),
                    "Use a decimal number.",
                    serde_json::json!({ "path": path, "value": raw }),
                )
            })?;
            serde_json::Number::from_f64(value)
                .map(JsonValue::Number)
                .ok_or_else(|| {
                    CoreError::classified(
                        ErrorClass::Config,
                        "invalid_session_config_override_float",
                        format!("Invalid float value for --with {path}."),
                        "Use a finite decimal number.",
                        serde_json::json!({ "path": path, "value": raw }),
                    )
                })
        }
        SessionConfigOverrideKind::Int => {
            let value = raw.trim().parse::<i64>().map_err(|_| {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_int",
                    format!("Invalid integer value for --with {path}."),
                    "Use a whole number.",
                    serde_json::json!({ "path": path, "value": raw }),
                )
            })?;
            Ok(JsonValue::Number(value.into()))
        }
        SessionConfigOverrideKind::String => Ok(JsonValue::String(raw.to_string())),
        SessionConfigOverrideKind::StringList => {
            let value = serde_json::from_str::<JsonValue>(raw.trim()).map_err(|source| {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_string_list",
                    format!("Invalid string-list value for --with {path}."),
                    "Use a JSON array of strings, for example `[\"editor\", \"workspace\", \"cpu\"]`.",
                    serde_json::json!({
                        "path": path,
                        "value": raw,
                        "error": source.to_string(),
                    }),
                )
            })?;
            let Some(items) = value.as_array() else {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_string_list",
                    format!("Invalid string-list value for --with {path}."),
                    "Use a JSON array of strings, for example `[\"editor\", \"workspace\", \"cpu\"]`.",
                    serde_json::json!({ "path": path, "value": raw }),
                ));
            };
            if items.iter().any(|item| !item.is_string()) {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_string_list",
                    format!("Invalid string-list value for --with {path}."),
                    "Every array item must be a string.",
                    serde_json::json!({ "path": path, "value": raw }),
                ));
            }
            Ok(value)
        }
        SessionConfigOverrideKind::StringListMap => {
            let value = serde_json::from_str::<JsonValue>(raw.trim()).map_err(|source| {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_string_list_map",
                    format!("Invalid string-list-map value for --with {path}."),
                    "Use a JSON object whose values are arrays of strings.",
                    serde_json::json!({
                        "path": path,
                        "value": raw,
                        "error": source.to_string(),
                    }),
                )
            })?;
            let Some(entries) = value.as_object() else {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_string_list_map",
                    format!("Invalid string-list-map value for --with {path}."),
                    "Use a JSON object whose values are arrays of strings.",
                    serde_json::json!({ "path": path, "value": raw }),
                ));
            };
            let all_values_are_string_lists = entries.values().all(|entry| {
                entry
                    .as_array()
                    .map(|items| items.iter().all(JsonValue::is_string))
                    .unwrap_or(false)
            });
            if !all_values_are_string_lists {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_session_config_override_string_list_map",
                    format!("Invalid string-list-map value for --with {path}."),
                    "Every object value must be an array of strings.",
                    serde_json::json!({ "path": path, "value": raw }),
                ));
            }
            Ok(value)
        }
    }
}

pub(super) fn apply_session_config_patch(
    root: &mut JsonValue,
    patch: &SessionConfigPatch,
) -> Result<(), CoreError> {
    let segments = patch.path.split('.').collect::<Vec<_>>();
    let Some((last, parents)) = segments.split_last() else {
        return Err(CoreError::usage(
            "yzx --with requires a non-empty config path.",
        ));
    };
    let mut object = root.as_object_mut().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Config,
            "session_config_override_root_not_object",
            "Yazelix can only apply --with patches to a settings JSON object.",
            "Use a complete settings.jsonc object, then retry.",
            serde_json::json!({}),
        )
    })?;
    for segment in parents {
        let value = object
            .entry((*segment).to_string())
            .or_insert_with(|| JsonValue::Object(JsonMap::new()));
        object = value.as_object_mut().ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "session_config_override_parent_not_object",
                format!(
                    "Cannot apply --with {} because {segment} is not an object.",
                    patch.path
                ),
                "Fix the settings.jsonc structure or choose a supported config path.",
                serde_json::json!({ "path": patch.path, "segment": segment }),
            )
        })?;
    }
    object.insert((*last).to_string(), patch.value.clone());
    Ok(())
}
