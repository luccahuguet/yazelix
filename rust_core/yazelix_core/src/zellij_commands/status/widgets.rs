//! Status-bus, cursor, and zjstatus widget rendering.

use super::STATUS_BUS_SCHEMA_VERSION;
use crate::bridge::{CoreError, ErrorClass};
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use serde_json::{Value, json};
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use yazelix_bar::{CursorWidgetFacts, render_cursor_status_widget};

pub(in crate::zellij_commands) const CURSOR_NAME_ENV: &str = "YAZELIX_CURSOR_NAME";
pub(in crate::zellij_commands) const CURSOR_COLOR_ENV: &str = "YAZELIX_CURSOR_COLOR";
pub(in crate::zellij_commands) const CURSOR_FAMILY_ENV: &str = "YAZELIX_CURSOR_FAMILY";
pub(in crate::zellij_commands) const CURSOR_DIVIDER_ENV: &str = "YAZELIX_CURSOR_DIVIDER";
pub(in crate::zellij_commands) const CURSOR_PRIMARY_COLOR_ENV: &str =
    "YAZELIX_CURSOR_PRIMARY_COLOR";
pub(in crate::zellij_commands) const CURSOR_SECONDARY_COLOR_ENV: &str =
    "YAZELIX_CURSOR_SECONDARY_COLOR";
pub(in crate::zellij_commands) const TERMINAL_ENV: &str = "YAZELIX_TERMINAL";

pub(in crate::zellij_commands) fn cursor_status_value_from_env() -> Option<Value> {
    let terminal = env::var_os(TERMINAL_ENV);
    let cursor_name = env::var_os(CURSOR_NAME_ENV);
    let cursor_color = env::var_os(CURSOR_COLOR_ENV);
    let cursor_family = env::var_os(CURSOR_FAMILY_ENV);
    let cursor_divider = env::var_os(CURSOR_DIVIDER_ENV);
    let cursor_primary_color = env::var_os(CURSOR_PRIMARY_COLOR_ENV);
    let cursor_secondary_color = env::var_os(CURSOR_SECONDARY_COLOR_ENV);
    cursor_status_value(
        terminal.as_deref(),
        cursor_name.as_deref(),
        cursor_color.as_deref(),
        cursor_family.as_deref(),
        cursor_divider.as_deref(),
        cursor_primary_color.as_deref(),
        cursor_secondary_color.as_deref(),
    )
}

pub(in crate::zellij_commands) fn cursor_status_value(
    terminal: Option<&OsStr>,
    cursor_name: Option<&OsStr>,
    cursor_color: Option<&OsStr>,
    cursor_family: Option<&OsStr>,
    cursor_divider: Option<&OsStr>,
    cursor_primary_color: Option<&OsStr>,
    cursor_secondary_color: Option<&OsStr>,
) -> Option<Value> {
    let name = cursor_name
        .map(|value| value.to_string_lossy().trim().to_string())
        .filter(|value| !value.is_empty())?;
    let terminal = terminal
        .map(|value| value.to_string_lossy().trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let mut cursor = json!({
        "terminal": terminal,
        "name": name,
    });
    if let Some(color) = cursor_color
        .map(|value| value.to_string_lossy())
        .and_then(|value| normalize_status_hex_color(value.as_ref()))
    {
        cursor["color"] = json!(color);
    }
    let family = cursor_family
        .map(|value| value.to_string_lossy())
        .and_then(|value| normalize_status_cursor_family(value.as_ref()));
    if let Some(family) = family {
        let is_split = family == "split";
        cursor["family"] = json!(family);
        if is_split {
            if let Some(divider) = cursor_divider
                .map(|value| value.to_string_lossy())
                .and_then(|value| normalize_status_cursor_divider(value.as_ref()))
            {
                cursor["divider"] = json!(divider);
            }
            if let Some(color) = cursor_primary_color
                .map(|value| value.to_string_lossy())
                .and_then(|value| normalize_status_hex_color(value.as_ref()))
            {
                cursor["primary_color"] = json!(color);
            }
            if let Some(color) = cursor_secondary_color
                .map(|value| value.to_string_lossy())
                .and_then(|value| normalize_status_hex_color(value.as_ref()))
            {
                cursor["secondary_color"] = json!(color);
            }
        }
    }

    Some(cursor)
}

pub(in crate::zellij_commands) fn render_status_bus_workspace_widget(value: &Value) -> String {
    let root = nested_str(value, &["workspace", "root"]).unwrap_or("");
    Path::new(root)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("none")
        .to_string()
}

pub(in crate::zellij_commands) fn render_zjstatus_workspace_widget(value: &Value) -> String {
    if nested_str(value, &["workspace", "root"])
        .map(str::trim)
        .filter(|root| !root.is_empty())
        .is_none()
    {
        return String::new();
    }
    format!(" [{}]", render_status_bus_workspace_widget(value))
}

pub(in crate::zellij_commands) fn render_zjstatus_cursor_widget(cache: &Value) -> String {
    render_cursor_status_widget(&cursor_widget_facts_from_cache(cache))
}

pub(in crate::zellij_commands) fn normalize_status_hex_color(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    let valid = normalized.len() == 7
        && normalized.starts_with('#')
        && normalized[1..].bytes().all(|byte| byte.is_ascii_hexdigit());
    valid.then_some(normalized)
}

pub(in crate::zellij_commands) fn normalize_status_cursor_family(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "mono" | "split" | "curated_template" => Some(normalized),
        _ => None,
    }
}

pub(in crate::zellij_commands) fn normalize_status_cursor_divider(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "vertical" | "horizontal" => Some(normalized),
        _ => None,
    }
}

