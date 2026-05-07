//! Status-bus, status-cache, and status-widget commands for Yazelix/Zellij.

use crate::bridge::{CoreError, ErrorClass};
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use serde_json::{Value, json};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod agent_usage;
pub(super) use agent_usage::*;

pub(super) const STATUS_BUS_SCHEMA_VERSION: i64 = 1;
pub(super) const STATUS_BAR_CACHE_SCHEMA_VERSION: i64 = 1;
pub(super) const ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION: i64 = 1;
pub(super) const DEFAULT_CURSOR_WIDGET_COLOR: &str = "#00ff88";
pub(super) const CURSOR_STATUS_GLYPH: &str = "█";
pub(super) const CURSOR_STATUS_VERTICAL_SPLIT_GLYPH: &str = "▌";
pub(super) const CURSOR_STATUS_HORIZONTAL_SPLIT_GLYPH: &str = "▀";
pub(super) const CURSOR_NAME_ENV: &str = "YAZELIX_CURSOR_NAME";
pub(super) const CURSOR_COLOR_ENV: &str = "YAZELIX_CURSOR_COLOR";
pub(super) const CURSOR_FAMILY_ENV: &str = "YAZELIX_CURSOR_FAMILY";
pub(super) const CURSOR_DIVIDER_ENV: &str = "YAZELIX_CURSOR_DIVIDER";
pub(super) const CURSOR_PRIMARY_COLOR_ENV: &str = "YAZELIX_CURSOR_PRIMARY_COLOR";
pub(super) const CURSOR_SECONDARY_COLOR_ENV: &str = "YAZELIX_CURSOR_SECONDARY_COLOR";
pub(super) const TERMINAL_ENV: &str = "YAZELIX_TERMINAL";

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
pub(super) struct ZellijStatusCacheRefreshTokenusageWindowedArgs {
    path: Option<PathBuf>,
    max_age_seconds: Option<u64>,
    error_backoff_seconds: Option<u64>,
    timeout_ms: Option<u64>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijStatusCacheRefreshCodexUsageArgs {
    path: Option<PathBuf>,
    max_age_seconds: Option<u64>,
    error_backoff_seconds: Option<u64>,
    timeout_ms: Option<u64>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ZellijStatusCacheRefreshOpenCodeGoUsageArgs {
    path: Option<PathBuf>,
    max_age_seconds: Option<u64>,
    error_backoff_seconds: Option<u64>,
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

pub(super) fn parse_zellij_status_cache_refresh_tokenusage_windowed_args(
    args: &[String],
) -> Result<ZellijStatusCacheRefreshTokenusageWindowedArgs, CoreError> {
    let mut parsed = ZellijStatusCacheRefreshTokenusageWindowedArgs::default();
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
            "--timeout-ms" => {
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
                    "Unknown argument for zellij status-cache-refresh-claude-usage: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-cache-refresh-claude-usage accepts only flags".to_string(),
                ));
            }
        }
    }

    Ok(parsed)
}

pub(super) fn parse_zellij_status_cache_refresh_codex_usage_args(
    args: &[String],
) -> Result<ZellijStatusCacheRefreshCodexUsageArgs, CoreError> {
    let mut parsed = ZellijStatusCacheRefreshCodexUsageArgs::default();
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
            "--timeout-ms" => {
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
                    "Unknown argument for zellij status-cache-refresh-codex-usage: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-cache-refresh-codex-usage accepts only flags".to_string(),
                ));
            }
        }
    }

    Ok(parsed)
}

