//! Status-bus, status-cache, and status-widget commands for Yazelix/Zellij.

use crate::bridge::{CoreError, ErrorClass};
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use rusqlite::{Connection, OpenFlags, params};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(super) const STATUS_BUS_SCHEMA_VERSION: i64 = 1;
pub(super) const STATUS_BAR_CACHE_SCHEMA_VERSION: i64 = 1;
pub(super) const ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION: i64 = 1;
pub(super) const CLAUDE_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
pub(super) const CODEX_USAGE_CACHE_SCHEMA_VERSION: i64 = 2;
pub(super) const OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
pub(super) const CODEX_USAGE_WINDOW_SEPARATOR: &str = " · ";
pub(super) const CLAUDE_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub(super) const CODEX_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub(super) const OPENCODE_GO_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub(super) const OPENCODE_GO_PROVIDER_ID: &str = "opencode-go";
pub(super) const OPENCODE_GO_FIVE_HOUR_SECONDS: u64 = 5 * 60 * 60;
pub(super) const OPENCODE_GO_WEEK_SECONDS: u64 = 7 * 24 * 60 * 60;
pub(super) const OPENCODE_GO_MONTH_SECONDS: u64 = 30 * 24 * 60 * 60;
pub(super) const OPENCODE_GO_FIVE_HOUR_LIMIT_USD: f64 = 12.0;
pub(super) const OPENCODE_GO_WEEKLY_LIMIT_USD: f64 = 30.0;
pub(super) const OPENCODE_GO_MONTHLY_LIMIT_USD: f64 = 60.0;
pub(super) const MINUTE_SECONDS: u64 = 60;
pub(super) const HOUR_SECONDS: u64 = 60 * MINUTE_SECONDS;
pub(super) const DAY_SECONDS: u64 = 24 * HOUR_SECONDS;
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
pub(super) struct ZellijStatusBusWorkspaceArgs {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WindowedUsageDisplay {
    Both,
    Token,
    Quota,
}

impl WindowedUsageDisplay {
    fn parse(raw: &str) -> Self {
        match raw.trim() {
            "token" | "tokens" => Self::Token,
            "quota" => Self::Quota,
            _ => Self::Both,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AgentUsageWidgetSettings {
    pub(super) claude_display: WindowedUsageDisplay,
    pub(super) codex_display: WindowedUsageDisplay,
    pub(super) opencode_go_display: WindowedUsageDisplay,
    pub(super) claude_periods: Vec<WindowedUsagePeriod>,
    pub(super) opencode_go_periods: Vec<WindowedUsagePeriod>,
}

impl Default for AgentUsageWidgetSettings {
    fn default() -> Self {
        Self {
            claude_display: WindowedUsageDisplay::Both,
            codex_display: WindowedUsageDisplay::Quota,
            opencode_go_display: WindowedUsageDisplay::Both,
            claude_periods: default_windowed_usage_periods().to_vec(),
            opencode_go_periods: default_opencode_go_usage_periods(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct WindowedUsageFacts {
    pub(super) updated_at_unix_seconds: Option<u64>,
    pub(super) five_hour_tokens: Option<u64>,
    pub(super) weekly_tokens: Option<u64>,
    pub(super) monthly_tokens: Option<u64>,
    pub(super) five_hour_remaining_percent: Option<u64>,
    pub(super) weekly_remaining_percent: Option<u64>,
    pub(super) monthly_remaining_percent: Option<u64>,
    pub(super) five_hour_reset_at_unix_seconds: Option<u64>,
    pub(super) weekly_reset_at_unix_seconds: Option<u64>,
    pub(super) five_hour_window_seconds: Option<u64>,
    pub(super) weekly_window_seconds: Option<u64>,
    pub(super) error: Option<String>,
}

impl WindowedUsageFacts {
    fn has_tokens(&self) -> bool {
        self.five_hour_tokens.is_some()
            || self.weekly_tokens.is_some()
            || self.monthly_tokens.is_some()
    }

    fn has_quota(&self) -> bool {
        self.five_hour_remaining_percent.is_some()
            || self.weekly_remaining_percent.is_some()
            || self.monthly_remaining_percent.is_some()
    }

    fn is_empty(&self) -> bool {
        !self.has_tokens() && !self.has_quota()
    }

    fn codex_window_reset_label(&self, period: WindowedUsagePeriod) -> Option<String> {
        let now = self.updated_at_unix_seconds?;
        let (reset_at, window_seconds) = match period {
            WindowedUsagePeriod::FiveHour => (
                self.five_hour_reset_at_unix_seconds?,
                self.five_hour_window_seconds?,
            ),
            WindowedUsagePeriod::Weekly => (
                self.weekly_reset_at_unix_seconds?,
                self.weekly_window_seconds?,
            ),
            WindowedUsagePeriod::Monthly => return None,
        };
        format_reset_window_label(reset_at, window_seconds, now)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenusageWindowedProvider {
    Claude,
    Codex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WindowedUsagePeriod {
    FiveHour,
    Weekly,
    Monthly,
}

impl WindowedUsagePeriod {
    fn parse_config(raw: &str) -> Option<Self> {
        match raw.trim() {
            "5h" | "five_hour" | "five-hour" | "rolling" => Some(Self::FiveHour),
            "week" | "weekly" | "wk" => Some(Self::Weekly),
            "month" | "monthly" | "mon" | "mo" => Some(Self::Monthly),
            _ => None,
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            Self::FiveHour => "5h",
            Self::Weekly => "wk",
            Self::Monthly => "mo",
        }
    }
}

pub(super) fn default_windowed_usage_periods() -> &'static [WindowedUsagePeriod] {
    &[WindowedUsagePeriod::FiveHour, WindowedUsagePeriod::Weekly]
}

pub(super) fn default_opencode_go_usage_periods() -> Vec<WindowedUsagePeriod> {
    vec![
        WindowedUsagePeriod::FiveHour,
        WindowedUsagePeriod::Weekly,
        WindowedUsagePeriod::Monthly,
    ]
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

pub(super) fn parse_zellij_status_bus_workspace_args(
    args: &[String],
) -> Result<ZellijStatusBusWorkspaceArgs, CoreError> {
    let mut parsed = ZellijStatusBusWorkspaceArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij status-bus-workspace: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-bus-workspace accepts no positional arguments".to_string(),
                ));
            }
        }
    }
    Ok(parsed)
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

pub(super) fn print_zellij_status_bus_workspace_help() {
    println!("Render the workspace status-bus fact for zjstatus");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-bus-workspace");
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

pub fn run_zellij_status_bus_workspace(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_bus_workspace_args(args)?;
    if parsed.help {
        print_zellij_status_bus_workspace_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        return Ok(0);
    }

    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    let value = decode_status_bus_snapshot(&response)?;
    print_optional_zjstatus_segment(render_zjstatus_workspace_widget(&value));
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

pub(super) fn codex_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    agent_usage_shared_cache_path_from_status_cache_path(
        status_cache_path,
        "codex_usage_cache",
        CODEX_USAGE_CACHE_SCHEMA_VERSION,
    )
}

pub(super) fn claude_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    agent_usage_shared_cache_path_from_status_cache_path(
        status_cache_path,
        "claude_usage_cache",
        CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
    )
}

pub(super) fn opencode_go_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    agent_usage_shared_cache_path_from_status_cache_path(
        status_cache_path,
        "opencode_go_usage_cache",
        OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
    )
}

pub(super) fn agent_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
    file_stem: &str,
    schema_version: i64,
) -> Option<PathBuf> {
    let state_dir = status_cache_path.parent()?.parent()?.parent()?;
    Some(
        state_dir
            .join("agent_usage")
            .join(format!("{file_stem}_v{schema_version}.json")),
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

pub(super) fn hydrate_status_cache_codex_usage(cache: &mut Value, status_cache_path: &Path) {
    let Some(shared_path) = codex_usage_shared_cache_path_from_status_cache_path(status_cache_path)
    else {
        return;
    };
    let Some(shared_cache) = read_codex_usage_shared_cache_value(&shared_path) else {
        return;
    };
    let Some(codex) = shared_cache.get("codex").cloned() else {
        return;
    };
    cache["codex_usage"] = codex;
}

pub(super) fn hydrate_status_cache_claude_usage(cache: &mut Value, status_cache_path: &Path) {
    let Some(shared_path) =
        claude_usage_shared_cache_path_from_status_cache_path(status_cache_path)
    else {
        return;
    };
    let Some(shared_cache) = read_claude_usage_shared_cache_value(&shared_path) else {
        return;
    };
    let Some(claude) = shared_cache.get("claude").cloned() else {
        return;
    };
    cache["claude_usage"] = claude;
}

pub(super) fn hydrate_status_cache_opencode_go_usage(cache: &mut Value, status_cache_path: &Path) {
    let Some(shared_path) =
        opencode_go_usage_shared_cache_path_from_status_cache_path(status_cache_path)
    else {
        return;
    };
    let Some(shared_cache) = read_opencode_go_usage_shared_cache_value(&shared_path) else {
        return;
    };
    let Some(opencode_go) = shared_cache.get("opencode_go").cloned() else {
        return;
    };
    cache["opencode_go_usage"] = opencode_go;
}

pub(super) fn read_codex_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64) != Some(CODEX_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub(super) fn read_claude_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64)
        != Some(CLAUDE_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub(super) fn read_opencode_go_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64)
        != Some(OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub(super) fn read_tokenusage_windowed_usage_shared_cache_value(
    path: &Path,
    provider: TokenusageWindowedProvider,
) -> Option<Value> {
    match provider {
        TokenusageWindowedProvider::Claude => read_claude_usage_shared_cache_value(path),
        TokenusageWindowedProvider::Codex => read_codex_usage_shared_cache_value(path),
    }
}

pub(super) fn tokenusage_windowed_usage_cache_schema_version(
    provider: TokenusageWindowedProvider,
) -> i64 {
    match provider {
        TokenusageWindowedProvider::Claude => CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
        TokenusageWindowedProvider::Codex => CODEX_USAGE_CACHE_SCHEMA_VERSION,
    }
}

pub(super) fn tokenusage_windowed_usage_cache_key(
    provider: TokenusageWindowedProvider,
) -> &'static str {
    match provider {
        TokenusageWindowedProvider::Claude => "claude",
        TokenusageWindowedProvider::Codex => "codex",
    }
}

pub(super) fn tokenusage_windowed_usage_label(
    provider: TokenusageWindowedProvider,
) -> &'static str {
    match provider {
        TokenusageWindowedProvider::Claude => "Claude",
        TokenusageWindowedProvider::Codex => "Codex",
    }
}

pub(super) fn tokenusage_windowed_usage_error_prefix(
    provider: TokenusageWindowedProvider,
) -> &'static str {
    match provider {
        TokenusageWindowedProvider::Claude => "claude_usage_cache",
        TokenusageWindowedProvider::Codex => "codex_usage_cache",
    }
}

pub(super) fn tokenusage_windowed_usage_lock_name(provider: TokenusageWindowedProvider) -> String {
    match provider {
        TokenusageWindowedProvider::Claude => format!(
            ".claude_usage_cache_v{}.lock",
            CLAUDE_USAGE_CACHE_SCHEMA_VERSION
        ),
        TokenusageWindowedProvider::Codex => format!(
            ".codex_usage_cache_v{}.lock",
            CODEX_USAGE_CACHE_SCHEMA_VERSION
        ),
    }
}

pub(super) fn tokenusage_windowed_usage_lock_stale_after_seconds(
    provider: TokenusageWindowedProvider,
) -> u64 {
    match provider {
        TokenusageWindowedProvider::Claude => CLAUDE_USAGE_LOCK_STALE_AFTER_SECONDS,
        TokenusageWindowedProvider::Codex => CODEX_USAGE_LOCK_STALE_AFTER_SECONDS,
    }
}

pub(super) fn status_bar_cache_status_bus(cache: &Value) -> Option<&Value> {
    let status_bus = cache.get("status_bus")?;
    if status_bus.get("schema_version").and_then(Value::as_i64) == Some(STATUS_BUS_SCHEMA_VERSION) {
        Some(status_bus)
    } else {
        None
    }
}

#[cfg(test)]
pub(super) fn render_status_cache_widget(cache: &Value, widget: &str) -> Result<String, CoreError> {
    render_status_cache_widget_with_agent_usage_settings(
        cache,
        widget,
        &AgentUsageWidgetSettings::default(),
    )
}

pub(super) fn render_status_cache_widget_with_agent_usage_settings(
    cache: &Value,
    widget: &str,
    settings: &AgentUsageWidgetSettings,
) -> Result<String, CoreError> {
    let status_bus = status_bar_cache_status_bus(cache);
    match widget {
        "workspace" => Ok(status_bus
            .map(render_zjstatus_workspace_widget)
            .unwrap_or_default()),
        "cursor" => Ok(render_zjstatus_cursor_widget(cache)),
        "claude_usage" => Ok(render_windowed_usage_segment(
            cache,
            "claude_usage",
            "claude",
            settings.claude_periods.as_slice(),
            settings.claude_display,
        )),
        "codex_usage" => Ok(render_codex_usage_segment(cache, settings.codex_display)),
        "opencode_go_usage" => Ok(render_windowed_usage_segment(
            cache,
            "opencode_go_usage",
            "go",
            settings.opencode_go_periods.as_slice(),
            settings.opencode_go_display,
        )),
        _ => Err(CoreError::usage(format!(
            "zellij status-cache-widget requires one of: {}",
            status_cache_widget_names().join(", ")
        ))),
    }
}

pub(super) fn status_cache_widget_names() -> Vec<&'static str> {
    vec![
        "workspace",
        "cursor",
        "claude_usage",
        "codex_usage",
        "opencode_go_usage",
    ]
}

pub(super) fn render_codex_usage_segment(cache: &Value, display: WindowedUsageDisplay) -> String {
    let Some(entry) = cache.get("codex_usage") else {
        return String::new();
    };
    let facts = windowed_usage_facts_from_cache_entry(entry);
    let summary = render_codex_usage_summary(&facts, display);
    if summary.is_empty() {
        String::new()
    } else {
        render_agent_usage_widget("codex", &summary)
    }
}

pub(super) fn render_codex_usage_summary(
    facts: &WindowedUsageFacts,
    display: WindowedUsageDisplay,
) -> String {
    let mut parts = Vec::new();
    for period in default_windowed_usage_periods() {
        let (tokens, remaining_percent) = match period {
            WindowedUsagePeriod::FiveHour => {
                (facts.five_hour_tokens, facts.five_hour_remaining_percent)
            }
            WindowedUsagePeriod::Weekly => (facts.weekly_tokens, facts.weekly_remaining_percent),
            WindowedUsagePeriod::Monthly => (facts.monthly_tokens, facts.monthly_remaining_percent),
        };
        let label = facts
            .codex_window_reset_label(*period)
            .unwrap_or_else(|| period.short_label().to_string());
        if let Some(part) = render_codex_usage_window(&label, tokens, remaining_percent, display) {
            parts.push(part);
        }
    }
    parts.join(CODEX_USAGE_WINDOW_SEPARATOR)
}

pub(super) fn render_codex_usage_window(
    label: &str,
    tokens: Option<u64>,
    remaining_percent: Option<u64>,
    display: WindowedUsageDisplay,
) -> Option<String> {
    let mut pieces = vec![label.to_string()];
    match display {
        WindowedUsageDisplay::Token => {
            pieces.push(format_agent_usage_token_count(tokens?));
        }
        WindowedUsageDisplay::Quota => {
            pieces.push(match remaining_percent {
                Some(percent) => format_quota_percent(percent),
                None if tokens.is_some() => "n/a".to_string(),
                None => return None,
            });
        }
        WindowedUsageDisplay::Both => {
            if let Some(tokens) = tokens {
                pieces.push(format_agent_usage_token_count(tokens));
            }
            if let Some(remaining_percent) = remaining_percent {
                pieces.push(format_quota_percent(remaining_percent));
            }
            if pieces.len() == 1 {
                return None;
            }
        }
    }
    Some(pieces.join(" "))
}

pub(super) fn render_windowed_usage_segment(
    cache: &Value,
    cache_key: &str,
    label: &str,
    periods: &[WindowedUsagePeriod],
    display: WindowedUsageDisplay,
) -> String {
    let Some(entry) = cache.get(cache_key) else {
        return String::new();
    };
    let facts = windowed_usage_facts_from_cache_entry(entry);
    let summary = render_windowed_usage_summary(&facts, periods, display);
    if summary.is_empty() {
        String::new()
    } else {
        render_agent_usage_widget(label, &summary)
    }
}

pub(super) fn render_windowed_usage_summary(
    facts: &WindowedUsageFacts,
    periods: &[WindowedUsagePeriod],
    display: WindowedUsageDisplay,
) -> String {
    let mut parts = Vec::new();
    for period in periods {
        let (tokens, remaining_percent) = match period {
            WindowedUsagePeriod::FiveHour => {
                (facts.five_hour_tokens, facts.five_hour_remaining_percent)
            }
            WindowedUsagePeriod::Weekly => (facts.weekly_tokens, facts.weekly_remaining_percent),
            WindowedUsagePeriod::Monthly => (facts.monthly_tokens, facts.monthly_remaining_percent),
        };
        if let Some(part) =
            render_windowed_usage_window(period.short_label(), tokens, remaining_percent, display)
        {
            parts.push(part);
        }
    }
    parts.join(" ")
}

pub(super) fn render_windowed_usage_window(
    label: &str,
    tokens: Option<u64>,
    remaining_percent: Option<u64>,
    display: WindowedUsageDisplay,
) -> Option<String> {
    let mut pieces = vec![label.to_string()];
    match display {
        WindowedUsageDisplay::Token => {
            pieces.push(format_agent_usage_token_count(tokens?));
        }
        WindowedUsageDisplay::Quota => {
            pieces.push(match remaining_percent {
                Some(percent) => format_quota_percent(percent),
                None if tokens.is_some() => "n/a".to_string(),
                None => return None,
            });
        }
        WindowedUsageDisplay::Both => {
            if let Some(tokens) = tokens {
                pieces.push(format_agent_usage_token_count(tokens));
            }
            if let Some(remaining_percent) = remaining_percent {
                pieces.push(format_quota_percent(remaining_percent));
            }
            if pieces.len() == 1 {
                return None;
            }
        }
    }
    Some(pieces.join("|"))
}

pub(super) fn windowed_usage_facts_from_cache_entry(entry: &Value) -> WindowedUsageFacts {
    WindowedUsageFacts {
        updated_at_unix_seconds: entry.get("updated_at_unix_seconds").and_then(Value::as_u64),
        five_hour_tokens: entry.get("five_hour_tokens").and_then(Value::as_u64),
        weekly_tokens: entry.get("weekly_tokens").and_then(Value::as_u64),
        monthly_tokens: entry.get("monthly_tokens").and_then(Value::as_u64),
        five_hour_remaining_percent: entry
            .get("five_hour_remaining_percent")
            .and_then(Value::as_u64),
        weekly_remaining_percent: entry
            .get("weekly_remaining_percent")
            .and_then(Value::as_u64),
        monthly_remaining_percent: entry
            .get("monthly_remaining_percent")
            .and_then(Value::as_u64),
        five_hour_reset_at_unix_seconds: entry
            .get("five_hour_reset_at_unix_seconds")
            .and_then(Value::as_u64),
        weekly_reset_at_unix_seconds: entry
            .get("weekly_reset_at_unix_seconds")
            .and_then(Value::as_u64),
        five_hour_window_seconds: entry
            .get("five_hour_window_seconds")
            .and_then(Value::as_u64),
        weekly_window_seconds: entry.get("weekly_window_seconds").and_then(Value::as_u64),
        error: entry
            .get("error")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

pub(super) fn refresh_codex_usage_shared_cache(
    shared_path: &Path,
    path_var: Option<&OsStr>,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
    timeout: Duration,
) -> Result<bool, CoreError> {
    refresh_tokenusage_windowed_usage_shared_cache(
        shared_path,
        TokenusageWindowedProvider::Codex,
        path_var,
        now,
        max_age_seconds,
        error_backoff_seconds,
        timeout,
    )
}

pub(super) fn refresh_tokenusage_windowed_usage_shared_cache(
    shared_path: &Path,
    provider: TokenusageWindowedProvider,
    path_var: Option<&OsStr>,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
    timeout: Duration,
) -> Result<bool, CoreError> {
    if tokenusage_windowed_usage_shared_cache_is_fresh(shared_path, provider, now, max_age_seconds)
    {
        return Ok(false);
    }
    if tokenusage_windowed_usage_shared_cache_is_backing_off(shared_path, provider, now) {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_tokenusage_windowed_usage_cache_lock(shared_path, provider, now)?
    else {
        return Ok(false);
    };
    if tokenusage_windowed_usage_shared_cache_is_fresh(shared_path, provider, now, max_age_seconds)
        || tokenusage_windowed_usage_shared_cache_is_backing_off(shared_path, provider, now)
    {
        return Ok(false);
    }

    let quota_backoff_until =
        tokenusage_windowed_usage_quota_backoff_until(shared_path, provider, now);
    let previous_facts = read_tokenusage_windowed_usage_shared_cache_value(shared_path, provider)
        .and_then(|cache| {
            cache
                .get(tokenusage_windowed_usage_cache_key(provider))
                .map(windowed_usage_facts_from_cache_entry)
        });
    let mut facts = collect_tokenusage_windowed_usage_facts(
        provider,
        path_var,
        timeout,
        quota_backoff_until.is_none(),
    );
    let quota_probe_failed = quota_backoff_until.is_none() && !facts.has_quota();
    preserve_previous_tokenusage_window_tokens(provider, &mut facts, previous_facts.as_ref());
    preserve_previous_tokenusage_window_quota(provider, &mut facts, previous_facts.as_ref(), now);
    let mut entry = serde_json::Map::new();
    entry.insert("updated_at_unix_seconds".to_string(), json!(now));
    if let Some(tokens) = facts.five_hour_tokens {
        entry.insert("five_hour_tokens".to_string(), json!(tokens));
    }
    if let Some(tokens) = facts.weekly_tokens {
        entry.insert("weekly_tokens".to_string(), json!(tokens));
    }
    if let Some(percent) = facts.five_hour_remaining_percent {
        entry.insert("five_hour_remaining_percent".to_string(), json!(percent));
    }
    if let Some(percent) = facts.weekly_remaining_percent {
        entry.insert("weekly_remaining_percent".to_string(), json!(percent));
    }
    if let Some(reset_at) = facts.five_hour_reset_at_unix_seconds {
        entry.insert(
            "five_hour_reset_at_unix_seconds".to_string(),
            json!(reset_at),
        );
    }
    if let Some(reset_at) = facts.weekly_reset_at_unix_seconds {
        entry.insert("weekly_reset_at_unix_seconds".to_string(), json!(reset_at));
    }
    if let Some(window_seconds) = facts.five_hour_window_seconds {
        entry.insert(
            "five_hour_window_seconds".to_string(),
            json!(window_seconds),
        );
    }
    if let Some(window_seconds) = facts.weekly_window_seconds {
        entry.insert("weekly_window_seconds".to_string(), json!(window_seconds));
    }
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        entry.insert("error".to_string(), json!(error));
        if facts.is_empty() {
            entry.insert(
                "backoff_until_unix_seconds".to_string(),
                json!(now.saturating_add(error_backoff_seconds)),
            );
        }
    }
    if let Some(backoff_until) = quota_backoff_until {
        entry.insert(
            "quota_backoff_until_unix_seconds".to_string(),
            json!(backoff_until),
        );
    } else if facts.has_tokens() && (quota_probe_failed || !facts.has_quota()) {
        entry.insert(
            "quota_backoff_until_unix_seconds".to_string(),
            json!(now.saturating_add(error_backoff_seconds)),
        );
    }
    let status = if facts.is_empty() {
        "error"
    } else if facts.has_tokens()
        && facts.has_quota()
        && !quota_probe_failed
        && quota_backoff_until.is_none()
    {
        "ok"
    } else {
        "partial"
    };
    entry.insert("status".to_string(), json!(status));

    let cache = json!({
        "schema_version": tokenusage_windowed_usage_cache_schema_version(provider),
        tokenusage_windowed_usage_cache_key(provider): Value::Object(entry),
    });
    write_json_value_atomic(
        shared_path,
        &cache,
        tokenusage_windowed_usage_error_prefix(provider),
    )?;
    Ok(true)
}

pub(super) fn preserve_previous_tokenusage_window_tokens(
    provider: TokenusageWindowedProvider,
    facts: &mut WindowedUsageFacts,
    previous: Option<&WindowedUsageFacts>,
) {
    let Some(previous) = previous else {
        return;
    };
    if !tokenusage_windowed_usage_facts_are_complete(provider, previous) {
        return;
    }

    if facts.five_hour_tokens.is_none()
        && tokenusage_window_identity_matches(
            facts.five_hour_reset_at_unix_seconds,
            facts.five_hour_window_seconds,
            previous.five_hour_reset_at_unix_seconds,
            previous.five_hour_window_seconds,
        )
    {
        facts.five_hour_tokens = previous.five_hour_tokens;
    }
    if facts.weekly_tokens.is_none()
        && tokenusage_window_identity_matches(
            facts.weekly_reset_at_unix_seconds,
            facts.weekly_window_seconds,
            previous.weekly_reset_at_unix_seconds,
            previous.weekly_window_seconds,
        )
    {
        facts.weekly_tokens = previous.weekly_tokens;
    }
}

pub(super) fn preserve_previous_tokenusage_window_quota(
    provider: TokenusageWindowedProvider,
    facts: &mut WindowedUsageFacts,
    previous: Option<&WindowedUsageFacts>,
    now: u64,
) {
    let Some(previous) = previous else {
        return;
    };

    if facts.five_hour_remaining_percent.is_none()
        && previous_quota_window_is_still_valid(
            provider,
            previous.five_hour_reset_at_unix_seconds,
            previous.five_hour_window_seconds,
            now,
        )
    {
        facts.five_hour_remaining_percent = previous.five_hour_remaining_percent;
        facts.five_hour_reset_at_unix_seconds = previous.five_hour_reset_at_unix_seconds;
        facts.five_hour_window_seconds = previous.five_hour_window_seconds;
    }
    if facts.weekly_remaining_percent.is_none()
        && previous_quota_window_is_still_valid(
            provider,
            previous.weekly_reset_at_unix_seconds,
            previous.weekly_window_seconds,
            now,
        )
    {
        facts.weekly_remaining_percent = previous.weekly_remaining_percent;
        facts.weekly_reset_at_unix_seconds = previous.weekly_reset_at_unix_seconds;
        facts.weekly_window_seconds = previous.weekly_window_seconds;
    }
}

pub(super) fn previous_quota_window_is_still_valid(
    provider: TokenusageWindowedProvider,
    reset_at_unix_seconds: Option<u64>,
    window_seconds: Option<u64>,
    now: u64,
) -> bool {
    match provider {
        TokenusageWindowedProvider::Claude => true,
        TokenusageWindowedProvider::Codex => {
            reset_at_unix_seconds.is_some_and(|reset_at| now < reset_at)
                && window_seconds.is_some_and(|seconds| seconds > 0)
        }
    }
}

pub(super) fn tokenusage_window_identity_matches(
    reset_at: Option<u64>,
    window_seconds: Option<u64>,
    previous_reset_at: Option<u64>,
    previous_window_seconds: Option<u64>,
) -> bool {
    reset_at.is_some()
        && window_seconds.is_some()
        && reset_at == previous_reset_at
        && window_seconds == previous_window_seconds
}

#[cfg(test)]
pub(super) fn codex_usage_shared_cache_is_fresh(
    path: &Path,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    tokenusage_windowed_usage_shared_cache_is_fresh(
        path,
        TokenusageWindowedProvider::Codex,
        now,
        max_age_seconds,
    )
}

pub(super) fn tokenusage_windowed_usage_facts_are_complete(
    provider: TokenusageWindowedProvider,
    facts: &WindowedUsageFacts,
) -> bool {
    let has_token_and_quota = facts.five_hour_tokens.is_some()
        && facts.weekly_tokens.is_some()
        && facts.five_hour_remaining_percent.is_some()
        && facts.weekly_remaining_percent.is_some();
    let has_reset_window = match provider {
        TokenusageWindowedProvider::Claude => true,
        TokenusageWindowedProvider::Codex => {
            facts.five_hour_reset_at_unix_seconds.is_some()
                && facts.weekly_reset_at_unix_seconds.is_some()
                && facts.five_hour_window_seconds.is_some()
                && facts.weekly_window_seconds.is_some()
        }
    };
    has_token_and_quota && has_reset_window
}

#[cfg(test)]
pub(super) fn codex_usage_shared_cache_is_backing_off(path: &Path, now: u64) -> bool {
    tokenusage_windowed_usage_shared_cache_is_backing_off(
        path,
        TokenusageWindowedProvider::Codex,
        now,
    )
}

pub(super) fn tokenusage_windowed_usage_shared_cache_is_fresh(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    let Some(cache) = read_tokenusage_windowed_usage_shared_cache_value(path, provider) else {
        return false;
    };
    let cache_key = tokenusage_windowed_usage_cache_key(provider);
    cache
        .get(cache_key)
        .and_then(|entry| entry.get("updated_at_unix_seconds"))
        .and_then(Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get(cache_key)
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| {
                        tokenusage_windowed_usage_facts_are_complete(provider, &facts)
                    })
        })
}

pub(super) fn tokenusage_windowed_usage_shared_cache_is_backing_off(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> bool {
    read_tokenusage_windowed_usage_shared_cache_value(path, provider)
        .and_then(|cache| {
            let entry = cache.get(tokenusage_windowed_usage_cache_key(provider))?;
            let facts = windowed_usage_facts_from_cache_entry(entry);
            if !facts.is_empty() && !tokenusage_windowed_usage_facts_are_complete(provider, &facts)
            {
                return None;
            }
            entry.get("backoff_until_unix_seconds")?.as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

#[cfg(test)]
pub(super) fn tokenusage_windowed_usage_quota_is_backing_off(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> bool {
    tokenusage_windowed_usage_quota_backoff_until(path, provider, now).is_some()
}

pub(super) fn tokenusage_windowed_usage_quota_backoff_until(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> Option<u64> {
    read_tokenusage_windowed_usage_shared_cache_value(path, provider)
        .and_then(|cache| {
            cache
                .get(tokenusage_windowed_usage_cache_key(provider))?
                .get("quota_backoff_until_unix_seconds")?
                .as_u64()
        })
        .filter(|backoff_until| now < *backoff_until)
}

pub(super) struct TokenusageWindowedUsageCacheLock {
    path: PathBuf,
}

impl Drop for TokenusageWindowedUsageCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

pub(super) fn try_acquire_tokenusage_windowed_usage_cache_lock(
    shared_path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> Result<Option<TokenusageWindowedUsageCacheLock>, CoreError> {
    let lock_path = shared_path.with_file_name(tokenusage_windowed_usage_lock_name(provider));
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                format!(
                    "{}_lock_parent_create_failed",
                    tokenusage_windowed_usage_error_prefix(provider)
                ),
                format!(
                    "Failed to create the Yazelix {} usage cache lock directory.",
                    tokenusage_windowed_usage_label(provider)
                ),
                "Check permissions for the Yazelix state directory, then retry.",
                &parent.display().to_string(),
                source,
            )
        })?;
    }
    match fs::create_dir(&lock_path) {
        Ok(()) => Ok(Some(TokenusageWindowedUsageCacheLock { path: lock_path })),
        Err(source) if source.kind() == ErrorKind::AlreadyExists => {
            if tokenusage_windowed_usage_cache_lock_is_stale(&lock_path, provider, now) {
                let _ = fs::remove_dir(&lock_path);
                return match fs::create_dir(&lock_path) {
                    Ok(()) => Ok(Some(TokenusageWindowedUsageCacheLock { path: lock_path })),
                    Err(source) if source.kind() == ErrorKind::AlreadyExists => Ok(None),
                    Err(source) => Err(CoreError::io(
                        format!(
                            "{}_lock_create_failed",
                            tokenusage_windowed_usage_error_prefix(provider)
                        ),
                        format!(
                            "Failed to acquire the Yazelix {} usage cache lock.",
                            tokenusage_windowed_usage_label(provider)
                        ),
                        "Check permissions for the Yazelix state directory, then retry.",
                        &lock_path.display().to_string(),
                        source,
                    )),
                };
            }
            Ok(None)
        }
        Err(source) => Err(CoreError::io(
            format!(
                "{}_lock_create_failed",
                tokenusage_windowed_usage_error_prefix(provider)
            ),
            format!(
                "Failed to acquire the Yazelix {} usage cache lock.",
                tokenusage_windowed_usage_label(provider)
            ),
            "Check permissions for the Yazelix state directory, then retry.",
            &lock_path.display().to_string(),
            source,
        )),
    }
}

pub(super) fn tokenusage_windowed_usage_cache_lock_is_stale(
    lock_path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> bool {
    fs::metadata(lock_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| {
            now.saturating_sub(duration.as_secs())
                > tokenusage_windowed_usage_lock_stale_after_seconds(provider)
        })
        .unwrap_or(false)
}

pub(super) fn refresh_opencode_go_usage_shared_cache(
    shared_path: &Path,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
) -> Result<bool, CoreError> {
    if opencode_go_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds) {
        return Ok(false);
    }
    if opencode_go_usage_shared_cache_is_backing_off(shared_path, now) {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_opencode_go_usage_cache_lock(shared_path, now)? else {
        return Ok(false);
    };
    if opencode_go_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds)
        || opencode_go_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }

    let db_paths = opencode_db_candidates_from_env();
    refresh_opencode_go_usage_shared_cache_from_dbs(
        shared_path,
        &db_paths,
        now,
        error_backoff_seconds,
    )
}

pub(super) fn refresh_opencode_go_usage_shared_cache_from_dbs(
    shared_path: &Path,
    db_paths: &[PathBuf],
    now: u64,
    error_backoff_seconds: u64,
) -> Result<bool, CoreError> {
    let facts = collect_opencode_go_usage_facts_from_dbs(db_paths, now);
    let mut opencode_go = serde_json::Map::new();
    opencode_go.insert("updated_at_unix_seconds".to_string(), json!(now));
    if let Some(tokens) = facts.five_hour_tokens {
        opencode_go.insert("five_hour_tokens".to_string(), json!(tokens));
    }
    if let Some(tokens) = facts.weekly_tokens {
        opencode_go.insert("weekly_tokens".to_string(), json!(tokens));
    }
    if let Some(tokens) = facts.monthly_tokens {
        opencode_go.insert("monthly_tokens".to_string(), json!(tokens));
    }
    if let Some(percent) = facts.five_hour_remaining_percent {
        opencode_go.insert("five_hour_remaining_percent".to_string(), json!(percent));
    }
    if let Some(percent) = facts.weekly_remaining_percent {
        opencode_go.insert("weekly_remaining_percent".to_string(), json!(percent));
    }
    if let Some(percent) = facts.monthly_remaining_percent {
        opencode_go.insert("monthly_remaining_percent".to_string(), json!(percent));
    }
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        opencode_go.insert("error".to_string(), json!(error));
        opencode_go.insert(
            "backoff_until_unix_seconds".to_string(),
            json!(now.saturating_add(error_backoff_seconds)),
        );
    }
    let status = if facts.is_empty() {
        "error"
    } else if facts.has_tokens() && facts.has_quota() {
        "ok"
    } else {
        "partial"
    };
    opencode_go.insert("status".to_string(), json!(status));

    let cache = json!({
        "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
        "opencode_go": Value::Object(opencode_go),
    });
    write_json_value_atomic(shared_path, &cache, "opencode_go_usage_cache")?;
    Ok(true)
}

pub(super) fn opencode_go_usage_shared_cache_is_fresh(
    path: &Path,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    let Some(cache) = read_opencode_go_usage_shared_cache_value(path) else {
        return false;
    };
    cache
        .get("opencode_go")
        .and_then(|opencode_go| opencode_go.get("updated_at_unix_seconds"))
        .and_then(Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get("opencode_go")
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| opencode_go_usage_facts_are_complete(&facts))
        })
}

pub(super) fn opencode_go_usage_facts_are_complete(facts: &WindowedUsageFacts) -> bool {
    facts.five_hour_tokens.is_some()
        && facts.weekly_tokens.is_some()
        && facts.monthly_tokens.is_some()
        && facts.five_hour_remaining_percent.is_some()
        && facts.weekly_remaining_percent.is_some()
        && facts.monthly_remaining_percent.is_some()
}

pub(super) fn opencode_go_usage_shared_cache_is_backing_off(path: &Path, now: u64) -> bool {
    read_opencode_go_usage_shared_cache_value(path)
        .and_then(|cache| {
            cache
                .get("opencode_go")?
                .get("backoff_until_unix_seconds")?
                .as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

pub(super) struct OpenCodeGoUsageCacheLock {
    path: PathBuf,
}

impl Drop for OpenCodeGoUsageCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

pub(super) fn try_acquire_opencode_go_usage_cache_lock(
    shared_path: &Path,
    now: u64,
) -> Result<Option<OpenCodeGoUsageCacheLock>, CoreError> {
    let lock_path = shared_path.with_file_name(format!(
        ".opencode_go_usage_cache_v{}.lock",
        OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION
    ));
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "opencode_go_usage_cache_lock_parent_create_failed",
                "Failed to create the Yazelix OpenCode Go usage cache lock directory.",
                "Check permissions for the Yazelix state directory, then retry.",
                &parent.display().to_string(),
                source,
            )
        })?;
    }
    match fs::create_dir(&lock_path) {
        Ok(()) => Ok(Some(OpenCodeGoUsageCacheLock { path: lock_path })),
        Err(source) if source.kind() == ErrorKind::AlreadyExists => {
            if opencode_go_usage_cache_lock_is_stale(&lock_path, now) {
                let _ = fs::remove_dir(&lock_path);
                return match fs::create_dir(&lock_path) {
                    Ok(()) => Ok(Some(OpenCodeGoUsageCacheLock { path: lock_path })),
                    Err(source) if source.kind() == ErrorKind::AlreadyExists => Ok(None),
                    Err(source) => Err(CoreError::io(
                        "opencode_go_usage_cache_lock_create_failed",
                        "Failed to acquire the Yazelix OpenCode Go usage cache lock.",
                        "Check permissions for the Yazelix state directory, then retry.",
                        &lock_path.display().to_string(),
                        source,
                    )),
                };
            }
            Ok(None)
        }
        Err(source) => Err(CoreError::io(
            "opencode_go_usage_cache_lock_create_failed",
            "Failed to acquire the Yazelix OpenCode Go usage cache lock.",
            "Check permissions for the Yazelix state directory, then retry.",
            &lock_path.display().to_string(),
            source,
        )),
    }
}

pub(super) fn opencode_go_usage_cache_lock_is_stale(lock_path: &Path, now: u64) -> bool {
    fs::metadata(lock_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| {
            now.saturating_sub(duration.as_secs()) > OPENCODE_GO_USAGE_LOCK_STALE_AFTER_SECONDS
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct OpenCodeGoUsageWindow {
    tokens: u64,
    cost_usd: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct OpenCodeGoUsageWindows {
    five_hour: OpenCodeGoUsageWindow,
    weekly: OpenCodeGoUsageWindow,
    monthly: OpenCodeGoUsageWindow,
}

pub(super) fn collect_opencode_go_usage_facts_from_dbs(
    db_paths: &[PathBuf],
    now: u64,
) -> WindowedUsageFacts {
    if db_paths.is_empty() {
        return WindowedUsageFacts {
            error: Some("missing OpenCode DB".to_string()),
            ..WindowedUsageFacts::default()
        };
    }

    let mut five_hour = OpenCodeGoUsageWindow::default();
    let mut weekly = OpenCodeGoUsageWindow::default();
    let mut monthly = OpenCodeGoUsageWindow::default();
    let mut opened_any = false;
    let mut first_error = None;

    for path in db_paths {
        match collect_opencode_go_usage_windows_from_db(path, now) {
            Ok(db_windows) => {
                opened_any = true;
                five_hour.tokens = five_hour.tokens.saturating_add(db_windows.five_hour.tokens);
                five_hour.cost_usd += db_windows.five_hour.cost_usd;
                weekly.tokens = weekly.tokens.saturating_add(db_windows.weekly.tokens);
                weekly.cost_usd += db_windows.weekly.cost_usd;
                monthly.tokens = monthly.tokens.saturating_add(db_windows.monthly.tokens);
                monthly.cost_usd += db_windows.monthly.cost_usd;
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(format!("{}: {error}", path.display()));
                }
            }
        }
    }

    if !opened_any {
        return WindowedUsageFacts {
            error: first_error.or_else(|| Some("OpenCode DB unavailable".to_string())),
            ..WindowedUsageFacts::default()
        };
    }

    let mut facts = WindowedUsageFacts::default();
    facts.five_hour_tokens = Some(five_hour.tokens);
    facts.five_hour_remaining_percent = Some(remaining_percent_from_cost_limit(
        five_hour.cost_usd,
        OPENCODE_GO_FIVE_HOUR_LIMIT_USD,
    ));
    facts.weekly_tokens = Some(weekly.tokens);
    facts.weekly_remaining_percent = Some(remaining_percent_from_cost_limit(
        weekly.cost_usd,
        OPENCODE_GO_WEEKLY_LIMIT_USD,
    ));
    facts.monthly_tokens = Some(monthly.tokens);
    facts.monthly_remaining_percent = Some(remaining_percent_from_cost_limit(
        monthly.cost_usd,
        OPENCODE_GO_MONTHLY_LIMIT_USD,
    ));
    if facts.is_empty() {
        facts.error = Some("OpenCode Go usage unavailable".to_string());
    }
    facts
}

pub(super) fn collect_opencode_go_usage_windows_from_db(
    path: &Path,
    now: u64,
) -> Result<OpenCodeGoUsageWindows, String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("failed to open OpenCode DB read-only: {error}"))?;
    connection
        .busy_timeout(Duration::from_millis(250))
        .map_err(|error| format!("failed to configure OpenCode DB read timeout: {error}"))?;
    let five_hour = query_opencode_go_usage_window(
        &connection,
        now.saturating_sub(OPENCODE_GO_FIVE_HOUR_SECONDS),
    )?;
    let weekly =
        query_opencode_go_usage_window(&connection, now.saturating_sub(OPENCODE_GO_WEEK_SECONDS))?;
    let monthly =
        query_opencode_go_usage_window(&connection, now.saturating_sub(OPENCODE_GO_MONTH_SECONDS))?;
    Ok(OpenCodeGoUsageWindows {
        five_hour,
        weekly,
        monthly,
    })
}

pub(super) fn query_opencode_go_usage_window(
    connection: &Connection,
    since_unix_seconds: u64,
) -> Result<OpenCodeGoUsageWindow, String> {
    connection
        .query_row(
            r#"
            SELECT
              COALESCE(SUM(
                COALESCE(
                  json_extract(data, '$.tokens.total'),
                  COALESCE(json_extract(data, '$.tokens.input'), 0) +
                  COALESCE(json_extract(data, '$.tokens.output'), 0) +
                  COALESCE(json_extract(data, '$.tokens.reasoning'), 0) +
                  COALESCE(json_extract(data, '$.tokens.cache.read'), 0) +
                  COALESCE(json_extract(data, '$.tokens.cache.write'), 0)
                )
              ), 0),
              COALESCE(SUM(COALESCE(json_extract(data, '$.cost'), 0.0)), 0.0)
            FROM message
            WHERE time_created >= ?1
              AND json_extract(data, '$.role') = 'assistant'
              AND json_extract(data, '$.providerID') = ?2
            "#,
            params![
                unix_millis_from_seconds_saturating(since_unix_seconds),
                OPENCODE_GO_PROVIDER_ID
            ],
            |row| {
                Ok(OpenCodeGoUsageWindow {
                    tokens: row.get::<_, i64>(0)?.max(0) as u64,
                    cost_usd: row.get::<_, f64>(1)?.max(0.0),
                })
            },
        )
        .map_err(|error| format!("failed to read OpenCode Go usage window: {error}"))
}

pub(super) fn unix_millis_from_seconds_saturating(seconds: u64) -> i64 {
    seconds.saturating_mul(1000).min(i64::MAX as u64) as i64
}

pub(super) fn remaining_percent_from_cost_limit(cost_usd: f64, limit_usd: f64) -> u64 {
    if limit_usd <= 0.0 {
        return 0;
    }
    (100.0 - (cost_usd / limit_usd * 100.0))
        .clamp(0.0, 100.0)
        .round() as u64
}

pub(super) fn opencode_db_candidates_from_env() -> Vec<PathBuf> {
    opencode_db_candidates_from_values(
        env::var_os("OPENCODE_DB").map(PathBuf::from),
        env::var_os("OPENCODE_DATA_DIR").map(PathBuf::from),
        env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        env::var_os("HOME").map(PathBuf::from),
    )
}

pub(super) fn opencode_db_candidates_from_values(
    opencode_db: Option<PathBuf>,
    opencode_data_dir: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Vec<PathBuf> {
    let data_dir = opencode_data_dir
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| {
            xdg_data_home
                .filter(|path| !path.as_os_str().is_empty())
                .map(|path| path.join("opencode"))
        })
        .or_else(|| {
            home.filter(|path| !path.as_os_str().is_empty())
                .map(|path| path.join(".local").join("share").join("opencode"))
        });

    let mut candidates = Vec::new();
    if let Some(path) = opencode_db.filter(|path| !path.as_os_str().is_empty()) {
        if path.is_absolute() {
            push_unique_path(&mut candidates, path);
        } else if let Some(data_dir) = data_dir.as_ref() {
            push_unique_path(&mut candidates, data_dir.join(path));
        } else {
            push_unique_path(&mut candidates, path);
        }
    }

    if let Some(data_dir) = data_dir {
        push_unique_path(&mut candidates, data_dir.join("opencode.db"));
        if let Ok(entries) = fs::read_dir(data_dir) {
            let mut channel_dbs = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("opencode-") && name.ends_with(".db"))
                })
                .collect::<Vec<_>>();
            channel_dbs.sort();
            for path in channel_dbs {
                push_unique_path(&mut candidates, path);
            }
        }
    }

    candidates
}

