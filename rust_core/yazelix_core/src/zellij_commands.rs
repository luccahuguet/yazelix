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
use serde_json::{Value, json};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";
const STATUS_BUS_SCHEMA_VERSION: i64 = 1;
const EDITOR_PANE_NAME: &str = "editor";
pub const INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS: &[&str] = &[
    "pipe",
    "get-workspace-root",
    "inspect-session",
    "status-bus",
    "status-bus-workspace",
    "status-bus-ai-activity",
    "status-bus-token-budget",
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
struct ZellijStatusBusWidgetArgs {
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ZellijAgentUsageArgs {
    provider: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentUsageProvider {
    Claude,
    Codex,
    Amp,
    Opencode,
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

fn parse_zellij_status_bus_widget_args(
    args: &[String],
    subcommand: &str,
) -> Result<ZellijStatusBusWidgetArgs, CoreError> {
    let mut parsed = ZellijStatusBusWidgetArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for zellij {subcommand}: {other}"
                )));
            }
            _ => {
                return Err(CoreError::usage(format!(
                    "zellij {subcommand} accepts no positional arguments"
                )));
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

fn print_zellij_status_bus_ai_activity_help() {
    println!("Render the AI activity status-bus fact for zjstatus");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-bus-ai-activity");
}

fn print_zellij_status_bus_token_budget_help() {
    println!("Render the AI token-budget status-bus fact for zjstatus");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij status-bus-token-budget");
}

fn print_zellij_agent_usage_help() {
    println!("Render an opt-in ccusage provider summary for zjstatus");
    println!();
    println!("Usage:");
    println!("  yzx_control zellij agent-usage <claude|codex|amp|opencode>");
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
        println!("outside");
        return Ok(0);
    }

    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    let value = decode_status_bus_snapshot(&response)?;
    println!("{}", render_status_bus_workspace_widget(&value));
    Ok(0)
}

pub fn run_zellij_status_bus_ai_activity(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_bus_widget_args(args, "status-bus-ai-activity")?;
    if parsed.help {
        print_zellij_status_bus_ai_activity_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        println!("unknown");
        return Ok(0);
    }

    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    let value = decode_status_bus_snapshot(&response)?;
    println!("{}", render_status_bus_ai_activity_widget(&value));
    Ok(0)
}

pub fn run_zellij_status_bus_token_budget(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_status_bus_widget_args(args, "status-bus-token-budget")?;
    if parsed.help {
        print_zellij_status_bus_token_budget_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        println!("unknown");
        return Ok(0);
    }

    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    let value = decode_status_bus_snapshot(&response)?;
    println!("{}", render_status_bus_token_budget_widget(&value));
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
            "zellij agent-usage requires one of: claude, codex, amp, opencode".to_string(),
        ));
    };

    let Some(binary_path) = find_command_in_path(agent_usage_binary(provider)) else {
        println!();
        return Ok(0);
    };

    let output = Command::new(binary_path)
        .args(["blocks", "--active", "--json"])
        .output()
        .map_err(|source| {
            CoreError::io(
                "agent_usage_failed",
                "Failed to run the configured ccusage provider.",
                "Ensure the opt-in agent usage package is healthy, then retry.",
                agent_usage_binary(provider),
                source,
            )
        })?;

    if !output.status.success() {
        println!();
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let summary = agent_usage_summary_from_json(&stdout);
    if summary.is_empty() {
        println!();
    } else {
        println!("{}", render_agent_usage_widget(provider, &summary));
    }
    Ok(0)
}

fn render_status_bus_workspace_widget(value: &Value) -> String {
    let root = nested_str(value, &["workspace", "root"]).unwrap_or("");
    let workspace = Path::new(root)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("none");
    let focus = value
        .get("focus_context")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|focus| !focus.is_empty())
        .unwrap_or("unknown");
    let sidebar = nested_bool(value, &["layout", "sidebar_collapsed"]);
    let sidebar_marker = match sidebar {
        Some(true) => "side:closed",
        Some(false) => "side:open",
        _ => "side:?",
    };
    format!("{workspace}/{focus}/{sidebar_marker}")
}

fn render_status_bus_ai_activity_widget(value: &Value) -> String {
    let Some(activity_facts) = nested_array(value, &["extensions", "ai_pane_activity"]) else {
        return "unknown".to_string();
    };
    let Some(selected) = activity_facts
        .iter()
        .max_by_key(|fact| ai_activity_state_rank(ai_activity_state(fact)))
    else {
        return "unknown".to_string();
    };
    let state = ai_activity_state(selected);
    let provider = selected
        .get("provider")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|provider| !provider.is_empty())
        .unwrap_or("ai");
    if state == "unknown" && provider == "ai" {
        "unknown".to_string()
    } else {
        format!("{provider}:{state}")
    }
}

