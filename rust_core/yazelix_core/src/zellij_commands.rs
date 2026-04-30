// Test lane: default
//! Zellij integration commands for `yzx_control`.
//!
//! These are thin wrappers around `zellij action pipe --plugin yazelix_pane_orchestrator`
//! used by Rust-owned public commands and the remaining shell/process wrappers.

use crate::bridge::{CoreError, ErrorClass};
use crate::compute_runtime_env;
use crate::control_plane::{
    home_dir_from_env, json_map_to_child_env, runtime_dir_from_env, runtime_env_request,
};
use crate::session_facts::compute_session_facts_from_env;
use crate::workspace_commands::{
    command_is_available, compute_integration_facts_from_env, run_ya_emit_to,
    sync_sidebar_to_directory,
};
use rusqlite::{Connection, OpenFlags, params};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";
const STATUS_BUS_SCHEMA_VERSION: i64 = 1;
const STATUS_BAR_CACHE_SCHEMA_VERSION: i64 = 1;
const ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION: i64 = 1;
const CODEX_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
const OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
const AGENT_USAGE_SEED_MAX_AGE_SECONDS: u64 = 180;
const AGENT_USAGE_SEED_CANDIDATE_LIMIT: usize = 16;
const CODEX_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
const OPENCODE_GO_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
const OPENCODE_GO_PROVIDER_ID: &str = "opencode-go";
const OPENCODE_GO_FIVE_HOUR_SECONDS: u64 = 5 * 60 * 60;
const OPENCODE_GO_WEEK_SECONDS: u64 = 7 * 24 * 60 * 60;
const OPENCODE_GO_MONTH_SECONDS: u64 = 30 * 24 * 60 * 60;
const OPENCODE_GO_FIVE_HOUR_LIMIT_USD: f64 = 12.0;
const OPENCODE_GO_WEEKLY_LIMIT_USD: f64 = 30.0;
const OPENCODE_GO_MONTHLY_LIMIT_USD: f64 = 60.0;
const EDITOR_PANE_NAME: &str = "editor";
pub const INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS: &[&str] = &[
    "pipe",
    "get-workspace-root",
    "inspect-session",
    "status-bus",
    "status-bus-workspace",
    "status-cache-write",
    "status-cache-heartbeat",
    "status-cache-widget",
    "status-cache-refresh-agent-usage",
    "status-cache-refresh-codex-usage",
    "status-cache-refresh-opencode-go-usage",
    "agent-usage",
    "retarget",
    "open-editor",
    "open-editor-cwd",
    "open-terminal",
];
const EDITOR_PANE_ENV_OVERRIDE_KEYS: &[&str] = &[
    "PATH",
    "YAZELIX_RUNTIME_DIR",
    "YAZELIX_SESSION_CONFIG_PATH",
    "YAZELIX_SESSION_FACTS_PATH",
    "IN_YAZELIX_SHELL",
    "NIX_CONFIG",
    "ZELLIJ_DEFAULT_LAYOUT",
    "YAZI_CONFIG_HOME",
    "YAZELIX_MANAGED_HELIX_BINARY",
    "EDITOR",
    "VISUAL",
    "HELIX_RUNTIME",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijPipeArgs {
    command: Option<String>,
    payload: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijGetWorkspaceRootArgs {
    include_bootstrap: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijInspectSessionArgs {
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusBusArgs {
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusBusWorkspaceArgs {
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusCacheWriteArgs {
    path: Option<PathBuf>,
    payload: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusCacheHeartbeatArgs {
    path: Option<PathBuf>,
    payload: Option<String>,
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusCacheWidgetArgs {
    widget: Option<String>,
    path: Option<PathBuf>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusCacheRefreshAgentUsageArgs {
    path: Option<PathBuf>,
    max_age_seconds: Option<u64>,
    timeout_ms: Option<u64>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusCacheRefreshCodexUsageArgs {
    path: Option<PathBuf>,
    max_age_seconds: Option<u64>,
    error_backoff_seconds: Option<u64>,
    timeout_ms: Option<u64>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijStatusCacheRefreshOpenCodeGoUsageArgs {
    path: Option<PathBuf>,
    max_age_seconds: Option<u64>,
    error_backoff_seconds: Option<u64>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijAgentUsageArgs {
    provider: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentUsageDisplay {
    Both,
    Tokens,
    Money,
}

impl AgentUsageDisplay {
    fn parse(raw: &str) -> Self {
        match raw.trim() {
            "tokens" => Self::Tokens,
            "money" => Self::Money,
            _ => Self::Both,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowedUsageDisplay {
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
struct AgentUsageWidgetSettings {
    display: AgentUsageDisplay,
    codex_display: WindowedUsageDisplay,
    opencode_go_display: WindowedUsageDisplay,
    opencode_go_periods: Vec<WindowedUsagePeriod>,
    claude_periods: Vec<AgentUsagePeriod>,
}

impl Default for AgentUsageWidgetSettings {
    fn default() -> Self {
        Self {
            display: AgentUsageDisplay::Tokens,
            codex_display: WindowedUsageDisplay::Both,
            opencode_go_display: WindowedUsageDisplay::Both,
            opencode_go_periods: default_opencode_go_usage_periods(),
            claude_periods: vec![AgentUsagePeriod::Daily],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentUsageRefreshConfig {
    widgets: Option<BTreeSet<String>>,
    claude_periods: Vec<AgentUsagePeriod>,
}

impl Default for AgentUsageRefreshConfig {
    fn default() -> Self {
        Self {
            widgets: None,
            claude_periods: vec![AgentUsagePeriod::Daily],
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct WindowedUsageFacts {
    five_hour_tokens: Option<u64>,
    weekly_tokens: Option<u64>,
    monthly_tokens: Option<u64>,
    five_hour_remaining_percent: Option<u64>,
    weekly_remaining_percent: Option<u64>,
    monthly_remaining_percent: Option<u64>,
    error: Option<String>,
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
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AgentUsageFacts {
    tokens: Option<String>,
    cost: Option<String>,
    remaining: Option<String>,
}

impl AgentUsageFacts {
    fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(tokens) = self.tokens.as_deref().filter(|value| !value.is_empty()) {
            parts.push(tokens);
        }
        if let Some(cost) = self.cost.as_deref().filter(|value| !value.is_empty()) {
            parts.push(cost);
        }
        if let Some(remaining) = self.remaining.as_deref().filter(|value| !value.is_empty()) {
            parts.push(remaining);
        }
        parts.join(" ")
    }

    fn display_summary(&self, display: AgentUsageDisplay) -> String {
        match display {
            AgentUsageDisplay::Both => self.summary(),
            AgentUsageDisplay::Tokens => self.tokens.clone().unwrap_or_default(),
            AgentUsageDisplay::Money => self.cost.clone().unwrap_or_default(),
        }
    }

    fn is_empty(&self) -> bool {
        self.tokens.is_none() && self.cost.is_none() && self.remaining.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentUsageProvider {
    Claude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentUsagePeriod {
    Daily,
    Monthly,
}

impl AgentUsagePeriod {
    fn parse_config(raw: &str) -> Option<Self> {
        match raw.trim() {
            "day" | "daily" | "d" => Some(Self::Daily),
            "month" | "monthly" | "mon" => Some(Self::Monthly),
            _ => None,
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            Self::Daily => "d",
            Self::Monthly => "mon",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowedUsagePeriod {
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

fn default_windowed_usage_periods() -> &'static [WindowedUsagePeriod] {
    &[WindowedUsagePeriod::FiveHour, WindowedUsagePeriod::Weekly]
}

fn default_opencode_go_usage_periods() -> Vec<WindowedUsagePeriod> {
    default_windowed_usage_periods().to_vec()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AgentUsageTarget {
    provider: AgentUsageProvider,
    period: AgentUsagePeriod,
    cache_key: &'static str,
    label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentUsageJsonDialect {
    Tokenusage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentUsageCommand {
    binary: &'static str,
    args: Vec<&'static str>,
    dialect: AgentUsageJsonDialect,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijOpenEditorArgs {
    targets: Vec<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijOpenEditorCwdArgs {
    target: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManagedEditorOpenStatus {
    Ok,
    Missing,
}

fn run_pane_orchestrator_command(command_name: &str, payload: &str) -> Result<String, CoreError> {
    let output = Command::new("zellij")
        .args([
            "action",
            "pipe",
            "--plugin",
            PANE_ORCHESTRATOR_PLUGIN_ALIAS,
            "--name",
            command_name,
            "--",
            payload,
        ])
        .output()
        .map_err(|source| {
            CoreError::io(
                "pane_orchestrator_pipe_failed",
                format!(
                    "Failed to run the Yazelix pane-orchestrator command `{command_name}`."
                ),
                "Run this command inside an active Yazelix/Zellij session with the pane orchestrator loaded, then retry.",
                "zellij",
                source,
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let details = if stderr.is_empty() {
            format!("exit code {}", output.status.code().unwrap_or(1))
        } else {
            stderr
        };
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "pane_orchestrator_pipe_failed",
            format!("Pane orchestrator pipe failed for `{command_name}`: {details}"),
            "Run this command inside an active Yazelix/Zellij session with the pane orchestrator loaded, then retry.",
            json!({ "command": command_name }),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_zellij_pipe_args(args: &[String]) -> Result<ZellijPipeArgs, CoreError> {
    let mut parsed = ZellijPipeArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
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
                    "Unknown argument for zellij pipe: {other}"
                )));
            }
            other => {
                if parsed.command.is_some() {
                    return Err(CoreError::usage(
                        "zellij pipe accepts only one command name".to_string(),
                    ));
                }
                parsed.command = Some(other.to_string());
            }
        }
    }

    Ok(parsed)
}

fn print_zellij_pipe_help() {
    println!("Send a command to the Yazelix pane orchestrator plugin");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij pipe <command> [--payload <json>]");
    println!();
    println!("Examples:");
    println!("  yzx_control zellij pipe focus_sidebar");
    println!("  yzx_control zellij pipe get_active_tab_session_state");
    println!("  yzx_control zellij pipe open_transient_pane --payload '{{\"kind\":\"popup\"}}'");
}

pub fn run_zellij_pipe(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_pipe_args(args)?;
    if parsed.help {
        print_zellij_pipe_help();
        return Ok(0);
    }

    let command = parsed.command.ok_or_else(|| {
        CoreError::usage(
            "zellij pipe requires a command name. Try `yzx_control zellij pipe --help`."
                .to_string(),
        )
    })?;

    let payload = parsed.payload.as_deref().unwrap_or("");
    let response = run_pane_orchestrator_command(&command, payload)?;
    println!("{}", response);
    Ok(0)
}

fn parse_zellij_get_workspace_root_args(
    args: &[String],
) -> Result<ZellijGetWorkspaceRootArgs, CoreError> {
    let mut parsed = ZellijGetWorkspaceRootArgs::default();
    for arg in args {
        match arg.as_str() {
            "--include-bootstrap" => parsed.include_bootstrap = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij get-workspace-root: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij get-workspace-root accepts no positional arguments".to_string(),
                ));
            }
        }
    }
    Ok(parsed)
}

fn print_zellij_get_workspace_root_help() {
    println!("Get the current tab workspace root from the pane orchestrator");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij get-workspace-root [--include-bootstrap]");
}

pub fn internal_zellij_control_subcommands_usage() -> String {
    INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS.join("|")
}

fn parse_zellij_inspect_session_args(
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

fn print_zellij_inspect_session_help() {
    println!("Inspect the current tab session state from the pane orchestrator");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij inspect-session [--json]");
}

fn parse_zellij_status_bus_args(args: &[String]) -> Result<ZellijStatusBusArgs, CoreError> {
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

fn print_zellij_status_bus_help() {
    println!("Read the current versioned Yazelix status-bus snapshot");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-bus [--json]");
}

fn parse_zellij_status_bus_workspace_args(
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

fn parse_zellij_status_cache_write_args(
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

fn parse_zellij_status_cache_widget_args(
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

fn parse_zellij_status_cache_heartbeat_args(
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

fn parse_zellij_status_cache_refresh_agent_usage_args(
    args: &[String],
) -> Result<ZellijStatusCacheRefreshAgentUsageArgs, CoreError> {
    let mut parsed = ZellijStatusCacheRefreshAgentUsageArgs::default();
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
                    "Unknown argument for zellij status-cache-refresh-agent-usage: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(
                    "zellij status-cache-refresh-agent-usage accepts only flags".to_string(),
                ));
            }
        }
    }

    Ok(parsed)
}

fn parse_zellij_status_cache_refresh_codex_usage_args(
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

fn parse_zellij_status_cache_refresh_opencode_go_usage_args(
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

fn parse_zellij_agent_usage_args(args: &[String]) -> Result<ZellijAgentUsageArgs, CoreError> {
    let mut parsed = ZellijAgentUsageArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij agent-usage: {other}"
                )));
            }
            other => {
                if parsed.provider.is_some() {
                    return Err(CoreError::usage(
                        "zellij agent-usage accepts exactly one provider".to_string(),
                    ));
                }
                parsed.provider = Some(other.to_string());
            }
        }
    }
    Ok(parsed)
}

fn print_zellij_status_bus_workspace_help() {
    println!("Render the workspace status-bus fact for zjstatus");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-bus-workspace");
}

fn print_zellij_status_cache_write_help() {
    println!("Write the window-local cached status-bar facts");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-cache-write --payload <json> [--path <path>]");
}

fn print_zellij_status_cache_widget_help() {
    println!("Render one status-bar widget from the window-local cache");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-cache-widget <widget> [--path <path>]");
}

fn print_zellij_status_cache_heartbeat_help() {
    println!("Read or update window-local pane-orchestrator heartbeat facts");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-cache-heartbeat [--json] [--path <path>]");
}

fn print_zellij_status_cache_refresh_agent_usage_help() {
    println!("Refresh cached agent-usage facts for status-bar widgets");
    println!();
    println!("Usage:");
    println!(
        "  yzx_control zellij status-cache-refresh-agent-usage [--path <path>] [--max-age-seconds <n>] [--timeout-ms <n>]"
    );
}

fn print_zellij_status_cache_refresh_codex_usage_help() {
    println!("Refresh cached Codex 5h/week usage and quota facts for status-bar widgets");
    println!();
    println!("Usage:");
    println!(
        "  yzx_control zellij status-cache-refresh-codex-usage [--path <path>] [--max-age-seconds <n>] [--error-backoff-seconds <n>] [--timeout-ms <n>]"
    );
}

fn print_zellij_status_cache_refresh_opencode_go_usage_help() {
    println!("Refresh cached OpenCode Go 5h/week usage and quota facts for status-bar widgets");
    println!();
    println!("Usage:");
    println!(
        "  yzx_control zellij status-cache-refresh-opencode-go-usage [--path <path>] [--max-age-seconds <n>] [--error-backoff-seconds <n>]"
    );
}

fn print_zellij_agent_usage_help() {
    println!("Render an opt-in agent usage provider summary for zjstatus");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij agent-usage <claude>");
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
    let agent_usage_seed =
        agent_usage_seed_for_status_cache_path(&path, previous_cache.as_ref(), now);
    let mut cache = build_status_bar_cache_at(
        status_bus,
        agent_usage_seed
            .as_ref()
            .map(|seed| seed.agent_usage.clone())
            .unwrap_or_else(|| json!({})),
        agent_usage_seed.and_then(|seed| seed.updated_at_unix_seconds),
        now,
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
    let Some(mut cache) = read_status_bar_cache_value(&path) else {
        return Ok(0);
    };
    if widget == "codex_usage" {
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

pub fn run_zellij_status_cache_refresh_agent_usage(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_cache_refresh_agent_usage_args(args)?;
    if parsed.help {
        print_zellij_status_cache_refresh_agent_usage_help();
        return Ok(0);
    }

    let path = match parsed.path.or_else(status_bar_cache_path_from_env) {
        Some(path) => path,
        None => return Ok(0),
    };
    let Some(mut cache) = read_status_bar_cache_value(&path) else {
        return Ok(0);
    };
    let max_age_seconds = parsed.max_age_seconds.unwrap_or(120);
    let timeout = Duration::from_millis(parsed.timeout_ms.unwrap_or(1500).max(1));
    let refresh_config = agent_usage_refresh_config_from_status_cache_path(&path);
    if refresh_status_bar_cache_agent_usage_value(
        &mut cache,
        env::var_os("PATH").as_deref(),
        &refresh_config,
        unix_time_seconds(),
        max_age_seconds,
        timeout,
    ) {
        write_status_bar_cache_value(&path, &cache)?;
        mark_status_cache_refresh_finished(&path, "agent_usage")?;
    }
    Ok(0)
}

pub fn run_zellij_agent_usage(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_agent_usage_args(args)?;
    if parsed.help {
        print_zellij_agent_usage_help();
        return Ok(0);
    }
    let Some(provider) = parsed
        .provider
        .as_deref()
        .and_then(parse_agent_usage_provider)
    else {
        return Err(CoreError::usage(
            "zellij agent-usage requires one of: claude".to_string(),
        ));
    };
    let target = default_agent_usage_target_for_provider(provider);
    let path_var = env::var_os("PATH");

    if let Some(path_var) = path_var.as_deref() {
        for command in agent_usage_command_candidates(target) {
            let Some(binary_path) = find_command_in_path_var(path_var, command.binary) else {
                continue;
            };
            let output = Command::new(binary_path)
                .args(command.args.as_slice())
                .output()
                .map_err(|source| {
                    CoreError::io(
                        "agent_usage_failed",
                        "Failed to run the configured agent usage provider.",
                        "Ensure the opt-in agent usage package is healthy, then retry.",
                        command.binary,
                        source,
                    )
                })?;
            if !output.status.success() {
                continue;
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let summary = agent_usage_summary_from_json(target, command.dialect, &stdout);
            if !summary.is_empty() {
                println!("{}", render_agent_usage_widget(target.label, &summary));
                return Ok(0);
            }
        }
    }

    println!();
    Ok(0)
}

fn status_bar_cache_path_from_env() -> Option<PathBuf> {
    status_bar_cache_path_from_values(
        env::var_os("YAZELIX_STATUS_BAR_CACHE_PATH").map(PathBuf::from),
        env::var_os("YAZELIX_SESSION_CONFIG_PATH").map(PathBuf::from),
    )
    .or_else(status_bar_cache_path_from_parent_process_env)
}

fn status_bar_cache_path_from_values(
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

fn session_config_path_from_env() -> Option<PathBuf> {
    session_config_path_from_values(
        env::var_os("YAZELIX_SESSION_CONFIG_PATH")
            .map(PathBuf::from)
            .or_else(session_config_path_from_parent_process_env),
        env::var_os("YAZELIX_STATUS_BAR_CACHE_PATH")
            .map(PathBuf::from)
            .or_else(status_bar_cache_path_from_parent_process_env),
    )
}

fn session_config_path_from_values(
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
fn status_bar_cache_path_from_parent_process_env() -> Option<PathBuf> {
    path_from_parent_process_env(status_bar_cache_path_from_environ_bytes)
}

#[cfg(target_os = "linux")]
fn session_config_path_from_parent_process_env() -> Option<PathBuf> {
    path_from_parent_process_env(session_config_path_from_environ_bytes)
}

#[cfg(target_os = "linux")]
fn path_from_parent_process_env(extract: fn(&[u8]) -> Option<PathBuf>) -> Option<PathBuf> {
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
fn status_bar_cache_path_from_parent_process_env() -> Option<PathBuf> {
    None
}

#[cfg(not(target_os = "linux"))]
fn session_config_path_from_parent_process_env() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "linux")]
fn parent_pid(pid: u32) -> Option<u32> {
    let stat_path = PathBuf::from("/proc").join(pid.to_string()).join("stat");
    let raw = fs::read_to_string(stat_path).ok()?;
    let after_name = raw.rsplit_once(") ")?.1;
    let mut fields = after_name.split_whitespace();
    fields.next()?;
    fields.next()?.parse().ok()
}

fn status_bar_cache_path_from_environ_bytes(raw: &[u8]) -> Option<PathBuf> {
    status_bar_cache_path_from_values(
        environ_path_value(raw, b"YAZELIX_STATUS_BAR_CACHE_PATH="),
        session_config_path_from_environ_bytes(raw),
    )
}

fn session_config_path_from_environ_bytes(raw: &[u8]) -> Option<PathBuf> {
    environ_path_value(raw, b"YAZELIX_SESSION_CONFIG_PATH=")
}

fn environ_path_value(raw: &[u8], prefix: &[u8]) -> Option<PathBuf> {
    raw.split(|byte| *byte == 0).find_map(|item| {
        let value = item.strip_prefix(prefix)?;
        (!value.is_empty()).then(|| PathBuf::from(String::from_utf8_lossy(value).to_string()))
    })
}

fn missing_status_bar_cache_path_error() -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_status_bar_cache_path",
        "Yazelix status-bar cache path is not available.",
        "Start a fresh Yazelix window so the launch-scoped session environment is available.",
        json!({}),
    )
}

fn codex_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    let state_dir = status_cache_path.parent()?.parent()?.parent()?;
    Some(state_dir.join("agent_usage").join("codex_usage_cache.json"))
}

fn opencode_go_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    let state_dir = status_cache_path.parent()?.parent()?.parent()?;
    Some(
        state_dir
            .join("agent_usage")
            .join("opencode_go_usage_cache.json"),
    )
}

fn build_status_bar_cache_at(
    status_bus: Value,
    agent_usage: Value,
    agent_usage_updated_at: Option<u64>,
    now: u64,
) -> Value {
    let mut cache = json!({
        "schema_version": STATUS_BAR_CACHE_SCHEMA_VERSION,
        "updated_at_unix_seconds": now,
        "status_bus": status_bus,
        "agent_usage": agent_usage,
    });
    if let Some(updated_at) = agent_usage_updated_at {
        cache["agent_usage_updated_at_unix_seconds"] = json!(updated_at);
    }
    cache
}

#[derive(Debug, Clone)]
struct AgentUsageCacheSeed {
    agent_usage: Value,
    updated_at_unix_seconds: Option<u64>,
}

fn agent_usage_seed_for_status_cache_path(
    path: &Path,
    previous_cache: Option<&Value>,
    now: u64,
) -> Option<AgentUsageCacheSeed> {
    previous_cache
        .and_then(agent_usage_seed_from_current_cache)
        .or_else(|| recent_sibling_agent_usage_seed(path, now))
}

fn agent_usage_seed_from_current_cache(cache: &Value) -> Option<AgentUsageCacheSeed> {
    let agent_usage = non_empty_agent_usage_value(cache)?;
    Some(AgentUsageCacheSeed {
        agent_usage,
        updated_at_unix_seconds: cache
            .get("agent_usage_updated_at_unix_seconds")
            .and_then(Value::as_u64),
    })
}

fn recent_sibling_agent_usage_seed(path: &Path, now: u64) -> Option<AgentUsageCacheSeed> {
    let sessions_dir = path.parent()?.parent()?;
    let mut candidates = status_bar_cache_sibling_candidates(sessions_dir, path);
    candidates.sort_by(|left, right| right.modified_unix_seconds.cmp(&left.modified_unix_seconds));

    candidates
        .into_iter()
        .take(AGENT_USAGE_SEED_CANDIDATE_LIMIT)
        .find_map(|candidate| {
            let cache = read_status_bar_cache_value(&candidate.path)?;
            let seed = agent_usage_seed_from_current_cache(&cache)?;
            let updated_at = seed.updated_at_unix_seconds?;
            if now.saturating_sub(updated_at) <= AGENT_USAGE_SEED_MAX_AGE_SECONDS {
                Some(seed)
            } else {
                None
            }
        })
}

#[derive(Debug, Clone)]
struct StatusBarCacheCandidate {
    path: PathBuf,
    modified_unix_seconds: u64,
}

fn status_bar_cache_sibling_candidates(
    sessions_dir: &Path,
    current_path: &Path,
) -> Vec<StatusBarCacheCandidate> {
    let Ok(entries) = fs::read_dir(sessions_dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let cache_path = entry.path().join("status_bar_cache.json");
            if cache_path == current_path {
                return None;
            }
            let modified_unix_seconds = fs::metadata(&cache_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())?;
            Some(StatusBarCacheCandidate {
                path: cache_path,
                modified_unix_seconds,
            })
        })
        .collect()
}

fn non_empty_agent_usage_value(cache: &Value) -> Option<Value> {
    cache
        .get("agent_usage")
        .filter(|usage| usage.as_object().is_some_and(|entries| !entries.is_empty()))
        .cloned()
}

fn unix_time_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn write_status_bar_cache_value(path: &Path, cache: &Value) -> Result<(), CoreError> {
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

fn decode_orchestrator_heartbeat_payload(raw: &str) -> Result<Value, CoreError> {
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

fn merge_status_bar_cache_orchestrator_heartbeat_value(
    path: &Path,
    heartbeat: Value,
) -> Result<(), CoreError> {
    let Some(mut cache) = read_status_bar_cache_value(path) else {
        return Ok(());
    };
    merge_orchestrator_heartbeat_into_cache(&mut cache, heartbeat);
    write_status_bar_cache_value(path, &cache)
}

fn mark_status_cache_refresh_finished(path: &Path, refresh_name: &str) -> Result<(), CoreError> {
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

fn merge_orchestrator_heartbeat_into_cache(cache: &mut Value, incoming: Value) {
    let existing = cache.get("orchestrator_heartbeat").cloned();
    cache["orchestrator_heartbeat"] = merge_orchestrator_heartbeat_values(existing, incoming);
}

fn merge_orchestrator_heartbeat_values(existing: Option<Value>, incoming: Value) -> Value {
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

fn merge_status_refresh_heartbeat_values(existing: Option<Value>, incoming: Value) -> Value {
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

fn write_json_value_atomic(
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

fn temporary_status_bar_cache_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("status_bar_cache.json");
    path.with_file_name(format!(".{file_name}.tmp"))
}

fn read_status_bar_cache_value(path: &Path) -> Option<Value> {
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

fn hydrate_status_cache_codex_usage(cache: &mut Value, status_cache_path: &Path) {
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

fn hydrate_status_cache_opencode_go_usage(cache: &mut Value, status_cache_path: &Path) {
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

fn read_codex_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64) != Some(CODEX_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

fn read_opencode_go_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64)
        != Some(OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

fn status_bar_cache_status_bus(cache: &Value) -> Option<&Value> {
    let status_bus = cache.get("status_bus")?;
    if status_bus.get("schema_version").and_then(Value::as_i64) == Some(STATUS_BUS_SCHEMA_VERSION) {
        Some(status_bus)
    } else {
        None
    }
}

#[cfg(test)]
fn render_status_cache_widget(cache: &Value, widget: &str) -> Result<String, CoreError> {
    render_status_cache_widget_with_agent_usage_display(cache, widget, AgentUsageDisplay::Both)
}

#[cfg(test)]
fn render_status_cache_widget_with_agent_usage_display(
    cache: &Value,
    widget: &str,
    agent_usage_display: AgentUsageDisplay,
) -> Result<String, CoreError> {
    let settings = AgentUsageWidgetSettings {
        display: agent_usage_display,
        ..AgentUsageWidgetSettings::default()
    };
    render_status_cache_widget_with_agent_usage_settings(cache, widget, &settings)
}

fn render_status_cache_widget_with_agent_usage_settings(
    cache: &Value,
    widget: &str,
    settings: &AgentUsageWidgetSettings,
) -> Result<String, CoreError> {
    let status_bus = status_bar_cache_status_bus(cache);
    match widget {
        "workspace" => Ok(status_bus
            .map(render_zjstatus_workspace_widget)
            .unwrap_or_default()),
        "claude_usage" => Ok(render_grouped_agent_usage_segment(
            cache,
            AgentUsageProvider::Claude,
            settings.claude_periods.as_slice(),
            settings.display,
        )),
        "codex_usage" => Ok(render_windowed_usage_segment(
            cache,
            "codex_usage",
            "codex",
            default_windowed_usage_periods(),
            settings.codex_display,
        )),
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

fn status_cache_widget_names() -> Vec<&'static str> {
    vec![
        "workspace",
        "claude_usage",
        "codex_usage",
        "opencode_go_usage",
    ]
}

fn render_grouped_agent_usage_segment(
    cache: &Value,
    provider: AgentUsageProvider,
    periods: &[AgentUsagePeriod],
    display: AgentUsageDisplay,
) -> String {
    let mut parts = Vec::new();
    for period in periods {
        let Some(target) = agent_usage_target_for_provider_period(provider, *period) else {
            continue;
        };
        let Some(entry) = cached_agent_usage_entry(cache, target) else {
            continue;
        };
        let facts = agent_usage_facts_from_cache_entry(entry);
        let summary = facts.display_summary(display);
        if !summary.is_empty() {
            parts.push(format!("{} {}", period.short_label(), summary));
        }
    }

    if parts.is_empty() {
        String::new()
    } else {
        render_agent_usage_widget(agent_usage_provider_label(provider), &parts.join(" | "))
    }
}

fn render_windowed_usage_segment(
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

fn render_windowed_usage_summary(
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

fn render_windowed_usage_window(
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
            pieces.push(format_quota_percent(remaining_percent?));
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

fn windowed_usage_facts_from_cache_entry(entry: &Value) -> WindowedUsageFacts {
    WindowedUsageFacts {
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
        error: entry
            .get("error")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

fn cached_agent_usage_entry(cache: &Value, target: AgentUsageTarget) -> Option<&Value> {
    cache.get("agent_usage")?.get(target.cache_key)
}

fn refresh_status_bar_cache_agent_usage_value(
    cache: &mut Value,
    path_var: Option<&OsStr>,
    refresh_config: &AgentUsageRefreshConfig,
    now: u64,
    max_age_seconds: u64,
    timeout: Duration,
) -> bool {
    if agent_usage_cache_is_fresh(cache, now, max_age_seconds) {
        return false;
    }

    let agent_usage = collect_agent_usage_entries(path_var, refresh_config, now, timeout);
    cache["agent_usage"] = agent_usage;
    cache["agent_usage_updated_at_unix_seconds"] = json!(now);
    true
}

fn agent_usage_cache_is_fresh(cache: &Value, now: u64, max_age_seconds: u64) -> bool {
    cache
        .get("agent_usage_updated_at_unix_seconds")
        .and_then(Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get("agent_usage")
                    .and_then(Value::as_object)
                    .is_some_and(|usage| !usage.is_empty())
        })
}

fn collect_agent_usage_entries(
    path_var: Option<&OsStr>,
    refresh_config: &AgentUsageRefreshConfig,
    now: u64,
    timeout: Duration,
) -> Value {
    let mut usage = serde_json::Map::new();
    for target in configured_agent_usage_targets(refresh_config) {
        let Some(facts) = agent_usage_facts_from_provider(target, path_var, timeout) else {
            continue;
        };
        let mut entry = serde_json::Map::new();
        entry.insert("updated_at_unix_seconds".to_string(), json!(now));
        entry.insert("summary".to_string(), json!(facts.summary()));
        if let Some(tokens) = facts.tokens {
            entry.insert("tokens".to_string(), json!(tokens));
        }
        if let Some(cost) = facts.cost {
            entry.insert("cost".to_string(), json!(cost));
        }
        if let Some(remaining) = facts.remaining {
            entry.insert("remaining".to_string(), json!(remaining));
        }
        usage.insert(target.cache_key.to_string(), Value::Object(entry));
    }
    Value::Object(usage)
}

fn refresh_codex_usage_shared_cache(
    shared_path: &Path,
    path_var: Option<&OsStr>,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
    timeout: Duration,
) -> Result<bool, CoreError> {
    if codex_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds) {
        return Ok(false);
    }
    if codex_usage_shared_cache_is_backing_off(shared_path, now) {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_codex_usage_cache_lock(shared_path, now)? else {
        return Ok(false);
    };
    if codex_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds)
        || codex_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }

    let facts = collect_codex_usage_facts(path_var, timeout);
    let mut codex = serde_json::Map::new();
    codex.insert("updated_at_unix_seconds".to_string(), json!(now));
    if let Some(tokens) = facts.five_hour_tokens {
        codex.insert("five_hour_tokens".to_string(), json!(tokens));
    }
    if let Some(tokens) = facts.weekly_tokens {
        codex.insert("weekly_tokens".to_string(), json!(tokens));
    }
    if let Some(percent) = facts.five_hour_remaining_percent {
        codex.insert("five_hour_remaining_percent".to_string(), json!(percent));
    }
    if let Some(percent) = facts.weekly_remaining_percent {
        codex.insert("weekly_remaining_percent".to_string(), json!(percent));
    }
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        codex.insert("error".to_string(), json!(error));
        codex.insert(
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
    codex.insert("status".to_string(), json!(status));

    let cache = json!({
        "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
        "codex": Value::Object(codex),
    });
    write_json_value_atomic(shared_path, &cache, "codex_usage_cache")?;
    Ok(true)
}

fn codex_usage_shared_cache_is_fresh(path: &Path, now: u64, max_age_seconds: u64) -> bool {
    let Some(cache) = read_codex_usage_shared_cache_value(path) else {
        return false;
    };
    cache
        .get("codex")
        .and_then(|codex| codex.get("updated_at_unix_seconds"))
        .and_then(Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get("codex")
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| !facts.is_empty())
        })
}

fn codex_usage_shared_cache_is_backing_off(path: &Path, now: u64) -> bool {
    read_codex_usage_shared_cache_value(path)
        .and_then(|cache| {
            cache
                .get("codex")?
                .get("backoff_until_unix_seconds")?
                .as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

struct CodexUsageCacheLock {
    path: PathBuf,
}

impl Drop for CodexUsageCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn try_acquire_codex_usage_cache_lock(
    shared_path: &Path,
    now: u64,
) -> Result<Option<CodexUsageCacheLock>, CoreError> {
    let lock_path = shared_path.with_file_name(".codex_usage_cache.lock");
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "codex_usage_cache_lock_parent_create_failed",
                "Failed to create the Yazelix Codex usage cache lock directory.",
                "Check permissions for the Yazelix state directory, then retry.",
                &parent.display().to_string(),
                source,
            )
        })?;
    }
    match fs::create_dir(&lock_path) {
        Ok(()) => Ok(Some(CodexUsageCacheLock { path: lock_path })),
        Err(source) if source.kind() == ErrorKind::AlreadyExists => {
            if codex_usage_cache_lock_is_stale(&lock_path, now) {
                let _ = fs::remove_dir(&lock_path);
                return match fs::create_dir(&lock_path) {
                    Ok(()) => Ok(Some(CodexUsageCacheLock { path: lock_path })),
                    Err(source) if source.kind() == ErrorKind::AlreadyExists => Ok(None),
                    Err(source) => Err(CoreError::io(
                        "codex_usage_cache_lock_create_failed",
                        "Failed to acquire the Yazelix Codex usage cache lock.",
                        "Check permissions for the Yazelix state directory, then retry.",
                        &lock_path.display().to_string(),
                        source,
                    )),
                };
            }
            Ok(None)
        }
        Err(source) => Err(CoreError::io(
            "codex_usage_cache_lock_create_failed",
            "Failed to acquire the Yazelix Codex usage cache lock.",
            "Check permissions for the Yazelix state directory, then retry.",
            &lock_path.display().to_string(),
            source,
        )),
    }
}

fn codex_usage_cache_lock_is_stale(lock_path: &Path, now: u64) -> bool {
    fs::metadata(lock_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| {
            now.saturating_sub(duration.as_secs()) > CODEX_USAGE_LOCK_STALE_AFTER_SECONDS
        })
        .unwrap_or(false)
}

fn refresh_opencode_go_usage_shared_cache(
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

fn refresh_opencode_go_usage_shared_cache_from_dbs(
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

fn opencode_go_usage_shared_cache_is_fresh(path: &Path, now: u64, max_age_seconds: u64) -> bool {
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
                    .is_some_and(|facts| !facts.is_empty())
        })
}

fn opencode_go_usage_shared_cache_is_backing_off(path: &Path, now: u64) -> bool {
    read_opencode_go_usage_shared_cache_value(path)
        .and_then(|cache| {
            cache
                .get("opencode_go")?
                .get("backoff_until_unix_seconds")?
                .as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

struct OpenCodeGoUsageCacheLock {
    path: PathBuf,
}

impl Drop for OpenCodeGoUsageCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn try_acquire_opencode_go_usage_cache_lock(
    shared_path: &Path,
    now: u64,
) -> Result<Option<OpenCodeGoUsageCacheLock>, CoreError> {
    let lock_path = shared_path.with_file_name(".opencode_go_usage_cache.lock");
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

fn opencode_go_usage_cache_lock_is_stale(lock_path: &Path, now: u64) -> bool {
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
struct OpenCodeGoUsageWindow {
    tokens: u64,
    cost_usd: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct OpenCodeGoUsageWindows {
    five_hour: OpenCodeGoUsageWindow,
    weekly: OpenCodeGoUsageWindow,
    monthly: OpenCodeGoUsageWindow,
}

fn collect_opencode_go_usage_facts_from_dbs(db_paths: &[PathBuf], now: u64) -> WindowedUsageFacts {
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
    if five_hour.tokens > 0 || five_hour.cost_usd > 0.0 {
        facts.five_hour_tokens = Some(five_hour.tokens);
        facts.five_hour_remaining_percent = Some(remaining_percent_from_cost_limit(
            five_hour.cost_usd,
            OPENCODE_GO_FIVE_HOUR_LIMIT_USD,
        ));
    }
    if weekly.tokens > 0 || weekly.cost_usd > 0.0 {
        facts.weekly_tokens = Some(weekly.tokens);
        facts.weekly_remaining_percent = Some(remaining_percent_from_cost_limit(
            weekly.cost_usd,
            OPENCODE_GO_WEEKLY_LIMIT_USD,
        ));
    }
    if monthly.tokens > 0 || monthly.cost_usd > 0.0 {
        facts.monthly_tokens = Some(monthly.tokens);
        facts.monthly_remaining_percent = Some(remaining_percent_from_cost_limit(
            monthly.cost_usd,
            OPENCODE_GO_MONTHLY_LIMIT_USD,
        ));
    }
    if facts.is_empty() {
        facts.error = Some("OpenCode Go usage unavailable".to_string());
    }
    facts
}

fn collect_opencode_go_usage_windows_from_db(
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

fn query_opencode_go_usage_window(
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

fn unix_millis_from_seconds_saturating(seconds: u64) -> i64 {
    seconds.saturating_mul(1000).min(i64::MAX as u64) as i64
}

fn remaining_percent_from_cost_limit(cost_usd: f64, limit_usd: f64) -> u64 {
    if limit_usd <= 0.0 {
        return 0;
    }
    (100.0 - (cost_usd / limit_usd * 100.0))
        .clamp(0.0, 100.0)
        .round() as u64
}

fn opencode_db_candidates_from_env() -> Vec<PathBuf> {
    opencode_db_candidates_from_values(
        env::var_os("OPENCODE_DB").map(PathBuf::from),
        env::var_os("OPENCODE_DATA_DIR").map(PathBuf::from),
        env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        env::var_os("HOME").map(PathBuf::from),
    )
}

fn opencode_db_candidates_from_values(
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

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

fn collect_codex_usage_facts(path_var: Option<&OsStr>, timeout: Duration) -> WindowedUsageFacts {
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
    match run_codex_usage_json_command(
        &binary_path,
        &[
            "blocks",
            "--active",
            "--json",
            "--offline",
            "--no-claude",
            "--no-antigravity",
        ],
        timeout,
    ) {
        Ok(Some(value)) => {
            facts.five_hour_tokens = codex_active_block_tokens_from_json(&value);
        }
        Ok(None) => facts.error = Some("active block unavailable".to_string()),
        Err(error) => facts.error = Some(error),
    }

    match run_codex_usage_json_command(
        &binary_path,
        &[
            "weekly",
            "--json",
            "--offline",
            "--no-claude",
            "--no-antigravity",
            "--order",
            "desc",
        ],
        timeout,
    ) {
        Ok(Some(value)) => {
            facts.weekly_tokens = codex_weekly_tokens_from_json(&value);
        }
        Ok(None) => {
            facts.error = facts
                .error
                .or_else(|| Some("weekly usage unavailable".to_string()))
        }
        Err(error) => facts.error = facts.error.or(Some(error)),
    }

    match run_codex_usage_json_command(
        &binary_path,
        &[
            "blocks",
            "--active",
            "--json",
            "--official-limits",
            "--no-claude",
            "--no-antigravity",
        ],
        timeout,
    ) {
        Ok(Some(value)) => {
            let quota = codex_quota_from_official_json(&value);
            facts.five_hour_remaining_percent = quota.five_hour_remaining_percent;
            facts.weekly_remaining_percent = quota.weekly_remaining_percent;
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

    facts
}

fn run_codex_usage_json_command(
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

fn codex_active_block_tokens_from_json(value: &Value) -> Option<u64> {
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

fn codex_weekly_tokens_from_json(value: &Value) -> Option<u64> {
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

fn codex_quota_from_official_json(value: &Value) -> WindowedUsageFacts {
    let Some(official) = value.get("official_codex") else {
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
        ..WindowedUsageFacts::default()
    }
}

fn remaining_percent_from_used(used_percent: f64) -> u64 {
    (100.0 - used_percent).clamp(0.0, 100.0).round() as u64
}

fn agent_usage_refresh_config_from_status_cache_path(
    status_cache_path: &Path,
) -> AgentUsageRefreshConfig {
    let Some(config) = normalized_session_config_for_status_cache_path(status_cache_path) else {
        return AgentUsageRefreshConfig::default();
    };
    agent_usage_refresh_config_from_normalized_config(&config)
}

fn agent_usage_refresh_config_from_normalized_config(config: &Value) -> AgentUsageRefreshConfig {
    AgentUsageRefreshConfig {
        widgets: agent_usage_widget_names_from_config(&config),
        claude_periods: agent_usage_periods_from_config(&config, "zellij_claude_usage_periods"),
    }
}

fn normalized_session_config_from_env() -> Option<Value> {
    let path = session_config_path_from_env()?;
    normalized_session_config_from_path(&path)
}

fn normalized_session_config_for_status_cache_path(status_cache_path: &Path) -> Option<Value> {
    normalized_session_config_from_status_cache_path(status_cache_path)
        .or_else(normalized_session_config_from_env)
}

fn normalized_session_config_from_status_cache_path(status_cache_path: &Path) -> Option<Value> {
    let path = session_config_path_from_values(None, Some(status_cache_path.to_path_buf()))?;
    normalized_session_config_from_path(&path)
}

fn normalized_session_config_from_path(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&raw)
        .ok()?
        .get("normalized_config")
        .cloned()
}

fn agent_usage_widget_settings_from_status_cache_path(
    status_cache_path: &Path,
) -> AgentUsageWidgetSettings {
    let Some(config) = normalized_session_config_for_status_cache_path(status_cache_path) else {
        return AgentUsageWidgetSettings::default();
    };
    AgentUsageWidgetSettings {
        display: config
            .get("zellij_agent_usage_display")
            .and_then(Value::as_str)
            .map(AgentUsageDisplay::parse)
            .unwrap_or(AgentUsageDisplay::Tokens),
        codex_display: config
            .get("zellij_codex_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Both),
        opencode_go_display: config
            .get("zellij_opencode_go_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Both),
        opencode_go_periods: windowed_usage_periods_from_config(
            &config,
            "zellij_opencode_go_usage_periods",
        ),
        claude_periods: agent_usage_periods_from_config(&config, "zellij_claude_usage_periods"),
    }
}

fn usage_widget_enabled_from_status_cache_path(status_cache_path: &Path, widget: &str) -> bool {
    normalized_session_config_for_status_cache_path(status_cache_path)
        .and_then(|config| agent_usage_widget_names_from_config(&config))
        .is_some_and(|widgets| widgets.contains(widget))
}

fn agent_usage_widget_names_from_config(config: &Value) -> Option<BTreeSet<String>> {
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

fn agent_usage_periods_from_config(config: &Value, key: &str) -> Vec<AgentUsagePeriod> {
    let periods = config
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter_map(AgentUsagePeriod::parse_config)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if periods.is_empty() {
        vec![AgentUsagePeriod::Daily]
    } else {
        periods
    }
}

fn windowed_usage_periods_from_config(config: &Value, key: &str) -> Vec<WindowedUsagePeriod> {
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
        default_opencode_go_usage_periods()
    } else {
        periods
    }
}

fn agent_usage_facts_from_cache_entry(entry: &Value) -> AgentUsageFacts {
    let mut facts = AgentUsageFacts {
        tokens: entry
            .get("tokens")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        cost: entry
            .get("cost")
            .or_else(|| entry.get("money"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        remaining: entry
            .get("remaining")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    };

    if facts.is_empty() {
        if let Some(summary) = entry
            .get("summary")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
        {
            facts = agent_usage_facts_from_summary(summary);
        }
    }
    facts
}

fn agent_usage_facts_from_summary(summary: &str) -> AgentUsageFacts {
    let mut facts = AgentUsageFacts::default();
    for part in summary.split_whitespace() {
        if part.starts_with('$') {
            if facts.cost.is_none() {
                facts.cost = Some(part.to_string());
            }
        } else if looks_like_token_count(part) {
            if facts.tokens.is_none() {
                facts.tokens = Some(part.to_string());
            }
        } else if facts.remaining.is_none() {
            facts.remaining = Some(part.to_string());
        }
    }
    facts
}

fn looks_like_token_count(value: &str) -> bool {
    let raw = value.trim();
    if raw.is_empty() {
        return false;
    }
    raw.chars().all(|c| c.is_ascii_digit())
        || ['k', 'M', 'B'].iter().any(|suffix| {
            raw.strip_suffix(*suffix).is_some_and(|number| {
                !number.is_empty() && number.chars().all(|c| c.is_ascii_digit() || c == '.')
            })
        })
}

fn agent_usage_facts_from_provider(
    target: AgentUsageTarget,
    path_var: Option<&OsStr>,
    timeout: Duration,
) -> Option<AgentUsageFacts> {
    let path_var = path_var?;
    for command in agent_usage_command_candidates(target) {
        let Some(binary_path) = find_command_in_path_var(path_var, command.binary) else {
            continue;
        };
        let output =
            run_agent_usage_command_with_timeout(&binary_path, command.args.as_slice(), timeout)
                .ok()??;
        if !output.status.success() {
            continue;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(facts) = agent_usage_facts_from_json(target, command.dialect, &stdout) {
            return Some(facts);
        }
    }
    None
}

fn run_agent_usage_command_with_timeout(
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

fn render_status_bus_workspace_widget(value: &Value) -> String {
    let root = nested_str(value, &["workspace", "root"]).unwrap_or("");
    Path::new(root)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("none")
        .to_string()
}

fn render_zjstatus_workspace_widget(value: &Value) -> String {
    if nested_str(value, &["workspace", "root"])
        .map(str::trim)
        .filter(|root| !root.is_empty())
        .is_none()
    {
        return String::new();
    }
    format!(" [{}]", render_status_bus_workspace_widget(value))
}

fn print_optional_zjstatus_segment(segment: String) {
    if !segment.is_empty() {
        println!("{segment}");
    }
}

fn parse_agent_usage_provider(raw: &str) -> Option<AgentUsageProvider> {
    match raw.trim() {
        "claude" => Some(AgentUsageProvider::Claude),
        _ => None,
    }
}

fn agent_usage_binary(_provider: AgentUsageProvider) -> &'static str {
    "tu"
}

fn default_agent_usage_target_for_provider(provider: AgentUsageProvider) -> AgentUsageTarget {
    match provider {
        AgentUsageProvider::Claude => CLAUDE_USAGE_TARGET,
    }
}

const CLAUDE_USAGE_TARGET: AgentUsageTarget = AgentUsageTarget {
    provider: AgentUsageProvider::Claude,
    period: AgentUsagePeriod::Daily,
    cache_key: "claude",
    label: "claude|d",
};
const CLAUDE_DAILY_USAGE_TARGET: AgentUsageTarget = AgentUsageTarget {
    provider: AgentUsageProvider::Claude,
    period: AgentUsagePeriod::Daily,
    cache_key: "claude_daily",
    label: "claude|d",
};
const CLAUDE_MONTHLY_USAGE_TARGET: AgentUsageTarget = AgentUsageTarget {
    provider: AgentUsageProvider::Claude,
    period: AgentUsagePeriod::Monthly,
    cache_key: "claude_monthly",
    label: "claude|mon",
};

fn agent_usage_target_for_provider_period(
    provider: AgentUsageProvider,
    period: AgentUsagePeriod,
) -> Option<AgentUsageTarget> {
    match (provider, period) {
        (AgentUsageProvider::Claude, AgentUsagePeriod::Daily) => Some(CLAUDE_DAILY_USAGE_TARGET),
        (AgentUsageProvider::Claude, AgentUsagePeriod::Monthly) => {
            Some(CLAUDE_MONTHLY_USAGE_TARGET)
        }
    }
}

fn agent_usage_provider_label(_provider: AgentUsageProvider) -> &'static str {
    "claude"
}

fn configured_agent_usage_targets(
    refresh_config: &AgentUsageRefreshConfig,
) -> Vec<AgentUsageTarget> {
    let mut targets = Vec::new();
    let Some(widgets) = refresh_config.widgets.as_ref() else {
        targets.push(CLAUDE_DAILY_USAGE_TARGET);
        return targets;
    };

    if widgets.contains("claude_usage") {
        for period in &refresh_config.claude_periods {
            if let Some(target) =
                agent_usage_target_for_provider_period(AgentUsageProvider::Claude, *period)
            {
                push_unique_agent_usage_target(&mut targets, target);
            }
        }
    }
    targets
}

fn push_unique_agent_usage_target(targets: &mut Vec<AgentUsageTarget>, target: AgentUsageTarget) {
    if !targets.contains(&target) {
        targets.push(target);
    }
}

fn agent_usage_command_candidates(target: AgentUsageTarget) -> Vec<AgentUsageCommand> {
    let (args, dialect) = match (target.provider, target.period) {
        (AgentUsageProvider::Claude, AgentUsagePeriod::Daily) => (
            vec![
                "today",
                "--json",
                "--offline",
                "--no-codex",
                "--no-antigravity",
            ],
            AgentUsageJsonDialect::Tokenusage,
        ),
        (AgentUsageProvider::Claude, AgentUsagePeriod::Monthly) => (
            vec![
                "monthly",
                "--json",
                "--offline",
                "--no-codex",
                "--no-antigravity",
                "--order",
                "desc",
            ],
            AgentUsageJsonDialect::Tokenusage,
        ),
    };
    vec![AgentUsageCommand {
        binary: agent_usage_binary(target.provider),
        args,
        dialect,
    }]
}

fn find_command_in_path_var(path_var: &OsStr, command_name: &str) -> Option<PathBuf> {
    env::split_paths(path_var)
        .map(|entry| entry.join(command_name))
        .find(|candidate| candidate.is_file())
}

fn agent_usage_summary_from_json(
    target: AgentUsageTarget,
    dialect: AgentUsageJsonDialect,
    raw: &str,
) -> String {
    agent_usage_facts_from_json(target, dialect, raw)
        .map(|facts| facts.summary())
        .unwrap_or_default()
}

fn agent_usage_facts_from_json(
    target: AgentUsageTarget,
    dialect: AgentUsageJsonDialect,
    raw: &str,
) -> Option<AgentUsageFacts> {
    let Some(json_raw) = extract_json_object(raw) else {
        return None;
    };
    let Ok(value) = serde_json::from_str::<Value>(json_raw) else {
        return None;
    };
    let selected = match dialect {
        AgentUsageJsonDialect::Tokenusage => tokenusage_agent_usage_value(target.period, &value),
    };

    agent_usage_facts_from_usage_value(selected)
}

fn tokenusage_agent_usage_value(period: AgentUsagePeriod, value: &Value) -> &Value {
    match period {
        AgentUsagePeriod::Daily => value.get("overview").unwrap_or(value),
        AgentUsagePeriod::Monthly => value
            .get("monthly")
            .and_then(Value::as_array)
            .and_then(|rows| rows.first())
            .unwrap_or(value),
    }
}

fn agent_usage_facts_from_usage_value(selected: &Value) -> Option<AgentUsageFacts> {
    let facts = AgentUsageFacts {
        tokens: first_u64_at(
            selected,
            &[
                &["totalTokens"],
                &["total_tokens"],
                &["usage", "totalTokens"],
                &["usage", "total_tokens"],
                &["totals", "totalTokens"],
                &["totals", "total_tokens"],
                &["totals", "tokens"],
            ],
        )
        .map(format_agent_usage_token_count),
        cost: first_f64_at(
            selected,
            &[
                &["costUSD"],
                &["cost_usd"],
                &["totalCost"],
                &["total_cost"],
                &["totals", "costUSD"],
                &["totals", "cost_usd"],
                &["totals", "totalCost"],
                &["totals", "total_cost"],
            ],
        )
        .map(format_agent_usage_cost),
        remaining: first_i64_at(
            selected,
            &[
                &["remainingMinutes"],
                &["remaining_minutes"],
                &["projection", "remainingMinutes"],
                &["projection", "remaining_minutes"],
            ],
        )
        .filter(|minutes| *minutes > 0)
        .map(format_remaining_minutes),
    };
    (!facts.is_empty()).then_some(facts)
}

fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    (start <= end).then_some(&raw[start..=end])
}

fn render_agent_usage_widget(label: &str, summary: &str) -> String {
    format!(" [{label} {summary}]")
}

fn first_u64_at(value: &Value, paths: &[&[&str]]) -> Option<u64> {
    paths
        .iter()
        .find_map(|path| nested_value(value, path)?.as_u64())
}

fn first_i64_at(value: &Value, paths: &[&[&str]]) -> Option<i64> {
    paths
        .iter()
        .find_map(|path| nested_value(value, path)?.as_i64())
}

fn first_f64_at(value: &Value, paths: &[&[&str]]) -> Option<f64> {
    paths
        .iter()
        .find_map(|path| nested_value(value, path)?.as_f64())
}

fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn format_agent_usage_cost(cost: f64) -> String {
    if cost >= 10.0 {
        format!("${cost:.0}")
    } else if cost >= 1.0 {
        format!("${cost:.2}")
    } else {
        format!("${cost:.3}")
    }
}

fn format_agent_usage_token_count(tokens: u64) -> String {
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

fn format_scaled_agent_usage_count(value: f64, suffix: &str) -> String {
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

fn format_quota_percent(percent: u64) -> String {
    format!("{}%", percent.min(100))
}

fn format_remaining_minutes(minutes: i64) -> String {
    if minutes >= 60 {
        let hours = minutes / 60;
        let remaining = minutes % 60;
        if remaining == 0 {
            format!("{hours}h")
        } else {
            format!("{hours}h{remaining}m")
        }
    } else {
        format!("{minutes}m")
    }
}

fn decode_status_bus_snapshot(raw: &str) -> Result<Value, CoreError> {
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

fn render_session_state_inspection_lines(value: &Value) -> Vec<String> {
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

fn nested_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn nested_bool(value: &Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_bool()
}

fn parse_zellij_open_editor_args(args: &[String]) -> Result<ZellijOpenEditorArgs, CoreError> {
    let mut parsed = ZellijOpenEditorArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij open-editor: {other}"
                )));
            }
            other => parsed.targets.push(other.to_string()),
        }
    }
    Ok(parsed)
}

fn parse_zellij_open_editor_cwd_args(
    args: &[String],
) -> Result<ZellijOpenEditorCwdArgs, CoreError> {
    let mut parsed = ZellijOpenEditorCwdArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij open-editor-cwd: {other}"
                )));
            }
            other => {
                if parsed.target.is_some() {
                    return Err(CoreError::usage(
                        "zellij open-editor-cwd accepts only one target path".to_string(),
                    ));
                }
                parsed.target = Some(other.to_string());
            }
        }
    }
    Ok(parsed)
}

fn print_zellij_open_editor_help() {
    println!("Open one or more files in the configured editor from a Yazi-managed flow");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij open-editor <path> [path ...]");
}

fn print_zellij_open_editor_cwd_help() {
    println!("Open a directory in the managed editor pane from the Yazi zoxide flow");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij open-editor-cwd <path>");
}

fn current_tab_workspace_root_from_json(raw: &str, include_bootstrap: bool) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(raw).ok()?;
    let workspace = parsed.get("workspace")?;
    let root = workspace.get("root")?.as_str()?.trim();
    if root.is_empty() {
        return None;
    }
    let source = workspace
        .get("source")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if !include_bootstrap && source == "bootstrap" {
        return None;
    }
    Some(root.to_string())
}

pub fn run_zellij_get_workspace_root(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_get_workspace_root_args(args)?;
    if parsed.help {
        print_zellij_get_workspace_root_help();
        return Ok(0);
    }

    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    match current_tab_workspace_root_from_json(&response, parsed.include_bootstrap) {
        Some(root) => {
            println!("{}", root);
            Ok(0)
        }
        None => {
            eprintln!("No workspace root available in the current tab session state.");
            Ok(1)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijRetargetArgs {
    target: Option<String>,
    editor: Option<String>,
    help: bool,
}

fn parse_zellij_retarget_args(args: &[String]) -> Result<ZellijRetargetArgs, CoreError> {
    let mut parsed = ZellijRetargetArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--editor" => {
                parsed.editor = Some(
                    iter.next()
                        .ok_or_else(|| CoreError::usage("--editor requires a value".to_string()))?
                        .to_string(),
                );
            }
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij retarget: {other}"
                )));
            }
            other => {
                if parsed.target.is_some() {
                    return Err(CoreError::usage(
                        "zellij retarget accepts only one target path".to_string(),
                    ));
                }
                parsed.target = Some(other.to_string());
            }
        }
    }

    Ok(parsed)
}

fn print_zellij_retarget_help() {
    println!("Retarget the current tab workspace without changing the focused pane cwd");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij retarget <path> [--editor <kind>]");
    println!();
    println!(
        "This is the internal workspace-retarget primitive that does not cd the focused pane."
    );
}

fn resolve_target_dir(target_path: &str) -> Result<PathBuf, CoreError> {
    let path = PathBuf::from(target_path);
    let expanded = if path.is_absolute() {
        path
    } else {
        env::current_dir()
            .map_err(|source| {
                CoreError::io(
                    "retarget_cwd",
                    "Could not read the current working directory.",
                    "cd into a valid directory, then retry.",
                    ".",
                    source,
                )
            })?
            .join(path)
    };

    let canonical = std::fs::canonicalize(&expanded).unwrap_or(expanded);

    if !canonical.exists() {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "missing_workspace_target",
            format!("Path does not exist: {}", canonical.display()),
            "Choose an existing directory or file path, then retry.",
            json!({ "path": canonical.display().to_string() }),
        ));
    }

    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Ok(canonical
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| canonical.to_path_buf()))
    }
}

fn resolve_existing_target_path(target_path: &str) -> Result<PathBuf, CoreError> {
    let path = PathBuf::from(target_path);
    let expanded = if path.is_absolute() {
        path
    } else {
        env::current_dir()
            .map_err(|source| {
                CoreError::io(
                    "editor_target_cwd",
                    "Could not read the current working directory.",
                    "cd into a valid directory, then retry.",
                    ".",
                    source,
                )
            })?
            .join(path)
    };

    let canonical = std::fs::canonicalize(&expanded).unwrap_or(expanded);
    if !canonical.exists() {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "missing_editor_target",
            format!("Path does not exist: {}", canonical.display()),
            "Choose an existing file or directory path, then retry.",
            json!({ "path": canonical.display().to_string() }),
        ));
    }

    Ok(canonical)
}

fn resolve_existing_target_paths(targets: &[String]) -> Result<Vec<PathBuf>, CoreError> {
    let mut resolved = Vec::new();
    for target in targets {
        let path = resolve_existing_target_path(target)?;
        if !resolved.iter().any(|existing| existing == &path) {
            resolved.push(path);
        }
    }
    Ok(resolved)
}

fn resolve_editor_working_dir(target_path: &Path) -> PathBuf {
    let target_dir = if target_path.is_dir() {
        target_path.to_path_buf()
    } else {
        target_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| target_path.to_path_buf())
    };

    let git_output = Command::new("git")
        .arg("-C")
        .arg(&target_dir)
        .args(["rev-parse", "--show-toplevel"])
        .output();
    match git_output {
        Ok(output) if output.status.success() => {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if root.is_empty() {
                target_dir
            } else {
                PathBuf::from(root)
            }
        }
        _ => target_dir,
    }
}

fn workspace_tab_name(workspace_root: &std::path::Path) -> String {
    workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("unnamed")
        .to_string()
}

fn parse_workspace_retarget_response(raw: &str) -> serde_json::Value {
    let trimmed = raw.trim();
    match trimmed {
        "missing" | "not_ready" | "permissions_denied" | "invalid_payload" => {
            json!({"status": trimmed})
        }
        _ => match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(mut result) => {
                let sidebar_yazi_id = result
                    .get("sidebar_yazi_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().to_string());
                let sidebar_yazi_cwd = result
                    .get("sidebar_yazi_cwd")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                if let Some(id) = sidebar_yazi_id.as_ref() {
                    if !id.is_empty() {
                        if let Some(obj) = result.as_object_mut() {
                            obj.insert(
                                "sidebar_state".to_string(),
                                json!({
                                    "yazi_id": id,
                                    "cwd": sidebar_yazi_cwd,
                                }),
                            );
                            obj.remove("sidebar_yazi_id");
                            obj.remove("sidebar_yazi_cwd");
                        }
                    }
                }
                result
            }
            Err(_) => json!({"status": "error", "reason": trimmed}),
        },
    }
}

fn workspace_retarget_status(result: &Value) -> &str {
    result
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("error")
}

fn sidebar_state_from_retarget_response(
    result: &Value,
) -> Option<crate::workspace_commands::SidebarState> {
    let sidebar = result.get("sidebar_state")?;
    let yazi_id = sidebar.get("yazi_id")?.as_str()?.trim();
    if yazi_id.is_empty() {
        return None;
    }
    let cwd = sidebar
        .get("cwd")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");

    Some(crate::workspace_commands::SidebarState {
        yazi_id: yazi_id.to_string(),
        cwd: cwd.to_string(),
    })
}

fn retarget_workspace_without_focused_cd(
    target_path: &Path,
    editor_kind: Option<&str>,
) -> Result<Value, CoreError> {
    let target_dir = if target_path.is_dir() {
        target_path.to_path_buf()
    } else {
        target_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| target_path.to_path_buf())
    };
    retarget_workspace_dir_without_focused_cd(&target_dir, editor_kind)
}

fn retarget_workspace_dir_without_focused_cd(
    target_dir: &Path,
    editor_kind: Option<&str>,
) -> Result<Value, CoreError> {
    let payload = json!({
        "workspace_root": target_dir.display().to_string(),
        "cd_focused_pane": false,
        "editor": editor_kind
            .map(str::trim)
            .filter(|editor| !editor.is_empty())
            .map(|editor| Value::String(editor.to_string()))
            .unwrap_or(Value::Null),
    })
    .to_string();
    let response = run_pane_orchestrator_command("retarget_workspace", &payload)?;
    Ok(parse_workspace_retarget_response(&response))
}

fn resolve_runtime_editor_launch() -> Result<(serde_json::Map<String, Value>, String), CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let facts = compute_session_facts_from_env()?;
    let mut normalized = serde_json::Map::new();
    normalized.insert("enable_sidebar".to_string(), json!(facts.enable_sidebar));
    normalized.insert(
        "initial_sidebar_state".to_string(),
        json!(facts.initial_sidebar_state),
    );
    if let Some(editor_command) = facts.editor_command {
        normalized.insert("editor_command".to_string(), json!(editor_command));
    }
    if let Some(helix_runtime_path) = facts.helix_runtime_path {
        normalized.insert("helix_runtime_path".to_string(), json!(helix_runtime_path));
    }
    let runtime_env =
        compute_runtime_env(&runtime_env_request(runtime_dir, &normalized)?)?.runtime_env;
    let editor = runtime_env
        .get("EDITOR")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|editor| !editor.is_empty())
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "editor_command_missing",
                "EDITOR is not configured for the Yazelix runtime.",
                "Set [editor].command in yazelix.toml or export EDITOR before running this command.",
                json!({}),
            )
        })?
        .to_string();
    Ok((runtime_env, editor))
}

fn pane_env_assignment(value: OsString) -> String {
    value.to_string_lossy().to_string()
}

fn build_editor_pane_env_assignments(
    runtime_env: &serde_json::Map<String, Value>,
    yazi_id: Option<&str>,
) -> Vec<String> {
    let mut env_assignments = json_map_to_child_env(runtime_env)
        .into_iter()
        .map(|(key, value)| {
            (
                key.to_string_lossy().to_string(),
                pane_env_assignment(value),
            )
        })
        .collect::<Vec<_>>();

    for key in EDITOR_PANE_ENV_OVERRIDE_KEYS {
        if let Ok(value) = env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                if let Some(existing) = env_assignments.iter_mut().find(|(name, _)| name == key) {
                    existing.1 = trimmed.to_string();
                } else {
                    env_assignments.push(((*key).to_string(), trimmed.to_string()));
                }
            }
        }
    }

    if let Some(yazi_id) = yazi_id.map(str::trim).filter(|id| !id.is_empty()) {
        if let Some(existing) = env_assignments
            .iter_mut()
            .find(|(name, _)| name == "YAZI_ID")
        {
            existing.1 = yazi_id.to_string();
        } else {
            env_assignments.push(("YAZI_ID".to_string(), yazi_id.to_string()));
        }
    }

    env_assignments
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect()
}

fn run_zellij_editor_pane(
    working_dir: &Path,
    runtime_env: &serde_json::Map<String, Value>,
    yazi_id: Option<&str>,
    editor_argv: &[String],
) -> Result<(), CoreError> {
    let env_args = build_editor_pane_env_assignments(runtime_env, yazi_id);
    let output = Command::new("zellij")
        .arg("run")
        .arg("--name")
        .arg(EDITOR_PANE_NAME)
        .arg("--cwd")
        .arg(working_dir)
        .arg("--")
        .arg("env")
        .args(env_args)
        .args(editor_argv)
        .output()
        .map_err(|source| {
            CoreError::io(
                "zellij_run_failed",
                "Failed to open a new editor pane through Zellij.",
                "Run this command inside an active Yazelix/Zellij session, then retry.",
                "zellij",
                source,
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let details = if stderr.is_empty() {
        format!("exit code {}", output.status.code().unwrap_or(1))
    } else {
        stderr
    };
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "open_editor_pane_failed",
        format!("Failed to open a new editor pane: {details}"),
        "Ensure Zellij is available in the current Yazelix session, then retry.",
        json!({ "cwd": working_dir.display().to_string() }),
    ))
}

fn open_files_in_managed_editor(
    editor_kind: &str,
    file_paths: &[PathBuf],
    working_dir: &Path,
) -> Result<ManagedEditorOpenStatus, CoreError> {
    let file_path_strings = file_paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    let first_file_path = file_path_strings.first().cloned().unwrap_or_default();
    let payload = json!({
        "editor": editor_kind,
        "file_path": first_file_path,
        "file_paths": file_path_strings,
        "working_dir": working_dir.display().to_string(),
    })
    .to_string();
    let response = run_pane_orchestrator_command("open_file", &payload)?;
    match response.trim() {
        "ok" | "opened" | "focused" => Ok(ManagedEditorOpenStatus::Ok),
        "missing" => Ok(ManagedEditorOpenStatus::Missing),
        other => Err(CoreError::classified(
            ErrorClass::Runtime,
            "managed_editor_open_failed",
            format!("Managed editor open failed: {other}"),
            "Ensure the Yazelix pane orchestrator is loaded and the managed editor pane title is `editor`, then retry.",
            json!({ "response": response }),
        )),
    }
}

fn sync_current_yazi_to_directory(
    ya_command: &str,
    home_dir: &Path,
    yazi_id: &str,
    target_path: &Path,
) {
    if !command_is_available(ya_command, home_dir) {
        return;
    }

    let target_dir = if target_path.is_dir() {
        target_path
    } else {
        target_path.parent().unwrap_or(target_path)
    };
    let target = target_dir.to_string_lossy().to_string();
    let _ = run_ya_emit_to(ya_command, home_dir, yazi_id, "cd", &[target.as_str()]);
}

pub fn run_zellij_retarget(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_retarget_args(args)?;
    if parsed.help {
        print_zellij_retarget_help();
        return Ok(0);
    }

    let target = parsed.target.ok_or_else(|| {
        CoreError::usage(
            "zellij retarget requires a target path. Try `yzx_control zellij retarget --help`."
                .to_string(),
        )
    })?;

    let target_dir = resolve_target_dir(&target)?;
    let tab_name = workspace_tab_name(&target_dir);

    let payload = json!({
        "workspace_root": target_dir.display().to_string(),
        "cd_focused_pane": false,
        "editor": parsed.editor.filter(|e| !e.trim().is_empty()),
    })
    .to_string();

    let response = run_pane_orchestrator_command("retarget_workspace", &payload)?;
    let result = parse_workspace_retarget_response(&response);

    let status = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("error");
    match status {
        "ok" => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "workspace_root": target_dir.display().to_string(),
                    "tab_name": tab_name,
                    "editor_status": result.get("editor_status").and_then(|v| v.as_str()).unwrap_or(""),
                    "sidebar_state": result.get("sidebar_state"),
                })
            );
            Ok(0)
        }
        "not_ready" => {
            eprintln!("❌ Yazelix tab state is not ready yet.");
            eprintln!(
                "   Wait a moment for the pane orchestrator plugin to finish loading, then try again."
            );
            Ok(1)
        }
        "permissions_denied" => {
            eprintln!(
                "❌ The Yazelix pane orchestrator plugin is missing required Zellij permissions."
            );
            eprintln!("   Run `yzx doctor --fix`, then restart Yazelix.");
            Ok(1)
        }
        _ => {
            let reason = result
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            eprintln!("❌ Failed to retarget workspace: {}", reason);
            Ok(1)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijOpenTerminalArgs {
    target: Option<String>,
    help: bool,
}

fn parse_zellij_open_terminal_args(args: &[String]) -> Result<ZellijOpenTerminalArgs, CoreError> {
    let mut parsed = ZellijOpenTerminalArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij open-terminal: {other}"
                )));
            }
            other => {
                if parsed.target.is_some() {
                    return Err(CoreError::usage(
                        "zellij open-terminal accepts only one target path".to_string(),
                    ));
                }
                parsed.target = Some(other.to_string());
            }
        }
    }
    Ok(parsed)
}

fn print_zellij_open_terminal_help() {
    println!("Open a new terminal pane in the given directory via the pane orchestrator");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij open-terminal <path>");
}

pub fn run_zellij_open_editor(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_open_editor_args(args)?;
    if parsed.help {
        print_zellij_open_editor_help();
        return Ok(0);
    }

    if parsed.targets.is_empty() {
        return Err(CoreError::usage(
            "zellij open-editor requires at least one target path. Try `yzx_control zellij open-editor --help`."
                .to_string(),
        ));
    }
    let target_paths = resolve_existing_target_paths(&parsed.targets)?;
    let primary_target_path = target_paths.first().ok_or_else(|| {
        CoreError::usage(
            "zellij open-editor requires at least one target path. Try `yzx_control zellij open-editor --help`."
                .to_string(),
        )
    })?;
    let integration_facts = compute_integration_facts_from_env()?;
    let (runtime_env, editor_command) = resolve_runtime_editor_launch()?;
    let editor_kind = integration_facts.managed_editor_kind.trim().to_string();
    let yazi_id = env::var("YAZI_ID").unwrap_or_default();
    let editor_working_dir = resolve_editor_working_dir(primary_target_path);

    if editor_kind == "helix" || editor_kind == "neovim" {
        let open_status =
            open_files_in_managed_editor(&editor_kind, &target_paths, &editor_working_dir)?;
        if open_status == ManagedEditorOpenStatus::Missing {
            let mut editor_argv = vec![editor_command.clone()];
            editor_argv.extend(target_paths.iter().map(|path| path.display().to_string()));
            run_zellij_editor_pane(
                &editor_working_dir,
                &runtime_env,
                Some(yazi_id.as_str()),
                &editor_argv,
            )?;
        }
    } else {
        let mut editor_argv = vec![editor_command];
        editor_argv.extend(target_paths.iter().map(|path| path.display().to_string()));
        run_zellij_editor_pane(
            &editor_working_dir,
            &runtime_env,
            Some(yazi_id.as_str()),
            &editor_argv,
        )?;
    }

    if let Ok(retarget_result) =
        retarget_workspace_dir_without_focused_cd(&editor_working_dir, None)
    {
        if workspace_retarget_status(&retarget_result) == "ok" {
            if integration_facts.enable_sidebar {
                if let Some(sidebar_state) = sidebar_state_from_retarget_response(&retarget_result)
                {
                    let _ = sync_sidebar_to_directory(
                        &integration_facts.ya_command,
                        &home_dir_from_env()?,
                        &sidebar_state,
                        primary_target_path,
                    );
                }
            } else if !yazi_id.trim().is_empty() {
                sync_current_yazi_to_directory(
                    &integration_facts.ya_command,
                    &home_dir_from_env()?,
                    yazi_id.as_str(),
                    primary_target_path,
                );
            }
        }
    }

    Ok(0)
}

pub fn run_zellij_open_editor_cwd(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_open_editor_cwd_args(args)?;
    if parsed.help {
        print_zellij_open_editor_cwd_help();
        return Ok(0);
    }

    let target = parsed.target.ok_or_else(|| {
        CoreError::usage(
            "zellij open-editor-cwd requires a target path. Try `yzx_control zellij open-editor-cwd --help`."
                .to_string(),
        )
    })?;
    let target_dir = resolve_target_dir(&target)?;
    let integration_facts = compute_integration_facts_from_env()?;
    let editor_kind = integration_facts.managed_editor_kind.trim().to_string();
    if editor_kind.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "managed_editor_missing",
            "No managed editor is configured for the current Yazelix runtime.",
            "Set the configured editor to Helix or Neovim before using the Yazi zoxide editor flow.",
            json!({}),
        ));
    }

    let retarget_result =
        retarget_workspace_without_focused_cd(&target_dir, Some(editor_kind.as_str()))?;
    let status = workspace_retarget_status(&retarget_result);
    if status != "ok" {
        let reason = retarget_result
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or(status);
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "retarget_workspace_failed",
            format!("Failed to retarget the current workspace: {reason}"),
            "Ensure the pane orchestrator plugin is loaded and the current tab is ready, then retry.",
            json!({ "status": status }),
        ));
    }

    match retarget_result
        .get("editor_status")
        .and_then(Value::as_str)
        .unwrap_or("")
    {
        "missing" => {
            let (runtime_env, editor_command) = resolve_runtime_editor_launch()?;
            let yazi_id = env::var("YAZI_ID").unwrap_or_default();
            let mut editor_argv = vec![editor_command];
            if editor_kind == "helix" {
                editor_argv.push(target_dir.display().to_string());
            }
            run_zellij_editor_pane(
                &target_dir,
                &runtime_env,
                Some(yazi_id.as_str()),
                &editor_argv,
            )?;
        }
        "unsupported_editor" => {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "unsupported_managed_editor",
                format!(
                    "Unsupported managed editor kind for workspace retarget: {}",
                    editor_kind
                ),
                "Configure Helix or Neovim as the managed editor, then retry.",
                json!({ "editor_kind": editor_kind }),
            ));
        }
        _ => {}
    }

    if integration_facts.enable_sidebar {
        if let Some(sidebar_state) = sidebar_state_from_retarget_response(&retarget_result) {
            let _ = sync_sidebar_to_directory(
                &integration_facts.ya_command,
                &home_dir_from_env()?,
                &sidebar_state,
                &target_dir,
            );
        }
    }

    Ok(0)
}

pub fn run_zellij_open_terminal(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_open_terminal_args(args)?;
    if parsed.help {
        print_zellij_open_terminal_help();
        return Ok(0);
    }

    let target = parsed.target.ok_or_else(|| {
        CoreError::usage(
            "zellij open-terminal requires a target path. Try `yzx_control zellij open-terminal --help`."
                .to_string(),
        )
    })?;

    let target_dir = resolve_target_dir(&target)?;

    let payload = json!({
        "cwd": target_dir.display().to_string(),
    })
    .to_string();

    let response = run_pane_orchestrator_command("open_terminal_in_cwd", &payload)?;
    if response.trim() != "ok" {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "open_terminal_failed",
            format!(
                "Pane orchestrator failed to open directory pane in '{}': {}",
                target_dir.display(),
                response
            ),
            "Ensure the pane orchestrator plugin is loaded and the current tab is ready, then retry.",
            json!({ "path": target_dir.display().to_string(), "response": response }),
        ));
    }

    let retarget_payload = json!({
        "workspace_root": target_dir.display().to_string(),
        "cd_focused_pane": false,
        "editor": None::<String>,
    })
    .to_string();

    let retarget_response = run_pane_orchestrator_command("retarget_workspace", &retarget_payload)?;
    let retarget_result = parse_workspace_retarget_response(&retarget_response);
    let retarget_status = retarget_result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("error");

    match retarget_status {
        "ok" => {
            println!(
                "{}",
                json!({
                    "status": "ok",
                    "workspace_root": target_dir.display().to_string(),
                })
            );
            Ok(0)
        }
        _ => {
            eprintln!(
                "⚠️  Terminal pane opened, but workspace retarget failed: {}",
                retarget_response
            );
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS_CACHE_TEST_PAYLOAD: &str = r#"{"schema_version":1,"active_tab_position":0,"workspace":{"root":"/tmp/yazelix-demo","source":"explicit"},"managed_panes":{"editor_pane_id":"terminal:1","sidebar_pane_id":"terminal:2"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[]}}"#;

    fn status_cache_test_status_bus() -> Value {
        serde_json::from_str(STATUS_CACHE_TEST_PAYLOAD).unwrap()
    }

    // Defends: workspace root parsing respects the bootstrap exclusion flag.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn current_tab_workspace_root_excludes_bootstrap_when_requested() {
        let json = r#"{"workspace":{"root":"/tmp/demo","source":"bootstrap"}}"#;
        assert_eq!(current_tab_workspace_root_from_json(json, false), None);
        assert_eq!(
            current_tab_workspace_root_from_json(json, true),
            Some("/tmp/demo".to_string())
        );
    }

    // Defends: workspace root parsing includes non-bootstrap sources.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn current_tab_workspace_root_includes_plugin_source() {
        let json = r#"{"workspace":{"root":"/tmp/demo","source":"plugin"}}"#;
        assert_eq!(
            current_tab_workspace_root_from_json(json, false),
            Some("/tmp/demo".to_string())
        );
    }

    // Defends: retarget response parsing extracts sidebar state correctly.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parse_retarget_response_extracts_sidebar_state() {
        let raw = r#"{"status":"ok","editor_status":"ok","sidebar_yazi_id":"yazi-123","sidebar_yazi_cwd":"/home/sidebar"}"#;
        let parsed = parse_workspace_retarget_response(raw);
        assert_eq!(parsed.get("status").and_then(|v| v.as_str()), Some("ok"));
        let sidebar = parsed.get("sidebar_state").unwrap();
        assert_eq!(
            sidebar.get("yazi_id").and_then(|v| v.as_str()),
            Some("yazi-123")
        );
        assert_eq!(
            sidebar.get("cwd").and_then(|v| v.as_str()),
            Some("/home/sidebar")
        );
    }

    // Defends: retarget response parsing handles simple error strings.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn parse_retarget_response_handles_error_strings() {
        assert_eq!(
            parse_workspace_retarget_response("missing"),
            json!({"status": "missing"})
        );
        assert_eq!(
            parse_workspace_retarget_response("permissions_denied"),
            json!({"status": "permissions_denied"})
        );
    }

    // Defends: Yazi selected-file expansion can pass multiple paths through the public open-editor parser in one action.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parse_open_editor_accepts_multiple_targets() {
        let parsed = parse_zellij_open_editor_args(&[
            "/tmp/project/one.txt".to_string(),
            "/tmp/project/two.txt".to_string(),
        ])
        .unwrap();

        assert_eq!(
            parsed.targets,
            vec!["/tmp/project/one.txt", "/tmp/project/two.txt"]
        );
    }

    // Defends: maintainer session inspection renders the stable active-tab snapshot fields used to debug workspace routing.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn session_inspection_lines_include_workspace_layout_and_sidebar_identity() {
        let value: Value = serde_json::from_str(
            r#"{"schema_version":1,"active_tab_position":2,"workspace":{"root":"/tmp/project","source":"explicit"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"managed_panes":{"editor_pane_id":"terminal:7","sidebar_pane_id":"terminal:8"},"sidebar_yazi":{"yazi_id":"yazi-123","cwd":"/tmp/project"},"extensions":{"ai_pane_activity":[{"tab_position":2,"provider":"codex","pane_id":"terminal:9","activity":"thinking","state":"thinking"}]}}"#,
        )
        .unwrap();
        let rendered = render_session_state_inspection_lines(&value).join("\n");

        assert!(rendered.contains("workspace: /tmp/project (explicit)"));
        assert!(rendered.contains("layout: active_swap_layout_name=single_open"));
        assert!(rendered.contains("managed_panes: editor=terminal:7, sidebar=terminal:8"));
        assert!(rendered.contains("sidebar_yazi: id=yazi-123, cwd=/tmp/project"));
    }

    // Defends: status-bus consumers reject unsupported producer schema versions instead of parsing stale payloads optimistically.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_bus_decode_rejects_unsupported_schema_version() {
        let err = decode_status_bus_snapshot(
            r#"{"schema_version":99,"active_tab_position":0,"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"unknown","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null}}"#,
        )
        .unwrap_err();

        assert!(
            err.message()
                .contains("Unsupported pane-orchestrator status-bus schema_version")
        );
        assert!(
            err.remediation()
                .contains("supports status-bus schema_version 1")
        );
    }

    // Defends: the bar workspace widget formats only status-bus facts from a fixture payload.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_bus_workspace_widget_formats_fixture_payload() {
        let value = decode_status_bus_snapshot(
            r#"{"schema_version":1,"active_tab_position":0,"workspace":{"root":"/tmp/yazelix-demo","source":"explicit"},"managed_panes":{"editor_pane_id":"terminal:1","sidebar_pane_id":"terminal:2"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[]}}"#,
        )
        .unwrap();

        assert_eq!(render_status_bus_workspace_widget(&value), "yazelix-demo");
    }

    // Regression: zjstatus command widgets return plain text while the template owns style markup, so command stdout cannot print literal `#[fg=...]` tags.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn zjstatus_status_bus_workspace_widget_renders_plain_segment_and_hides_missing_facts() {
        let value = decode_status_bus_snapshot(
            r#"{"schema_version":1,"active_tab_position":0,"workspace":{"root":"/tmp/yazelix-demo","source":"explicit"},"managed_panes":{"editor_pane_id":"terminal:1","sidebar_pane_id":"terminal:2"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[{"tab_position":0,"provider":"claude","pane_id":"terminal:2","activity":"thinking","state":"thinking"}]}}"#,
        )
        .unwrap();
        let empty = decode_status_bus_snapshot(
            r#"{"schema_version":1,"active_tab_position":0,"workspace":null,"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"other","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[]}}"#,
        )
        .unwrap();

        assert_eq!(render_zjstatus_workspace_widget(&value), " [yazelix-demo]");
        assert!(!render_zjstatus_workspace_widget(&value).contains("#["));
        assert_eq!(render_zjstatus_workspace_widget(&empty), "");
    }

    // Regression: zjstatus reads dynamic widgets from a local cache instead of invoking Zellij pipes from every bar command.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_round_trip_renders_cached_workspace_fact() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join("window_a").join("status_bar_cache.json");

        run_zellij_status_cache_write(&[
            "--path".to_string(),
            cache_path.display().to_string(),
            "--payload".to_string(),
            STATUS_CACHE_TEST_PAYLOAD.to_string(),
        ])
        .unwrap();
        let cache = read_status_bar_cache_value(&cache_path).unwrap();

        assert_eq!(
            render_status_cache_widget(&cache, "workspace").unwrap(),
            " [yazelix-demo]"
        );
        assert!(
            !render_status_cache_widget(&cache, "workspace")
                .unwrap()
                .contains("#[")
        );
    }

    // Defends: heartbeat updates merge into the window-local cache without replacing status-bus or usage facts.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_heartbeat_merge_preserves_cached_session_facts() {
        let mut cache = build_status_bar_cache_at(
            status_cache_test_status_bus(),
            json!({
                "claude_daily": {
                    "summary": "42k $0.42",
                    "tokens": "42k",
                    "cost": "$0.42"
                }
            }),
            Some(1_000),
            1_000,
        );
        let status_bus_before = cache.get("status_bus").cloned();
        let agent_usage_before = cache.get("agent_usage").cloned();

        merge_orchestrator_heartbeat_into_cache(
            &mut cache,
            json!({
                "schema_version": 1,
                "heartbeat_at_unix_seconds": 2_000,
                "last_pipe": {
                    "name": "toggle_transient_pane",
                    "at_unix_seconds": 1_990
                },
                "status_refreshes": {
                    "codex_usage": {
                        "started_at_unix_seconds": 1_980
                    }
                }
            }),
        );
        merge_orchestrator_heartbeat_into_cache(
            &mut cache,
            json!({
                "schema_version": 1,
                "status_refreshes": {
                    "codex_usage": {
                        "finished_at_unix_seconds": 2_010
                    }
                }
            }),
        );

        assert_eq!(cache.get("status_bus").cloned(), status_bus_before);
        assert_eq!(cache.get("agent_usage").cloned(), agent_usage_before);
        assert_eq!(
            cache
                .pointer("/orchestrator_heartbeat/last_pipe/name")
                .and_then(Value::as_str),
            Some("toggle_transient_pane")
        );
        assert_eq!(
            cache
                .pointer(
                    "/orchestrator_heartbeat/status_refreshes/codex_usage/started_at_unix_seconds"
                )
                .and_then(Value::as_u64),
            Some(1_980)
        );
        assert_eq!(
            cache
                .pointer(
                    "/orchestrator_heartbeat/status_refreshes/codex_usage/finished_at_unix_seconds"
                )
                .and_then(Value::as_u64),
            Some(2_010)
        );
    }

    // Regression: status-bus cache rewrites must not erase heartbeat facts used to debug orchestrator stalls.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_write_preserves_existing_heartbeat() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join("window_a").join("status_bar_cache.json");
        let mut cache =
            build_status_bar_cache_at(status_cache_test_status_bus(), json!({}), None, 1_000);
        merge_orchestrator_heartbeat_into_cache(
            &mut cache,
            json!({
                "schema_version": 1,
                "heartbeat_at_unix_seconds": 2_000,
                "last_timer_at_unix_seconds": 1_990
            }),
        );
        write_status_bar_cache_value(&cache_path, &cache).unwrap();

        run_zellij_status_cache_write(&[
            "--path".to_string(),
            cache_path.display().to_string(),
            "--payload".to_string(),
            STATUS_CACHE_TEST_PAYLOAD.to_string(),
        ])
        .unwrap();

        let updated_cache = read_status_bar_cache_value(&cache_path).unwrap();
        assert_eq!(
            updated_cache
                .pointer("/orchestrator_heartbeat/last_timer_at_unix_seconds")
                .and_then(Value::as_u64),
            Some(1_990)
        );
    }

    // Regression: new Yazelix windows should first-paint agent usage from recent sibling cache facts instead of waiting for provider refresh.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_write_seeds_agent_usage_from_recent_sibling_session_cache() {
        let temp = tempfile::tempdir().unwrap();
        let sessions_dir = temp.path().join("sessions");
        let old_cache_path = sessions_dir.join("window_a").join("status_bar_cache.json");
        let new_cache_path = sessions_dir.join("window_b").join("status_bar_cache.json");
        let now = unix_time_seconds();
        let old_cache = build_status_bar_cache_at(
            status_cache_test_status_bus(),
            json!({
                "claude_daily": {
                    "summary": "42k $0.42",
                    "tokens": "42k",
                    "cost": "$0.42"
                }
            }),
            Some(now),
            now,
        );
        write_status_bar_cache_value(&old_cache_path, &old_cache).unwrap();

        run_zellij_status_cache_write(&[
            "--path".to_string(),
            new_cache_path.display().to_string(),
            "--payload".to_string(),
            STATUS_CACHE_TEST_PAYLOAD.to_string(),
        ])
        .unwrap();
        let new_cache = read_status_bar_cache_value(&new_cache_path).unwrap();

        assert_eq!(
            render_status_cache_widget(&new_cache, "claude_usage").unwrap(),
            " [claude d 42k $0.42]"
        );
        assert_eq!(
            new_cache
                .get("agent_usage_updated_at_unix_seconds")
                .and_then(Value::as_u64),
            Some(now)
        );
    }

    // Defends: first-paint seeding refuses old sibling usage facts so a new day or stale session cannot render misleading usage indefinitely.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_write_ignores_stale_sibling_agent_usage_seed() {
        let temp = tempfile::tempdir().unwrap();
        let sessions_dir = temp.path().join("sessions");
        let old_cache_path = sessions_dir.join("window_a").join("status_bar_cache.json");
        let new_cache_path = sessions_dir.join("window_b").join("status_bar_cache.json");
        let now = unix_time_seconds();
        let stale_updated_at = now.saturating_sub(AGENT_USAGE_SEED_MAX_AGE_SECONDS + 1);
        let old_cache = build_status_bar_cache_at(
            status_cache_test_status_bus(),
            json!({
                "claude_daily": {
                    "summary": "42k $0.42",
                    "tokens": "42k",
                    "cost": "$0.42"
                }
            }),
            Some(stale_updated_at),
            stale_updated_at,
        );
        write_status_bar_cache_value(&old_cache_path, &old_cache).unwrap();

        run_zellij_status_cache_write(&[
            "--path".to_string(),
            new_cache_path.display().to_string(),
            "--payload".to_string(),
            STATUS_CACHE_TEST_PAYLOAD.to_string(),
        ])
        .unwrap();
        let new_cache = read_status_bar_cache_value(&new_cache_path).unwrap();

        assert_eq!(
            render_status_cache_widget(&new_cache, "claude_usage").unwrap(),
            ""
        );
        assert!(
            new_cache
                .get("agent_usage_updated_at_unix_seconds")
                .is_none()
        );
    }

    // Defends: cache readers stay quiet when a launch-scoped cache has not been written yet.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn missing_status_cache_file_renders_no_widget_segment() {
        let temp = tempfile::tempdir().unwrap();

        assert!(read_status_bar_cache_value(&temp.path().join("missing.json")).is_none());
    }

    // Regression: zjstatus command execution can strip direct Yazelix cache env even though its Zellij parent still carries the launch env.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_path_can_be_recovered_from_process_environ_bytes() {
        let explicit = status_bar_cache_path_from_environ_bytes(
            b"PATH=/bin\0YAZELIX_STATUS_BAR_CACHE_PATH=/tmp/window/status_bar_cache.json\0YAZELIX_SESSION_CONFIG_PATH=/tmp/other/config_snapshot.json\0",
        );
        assert_eq!(
            explicit,
            Some(PathBuf::from("/tmp/window/status_bar_cache.json"))
        );

        let derived = status_bar_cache_path_from_environ_bytes(
            b"PATH=/bin\0YAZELIX_SESSION_CONFIG_PATH=/tmp/session/config_snapshot.json\0",
        );
        assert_eq!(
            derived,
            Some(PathBuf::from("/tmp/session/status_bar_cache.json"))
        );
    }

    // Regression: zjstatus command execution can preserve only the cache path, so usage refresh still needs the sibling config snapshot.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn session_config_path_can_be_recovered_from_cache_path() {
        assert_eq!(
            session_config_path_from_values(
                None,
                Some(PathBuf::from("/tmp/session/status_bar_cache.json")),
            ),
            Some(PathBuf::from("/tmp/session/config_snapshot.json"))
        );
        assert_eq!(
            session_config_path_from_environ_bytes(
                b"PATH=/bin\0YAZELIX_SESSION_CONFIG_PATH=/tmp/session/config_snapshot.json\0",
            ),
            Some(PathBuf::from("/tmp/session/config_snapshot.json"))
        );
    }

    // Regression: refresh commands receive an explicit cache path from the plugin, so they must recover the sibling config snapshot without relying on ambient env.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn usage_widget_settings_can_be_recovered_from_cache_path() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join("window").join("status_bar_cache.json");
        let config_path = temp.path().join("window").join("config_snapshot.json");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(
            &config_path,
            json!({
                "normalized_config": {
                    "zellij_widget_tray": ["opencode_go_usage"],
                    "zellij_opencode_go_usage_display": "quota",
                    "zellij_opencode_go_usage_periods": ["5h", "month"]
                }
            })
            .to_string(),
        )
        .unwrap();

        assert!(usage_widget_enabled_from_status_cache_path(
            &cache_path,
            "opencode_go_usage"
        ));
        let settings = agent_usage_widget_settings_from_status_cache_path(&cache_path);
        assert_eq!(settings.opencode_go_display, WindowedUsageDisplay::Quota);
        assert_eq!(
            settings.opencode_go_periods,
            vec![WindowedUsagePeriod::FiveHour, WindowedUsagePeriod::Monthly]
        );
    }

    // Regression: agent usage refresh must honor grouped Claude period settings from the launch snapshot while dedicated windowed widgets use separate refresh paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn agent_usage_refresh_config_reads_grouped_periods_from_snapshot_path() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config_snapshot.json");
        fs::write(
            &config_path,
            json!({
                "normalized_config": {
                    "zellij_widget_tray": ["codex_usage", "claude_usage", "opencode_go_usage"],
                    "zellij_claude_usage_periods": ["day", "month"],
                    "zellij_agent_usage_display": "both"
                }
            })
            .to_string(),
        )
        .unwrap();
        let config = normalized_session_config_from_path(&config_path).unwrap();
        let refresh_config = agent_usage_refresh_config_from_normalized_config(&config);

        assert_eq!(
            configured_agent_usage_targets(&refresh_config),
            vec![CLAUDE_DAILY_USAGE_TARGET, CLAUDE_MONTHLY_USAGE_TARGET]
        );
    }

    // Defends: cached agent usage widgets consume precomputed summaries instead of running provider binaries from zjstatus.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_agent_usage_renders_precomputed_summary() {
        let cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "agent_usage": {
                "claude_daily": {"summary": "123k $1.23"}
            }
        });

        assert_eq!(
            render_status_cache_widget(&cache, "claude_usage").unwrap(),
            " [claude d 123k $1.23]"
        );
        assert!(
            !render_status_cache_widget(&cache, "claude_usage")
                .unwrap()
                .contains("#[")
        );
        assert_eq!(
            render_status_cache_widget(&cache, "codex_usage").unwrap(),
            ""
        );
    }

    // Defends: agent usage display modes can keep status-bar widgets compact without rerunning provider commands.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_agent_usage_display_modes_filter_cached_facts() {
        let cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "agent_usage": {
                "claude_daily": {
                    "summary": "94k $0.113 2h17m",
                    "tokens": "94k",
                    "cost": "$0.113",
                    "remaining": "2h17m"
                }
            }
        });

        assert_eq!(
            render_status_cache_widget_with_agent_usage_display(
                &cache,
                "claude_usage",
                AgentUsageDisplay::Both
            )
            .unwrap(),
            " [claude d 94k $0.113 2h17m]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_display(
                &cache,
                "claude_usage",
                AgentUsageDisplay::Tokens
            )
            .unwrap(),
            " [claude d 94k]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_display(
                &cache,
                "claude_usage",
                AgentUsageDisplay::Money
            )
            .unwrap(),
            " [claude d $0.113]"
        );
    }

    // Defends: Codex usage renders only the compact 5h/week token/quota contract selected by codex_usage_display.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_codex_usage_renders_5h_week_display_modes() {
        let cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "codex_usage": {
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64
            }
        });

        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "codex_usage",
                &AgentUsageWidgetSettings {
                    codex_display: WindowedUsageDisplay::Both,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [codex 5h|138M|49% wk|1.34B|80%]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "codex_usage",
                &AgentUsageWidgetSettings {
                    codex_display: WindowedUsageDisplay::Token,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [codex 5h|138M wk|1.34B]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "codex_usage",
                &AgentUsageWidgetSettings {
                    codex_display: WindowedUsageDisplay::Quota,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [codex 5h|49% wk|80%]"
        );
    }

    // Defends: OpenCode Go usage renders configurable 5h/week/month token/quota windows with the short `go` label.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_opencode_go_usage_renders_configured_window_display_modes() {
        let cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "opencode_go_usage": {
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "monthly_tokens": 2220000000u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64,
                "monthly_remaining_percent": 70u64
            }
        });

        let monthly_periods = vec![
            WindowedUsagePeriod::FiveHour,
            WindowedUsagePeriod::Weekly,
            WindowedUsagePeriod::Monthly,
        ];

        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "opencode_go_usage",
                &AgentUsageWidgetSettings {
                    opencode_go_display: WindowedUsageDisplay::Both,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [go 5h|138M|49% wk|1.34B|80%]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "opencode_go_usage",
                &AgentUsageWidgetSettings {
                    opencode_go_periods: monthly_periods.clone(),
                    opencode_go_display: WindowedUsageDisplay::Both,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [go 5h|138M|49% wk|1.34B|80% mo|2.22B|70%]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "opencode_go_usage",
                &AgentUsageWidgetSettings {
                    opencode_go_periods: monthly_periods.clone(),
                    opencode_go_display: WindowedUsageDisplay::Token,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [go 5h|138M wk|1.34B mo|2.22B]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "opencode_go_usage",
                &AgentUsageWidgetSettings {
                    opencode_go_periods: monthly_periods,
                    opencode_go_display: WindowedUsageDisplay::Quota,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [go 5h|49% wk|80% mo|70%]"
        );
    }

    // Defends: tokenusage JSON shape for Codex active-block, weekly, and official quota facts maps to the compact widget contract.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn codex_usage_json_parsers_read_tokenusage_windows_and_official_quota() {
        let active = json!({
            "blocks": [
                {"isActive": false, "totals": {"total_tokens": 10u64}},
                {"isActive": true, "totals": {"total_tokens": 138424632u64}}
            ]
        });
        let weekly = json!({
            "weekly": [
                {"totals": {"total_tokens": 1335519960u64}},
                {"totals": {"total_tokens": 1u64}}
            ]
        });
        let official = json!({
            "official_codex": {
                "primary_used_percent": 51.0,
                "secondary_used_percent": 20.0
            }
        });

        let quota = codex_quota_from_official_json(&official);

        assert_eq!(
            codex_active_block_tokens_from_json(&active),
            Some(138424632)
        );
        assert_eq!(codex_weekly_tokens_from_json(&weekly), Some(1335519960));
        assert_eq!(quota.five_hour_remaining_percent, Some(49));
        assert_eq!(quota.weekly_remaining_percent, Some(80));
    }

    // Regression: the dedicated Codex refresh writes a shared cache that new windows hydrate before rendering.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn status_cache_codex_usage_refresh_writes_shared_combined_cache() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let provider = bin_dir.join("tu");
        fs::write(
            &provider,
            r#"#!/usr/bin/env sh
if [ "$1" = "blocks" ] && [ "$2" = "--active" ]; then
  case " $* " in
    *" --official-limits "*)
      printf '%s\n' '{"official_codex":{"primary_used_percent":51.0,"secondary_used_percent":20.0}}'
      ;;
    *)
      printf '%s\n' '{"blocks":[{"isActive":true,"totals":{"total_tokens":138424632}}]}'
      ;;
  esac
  exit 0
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":1335519960}}]}'
  exit 0