pub(super) fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

pub(super) fn collect_tokenusage_windowed_usage_facts(
    provider: TokenusageWindowedProvider,
    path_var: Option<&OsStr>,
    timeout: Duration,
    include_quota: bool,
) -> WindowedUsageFacts {
    let Some(path_var) = path_var else {
        return WindowedUsageFacts {
            error: Some("missing PATH".to_string()),
            ..WindowedUsageFacts::default()
        };
    };
    let Some(binary_path) = find_command_in_path_var(path_var, "tu") else {
        return WindowedUsageFacts {
            error: Some("missing tu".to_string()),
            ..WindowedUsageFacts::default()
        };
    };

    let mut facts = WindowedUsageFacts::default();
    match run_tokenusage_json_command(
        &binary_path,
        tokenusage_active_block_args(provider).as_slice(),
        timeout,
    ) {
        Ok(Some(value)) => {
            facts.five_hour_tokens = tokenusage_active_block_tokens_from_json(&value);
        }
        Ok(None) => facts.error = Some("active block unavailable".to_string()),
        Err(error) => facts.error = Some(error),
    }

    match run_tokenusage_json_command(
        &binary_path,
        tokenusage_weekly_args(provider).as_slice(),
        timeout,
    ) {
        Ok(Some(value)) => {
            facts.weekly_tokens = tokenusage_weekly_tokens_from_json(&value);
        }
        Ok(None) => {
            facts.error = facts
                .error
                .or_else(|| Some("weekly usage unavailable".to_string()))
        }
        Err(error) => facts.error = facts.error.or(Some(error)),
    }

    if include_quota {
        match run_tokenusage_json_command(
            &binary_path,
            tokenusage_official_limits_args(provider).as_slice(),
            timeout,
        ) {
            Ok(Some(value)) => {
                let quota = tokenusage_quota_from_official_json(&value, provider);
                facts.five_hour_remaining_percent = quota.five_hour_remaining_percent;
                facts.weekly_remaining_percent = quota.weekly_remaining_percent;
                facts.five_hour_reset_at_unix_seconds = quota.five_hour_reset_at_unix_seconds;
                facts.weekly_reset_at_unix_seconds = quota.weekly_reset_at_unix_seconds;
                facts.five_hour_window_seconds = quota.five_hour_window_seconds;
                facts.weekly_window_seconds = quota.weekly_window_seconds;
                if !quota.has_quota() {
                    facts.error = facts
                        .error
                        .or_else(|| Some("quota unavailable".to_string()));
                }
            }
            Ok(None) => {
                facts.error = facts
                    .error
                    .or_else(|| Some("quota unavailable".to_string()))
            }
            Err(error) => facts.error = facts.error.or(Some(error)),
        }
    }

    facts
}