fn render_status_bus_token_budget_widget(value: &Value) -> String {
    let Some(token_budget_facts) = nested_array(value, &["extensions", "ai_token_budget"]) else {
        return "unknown".to_string();
    };
    let Some(selected) = token_budget_facts.iter().find(|fact| {
        fact.get("remaining_tokens")
            .and_then(Value::as_u64)
            .is_some()
            || fact.get("total_tokens").and_then(Value::as_u64).is_some()
    }) else {
        return "unknown".to_string();
    };
    let provider = selected
        .get("provider")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|provider| !provider.is_empty())
        .unwrap_or("ai");
    let remaining = selected
        .get("remaining_tokens")
        .and_then(Value::as_u64)
        .map(format_token_count)
        .unwrap_or_else(|| "?".to_string());
    let total = selected
        .get("total_tokens")
        .and_then(Value::as_u64)
        .map(format_token_count)
        .unwrap_or_else(|| "?".to_string());
    format!("{provider}:{remaining}/{total}")
}

fn ai_activity_state(fact: &Value) -> &str {
    fact.get("state")
        .and_then(Value::as_str)
        .or_else(|| fact.get("activity").and_then(Value::as_str))
        .map(str::trim)
        .filter(|state| !state.is_empty())
        .unwrap_or("unknown")
}

fn ai_activity_state_rank(state: &str) -> u8 {
    match state {
        "thinking" => 5,
        "active" | "streaming" => 4,
        "stale" => 3,
        "inactive" | "idle" => 2,
        _ => 1,
    }
}

fn format_token_count(tokens: u64) -> String {
    if tokens >= 1_000 {
        format!("{}k", tokens / 1_000)
    } else {
        tokens.to_string()
    }
}

fn parse_agent_usage_provider(raw: &str) -> Option<AgentUsageProvider> {
    match raw.trim() {
        "claude" | "ccusage" => Some(AgentUsageProvider::Claude),
        "codex" | "ccusage-codex" => Some(AgentUsageProvider::Codex),
        "amp" | "ccusage-amp" => Some(AgentUsageProvider::Amp),
        "opencode" | "ccusage-opencode" => Some(AgentUsageProvider::Opencode),
        _ => None,
    }
}

fn agent_usage_binary(provider: AgentUsageProvider) -> &'static str {
    match provider {
        AgentUsageProvider::Claude => "ccusage",
        AgentUsageProvider::Codex => "ccusage-codex",
        AgentUsageProvider::Amp => "ccusage-amp",
        AgentUsageProvider::Opencode => "ccusage-opencode",
    }
}

fn agent_usage_label(provider: AgentUsageProvider) -> &'static str {
    match provider {
        AgentUsageProvider::Claude => "claude",
        AgentUsageProvider::Codex => "codex",
        AgentUsageProvider::Amp => "amp",
        AgentUsageProvider::Opencode => "opencode",
    }
}

fn find_command_in_path(command_name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|entry| entry.join(command_name))
        .find(|candidate| candidate.is_file())
}

fn agent_usage_summary_from_json(raw: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return String::new();
    };
    let selected = value
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
        .or_else(|| value.get("block"))
        .unwrap_or(&value);

    let mut parts = Vec::new();
    if let Some(tokens) = first_u64_at(
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
    ) {
        parts.push(format_token_count(tokens));
    }
    if let Some(cost) = first_f64_at(
        selected,
        &[
            &["costUSD"],
            &["cost_usd"],
            &["totalCost"],
            &["total_cost"],
            &["totals", "costUSD"],
            &["totals", "cost_usd"],
        ],
    ) {
        parts.push(format_agent_usage_cost(cost));
    }
    if let Some(minutes) = first_i64_at(
        selected,
        &[
            &["remainingMinutes"],
            &["remaining_minutes"],
            &["projection", "remainingMinutes"],
            &["projection", "remaining_minutes"],
        ],
    ) {
        if minutes > 0 {
            parts.push(format_remaining_minutes(minutes));
        }
    }
    parts.join(" ")
}