fi
exit 64
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&provider).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&provider, permissions).unwrap();
        let status_cache_path = temp
            .path()
            .join("state")
            .join("sessions")
            .join("window_a")
            .join("status_bar_cache.json");
        let shared_path =
            codex_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();

        let refreshed = refresh_codex_usage_shared_cache(
            &shared_path,
            Some(bin_dir.as_os_str()),
            1_000,
            600,
            1_800,
            Duration::from_secs(1),
        )
        .unwrap();
        let mut cache =
            build_status_bar_cache_at(status_cache_test_status_bus(), json!({}), None, 1_000);
        hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);

        assert!(refreshed);
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "codex_usage",
                &AgentUsageWidgetSettings {
                    codex_display: WindowedUsageDisplay::Both,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [codex 5h|138M|49% wk|1.34B|80%]"
        );
    }

    fn write_opencode_go_usage_test_db(path: &Path, now: u64) {
        let connection = Connection::open(path).unwrap();
        connection
            .execute_batch(
                r#"
                CREATE TABLE message (
                    id text PRIMARY KEY,
                    session_id text NOT NULL,
                    time_created integer NOT NULL,
                    time_updated integer NOT NULL,
                    data text NOT NULL
                );
                "#,
            )
            .unwrap();
        let rows = [
            (
                "within_five_hour",
                now.saturating_sub(60),
                r#"{"role":"assistant","providerID":"opencode-go","cost":3.0,"tokens":{"input":1000000,"output":2000000,"reasoning":3000000,"cache":{"read":4000000,"write":5000000}}}"#,
            ),
            (
                "within_week",
                now.saturating_sub(6 * 60 * 60),
                r#"{"role":"assistant","providerID":"opencode-go","cost":12.0,"tokens":{"total":85000000}}"#,
            ),
            (
                "within_month",
                now.saturating_sub(8 * 24 * 60 * 60),
                r#"{"role":"assistant","providerID":"opencode-go","cost":15.0,"tokens":{"total":200000000}}"#,
            ),
            (
                "wrong_provider",
                now.saturating_sub(60),
                r#"{"role":"assistant","providerID":"opencode","cost":99.0,"tokens":{"total":900000000}}"#,
            ),
            (
                "wrong_role",
                now.saturating_sub(60),
                r#"{"role":"user","providerID":"opencode-go","cost":99.0,"tokens":{"total":900000000}}"#,
            ),
        ];
        for (id, created_at, data) in rows {
            let created_at = unix_millis_from_seconds_saturating(created_at);
            connection
                .execute(
                    "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES (?1, 'session', ?2, ?2, ?3)",
                    rusqlite::params![id, created_at, data],
                )
                .unwrap();
        }
    }

    // Defends: OpenCode Go usage reads only assistant rows from OpenCode's SQLite store and converts official dollar limits to remaining percentages.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn opencode_go_sqlite_reader_filters_provider_and_computes_quota_windows() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("opencode.db");
        let now = 2_000_000;
        write_opencode_go_usage_test_db(&db_path, now);

        let facts = collect_opencode_go_usage_facts_from_dbs(&[db_path], now);

        assert_eq!(facts.five_hour_tokens, Some(15_000_000));
        assert_eq!(facts.weekly_tokens, Some(100_000_000));
        assert_eq!(facts.monthly_tokens, Some(300_000_000));
        assert_eq!(facts.five_hour_remaining_percent, Some(75));
        assert_eq!(facts.weekly_remaining_percent, Some(50));
        assert_eq!(facts.monthly_remaining_percent, Some(50));
    }

    // Regression: the dedicated OpenCode Go refresh writes a shared cache that new windows hydrate before rendering.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_opencode_go_usage_refresh_writes_shared_combined_cache() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("opencode.db");
        let now = 2_000_000;
        write_opencode_go_usage_test_db(&db_path, now);
        let status_cache_path = temp
            .path()
            .join("state")
            .join("sessions")
            .join("window_a")
            .join("status_bar_cache.json");
        let shared_path =
            opencode_go_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();

        let refreshed =
            refresh_opencode_go_usage_shared_cache_from_dbs(&shared_path, &[db_path], now, 1_800)
                .unwrap();
        let mut cache =
            build_status_bar_cache_at(status_cache_test_status_bus(), json!({}), None, now);
        hydrate_status_cache_opencode_go_usage(&mut cache, &status_cache_path);

        assert!(refreshed);
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "opencode_go_usage",
                &AgentUsageWidgetSettings {
                    opencode_go_periods: vec![
                        WindowedUsagePeriod::FiveHour,
                        WindowedUsagePeriod::Weekly,
                        WindowedUsagePeriod::Monthly,
                    ],
                    opencode_go_display: WindowedUsageDisplay::Both,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [go 5h|15M|75% wk|100M|50% mo|300M|50%]"
        );
    }

    // Defends: shared Codex usage caches have explicit freshness and error backoff so multiple Yazelix windows do not stampede provider calls.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn codex_usage_shared_cache_respects_freshness_and_backoff() {
        let temp = tempfile::tempdir().unwrap();
        let shared_path = temp.path().join("codex_usage_cache.json");

        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();
        assert!(codex_usage_shared_cache_is_fresh(&shared_path, 1_100, 600));
        assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_700, 600));

        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 1_700u64,
                    "error": "quota unavailable",
                    "backoff_until_unix_seconds": 2_000u64
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();
        assert!(codex_usage_shared_cache_is_backing_off(&shared_path, 1_999));
        assert!(!codex_usage_shared_cache_is_backing_off(
            &shared_path,
            2_000
        ));
    }

    // Defends: Claude usage mirrors the grouped period contract used by the other provider widgets.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_grouped_claude_usage_renders_configured_periods_compactly() {
        let cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "agent_usage": {
                "claude_daily": {"tokens": "124M", "cost": "$6.69"},
                "claude_monthly": {"tokens": "1.58B", "cost": "$98"}
            }
        });
        let settings = AgentUsageWidgetSettings {
            display: AgentUsageDisplay::Both,
            claude_periods: vec![AgentUsagePeriod::Daily, AgentUsagePeriod::Monthly],
            ..AgentUsageWidgetSettings::default()
        };

        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(&cache, "claude_usage", &settings)
                .unwrap(),
            " [claude d 124M $6.69 | mon 1.58B $98]"
        );
    }

    // Regression: the agent-usage producer updates cached summaries from opt-in providers without making zjstatus run provider binaries.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn status_cache_agent_usage_refresh_writes_precomputed_summary() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let provider = bin_dir.join("tu");
        fs::write(
            &provider,
            r#"#!/usr/bin/env sh
if [ "$1" = "today" ]; then
  printf '%s\n' 'log prefix before json'
  printf '%s\n' '{"date":"2026-04-30","overview":{"totals":{"total_tokens":123456,"cost_usd":1.234}}}'
  exit 0
fi
exit 64
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&provider).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&provider, permissions).unwrap();
        let mut cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "agent_usage": {}
        });

        let refresh_config = AgentUsageRefreshConfig {
            widgets: Some(["claude_usage".to_string()].into_iter().collect()),
            ..AgentUsageRefreshConfig::default()
        };
        assert_eq!(
            configured_agent_usage_targets(&refresh_config),
            vec![CLAUDE_DAILY_USAGE_TARGET]
        );
        let refreshed = refresh_status_bar_cache_agent_usage_value(
            &mut cache,
            Some(bin_dir.as_os_str()),
            &refresh_config,
            1_000,
            120,
            Duration::from_secs(3),
        );

        assert!(refreshed);
        assert_eq!(
            cache
                .get("agent_usage")
                .and_then(|usage| usage.get("claude_daily"))
                .and_then(|entry| entry.get("summary"))
                .and_then(Value::as_str),
            Some("123k $1.23")
        );
        assert_eq!(
            render_status_cache_widget(&cache, "claude_usage").unwrap(),
            " [claude d 123k $1.23]"
        );
    }

    // Regression: Claude usage refresh writes the same grouped-period cache entries used by status rendering.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn status_cache_claude_usage_refresh_writes_grouped_period_summaries() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let provider = bin_dir.join("tu");
        fs::write(
            &provider,
            r#"#!/usr/bin/env sh
case "$1" in
  today)
    printf '%s\n' '{"overview":{"totals":{"total_tokens":123456,"cost_usd":1.234}}}'
    ;;
  monthly)
    printf '%s\n' '{"monthly":[{"totals":{"total_tokens":1576759000,"cost_usd":98.0}}]}'
    ;;
  *)
    exit 64
    ;;
