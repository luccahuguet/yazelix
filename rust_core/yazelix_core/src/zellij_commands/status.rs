//! Status-bus, status-cache, and status-widget commands for Yazelix/Zellij.

use crate::bridge::{CoreError, ErrorClass};
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use serde_json::{Value, json};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod agent_usage;
mod cache;
mod widgets;

pub(super) use agent_usage::*;
pub(super) use cache::*;
pub use widgets::probe_active_tab_session_state;
pub(super) use widgets::*;

pub(super) const STATUS_BUS_SCHEMA_VERSION: i64 = 1;
pub(super) const STATUS_BAR_CACHE_SCHEMA_VERSION: i64 = 1;
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijStatusCacheRefreshUsageArgs {
    path: Option<PathBuf>,
    max_age_seconds: Option<u64>,
    error_backoff_seconds: Option<u64>,
    timeout_ms: Option<u64>,
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
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij status-cache-write: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-cache-write accepts only --path and --payload".to_string(),
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

pub(super) fn parse_zellij_status_cache_refresh_usage_args(
    args: &[String],
    command_name: &str,
    allow_timeout: bool,
) -> Result<ZellijStatusCacheRefreshUsageArgs, CoreError> {
    let mut parsed = ZellijStatusCacheRefreshUsageArgs::default();
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
            "--max-age-seconds" => {
                let raw = iter.next().ok_or_else(|| {
                    CoreError::usage("--max-age-seconds requires a value".to_string())
                })?;
                parsed.max_age_seconds = Some(raw.parse::<u64>().map_err(|_| {
                    CoreError::usage("--max-age-seconds must be an integer".to_string())
                })?);
            }
            "--error-backoff-seconds" => {
                let raw = iter.next().ok_or_else(|| {
                    CoreError::usage("--error-backoff-seconds requires a value".to_string())
                })?;
                parsed.error_backoff_seconds = Some(raw.parse::<u64>().map_err(|_| {
                    CoreError::usage("--error-backoff-seconds must be an integer".to_string())
                })?);
            }
            "--timeout-ms" if allow_timeout => {
                let raw = iter
                    .next()
                    .ok_or_else(|| CoreError::usage("--timeout-ms requires a value".to_string()))?;
                parsed.timeout_ms = Some(raw.parse::<u64>().map_err(|_| {
                    CoreError::usage("--timeout-ms must be an integer".to_string())
                })?);
            }
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij {command_name}: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(format!(
                    "zellij {command_name} accepts only flags"
                )));
            }
        }
    }

    Ok(parsed)
}