pub(super) fn parse_zellij_status_cache_refresh_opencode_go_usage_args(
    args: &[String],
) -> Result<ZellijStatusCacheRefreshOpenCodeGoUsageArgs, CoreError> {
    let mut parsed = ZellijStatusCacheRefreshOpenCodeGoUsageArgs::default();
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
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij status-cache-refresh-opencode-go-usage: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-cache-refresh-opencode-go-usage accepts only flags".to_string(),
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

pub fn run_zellij_status_cache_refresh_codex_usage(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_refresh_codex_usage_args(args)?;
    if parsed.help {
        print_zellij_status_cache_refresh_codex_usage_help();
        return Ok(0);
    }

    let path = match parsed.path.or_else(status_bar_cache_path_from_env) {
        Some(path) => path,
        None => return Ok(0),
    };
    if !usage_widget_enabled_from_status_cache_path(&path, "codex_usage") {
        return Ok(0);
    }
    let Some(shared_path) = codex_usage_shared_cache_path_from_status_cache_path(&path) else {
        return Ok(0);
    };
    let timeout = Duration::from_millis(parsed.timeout_ms.unwrap_or(5_000).max(1));
    refresh_codex_usage_shared_cache(
        &shared_path,
        env::var_os("PATH").as_deref(),
        unix_time_seconds(),
        parsed.max_age_seconds.unwrap_or(600),
        parsed.error_backoff_seconds.unwrap_or(1_800),
        timeout,
    )?;
    mark_status_cache_refresh_finished(&path, "codex_usage")?;
    Ok(0)
}

pub fn run_zellij_status_cache_refresh_opencode_go_usage(
    args: &[String],
) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_refresh_opencode_go_usage_args(args)?;
    if parsed.help {
        print_zellij_status_cache_refresh_opencode_go_usage_help();
        return Ok(0);
    }

    let path = match parsed.path.or_else(status_bar_cache_path_from_env) {
        Some(path) => path,
        None => return Ok(0),
    };
    if !usage_widget_enabled_from_status_cache_path(&path, "opencode_go_usage") {
        return Ok(0);
    }
    let Some(shared_path) = opencode_go_usage_shared_cache_path_from_status_cache_path(&path)
    else {
        return Ok(0);
    };
    refresh_opencode_go_usage_shared_cache(
        &shared_path,
        unix_time_seconds(),
        parsed.max_age_seconds.unwrap_or(600),
        parsed.error_backoff_seconds.unwrap_or(1_800),
    )?;
    mark_status_cache_refresh_finished(&path, "opencode_go_usage")?;
    Ok(0)
}

pub fn run_zellij_status_cache_refresh_claude_usage(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_refresh_tokenusage_windowed_args(args)?;
    if parsed.help {
        print_zellij_status_cache_refresh_claude_usage_help();
        return Ok(0);
    }

    let path = match parsed.path.or_else(status_bar_cache_path_from_env) {
        Some(path) => path,
        None => return Ok(0),
    };
    if !usage_widget_enabled_from_status_cache_path(&path, "claude_usage") {
        return Ok(0);
    }
    let Some(shared_path) = claude_usage_shared_cache_path_from_status_cache_path(&path) else {
        return Ok(0);
    };
    let timeout = Duration::from_millis(parsed.timeout_ms.unwrap_or(5_000).max(1));
    refresh_tokenusage_windowed_usage_shared_cache(
        &shared_path,
        TokenusageWindowedProvider::Claude,
        env::var_os("PATH").as_deref(),
        unix_time_seconds(),
        parsed.max_age_seconds.unwrap_or(600),
        parsed.error_backoff_seconds.unwrap_or(1_800),
        timeout,
    )?;
    mark_status_cache_refresh_finished(&path, "claude_usage")?;
    Ok(0)
}

pub(super) fn status_bar_cache_path_from_env() -> Option<PathBuf> {
    status_bar_cache_path_from_values(
        env::var_os("YAZELIX_STATUS_BAR_CACHE_PATH").map(PathBuf::from),
        env::var_os("YAZELIX_SESSION_CONFIG_PATH").map(PathBuf::from),
    )
    .or_else(status_bar_cache_path_from_parent_process_env)
}

pub(super) fn status_bar_cache_path_from_values(
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

pub(super) fn session_config_path_from_env() -> Option<PathBuf> {
    session_config_path_from_values(
        env::var_os("YAZELIX_SESSION_CONFIG_PATH")
            .map(PathBuf::from)
            .or_else(session_config_path_from_parent_process_env),
        env::var_os("YAZELIX_STATUS_BAR_CACHE_PATH")
            .map(PathBuf::from)
            .or_else(status_bar_cache_path_from_parent_process_env),
    )
}

pub(super) fn session_config_path_from_values(
    session_config_path: Option<PathBuf>,
    cache_path: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(path) = session_config_path {
        return Some(path);
    }

    cache_path.and_then(|path| {
        path.parent()
            .map(|parent| parent.join("config_snapshot.json"))
    })
}

#[cfg(target_os = "linux")]
pub(super) fn status_bar_cache_path_from_parent_process_env() -> Option<PathBuf> {
    path_from_parent_process_env(status_bar_cache_path_from_environ_bytes)
}

#[cfg(target_os = "linux")]
pub(super) fn session_config_path_from_parent_process_env() -> Option<PathBuf> {
    path_from_parent_process_env(session_config_path_from_environ_bytes)
}

#[cfg(target_os = "linux")]
pub(super) fn path_from_parent_process_env(
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
pub(super) fn status_bar_cache_path_from_parent_process_env() -> Option<PathBuf> {
    None
}

#[cfg(not(target_os = "linux"))]
pub(super) fn session_config_path_from_parent_process_env() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "linux")]
pub(super) fn parent_pid(pid: u32) -> Option<u32> {
    let stat_path = PathBuf::from("/proc").join(pid.to_string()).join("stat");
    let raw = fs::read_to_string(stat_path).ok()?;
    let after_name = raw.rsplit_once(") ")?.1;
    let mut fields = after_name.split_whitespace();
    fields.next()?;
    fields.next()?.parse().ok()
}