pub(super) fn tokenusage_active_block_args(
    provider: TokenusageWindowedProvider,
) -> Vec<&'static str> {
    let mut args = vec!["blocks", "--active", "--json", "--offline"];
    args.extend(tokenusage_disabled_source_args(provider));
    args
}

pub(super) fn tokenusage_weekly_args(provider: TokenusageWindowedProvider) -> Vec<&'static str> {
    let mut args = vec!["weekly", "--json", "--offline"];
    args.extend(tokenusage_disabled_source_args(provider));
    args.extend(["--order", "desc"]);
    args
}

pub(super) fn tokenusage_official_limits_args(
    provider: TokenusageWindowedProvider,
) -> Vec<&'static str> {
    let mut args = vec!["blocks", "--active", "--json", "--official-limits"];
    args.extend(tokenusage_disabled_source_args(provider));
    args
}

pub(super) fn tokenusage_disabled_source_args(
    provider: TokenusageWindowedProvider,
) -> &'static [&'static str] {
    match provider {
        TokenusageWindowedProvider::Claude => &["--no-codex", "--no-antigravity"],
        TokenusageWindowedProvider::Codex => &["--no-claude", "--no-antigravity"],
    }
}

pub(super) fn run_tokenusage_json_command(
    binary_path: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<Option<Value>, String> {
    let output = run_agent_usage_command_with_timeout(binary_path, args, timeout)
        .map_err(|error| error.to_string())?;
    let Some(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(json_raw) = extract_json_object(&stdout) else {
        return Ok(None);
    };
    serde_json::from_str::<Value>(json_raw)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub(super) fn tokenusage_active_block_tokens_from_json(value: &Value) -> Option<u64> {
    value
        .get("blocks")
        .and_then(Value::as_array)
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|block| {
                    block
                        .get("isActive")
                        .or_else(|| block.get("is_active"))
                        .and_then(Value::as_bool)
                        == Some(true)
                })
                .or_else(|| blocks.first())
        })
        .and_then(|block| {
            first_u64_at(
                block,
                &[
                    &["totals", "total_tokens"],
                    &["totals", "totalTokens"],
                    &["total_tokens"],
                    &["totalTokens"],
                ],
            )
        })
}