pub(super) fn print_zellij_status_cache_write_help() {
    println!("Write the window-local cached status-bar facts");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-cache-write --payload <json> [--path <path>]");
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

pub(super) fn print_zellij_status_cache_refresh_claude_usage_help() {
    println!("Refresh cached Claude 5h/week usage and quota facts for status-bar widgets");
    println!();
    println!("Usage:");
    println!(
        "  yzx_control zellij status-cache-refresh-claude-usage [--path <path>] [--max-age-seconds <n>] [--error-backoff-seconds <n>] [--timeout-ms <n>]"
    );
}

pub(super) fn print_zellij_status_cache_refresh_codex_usage_help() {
    println!(
        "Refresh cached Codex 5h/week usage, quota, and reset-window facts for status-bar widgets"
    );
    println!();
    println!("Usage:");
    println!(
        "  yzx_control zellij status-cache-refresh-codex-usage [--path <path>] [--max-age-seconds <n>] [--error-backoff-seconds <n>] [--timeout-ms <n>]"
    );
}

pub(super) fn print_zellij_status_cache_refresh_opencode_go_usage_help() {
    println!(
        "Refresh cached OpenCode Go 5h/week/month usage and quota facts for status-bar widgets"
    );
    println!();
    println!("Usage:");
    println!(
        "  yzx_control zellij status-cache-refresh-opencode-go-usage [--path <path>] [--max-age-seconds <n>] [--error-backoff-seconds <n>]"
    );
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
    let previous_cache = read_status_bar_cache_value(&path);
    let now = unix_time_seconds();
    let mut cache = build_status_bar_cache_at(status_bus, now);
    merge_status_bar_cache_cursor_value(
        &mut cache,
        previous_cache
            .as_ref()
            .and_then(|cache| cache.get("cursor"))
            .cloned(),
    );
    if let Some(heartbeat) = previous_cache
        .as_ref()
        .and_then(|cache| cache.get("orchestrator_heartbeat"))
        .cloned()
    {
        cache["orchestrator_heartbeat"] = heartbeat;
    }
    write_status_bar_cache_value(&path, &cache)?;
    Ok(0)
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
    let Some(mut cache) = status_cache_value_for_widget_path(&path, widget, unix_time_seconds())
    else {
        return Ok(0);
    };
    if widget == "claude_usage" {
        hydrate_status_cache_claude_usage(&mut cache, &path);
    } else if widget == "codex_usage" {
        hydrate_status_cache_codex_usage(&mut cache, &path);
    } else if widget == "opencode_go_usage" {
        hydrate_status_cache_opencode_go_usage(&mut cache, &path);
    }
    print_optional_zjstatus_segment(render_status_cache_widget_with_agent_usage_settings(
        &cache,
        widget,
        &agent_usage_widget_settings_from_status_cache_path(&path),
    )?);
    Ok(0)
}

pub(super) fn status_cache_value_for_widget_path(
    path: &Path,
    widget: &str,
    now: u64,
) -> Option<Value> {
    read_status_bar_cache_value(path).or_else(|| first_paint_cache_for_widget(widget, now))
}

pub(super) fn first_paint_cache_for_widget(widget: &str, now: u64) -> Option<Value> {
    if matches!(widget, "claude_usage" | "codex_usage" | "opencode_go_usage") {
        return Some(json!({
            "schema_version": STATUS_BAR_CACHE_SCHEMA_VERSION,
            "updated_at_unix_seconds": now,
            "agent_usage": {},
        }));
    }

    if widget == "cursor" {
        return cursor_status_value_from_env().map(|cursor| {
            json!({
                "schema_version": STATUS_BAR_CACHE_SCHEMA_VERSION,
                "updated_at_unix_seconds": now,
                "agent_usage": {},
                "cursor": cursor,
            })
        });
    }

    None
}

fn run_zellij_status_cache_refresh_agent_usage(
    args: &[String],
    target: AgentUsageRefreshTarget,
    print_help: fn(),
) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_refresh_usage_args(
        args,
        target.command_name(),
        target.allow_timeout(),
    )?;
    if parsed.help {
        print_help();
        return Ok(0);
    }

    let path = match parsed.path.or_else(status_bar_cache_path_from_env) {
        Some(path) => path,
        None => return Ok(0),
    };
    let now = unix_time_seconds();
    refresh_agent_usage_shared_cache_for_status_cache_path(
        target,
        &path,
        env::var_os("PATH").as_deref(),
        now,
        parsed.max_age_seconds.unwrap_or(600),
        parsed.error_backoff_seconds.unwrap_or(1_800),
        Duration::from_millis(parsed.timeout_ms.unwrap_or(5_000).max(1)),
    )?;
    Ok(0)
}

pub fn run_zellij_status_cache_refresh_codex_usage(args: &[String]) -> Result<i32, CoreError> {
    run_zellij_status_cache_refresh_agent_usage(
        args,
        AgentUsageRefreshTarget::Codex,
        print_zellij_status_cache_refresh_codex_usage_help,
    )
}

pub fn run_zellij_status_cache_refresh_opencode_go_usage(
    args: &[String],
) -> Result<i32, CoreError> {
    run_zellij_status_cache_refresh_agent_usage(
        args,
        AgentUsageRefreshTarget::OpenCodeGo,
        print_zellij_status_cache_refresh_opencode_go_usage_help,
    )
}

pub fn run_zellij_status_cache_refresh_claude_usage(args: &[String]) -> Result<i32, CoreError> {
    run_zellij_status_cache_refresh_agent_usage(
        args,
        AgentUsageRefreshTarget::Claude,
        print_zellij_status_cache_refresh_claude_usage_help,
    )
}

#[cfg(test)]
mod tests;