esac
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&provider).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&provider, permissions).unwrap();
        let mut cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "agent_usage": {}
        });

        let refresh_config = AgentUsageRefreshConfig {
            widgets: Some(["claude_usage".to_string()].into_iter().collect()),
            claude_periods: vec![AgentUsagePeriod::Daily, AgentUsagePeriod::Monthly],
            ..AgentUsageRefreshConfig::default()
        };
        assert_eq!(
            configured_agent_usage_targets(&refresh_config),
            vec![CLAUDE_DAILY_USAGE_TARGET, CLAUDE_MONTHLY_USAGE_TARGET,]
        );
        let refreshed = refresh_status_bar_cache_agent_usage_value(
            &mut cache,
            Some(bin_dir.as_os_str()),
            &refresh_config,
            1_000,
            120,
            Duration::from_secs(3),
        );
        let settings = AgentUsageWidgetSettings {
            display: AgentUsageDisplay::Both,
            claude_periods: refresh_config.claude_periods.clone(),
            ..AgentUsageWidgetSettings::default()
        };

        assert!(refreshed);
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(&cache, "claude_usage", &settings)
                .unwrap(),
            " [claude d 123k $1.23 | mon 1.58B $98]"
        );
    }

    // Regression: the grouped agent-usage refresh must not invoke Codex providers now that codex_usage has a dedicated 5h/week cache.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn status_cache_agent_usage_refresh_does_not_run_codex_combined_widget() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let sentinel = temp.path().join("tu_was_invoked");
        let provider = bin_dir.join("tu");
        fs::write(
            &provider,
            format!(
                "#!/usr/bin/env sh\nprintf touched > '{}'\nprintf '%s\\n' '{{\"overview\":{{\"totals\":{{\"total_tokens\":123456}}}}}}'\n",
                sentinel.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&provider).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&provider, permissions).unwrap();
        let mut cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "agent_usage": {}
        });

        let refresh_config = AgentUsageRefreshConfig {
            widgets: Some(["codex_usage".to_string()].into_iter().collect()),
            ..AgentUsageRefreshConfig::default()
        };
        assert!(configured_agent_usage_targets(&refresh_config).is_empty());
        refresh_status_bar_cache_agent_usage_value(
            &mut cache,
            Some(bin_dir.as_os_str()),
            &refresh_config,
            1_000,
            120,
            Duration::from_secs(1),
        );

        assert_eq!(
            cache
                .get("agent_usage")
                .and_then(Value::as_object)
                .map(|usage| usage.len()),
            Some(0)
        );
        assert!(!sentinel.exists());
    }

    // Regression: hung agent-usage providers are killed quickly so the cache producer cannot recreate the CPU-spike failure mode.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn status_cache_agent_usage_refresh_times_out_hung_provider() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let provider = bin_dir.join("tu");
        fs::write(&provider, "#!/usr/bin/env sh\nsleep 5\n").unwrap();
        let mut permissions = fs::metadata(&provider).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&provider, permissions).unwrap();
        let mut cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": 10,
            "status_bus": {
                "schema_version": 1,
                "active_tab_position": 0,
                "workspace": null,
                "managed_panes": {"editor_pane_id": null, "sidebar_pane_id": null},
                "focus_context": "other",
                "layout": {"active_swap_layout_name": null, "sidebar_collapsed": null},
                "sidebar_yazi": null,
                "transient_panes": {"popup": null, "menu": null},
                "extensions": {"ai_pane_activity": []}
            },
            "agent_usage": {}
        });
        let started = Instant::now();

        let refresh_config = AgentUsageRefreshConfig {
            widgets: Some(["claude_usage".to_string()].into_iter().collect()),
            ..AgentUsageRefreshConfig::default()
        };
        let refreshed = refresh_status_bar_cache_agent_usage_value(
            &mut cache,
            Some(bin_dir.as_os_str()),
            &refresh_config,
            1_000,
            120,
            Duration::from_millis(50),
        );

        assert!(refreshed);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(
            cache
                .get("agent_usage")
                .and_then(Value::as_object)
                .unwrap()
                .len(),
            0
        );
    }

    // Regression: tokenusage monthly JSON includes all-history top-level totals; Yazelix must render the current row instead.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn tokenusage_json_uses_selected_period_row_instead_of_all_history_totals() {
        let monthly = agent_usage_summary_from_json(
            CLAUDE_MONTHLY_USAGE_TARGET,
            AgentUsageJsonDialect::Tokenusage,
            r#"{"monthly":[{"date":"2026-04","totals":{"total_tokens":1576759000,"cost_usd":98.0}}],"totals":{"total_tokens":999000,"cost_usd":99.9}}"#,
        );
        assert_eq!(monthly, "1.58B $98");
    }

    // Defends: configured Claude usage periods expand to tokenusage-backed cache targets used by the status bar.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn claude_usage_period_config_expands_to_tokenusage_targets() {
        let provider = parse_agent_usage_provider("claude").unwrap();
        let refresh_config = AgentUsageRefreshConfig {
            widgets: Some(["claude_usage".to_string()].into_iter().collect()),
            claude_periods: vec![AgentUsagePeriod::Daily, AgentUsagePeriod::Monthly],
        };

        assert_eq!(agent_usage_binary(provider), "tu");
        assert_eq!(
            configured_agent_usage_targets(&refresh_config),
            vec![CLAUDE_DAILY_USAGE_TARGET, CLAUDE_MONTHLY_USAGE_TARGET]
        );
    }
}