pub(super) fn status_bar_cache_path_from_environ_bytes(raw: &[u8]) -> Option<PathBuf> {
    status_bar_cache_path_from_values(
        environ_path_value(raw, b"YAZELIX_STATUS_BAR_CACHE_PATH="),
        session_config_path_from_environ_bytes(raw),
    )
}

pub(super) fn session_config_path_from_environ_bytes(raw: &[u8]) -> Option<PathBuf> {
    environ_path_value(raw, b"YAZELIX_SESSION_CONFIG_PATH=")
}

pub(super) fn environ_path_value(raw: &[u8], prefix: &[u8]) -> Option<PathBuf> {
    raw.split(|byte| *byte == 0).find_map(|item| {
        let value = item.strip_prefix(prefix)?;
        (!value.is_empty()).then(|| PathBuf::from(String::from_utf8_lossy(value).to_string()))
    })
}

pub(super) fn missing_status_bar_cache_path_error() -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_status_bar_cache_path",
        "Yazelix status-bar cache path is not available.",
        "Start a fresh Yazelix window so the launch-scoped session environment is available.",
        json!({}),
    )
}

pub(super) fn build_status_bar_cache_at(status_bus: Value, now: u64) -> Value {
    json!({
        "schema_version": STATUS_BAR_CACHE_SCHEMA_VERSION,
        "updated_at_unix_seconds": now,
        "status_bus": status_bus,
        "agent_usage": {},
    })
}

pub(super) fn merge_status_bar_cache_cursor_value(
    cache: &mut Value,
    previous_cursor: Option<Value>,
) {
    if let Some(cursor) = cursor_status_value_from_env().or(previous_cursor) {
        cache["cursor"] = cursor;
    }
}

pub(super) fn cursor_status_value_from_env() -> Option<Value> {
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

pub(super) fn cursor_status_value(
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

pub(super) fn unix_time_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(super) fn write_status_bar_cache_value(path: &Path, cache: &Value) -> Result<(), CoreError> {
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

pub(super) fn decode_orchestrator_heartbeat_payload(raw: &str) -> Result<Value, CoreError> {
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

pub(super) fn merge_status_bar_cache_orchestrator_heartbeat_value(
    path: &Path,
    heartbeat: Value,
) -> Result<(), CoreError> {
    let Some(mut cache) = read_status_bar_cache_value(path) else {
        return Ok(());
    };
    merge_orchestrator_heartbeat_into_cache(&mut cache, heartbeat);
    write_status_bar_cache_value(path, &cache)
}

pub(super) fn mark_status_cache_refresh_finished(
    path: &Path,
    refresh_name: &str,
) -> Result<(), CoreError> {
    let heartbeat = json!({
        "schema_version": ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION,
        "status_refreshes": {
            refresh_name: {
                "finished_at_unix_seconds": unix_time_seconds(),
            }
        }
    });
    merge_status_bar_cache_orchestrator_heartbeat_value(path, heartbeat)
}

pub(super) fn merge_orchestrator_heartbeat_into_cache(cache: &mut Value, incoming: Value) {
    let existing = cache.get("orchestrator_heartbeat").cloned();
    cache["orchestrator_heartbeat"] = merge_orchestrator_heartbeat_values(existing, incoming);
}

pub(super) fn merge_orchestrator_heartbeat_values(
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

pub(super) fn merge_status_refresh_heartbeat_values(
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

pub(super) fn write_json_value_atomic(
    path: &Path,
    value: &Value,
    error_prefix: &str,
) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                format!("{error_prefix}_parent_create_failed"),
                "Failed to create the Yazelix cache directory.",
                "Check permissions for the Yazelix state directory, then retry.",
                &parent.display().to_string(),
                source,
            )
        })?;
    }

    let serialized = format!(
        "{}\n",
        serde_json::to_string(value).map_err(|error| {
            CoreError::classified(
                ErrorClass::Runtime,
                format!("{error_prefix}_serialize_failed"),
                format!("Failed to serialize Yazelix cache payload: {error}"),
                "Retry the command. If this persists, report the cache payload.",
                json!({ "cache": value.clone() }),
            )
        })?
    );
    let tmp_path = temporary_status_bar_cache_path(path);
    fs::write(&tmp_path, serialized).map_err(|source| {
        CoreError::io(
            format!("{error_prefix}_write_failed"),
            "Failed to write the temporary Yazelix cache file.",
            "Check permissions for the Yazelix state directory, then retry.",
            &tmp_path.display().to_string(),
            source,
        )
    })?;
    fs::rename(&tmp_path, path).map_err(|source| {
        CoreError::io(
            format!("{error_prefix}_rename_failed"),
            "Failed to publish the Yazelix cache file.",
            "Check permissions for the Yazelix state directory, then retry.",
            &path.display().to_string(),
            source,
        )
    })?;
    Ok(())
}

pub(super) fn temporary_status_bar_cache_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("status_bar_cache.json");
    path.with_file_name(format!(".{file_name}.tmp"))
}

