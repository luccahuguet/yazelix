//! Status-bus, status-cache, and status-widget commands for Yazelix/Zellij.

use crate::bridge::{CoreError, ErrorClass};
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use serde_json::{Value, json};
use std::env;
use std::path::PathBuf;

mod agent_usage;
mod cache;
mod widgets;

pub(super) use agent_usage::*;
pub(super) use cache::*;
pub use widgets::probe_active_tab_session_state;
pub(super) use widgets::*;

pub(super) const STATUS_BUS_SCHEMA_VERSION: i64 = 1;
pub(super) const STATUS_BAR_CACHE_SCHEMA_VERSION: i64 = 1;
pub(super) const TAB_ACTIVITY_SNAPSHOT_SCHEMA_VERSION: i64 = 1;
pub(super) const ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION: i64 = 1;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijInspectSessionArgs {
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijStatusBusArgs {
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijStatusCacheWriteArgs {
    path: Option<PathBuf>,
    payload: Option<String>,
    tab_activity_payload: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijStatusCacheHeartbeatArgs {
    path: Option<PathBuf>,
    payload: Option<String>,
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijStatusCacheWidgetArgs {
    widget: Option<String>,
    path: Option<PathBuf>,
    help: bool,
}

pub(super) fn parse_zellij_inspect_session_args(
    args: &[String],
) -> Result<ZellijInspectSessionArgs, CoreError> {
    let mut parsed = ZellijInspectSessionArgs::default();
    for arg in args {
        match arg.as_str() {
            "--json" => parsed.json = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij inspect-session: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij inspect-session accepts no positional arguments".to_string(),
                ));
            }
        }
    }
    Ok(parsed)
}

pub(super) fn print_zellij_inspect_session_help() {
    println!("Inspect the current tab session state from the pane orchestrator");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij inspect-session [--json]");
}

pub(super) fn parse_zellij_status_bus_args(
    args: &[String],
) -> Result<ZellijStatusBusArgs, CoreError> {
    let mut parsed = ZellijStatusBusArgs::default();
    for arg in args {
        match arg.as_str() {
            "--json" => parsed.json = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij status-bus: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-bus accepts no positional arguments".to_string(),
                ));
            }
        }
    }
    Ok(parsed)
}

pub(super) fn print_zellij_status_bus_help() {
    println!("Read the current versioned Yazelix status-bus snapshot");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-bus [--json]");
}

pub(super) fn parse_zellij_status_cache_write_args(
    args: &[String],
) -> Result<ZellijStatusCacheWriteArgs, CoreError> {
    let mut parsed = ZellijStatusCacheWriteArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--path" => {
                parsed.path = Some(PathBuf::from(
                    iter.next()
                        .ok_or_else(|| CoreError::usage("--path requires a value".to_string()))?
                        .as_str(),
                ));
            }
            "--payload" => {
                parsed.payload = Some(
                    iter.next()
                        .ok_or_else(|| CoreError::usage("--payload requires a value".to_string()))?
                        .to_string(),
                );
            }
            "--tab-activity-payload" => {
                parsed.tab_activity_payload = Some(
                    iter.next()
                        .ok_or_else(|| {
                            CoreError::usage("--tab-activity-payload requires a value".to_string())
                        })?
                        .to_string(),
                );
            }
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij status-cache-write: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-cache-write accepts only --path, --payload, and --tab-activity-payload".to_string(),
                ));
            }
        }
    }

    Ok(parsed)
}

pub(super) fn parse_zellij_status_cache_widget_args(
    args: &[String],
) -> Result<ZellijStatusCacheWidgetArgs, CoreError> {
    let mut parsed = ZellijStatusCacheWidgetArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--path" => {
                parsed.path = Some(PathBuf::from(
                    iter.next()
                        .ok_or_else(|| CoreError::usage("--path requires a value".to_string()))?
                        .as_str(),
                ));
            }
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij status-cache-widget: {other}"
                )));
            }
            other => {
                if parsed.widget.is_some() {
                    return Err(CoreError::usage(
                        "zellij status-cache-widget accepts exactly one widget".to_string(),
                    ));
                }
                parsed.widget = Some(other.to_string());
            }
        }
    }

    Ok(parsed)
}

