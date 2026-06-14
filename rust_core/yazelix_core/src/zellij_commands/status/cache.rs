//! Status-bar cache path, IO, and heartbeat ownership.

use super::{ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION, STATUS_BAR_CACHE_SCHEMA_VERSION};
use crate::bridge::{CoreError, ErrorClass};
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(in crate::zellij_commands) fn status_bar_cache_path_from_env() -> Option<PathBuf> {
    status_bar_cache_path_from_values(
        env::var_os("YAZELIX_STATUS_BAR_CACHE_PATH").map(PathBuf::from),
        env::var_os("YAZELIX_SESSION_CONFIG_PATH").map(PathBuf::from),
    )
    .or_else(status_bar_cache_path_from_parent_process_env)
}

pub(in crate::zellij_commands) fn status_bar_cache_path_from_values(
    cache_path: Option<PathBuf>,
    session_config_path: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(path) = cache_path {
        return Some(path);
    }

    session_config_path.and_then(|path| {
        path.parent()
            .map(|parent| parent.join("status_bar_cache.json"))
    })
}

#[cfg(target_os = "linux")]
pub(in crate::zellij_commands) fn status_bar_cache_path_from_parent_process_env() -> Option<PathBuf>
{
    path_from_parent_process_env(status_bar_cache_path_from_environ_bytes)
}

#[cfg(target_os = "linux")]
pub(in crate::zellij_commands) fn path_from_parent_process_env(
    extract: fn(&[u8]) -> Option<PathBuf>,
) -> Option<PathBuf> {
    let mut pid = parent_pid(std::process::id())?;
    for _ in 0..4 {
        let env_path = PathBuf::from("/proc").join(pid.to_string()).join("environ");
        if let Ok(raw) = fs::read(env_path) {
            if let Some(path) = extract(&raw) {
                return Some(path);
            }
        }
        let next = parent_pid(pid)?;
        if next == pid || next <= 1 {
            break;
        }
        pid = next;
    }
    None
}

#[cfg(not(target_os = "linux"))]
pub(in crate::zellij_commands) fn status_bar_cache_path_from_parent_process_env() -> Option<PathBuf>
{
    None
}

#[cfg(target_os = "linux")]
pub(in crate::zellij_commands) fn parent_pid(pid: u32) -> Option<u32> {
    let stat_path = PathBuf::from("/proc").join(pid.to_string()).join("stat");
    let raw = fs::read_to_string(stat_path).ok()?;
    let after_name = raw.rsplit_once(") ")?.1;
    let mut fields = after_name.split_whitespace();
    fields.next()?;
    fields.next()?.parse().ok()
}

#[cfg(any(test, target_os = "linux"))]
pub(in crate::zellij_commands) fn status_bar_cache_path_from_environ_bytes(
    raw: &[u8],
) -> Option<PathBuf> {
    status_bar_cache_path_from_values(
        environ_path_value(raw, b"YAZELIX_STATUS_BAR_CACHE_PATH="),
        environ_path_value(raw, b"YAZELIX_SESSION_CONFIG_PATH="),
    )
}

#[cfg(any(test, target_os = "linux"))]
pub(in crate::zellij_commands) fn environ_path_value(raw: &[u8], prefix: &[u8]) -> Option<PathBuf> {
    raw.split(|byte| *byte == 0).find_map(|item| {
        let value = item.strip_prefix(prefix)?;
        (!value.is_empty()).then(|| PathBuf::from(String::from_utf8_lossy(value).to_string()))
    })
}

pub(in crate::zellij_commands) fn missing_status_bar_cache_path_error() -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_status_bar_cache_path",
        "Yazelix status-bar cache path is not available.",
        "Start a fresh Yazelix window so the launch-scoped session environment is available.",
        json!({}),
    )
}

pub(in crate::zellij_commands) fn build_status_bar_cache_at(status_bus: Value, now: u64) -> Value {
    json!({
        "schema_version": STATUS_BAR_CACHE_SCHEMA_VERSION,
        "updated_at_unix_seconds": now,
        "status_bus": status_bus,
        "agent_usage": {},
    })
}

pub(in crate::zellij_commands) fn build_status_bar_cache_with_tab_activity_at(
    status_bus: Value,
    tab_activity: Option<Value>,
    now: u64,
) -> Value {
    let mut cache = build_status_bar_cache_at(status_bus, now);
    if let Some(tab_activity) = tab_activity {
        cache["tab_activity"] = tab_activity;
    }
    cache
}