pub(super) fn tokenusage_weekly_tokens_from_json(value: &Value) -> Option<u64> {
    value
        .get("weekly")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| {
            first_u64_at(
                row,
                &[
                    &["totals", "total_tokens"],
                    &["totals", "totalTokens"],
                    &["total_tokens"],
                    &["totalTokens"],
                ],
            )
        })
}

pub(super) fn tokenusage_quota_from_official_json(
    value: &Value,
    provider: TokenusageWindowedProvider,
) -> WindowedUsageFacts {
    let official_key = match provider {
        TokenusageWindowedProvider::Claude => "official_claude",
        TokenusageWindowedProvider::Codex => "official_codex",
    };
    let Some(official) = value.get(official_key) else {
        return WindowedUsageFacts::default();
    };
    WindowedUsageFacts {
        five_hour_remaining_percent: official
            .get("primary_used_percent")
            .and_then(Value::as_f64)
            .map(remaining_percent_from_used),
        weekly_remaining_percent: official
            .get("secondary_used_percent")
            .and_then(Value::as_f64)
            .map(remaining_percent_from_used),
        five_hour_reset_at_unix_seconds: official.get("primary_resets_at").and_then(Value::as_u64),
        weekly_reset_at_unix_seconds: official.get("secondary_resets_at").and_then(Value::as_u64),
        five_hour_window_seconds: official
            .get("primary_window_mins")
            .and_then(Value::as_u64)
            .and_then(window_minutes_to_seconds),
        weekly_window_seconds: official
            .get("secondary_window_mins")
            .and_then(Value::as_u64)
            .and_then(window_minutes_to_seconds),
        ..WindowedUsageFacts::default()
    }
}