pub(super) fn read_status_bar_cache_value(path: &Path) -> Option<Value> {
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

pub(super) fn render_status_bus_workspace_widget(value: &Value) -> String {
    let root = nested_str(value, &["workspace", "root"]).unwrap_or("");
    Path::new(root)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("none")
        .to_string()
}

pub(super) fn render_zjstatus_workspace_widget(value: &Value) -> String {
    if nested_str(value, &["workspace", "root"])
        .map(str::trim)
        .filter(|root| !root.is_empty())
        .is_none()
    {
        return String::new();
    }
    format!(" [{}]", render_status_bus_workspace_widget(value))
}

pub(super) fn render_zjstatus_cursor_widget(cache: &Value) -> String {
    let Some(name) = cache
        .get("cursor")
        .and_then(|cursor| cursor.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(sanitize_zjstatus_cursor_name)
        .filter(|name| !name.is_empty())
    else {
        return String::new();
    };

    let color = cache
        .get("cursor")
        .and_then(|cursor| cursor.get("color"))
        .and_then(Value::as_str)
        .and_then(normalize_status_hex_color)
        .unwrap_or_else(|| DEFAULT_CURSOR_WIDGET_COLOR.to_string());

    if let Some((glyph, primary_color, secondary_color)) =
        cursor_widget_split_preview(cache, &color)
    {
        let glyph_segment = format!("#[fg={primary_color},bg={secondary_color},bold]{glyph}");
        return render_zjstatus_cursor_widget_frame(&color, &glyph_segment, &name);
    }

    let glyph_segment = format!("#[fg={color},bold]{CURSOR_STATUS_GLYPH}");
    render_zjstatus_cursor_widget_frame(&color, &glyph_segment, &name)
}

pub(super) fn render_zjstatus_cursor_widget_frame(
    accent_color: &str,
    glyph_segment: &str,
    name: &str,
) -> String {
    format!(
        " #[fg={accent_color},bg=default,bold][{glyph_segment}#[fg={accent_color},bg=default,bold] {name}]"
    )
}

pub(super) fn normalize_status_hex_color(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    let valid = normalized.len() == 7
        && normalized.starts_with('#')
        && normalized[1..].bytes().all(|byte| byte.is_ascii_hexdigit());
    valid.then_some(normalized)
}

pub(super) fn normalize_status_cursor_family(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "mono" | "split" | "curated_template" => Some(normalized),
        _ => None,
    }
}

pub(super) fn normalize_status_cursor_divider(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "vertical" | "horizontal" => Some(normalized),
        _ => None,
    }
}

pub(super) fn cursor_widget_split_preview(
    cache: &Value,
    fallback_primary_color: &str,
) -> Option<(&'static str, String, String)> {
    let cursor = cache.get("cursor")?;
    let family = cursor.get("family").and_then(Value::as_str)?.trim();
    if family != "split" {
        return None;
    }

    let glyph = match cursor.get("divider").and_then(Value::as_str)?.trim() {
        "vertical" => CURSOR_STATUS_VERTICAL_SPLIT_GLYPH,
        "horizontal" => CURSOR_STATUS_HORIZONTAL_SPLIT_GLYPH,
        _ => return None,
    };
    let primary_color = cursor
        .get("primary_color")
        .and_then(Value::as_str)
        .and_then(normalize_status_hex_color)
        .unwrap_or_else(|| fallback_primary_color.to_string());
    let secondary_color = cursor
        .get("secondary_color")
        .and_then(Value::as_str)
        .and_then(normalize_status_hex_color)?;

    Some((glyph, primary_color, secondary_color))
}

pub(super) fn sanitize_zjstatus_cursor_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '/' | '.'))
        .collect()
}

pub(super) fn print_optional_zjstatus_segment(segment: String) {
    if !segment.is_empty() {
        println!("{segment}");
    }
}

pub(super) fn decode_status_bus_snapshot(raw: &str) -> Result<Value, CoreError> {
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

pub(super) fn render_session_state_inspection_lines(value: &Value) -> Vec<String> {
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

pub(super) fn nested_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) fn nested_bool(value: &Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_bool()
}