pub(in crate::zellij_commands) fn cursor_widget_facts_from_cache(
    cache: &Value,
) -> CursorWidgetFacts {
    let Some(cursor) = cache.get("cursor") else {
        return CursorWidgetFacts::default();
    };
    CursorWidgetFacts {
        name: cursor
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        color: cursor
            .get("color")
            .and_then(Value::as_str)
            .map(str::to_string),
        family: cursor
            .get("family")
            .and_then(Value::as_str)
            .map(str::to_string),
        divider: cursor
            .get("divider")
            .and_then(Value::as_str)
            .map(str::to_string),
        primary_color: cursor
            .get("primary_color")
            .and_then(Value::as_str)
            .map(str::to_string),
        secondary_color: cursor
            .get("secondary_color")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

pub(in crate::zellij_commands) fn print_optional_zjstatus_segment(segment: String) {
    if !segment.is_empty() {
        println!("{segment}");
    }
}

pub(in crate::zellij_commands) fn decode_status_bus_snapshot(
    raw: &str,
) -> Result<Value, CoreError> {
    match raw.trim() {
        "permissions_denied" => {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "status_bus_permissions_denied",
                "Pane orchestrator permissions are not granted for the status bus.",
                "Run `yzx doctor --fix`, restart Yazelix, and retry.",
                json!({}),
            ));
        }
        "not_ready" | "missing" => {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "status_bus_not_ready",
                "Pane orchestrator status bus is not ready yet.",
                "Wait a moment and retry from inside the affected Yazelix session.",
                json!({}),
            ));
        }
        "invalid_payload" => {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "status_bus_invalid_request",
                "Pane orchestrator rejected the status-bus request.",
                "Restart Yazelix and retry. If this persists, rebuild the pane orchestrator wasm.",
                json!({}),
            ));
        }
        _ => {}
    }

    let value: Value = serde_json::from_str(raw).map_err(|error| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_status_bus_payload",
            format!("Pane orchestrator returned invalid status-bus JSON: {error}"),
            "Restart Yazelix and retry. If this persists, rebuild the pane orchestrator wasm.",
            json!({ "payload": raw }),
        )
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "missing_status_bus_schema_version",
                "Pane orchestrator status-bus payload is missing schema_version.",
                "Rebuild the pane orchestrator wasm so consumers can validate the status schema.",
                json!({ "payload": value.clone() }),
            )
        })?;
    if schema_version != STATUS_BUS_SCHEMA_VERSION {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "unsupported_status_bus_schema_version",
            format!("Unsupported pane-orchestrator status-bus schema_version: {schema_version}."),
            format!(
                "This Yazelix build supports status-bus schema_version {STATUS_BUS_SCHEMA_VERSION}. Update Yazelix or rebuild the pane orchestrator wasm so producer and consumer match."
            ),
            json!({
                "expected": STATUS_BUS_SCHEMA_VERSION,
                "actual": schema_version,
            }),
        ));
    }
    Ok(value)
}

pub fn probe_active_tab_session_state() -> Value {
    if env::var_os("ZELLIJ").is_none() {
        return json!({
            "available": false,
            "reason": "not_in_zellij"
        });
    }

    match run_pane_orchestrator_command("get_active_tab_session_state", "") {
        Ok(response) => match response.trim() {
            "permissions_denied" => json!({
                "available": false,
                "reason": "permissions_denied"
            }),
            "not_ready" | "missing" => json!({
                "available": false,
                "reason": "not_ready"
            }),
            "invalid_payload" => json!({
                "available": false,
                "reason": "invalid_payload"
            }),
            raw => serde_json::from_str(raw).unwrap_or_else(|_| {
                json!({
                    "available": false,
                    "reason": "invalid_json"
                })
            }),
        },
        Err(error) => json!({
            "available": false,
            "reason": "pipe_failed",
            "error": error.message()
        }),
    }
}

pub(in crate::zellij_commands) fn render_session_state_inspection_lines(
    value: &Value,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("Yazelix active tab session state".to_string());
    lines.push(format!(
        "  schema_version: {}",
        value
            .get("schema_version")
            .and_then(Value::as_i64)
            .map(|version| version.to_string())
            .unwrap_or_else(|| "unknown".into())
    ));
    lines.push(format!(
        "  active_tab_position: {}",
        value
            .get("active_tab_position")
            .and_then(Value::as_u64)
            .map(|position| position.to_string())
            .unwrap_or_else(|| "unknown".into())
    ));
    lines.push(format!(
        "  workspace: {} ({})",
        nested_str(value, &["workspace", "root"]).unwrap_or("none"),
        nested_str(value, &["workspace", "source"]).unwrap_or("unknown")
    ));
    lines.push(format!(
        "  focus_context: {}",
        value
            .get("focus_context")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    ));
    lines.push(format!(
        "  layout: active_swap_layout_name={}, sidebar_collapsed={}",
        nested_str(value, &["layout", "active_swap_layout_name"]).unwrap_or("none"),
        nested_bool(value, &["layout", "sidebar_collapsed"])
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into())
    ));
    lines.push(format!(
        "  managed_panes: editor={}, sidebar={}",
        nested_str(value, &["managed_panes", "editor_pane_id"]).unwrap_or("none"),
        nested_str(value, &["managed_panes", "sidebar_pane_id"]).unwrap_or("none")
    ));
    lines.push(format!(
        "  sidebar_yazi: id={}, cwd={}",
        nested_str(value, &["sidebar_yazi", "yazi_id"]).unwrap_or("none"),
        nested_str(value, &["sidebar_yazi", "cwd"]).unwrap_or("none")
    ));
    lines
}

pub(in crate::zellij_commands) fn nested_str<'a>(
    value: &'a Value,
    path: &[&str],
) -> Option<&'a str> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(in crate::zellij_commands) fn nested_bool(value: &Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_bool()
}