pub(super) fn window_minutes_to_seconds(minutes: u64) -> Option<u64> {
    minutes
        .checked_mul(MINUTE_SECONDS)
        .filter(|seconds| *seconds > 0)
}

pub(super) fn remaining_percent_from_used(used_percent: f64) -> u64 {
    (100.0 - used_percent).clamp(0.0, 100.0).round() as u64
}

pub(super) fn normalized_session_config_from_env() -> Option<Value> {
    let path = session_config_path_from_env()?;
    normalized_session_config_from_path(&path)
}

pub(super) fn normalized_session_config_for_status_cache_path(
    status_cache_path: &Path,
) -> Option<Value> {
    normalized_session_config_from_status_cache_path(status_cache_path)
        .or_else(normalized_session_config_from_env)
}

pub(super) fn normalized_session_config_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<Value> {
    let path = session_config_path_from_values(None, Some(status_cache_path.to_path_buf()))?;
    normalized_session_config_from_path(&path)
}

pub(super) fn normalized_session_config_from_path(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&raw)
        .ok()?
        .get("normalized_config")
        .cloned()
}

pub(super) fn agent_usage_widget_settings_from_status_cache_path(
    status_cache_path: &Path,
) -> AgentUsageWidgetSettings {
    let Some(config) = normalized_session_config_for_status_cache_path(status_cache_path) else {
        return AgentUsageWidgetSettings::default();
    };
    AgentUsageWidgetSettings {
        claude_display: config
            .get("zellij_claude_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Both),
        codex_display: config
            .get("zellij_codex_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Quota),
        opencode_go_display: config
            .get("zellij_opencode_go_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Both),
        claude_periods: windowed_usage_periods_from_config(
            &config,
            "zellij_claude_usage_periods",
            default_windowed_usage_periods(),
        ),
        opencode_go_periods: windowed_usage_periods_from_config(
            &config,
            "zellij_opencode_go_usage_periods",
            &default_opencode_go_usage_periods(),
        ),
    }
}

pub(super) fn usage_widget_enabled_from_status_cache_path(
    status_cache_path: &Path,
    widget: &str,
) -> bool {
    normalized_session_config_for_status_cache_path(status_cache_path)
        .and_then(|config| agent_usage_widget_names_from_config(&config))
        .is_some_and(|widgets| widgets.contains(widget))
}

pub(super) fn agent_usage_widget_names_from_config(config: &Value) -> Option<BTreeSet<String>> {
    Some(
        config
            .get("zellij_widget_tray")?
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|widget| !widget.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

pub(super) fn windowed_usage_periods_from_config(
    config: &Value,
    key: &str,
    default_periods: &[WindowedUsagePeriod],
) -> Vec<WindowedUsagePeriod> {
    let periods = config
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter_map(WindowedUsagePeriod::parse_config)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if periods.is_empty() {
        default_periods.to_vec()
    } else {
        periods
    }
}

pub(super) fn run_agent_usage_command_with_timeout(
    binary_path: &Path,
    args: &[&str],
    timeout: Duration,
) -> std::io::Result<Option<std::process::Output>> {
    let mut child = Command::new(binary_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map(Some);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(25));
    }
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

pub(super) fn find_command_in_path_var(path_var: &OsStr, command_name: &str) -> Option<PathBuf> {
    env::split_paths(path_var)
        .map(|entry| entry.join(command_name))
        .find(|candidate| candidate.is_file())
}

pub(super) fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    (start <= end).then_some(&raw[start..=end])
}

pub(super) fn render_agent_usage_widget(label: &str, summary: &str) -> String {
    format!(" [{label} {summary}]")
}

pub(super) fn first_u64_at(value: &Value, paths: &[&[&str]]) -> Option<u64> {
    paths
        .iter()
        .find_map(|path| nested_value(value, path)?.as_u64())
}

pub(super) fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

pub(super) fn format_agent_usage_token_count(tokens: u64) -> String {
    if tokens >= 1_000_000_000 {
        format_scaled_agent_usage_count(tokens as f64 / 1_000_000_000.0, "B")
    } else if tokens >= 1_000_000 {
        format_scaled_agent_usage_count(tokens as f64 / 1_000_000.0, "M")
    } else if tokens >= 1_000 {
        format!("{}k", tokens / 1_000)
    } else {
        tokens.to_string()
    }
}

pub(super) fn format_scaled_agent_usage_count(value: f64, suffix: &str) -> String {
    let raw = if value >= 100.0 {
        format!("{value:.0}")
    } else if value >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    };
    let trimmed = if raw.contains('.') {
        raw.trim_end_matches('0').trim_end_matches('.')
    } else {
        raw.as_str()
    };
    format!("{trimmed}{suffix}")
}

pub(super) fn format_reset_window_label(
    reset_at_unix_seconds: u64,
    window_seconds: u64,
    now_unix_seconds: u64,
) -> Option<String> {
    if window_seconds == 0 {
        return None;
    }
    let remaining_seconds = reset_at_unix_seconds
        .saturating_sub(now_unix_seconds)
        .min(window_seconds);
    let elapsed_seconds = window_seconds.saturating_sub(remaining_seconds);
    Some(format!(
        "{}/{}",
        format_reset_window_position_duration(elapsed_seconds, window_seconds),
        format_reset_window_total_duration(window_seconds)
    ))
}

pub(super) fn format_reset_window_position_duration(seconds: u64, window_seconds: u64) -> String {
    if window_seconds >= DAY_SECONDS {
        let days = seconds / DAY_SECONDS;
        let hours = (seconds % DAY_SECONDS) / HOUR_SECONDS;
        if days > 0 && hours > 0 {
            format!("{days}d{hours}h")
        } else if days > 0 {
            format!("{days}d")
        } else if hours > 0 {
            format!("{hours}h")
        } else {
            "0h".to_string()
        }
    } else if window_seconds >= HOUR_SECONDS {
        let hours = seconds / HOUR_SECONDS;
        let minutes = elapsed_minutes_after_hour(seconds);
        if hours > 0 && minutes > 0 {
            format!("{hours}h{minutes}m")
        } else if hours > 0 {
            format!("{hours}h")
        } else {
            format!("{minutes}m")
        }
    } else if window_seconds >= MINUTE_SECONDS {
        if seconds > 0 {
            format!("{}m", seconds.div_ceil(MINUTE_SECONDS))
        } else {
            "0m".to_string()
        }
    } else if seconds > 0 {
        format!("{seconds}s")
    } else {
        "0s".to_string()
    }
}

pub(super) fn elapsed_minutes_after_hour(seconds: u64) -> u64 {
    let minutes = (seconds % HOUR_SECONDS) / MINUTE_SECONDS;
    if seconds > 0 && seconds < HOUR_SECONDS && minutes == 0 {
        1
    } else {
        minutes
    }
}

pub(super) fn format_reset_window_total_duration(seconds: u64) -> String {
    if seconds % DAY_SECONDS == 0 {
        format!("{}d", seconds / DAY_SECONDS)
    } else if seconds % HOUR_SECONDS == 0 {
        format!("{}h", seconds / HOUR_SECONDS)
    } else if seconds % MINUTE_SECONDS == 0 {
        format!("{}m", seconds / MINUTE_SECONDS)
    } else {
        format!("{seconds}s")
    }
}

pub(super) fn format_quota_percent(percent: u64) -> String {
    format!("{}%", percent.min(100))
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