pub(in crate::zellij_commands) fn unix_time_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(in crate::zellij_commands) fn write_status_bar_cache_value(
    path: &Path,
    cache: &Value,
) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "status_bar_cache_parent_create_failed",
                "Failed to create the Yazelix status-bar cache directory.",
                "Check permissions for the Yazelix state directory, then restart Yazelix.",
                &parent.display().to_string(),
                source,
            )
        })?;
    }

    let serialized = format!(
        "{}\n",
        serde_json::to_string(cache).map_err(|error| {
            CoreError::classified(
                ErrorClass::Runtime,
                "status_bar_cache_serialize_failed",
                format!("Failed to serialize Yazelix status-bar cache: {error}"),
                "Restart Yazelix and retry. If this persists, report the status-bar cache payload.",
                json!({ "cache": cache.clone() }),
            )
        })?
    );
    let tmp_path = temporary_status_bar_cache_path(path);
    fs::write(&tmp_path, serialized).map_err(|source| {
        CoreError::io(
            "status_bar_cache_write_failed",
            "Failed to write the temporary Yazelix status-bar cache file.",
            "Check permissions for the Yazelix state directory, then restart Yazelix.",
            &tmp_path.display().to_string(),
            source,
        )
    })?;
    fs::rename(&tmp_path, path).map_err(|source| {
        CoreError::io(
            "status_bar_cache_rename_failed",
            "Failed to publish the Yazelix status-bar cache file.",
            "Check permissions for the Yazelix state directory, then restart Yazelix.",
            &path.display().to_string(),
            source,
        )
    })?;
    Ok(())
}

pub(in crate::zellij_commands) fn decode_orchestrator_heartbeat_payload(
    raw: &str,
) -> Result<Value, CoreError> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_orchestrator_heartbeat_payload",
            format!("Invalid pane-orchestrator heartbeat payload: {error}"),
            "Restart Yazelix and retry. If this persists, report the heartbeat payload.",
            json!({ "payload": raw }),
        )
    })?;
    if value.get("schema_version").and_then(Value::as_i64)
        != Some(ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION)
    {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "unsupported_orchestrator_heartbeat_schema",
            "Unsupported pane-orchestrator heartbeat schema.",
            "Restart Yazelix so the runtime and pane-orchestrator plugin agree on heartbeat format.",
            json!({ "payload": value }),
        ));
    }
    Ok(value)
}

pub(in crate::zellij_commands) fn merge_status_bar_cache_orchestrator_heartbeat_value(
    path: &Path,
    heartbeat: Value,
) -> Result<(), CoreError> {
    let Some(mut cache) = read_status_bar_cache_value(path) else {
        return Ok(());
    };
    merge_orchestrator_heartbeat_into_cache(&mut cache, heartbeat);
    write_status_bar_cache_value(path, &cache)
}

pub(in crate::zellij_commands) fn merge_orchestrator_heartbeat_into_cache(
    cache: &mut Value,
    incoming: Value,
) {
    let existing = cache.get("orchestrator_heartbeat").cloned();
    cache["orchestrator_heartbeat"] = merge_orchestrator_heartbeat_values(existing, incoming);
}

pub(in crate::zellij_commands) fn merge_orchestrator_heartbeat_values(
    existing: Option<Value>,
    incoming: Value,
) -> Value {
    let Some(Value::Object(mut merged)) = existing else {
        return incoming;
    };
    let Value::Object(incoming_object) = incoming else {
        return Value::Object(merged);
    };

    for (key, value) in incoming_object {
        if key == "status_refreshes" {
            let existing_refreshes = merged.remove("status_refreshes");
            merged.insert(
                key,
                merge_status_refresh_heartbeat_values(existing_refreshes, value),
            );
        } else {
            merged.insert(key, value);
        }
    }

    Value::Object(merged)
}

pub(in crate::zellij_commands) fn merge_status_refresh_heartbeat_values(
    existing: Option<Value>,
    incoming: Value,
) -> Value {
    let Some(Value::Object(mut merged)) = existing else {
        return incoming;
    };
    let Value::Object(incoming_object) = incoming else {
        return Value::Object(merged);
    };

    for (refresh_name, refresh_value) in incoming_object {
        let existing_refresh = merged.remove(&refresh_name);
        let merged_refresh = match (existing_refresh, refresh_value) {
            (Some(Value::Object(mut existing_fields)), Value::Object(incoming_fields)) => {
                for (field, value) in incoming_fields {
                    existing_fields.insert(field, value);
                }
                Value::Object(existing_fields)
            }
            (_, incoming_value) => incoming_value,
        };
        merged.insert(refresh_name, merged_refresh);
    }

    Value::Object(merged)
}

pub(in crate::zellij_commands) fn temporary_status_bar_cache_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("status_bar_cache.json");
    path.with_file_name(format!(".{file_name}.tmp"))
}

pub(in crate::zellij_commands) fn read_status_bar_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64) != Some(STATUS_BAR_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    if status_bar_cache_status_bus(&cache).is_none() {
        return None;
    }
    Some(cache)
}

pub(in crate::zellij_commands) fn status_bar_cache_status_bus(cache: &Value) -> Option<&Value> {
    let status_bus = cache.get("status_bus")?;
    if status_bus.get("schema_version").and_then(Value::as_i64)
        == Some(super::STATUS_BUS_SCHEMA_VERSION)
    {
        Some(status_bus)
    } else {
        None
    }
}