pub(super) fn parse_zellij_status_cache_heartbeat_args(
    args: &[String],
) -> Result<ZellijStatusCacheHeartbeatArgs, CoreError> {
    let mut parsed = ZellijStatusCacheHeartbeatArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--path" => {
                parsed.path = Some(PathBuf::from(
                    iter.next()
                        .ok_or_else(|| CoreError::usage("--path requires a value".to_string()))?
                        .as_str(),
                ));
            }
            "--payload" => {
                parsed.payload = Some(
                    iter.next()
                        .ok_or_else(|| CoreError::usage("--payload requires a value".to_string()))?
                        .to_string(),
                );
            }
            "--json" => parsed.json = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij status-cache-heartbeat: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-cache-heartbeat accepts only flags".to_string(),
                ));
            }
        }
    }

    Ok(parsed)
}

pub(super) fn print_zellij_status_cache_write_help() {
    println!("Write the window-local cached status-bar facts");
    println!();
    println!("Usage:");
    println!(
        "  yzx_control zellij status-cache-write --payload <json> [--tab-activity-payload <json>] [--path <path>]"
    );
}

pub(super) fn print_zellij_status_cache_widget_help() {
    println!("Render one status-bar widget from the window-local cache");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-cache-widget <widget> [--path <path>]");
}

pub(super) fn print_zellij_status_cache_heartbeat_help() {
    println!("Read or update window-local pane-orchestrator heartbeat facts");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-cache-heartbeat [--json] [--path <path>]");
}

pub fn run_zellij_inspect_session(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_inspect_session_args(args)?;
    if parsed.help {
        print_zellij_inspect_session_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        eprintln!("yzx_control zellij inspect-session only works inside a Yazelix/Zellij session.");
        return Ok(1);
    }

    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    match response.trim() {
        "permissions_denied" => {
            eprintln!("Pane orchestrator permissions are not granted.");
            Ok(1)
        }
        "not_ready" | "missing" => {
            eprintln!("Pane orchestrator session state is not ready yet.");
            Ok(1)
        }
        "invalid_payload" => {
            eprintln!("Pane orchestrator rejected the inspect-session request.");
            Ok(1)
        }
        raw => {
            let value: Value = serde_json::from_str(raw).map_err(|error| {
                CoreError::classified(
                    ErrorClass::Runtime,
                    "invalid_session_state_payload",
                    format!("Pane orchestrator returned invalid session-state JSON: {error}"),
                    "Restart Yazelix and retry. If this persists, rebuild the pane orchestrator wasm.",
                    json!({ "payload": raw }),
                )
            })?;
            if parsed.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
                );
            } else {
                for line in render_session_state_inspection_lines(&value) {
                    println!("{line}");
                }
            }
            Ok(0)
        }
    }
}

pub fn run_zellij_status_bus(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_bus_args(args)?;
    if parsed.help {
        print_zellij_status_bus_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        eprintln!("yzx_control zellij status-bus only works inside a Yazelix/Zellij session.");
        return Ok(1);
    }

    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    let value = decode_status_bus_snapshot(&response)?;
    if parsed.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
        );
    } else {
        println!("Yazelix status bus snapshot");
        for line in render_session_state_inspection_lines(&value) {
            println!("{line}");
        }
    }
    Ok(0)
}