fn render_agent_usage_widget(provider: AgentUsageProvider, summary: &str) -> String {
    let label = agent_usage_label(provider);
    format!("#[fg=#bb88ff,bold][{label} {summary}]")
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
    lines.push(format!(
        "  ai_activity: {}",
        render_status_bus_ai_activity_widget(value)
    ));
    lines.push(format!(
        "  token_budget: {}",
        render_status_bus_token_budget_widget(value)
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

fn nested_array<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Vec<Value>> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_array()
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

    if let Ok(retarget_result) = retarget_workspace_without_focused_cd(primary_target_path, None) {
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
            r#"{"schema_version":1,"active_tab_position":2,"workspace":{"root":"/tmp/project","source":"explicit"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"managed_panes":{"editor_pane_id":"terminal:7","sidebar_pane_id":"terminal:8"},"sidebar_yazi":{"yazi_id":"yazi-123","cwd":"/tmp/project"},"extensions":{"ai_pane_activity":[{"tab_position":2,"provider":"codex","pane_id":"terminal:9","activity":"thinking","state":"thinking"}],"ai_token_budget":[{"tab_position":2,"provider":"codex","remaining_tokens":120000,"total_tokens":200000}]}}"#,
        )
        .unwrap();
        let rendered = render_session_state_inspection_lines(&value).join("\n");

        assert!(rendered.contains("workspace: /tmp/project (explicit)"));
        assert!(rendered.contains("layout: active_swap_layout_name=single_open"));
        assert!(rendered.contains("managed_panes: editor=terminal:7, sidebar=terminal:8"));
        assert!(rendered.contains("sidebar_yazi: id=yazi-123, cwd=/tmp/project"));
        assert!(rendered.contains("ai_activity: codex:thinking"));
        assert!(rendered.contains("token_budget: codex:120k/200k"));
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
            r#"{"schema_version":1,"active_tab_position":0,"workspace":{"root":"/tmp/yazelix-demo","source":"explicit"},"managed_panes":{"editor_pane_id":"terminal:1","sidebar_pane_id":"terminal:2"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[],"ai_token_budget":[]}}"#,
        )
        .unwrap();

        assert_eq!(
            render_status_bus_workspace_widget(&value),
            "yazelix-demo/sidebar/side:open"
        );
    }

    // Defends: the AI activity widget consumes status-bus facts and prioritizes active/thinking over stale or idle states.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_bus_ai_activity_widget_formats_highest_priority_fact() {
        let value = decode_status_bus_snapshot(
            r#"{"schema_version":1,"active_tab_position":0,"workspace":null,"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"other","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[{"tab_position":0,"provider":"codex","pane_id":"terminal:1","activity":"stale","state":"stale"},{"tab_position":0,"provider":"claude","pane_id":"terminal:2","activity":"thinking","state":"thinking"}],"ai_token_budget":[]}}"#,
        )
        .unwrap();

        assert_eq!(
            render_status_bus_ai_activity_widget(&value),
            "claude:thinking"
        );
    }

    // Defends: the token-budget widget is a status-bus extension point and stays explicit when no provider facts exist.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_bus_token_budget_widget_formats_known_budget_and_unknown_empty_budget() {
        let value = decode_status_bus_snapshot(
            r#"{"schema_version":1,"active_tab_position":0,"workspace":null,"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"other","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[],"ai_token_budget":[{"tab_position":0,"provider":"codex","remaining_tokens":120000,"total_tokens":200000}]}}"#,
        )
        .unwrap();
        let empty = decode_status_bus_snapshot(
            r#"{"schema_version":1,"active_tab_position":0,"workspace":null,"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"other","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[],"ai_token_budget":[]}}"#,
        )
        .unwrap();

        assert_eq!(
            render_status_bus_token_budget_widget(&value),
            "codex:120k/200k"
        );
        assert_eq!(render_status_bus_token_budget_widget(&empty), "unknown");
    }

    // Defends: ccusage-backed tray widgets derive a compact, fully formatted segment from the active usage block.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn agent_usage_widget_formats_active_json_block() {
        let summary = agent_usage_summary_from_json(
            r#"{"blocks":[{"isActive":false,"totalTokens":10},{"isActive":true,"totalTokens":123456,"costUSD":1.234,"projection":{"remainingMinutes":137}}]}"#,
        );

        assert_eq!(summary, "123k $1.23 2h17m");
        assert_eq!(
            render_agent_usage_widget(AgentUsageProvider::Codex, &summary),
            "#[fg=#bb88ff,bold][codex 123k $1.23 2h17m]"
        );
    }

    // Defends: provider aliases map to the exact opt-in ccusage binaries used by flake and Home Manager package wiring.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn agent_usage_provider_aliases_map_to_binaries() {
        assert_eq!(
            parse_agent_usage_provider("claude").map(agent_usage_binary),
            Some("ccusage")
        );
        assert_eq!(
            parse_agent_usage_provider("ccusage-codex").map(agent_usage_binary),
            Some("ccusage-codex")
        );
        assert_eq!(
            parse_agent_usage_provider("amp").map(agent_usage_binary),
            Some("ccusage-amp")
        );
        assert_eq!(
            parse_agent_usage_provider("opencode").map(agent_usage_binary),
            Some("ccusage-opencode")
        );
    }
}