pub fn run_zellij_status_cache_write(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_write_args(args)?;
    if parsed.help {
        print_zellij_status_cache_write_help();
        return Ok(0);
    }

    let payload = parsed.payload.as_deref().ok_or_else(|| {
        CoreError::usage(
            "zellij status-cache-write requires --payload <status-bus-json>".to_string(),
        )
    })?;
    let path = parsed
        .path
        .or_else(status_bar_cache_path_from_env)
        .ok_or_else(missing_status_bar_cache_path_error)?;
    let status_bus = decode_status_bus_snapshot(payload)?;
    let tab_activity = parsed
        .tab_activity_payload
        .as_deref()
        .map(decode_tab_activity_snapshot)
        .transpose()?;
    let now = unix_time_seconds();
    let previous_cache = read_status_bar_cache_value(&path);
    let mut cache = build_status_bar_cache_with_tab_activity_at(status_bus, tab_activity, now);
    if let Some(heartbeat) = previous_cache
        .as_ref()
        .and_then(|cache| cache.get("orchestrator_heartbeat"))
        .cloned()
    {
        cache["orchestrator_heartbeat"] = heartbeat;
    }
    if cache.get("tab_activity").is_none() {
        if let Some(tab_activity) = previous_cache
            .as_ref()
            .and_then(|cache| cache.get("tab_activity"))
            .cloned()
        {
            cache["tab_activity"] = tab_activity;
        }
    }
    write_status_bar_cache_value(&path, &cache)?;
    Ok(0)
}

pub(in crate::zellij_commands) fn decode_tab_activity_snapshot(
    raw: &str,
) -> Result<Value, CoreError> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_tab_activity_payload",
            format!("Invalid Yazelix tab activity payload: {error}"),
            "Restart Yazelix so the status-bar cache producer and consumer agree on tab activity format.",
            json!({ "payload": raw }),
        )
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "missing_tab_activity_schema_version",
                "Yazelix tab activity payload is missing schema_version.",
                "Rebuild the pane orchestrator wasm so consumers can validate the tab activity schema.",
                json!({ "payload": value.clone() }),
            )
        })?;
    if schema_version != TAB_ACTIVITY_SNAPSHOT_SCHEMA_VERSION {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "unsupported_tab_activity_schema_version",
            format!("Unsupported Yazelix tab activity schema_version: {schema_version}."),
            format!(
                "This Yazelix build supports tab activity schema_version {TAB_ACTIVITY_SNAPSHOT_SCHEMA_VERSION}. Update Yazelix or rebuild the pane orchestrator wasm so producer and consumer match."
            ),
            json!({
                "expected": TAB_ACTIVITY_SNAPSHOT_SCHEMA_VERSION,
                "actual": schema_version,
            }),
        ));
    }
    Ok(value)
}

pub fn run_zellij_status_cache_heartbeat(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_heartbeat_args(args)?;
    if parsed.help {
        print_zellij_status_cache_heartbeat_help();
        return Ok(0);
    }

    let path = parsed
        .path
        .or_else(status_bar_cache_path_from_env)
        .ok_or_else(missing_status_bar_cache_path_error)?;

    if let Some(payload) = parsed.payload {
        let heartbeat = decode_orchestrator_heartbeat_payload(&payload)?;
        merge_status_bar_cache_orchestrator_heartbeat_value(&path, heartbeat)?;
        return Ok(0);
    }

    let Some(cache) = read_status_bar_cache_value(&path) else {
        return Ok(0);
    };
    let Some(heartbeat) = cache.get("orchestrator_heartbeat") else {
        if parsed.json {
            println!("{{}}");
        }
        return Ok(0);
    };

    if parsed.json {
        println!(
            "{}",
            serde_json::to_string_pretty(heartbeat).unwrap_or_else(|_| heartbeat.to_string())
        );
    } else {
        println!("{heartbeat}");
    }
    Ok(0)
}

pub fn run_zellij_status_cache_widget(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_widget_args(args)?;
    if parsed.help {
        print_zellij_status_cache_widget_help();
        return Ok(0);
    }

    let widget = parsed.widget.as_deref().ok_or_else(|| {
        CoreError::usage(
            "zellij status-cache-widget requires a widget name. Try `yzx_control zellij status-cache-widget --help`."
                .to_string(),
        )
    })?;
    let path = match parsed.path.or_else(status_bar_cache_path_from_env) {
        Some(path) => path,
        None => return Ok(0),
    };
    let Some(cache) = read_status_bar_cache_value(&path) else {
        return Ok(0);
    };
    print_optional_zjstatus_segment(render_status_cache_widget(&cache, widget)?);
    Ok(0)
}

#[cfg(test)]
mod tests;
