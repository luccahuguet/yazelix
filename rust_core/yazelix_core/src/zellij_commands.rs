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
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use crate::session_facts::compute_session_facts_from_env;
use crate::workspace_commands::{compute_integration_facts_from_env, sync_sidebar_to_directory};
use crate::workspace_session::{
    WorkspaceRetargetResult, current_tab_workspace_root_from_json,
    parse_workspace_retarget_response,
};
use serde_json::{Value, json};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

mod status;

#[cfg(test)]
use rusqlite::Connection;
use status::nested_bool;
#[cfg(test)]
use status::*;
pub use status::{
    probe_active_tab_session_state, run_zellij_inspect_session, run_zellij_status_bus,
    run_zellij_status_cache_heartbeat, run_zellij_status_cache_refresh_claude_usage,
    run_zellij_status_cache_refresh_codex_usage, run_zellij_status_cache_refresh_opencode_go_usage,
    run_zellij_status_cache_widget, run_zellij_status_cache_write,
};
#[cfg(test)]
use std::ffi::OsStr;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::time::Instant;

const EDITOR_PANE_CREATE_LAYOUT_SETTLE_MS: u64 = 80;
const OPEN_FILE_ORCHESTRATOR_RETRY_DELAYS_MS: &[u64] = &[50, 100, 200];
const EDITOR_PANE_NAME: &str = "editor";
pub const INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS: &[&str] = &[
    "pipe",
    "get-workspace-root",
    "inspect-session",
    "status-bus",
    "status-cache-write",
    "status-cache-heartbeat",
    "status-cache-widget",
    "status-cache-refresh-claude-usage",
    "status-cache-refresh-codex-usage",
    "status-cache-refresh-opencode-go-usage",
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
    NotReady,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CurrentSidebarYaziRegistration {
    pane_id: String,
    yazi_id: String,
    cwd: String,
}

pub(crate) fn run_pane_orchestrator_runtime_config_reload(
    payload: &str,
) -> Result<String, CoreError> {
    run_pane_orchestrator_command("reload_runtime_config", payload)
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

fn workspace_retarget_status(result: &WorkspaceRetargetResult) -> &str {
    result.status()
}

fn hide_sidebar_if_visible() -> Result<(), CoreError> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    let state = serde_json::from_str::<Value>(response.trim()).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "sidebar_state_parse_failed",
            format!("Could not parse active Yazelix tab state: {source}"),
            "Ensure the pane orchestrator plugin is loaded, then retry opening the file.",
            json!({ "response": response }),
        )
    })?;

    let sidebar_collapsed = nested_bool(&state, &["layout", "sidebar_collapsed"]);
    match sidebar_collapsed {
        Some(true) => return Ok(()),
        Some(false) | None => {}
    }

    let hide_response = run_pane_orchestrator_command("hide_sidebar", "")?;
    let trimmed = hide_response.trim();
    if sidebar_collapsed.is_none() && matches!(trimmed, "unknown_layout" | "missing") {
        return Ok(());
    }

    match trimmed {
        "ok" | "closed" | "focused" => Ok(()),
        other => Err(CoreError::classified(
            ErrorClass::Runtime,
            "hide_sidebar_failed",
            format!("Could not hide the managed sidebar before opening the editor: {other}"),
            "Ensure the pane orchestrator plugin is loaded, then retry.",
            json!({ "response": hide_response }),
        )),
    }
}

fn hide_sidebar_after_editor_pane_creation() -> Result<(), CoreError> {
    thread::sleep(Duration::from_millis(EDITOR_PANE_CREATE_LAYOUT_SETTLE_MS));
    hide_sidebar_if_visible()
}

fn retarget_workspace_without_focused_cd(
    target_path: &Path,
    editor_kind: Option<&str>,
) -> Result<WorkspaceRetargetResult, CoreError> {
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
) -> Result<WorkspaceRetargetResult, CoreError> {
    let payload = json!({
        "workspace_root": target_dir.display().to_string(),
        "cd_focused_pane": false,
        "editor": editor_kind
            .map(str::trim)
            .filter(|editor| !editor.is_empty())
            .map(|editor| Value::String(editor.to_string()))
            .unwrap_or(Value::Null),
        "sidebar_yazi": current_sidebar_yazi_registration()
            .map(|registration| {
                json!({
                    "pane_id": registration.pane_id,
                    "yazi_id": registration.yazi_id,
                    "cwd": registration.cwd,
                })
            })
            .unwrap_or(Value::Null),
    })
    .to_string();
    let response = run_pane_orchestrator_command("retarget_workspace", &payload)?;
    Ok(parse_workspace_retarget_response(&response))
}

fn current_sidebar_yazi_registration() -> Option<CurrentSidebarYaziRegistration> {
    let yazi_id = env::var("YAZI_ID").ok()?;
    let yazi_id = yazi_id.trim();
    if yazi_id.is_empty() {
        return None;
    }

    let pane_id = env::var("ZELLIJ_PANE_ID").ok()?;
    let pane_id = normalize_terminal_pane_id(&pane_id)?;

    let cwd = env::current_dir().ok()?.display().to_string();
    if cwd.trim().is_empty() {
        return None;
    }

    Some(CurrentSidebarYaziRegistration {
        pane_id,
        yazi_id: yazi_id.to_string(),
        cwd,
    })
}

fn normalize_terminal_pane_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.contains(':') {
        Some(trimmed.to_string())
    } else {
        Some(format!("terminal:{trimmed}"))
    }
}

fn resolve_runtime_editor_launch() -> Result<(serde_json::Map<String, Value>, String), CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let facts = compute_session_facts_from_env()?;
    let mut normalized = serde_json::Map::new();
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
                "Set editor.command in settings.jsonc or export EDITOR before running this command.",
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

    for retry_index in 0..=OPEN_FILE_ORCHESTRATOR_RETRY_DELAYS_MS.len() {
        match run_pane_orchestrator_command("open_file", &payload) {
            Ok(response) => match parse_managed_editor_open_response(&response)? {
                ManagedEditorOpenStatus::NotReady => {}
                status => return Ok(status),
            },
            Err(error) if is_transient_orchestrator_pipe_error(&error) => {}
            Err(error) => return Err(error),
        }

        if let Some(delay_ms) = OPEN_FILE_ORCHESTRATOR_RETRY_DELAYS_MS.get(retry_index) {
            thread::sleep(Duration::from_millis(*delay_ms));
        }
    }

    Ok(ManagedEditorOpenStatus::NotReady)
}

fn parse_managed_editor_open_response(
    response: &str,
) -> Result<ManagedEditorOpenStatus, CoreError> {
    match response.trim() {
        "ok" | "opened" | "focused" => Ok(ManagedEditorOpenStatus::Ok),
        "missing" => Ok(ManagedEditorOpenStatus::Missing),
        "not_ready" => Ok(ManagedEditorOpenStatus::NotReady),
        other => Err(CoreError::classified(
            ErrorClass::Runtime,
            "managed_editor_open_failed",
            format!("Managed editor open failed: {other}"),
            "Ensure the Yazelix pane orchestrator is loaded and the managed editor pane title is `editor`, then retry.",
            json!({ "response": response }),
        )),
    }
}

fn is_transient_orchestrator_pipe_error(error: &CoreError) -> bool {
    if error.code() != "pane_orchestrator_pipe_failed" {
        return false;
    }
    let message = error.message().to_ascii_lowercase();
    message.contains("timed out")
        || message.contains("timeout")
        || message.contains("no response")
        || message.contains("did not receive")
        || message.contains("not ready")
        || message.contains("not_ready")
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

    let status = result.status();
    match status {
        "ok" => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "workspace_root": target_dir.display().to_string(),
                    "tab_name": tab_name,
                    "editor_status": result.editor_status,
                    "sidebar_state": result.sidebar_state,
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
            let reason = result.reason.as_deref().unwrap_or("unknown error");
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
    let mut created_editor_pane = false;

    if integration_facts.hide_sidebar_on_file_open {
        hide_sidebar_if_visible()?;
    }

    if editor_kind == "helix" || editor_kind == "neovim" {
        let open_status =
            open_files_in_managed_editor(&editor_kind, &target_paths, &editor_working_dir)?;
        if matches!(
            open_status,
            ManagedEditorOpenStatus::Missing | ManagedEditorOpenStatus::NotReady
        ) {
            let mut editor_argv = vec![editor_command.clone()];
            editor_argv.extend(target_paths.iter().map(|path| path.display().to_string()));
            run_zellij_editor_pane(
                &editor_working_dir,
                &runtime_env,
                Some(yazi_id.as_str()),
                &editor_argv,
            )?;
            created_editor_pane = true;
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
        created_editor_pane = true;
    }

    if let Ok(retarget_result) =
        retarget_workspace_dir_without_focused_cd(&editor_working_dir, None)
    {
        if workspace_retarget_status(&retarget_result) == "ok" {
            if let Some(sidebar_state) = retarget_result.sidebar_state.as_ref() {
                let _ = sync_sidebar_to_directory(
                    &integration_facts.ya_command,
                    &home_dir_from_env()?,
                    &sidebar_state,
                    primary_target_path,
                );
            }
        }
    }

    if created_editor_pane && integration_facts.hide_sidebar_on_file_open {
        hide_sidebar_after_editor_pane_creation()?;
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

    if integration_facts.hide_sidebar_on_file_open {
        hide_sidebar_if_visible()?;
    }

    let retarget_result =
        retarget_workspace_without_focused_cd(&target_dir, Some(editor_kind.as_str()))?;
    let mut created_editor_pane = false;
    let status = workspace_retarget_status(&retarget_result);
    if status != "ok" {
        let reason = retarget_result.reason.as_deref().unwrap_or(status);
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "retarget_workspace_failed",
            format!("Failed to retarget the current workspace: {reason}"),
            "Ensure the pane orchestrator plugin is loaded and the current tab is ready, then retry.",
            json!({ "status": status }),
        ));
    }

    match retarget_result.editor_status.as_str() {
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
            created_editor_pane = true;
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

    if let Some(sidebar_state) = retarget_result.sidebar_state.as_ref() {
        let _ = sync_sidebar_to_directory(
            &integration_facts.ya_command,
            &home_dir_from_env()?,
            &sidebar_state,
            &target_dir,
        );
    }

    if created_editor_pane && integration_facts.hide_sidebar_on_file_open {
        hide_sidebar_after_editor_pane_creation()?;
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
    let retarget_status = retarget_result.status();

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

    // Defends: the cursor widget renders mono and split cursor previews from cached launch facts without widening the status segment.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_cursor_widget_renders_cached_launch_fact() {
        let mono = json!({
            "schema_version": STATUS_BAR_CACHE_SCHEMA_VERSION,
            "updated_at_unix_seconds": 1_000,
            "status_bus": status_cache_test_status_bus(),
            "agent_usage": {},
            "cursor": {
                "terminal": "ghostty",
                "name": "reef",
                "color": "#14D9A0",
                "family": "mono"
            }
        });
        let vertical_split = json!({
            "cursor": {
                "terminal": "ghostty",
                "name": "reef",
                "color": "#00e6ff",
                "family": "split",
                "divider": "vertical",
                "primary_color": "#00e6ff",
                "secondary_color": "#00ff66"
            }
        });
        let horizontal_split = json!({
            "cursor": {
                "terminal": "ghostty",
                "name": "magma",
                "color": "#ff1600",
                "family": "split",
                "divider": "horizontal",
                "primary_color": "#ff1600",
                "secondary_color": "#2a3340"
            }
        });
        let display_color_differs_from_split_primary = json!({
            "cursor": {
                "terminal": "ghostty",
                "name": "eclipse",
                "color": "#ffd400",
                "family": "split",
                "divider": "vertical",
                "primary_color": "#2e294e",
                "secondary_color": "#ffd400"
            }
        });
        let invalid_split = json!({
            "cursor": {
                "name": "magma",
                "color": "#ff1600",
                "family": "split",
                "divider": "horizontal",
                "primary_color": "#ff1600",
                "secondary_color": "hot"
            }
        });

        assert_eq!(
            render_status_cache_widget(&mono, "cursor").unwrap(),
            " #[fg=#14d9a0,bg=default,bold][#[fg=#14d9a0,bold]█#[fg=#14d9a0,bg=default,bold] reef]"
        );
        assert_eq!(
            render_status_cache_widget(&vertical_split, "cursor").unwrap(),
            " #[fg=#00e6ff,bg=default,bold][#[fg=#00e6ff,bg=#00ff66,bold]▌#[fg=#00e6ff,bg=default,bold] reef]"
        );
        assert_eq!(
            render_status_cache_widget(&horizontal_split, "cursor").unwrap(),
            " #[fg=#ff1600,bg=default,bold][#[fg=#ff1600,bg=#2a3340,bold]▀#[fg=#ff1600,bg=default,bold] magma]"
        );
        assert_eq!(
            render_status_cache_widget(&display_color_differs_from_split_primary, "cursor")
                .unwrap(),
            " #[fg=#ffd400,bg=default,bold][#[fg=#2e294e,bg=#ffd400,bold]▌#[fg=#ffd400,bg=default,bold] eclipse]"
        );
        assert_eq!(
            render_status_cache_widget(&invalid_split, "cursor").unwrap(),
            " #[fg=#ff1600,bg=default,bold][#[fg=#ff1600,bold]█#[fg=#ff1600,bg=default,bold] magma]"
        );
        assert_eq!(
            render_status_cache_widget(&json!({"cursor": {"name": "n/a"}}), "cursor").unwrap(),
            " #[fg=#00ff88,bg=default,bold][#[fg=#00ff88,bold]█#[fg=#00ff88,bg=default,bold] n/a]"
        );
        assert_eq!(
            render_status_cache_widget(&json!({"cursor": {"name": ""}}), "cursor").unwrap(),
            ""
        );
    }

    // Defends: cursor status facts are copied from launch env as small terminal-scoped data, not by parsing config on every bar refresh.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn cursor_status_value_uses_non_empty_launch_env_values() {
        assert_eq!(
            cursor_status_value(
                Some(OsStr::new("ghostty")),
                Some(OsStr::new("magma")),
                Some(OsStr::new("#FF1600")),
                Some(OsStr::new("split")),
                Some(OsStr::new("horizontal")),
                Some(OsStr::new("#FF1600")),
                Some(OsStr::new("#2A3340")),
            ),
            Some(json!({
                "terminal": "ghostty",
                "name": "magma",
                "color": "#ff1600",
                "family": "split",
                "divider": "horizontal",
                "primary_color": "#ff1600",
                "secondary_color": "#2a3340"
            }))
        );
        assert_eq!(
            cursor_status_value(
                Some(OsStr::new("ghostty")),
                Some(OsStr::new("  ")),
                Some(OsStr::new("#ff1600")),
                Some(OsStr::new("split")),
                Some(OsStr::new("horizontal")),
                Some(OsStr::new("#ff1600")),
                Some(OsStr::new("#2a3340")),
            ),
            None
        );
    }

    // Defends: heartbeat updates merge into the window-local cache without replacing status-bus or usage facts.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_heartbeat_merge_preserves_cached_session_facts() {
        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
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
        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
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

    // Regression: usage widgets should first-paint from recent sibling/shared caches before the new window writes its status-bus cache.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn usage_widgets_render_from_existing_caches_before_status_bus_write() {
        let temp = tempfile::tempdir().unwrap();
        let sessions_dir = temp.path().join("state").join("sessions");
        let new_cache_path = sessions_dir.join("window_b").join("status_bar_cache.json");
        let now = unix_time_seconds();

        let claude_shared_path =
            claude_usage_shared_cache_path_from_status_cache_path(&new_cache_path).unwrap();
        write_json_value_atomic(
            &claude_shared_path,
            &json!({
                "schema_version": CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
                "claude": {
                    "updated_at_unix_seconds": now,
                    "five_hour_tokens": 42_000_000u64,
                    "weekly_tokens": 420_000_000u64,
                    "five_hour_remaining_percent": 73u64,
                    "weekly_remaining_percent": 81u64,
                    "status": "ok"
                }
            }),
            "claude_usage_cache_test",
        )
        .unwrap();
        let codex_shared_path =
            codex_usage_shared_cache_path_from_status_cache_path(&new_cache_path).unwrap();
        write_json_value_atomic(
            &codex_shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": now,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64,
                    "five_hour_reset_at_unix_seconds": now + 2 * HOUR_SECONDS,
                    "weekly_reset_at_unix_seconds": now + 3 * DAY_SECONDS,
                    "five_hour_window_seconds": 5 * HOUR_SECONDS,
                    "weekly_window_seconds": 7 * DAY_SECONDS,
                    "status": "ok"
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();
        let opencode_go_shared_path =
            opencode_go_usage_shared_cache_path_from_status_cache_path(&new_cache_path).unwrap();
        write_json_value_atomic(
            &opencode_go_shared_path,
            &json!({
                "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
                "opencode_go": {
                    "updated_at_unix_seconds": now,
                    "five_hour_tokens": 0u64,
                    "five_hour_remaining_percent": 100u64,
                    "weekly_tokens": 85_000_000u64,
                    "weekly_remaining_percent": 60u64,
                    "monthly_tokens": 85_000_000u64,
                    "monthly_remaining_percent": 80u64,
                    "status": "ok"
                }
            }),
            "opencode_go_usage_cache_test",
        )
        .unwrap();

        let mut claude_cache =
            status_cache_value_for_widget_path(&new_cache_path, "claude_usage", now).unwrap();
        hydrate_status_cache_claude_usage(&mut claude_cache, &new_cache_path);
        assert_eq!(
            render_status_cache_widget(&claude_cache, "claude_usage").unwrap(),
            " [claude 5h|42M|73% wk|420M|81%]"
        );

        let mut codex_cache =
            status_cache_value_for_widget_path(&new_cache_path, "codex_usage", now).unwrap();
        hydrate_status_cache_codex_usage(&mut codex_cache, &new_cache_path);
        assert_eq!(
            render_status_cache_widget(&codex_cache, "codex_usage").unwrap(),
            " [codex 3h/5h 49% · 4d/7d 80%]"
        );

        let mut opencode_go_cache =
            status_cache_value_for_widget_path(&new_cache_path, "opencode_go_usage", now).unwrap();
        hydrate_status_cache_opencode_go_usage(&mut opencode_go_cache, &new_cache_path);
        assert_eq!(
            render_status_cache_widget(&opencode_go_cache, "opencode_go_usage").unwrap(),
            " [go 5h|0|100% wk|85M|60% mo|85M|80%]"
        );

        assert!(status_cache_value_for_widget_path(&new_cache_path, "workspace", now).is_none());
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
                    "zellij_widget_tray": ["claude_usage", "opencode_go_usage"],
                    "zellij_claude_usage_display": "quota",
                    "zellij_claude_usage_periods": ["week"],
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
        assert!(usage_widget_enabled_from_status_cache_path(
            &cache_path,
            "claude_usage"
        ));
        let settings = agent_usage_widget_settings_from_status_cache_path(&cache_path);
        assert_eq!(settings.claude_display, WindowedUsageDisplay::Quota);
        assert_eq!(settings.claude_periods, vec![WindowedUsagePeriod::Weekly]);
        assert_eq!(settings.codex_display, WindowedUsageDisplay::Quota);
        assert_eq!(settings.opencode_go_display, WindowedUsageDisplay::Quota);
        assert_eq!(
            settings.opencode_go_periods,
            vec![WindowedUsagePeriod::FiveHour, WindowedUsagePeriod::Monthly]
        );
    }

    // Defends: Claude usage mirrors the compact 5h/week token/quota contract selected by claude_usage_display.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_claude_usage_renders_5h_week_display_modes() {
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
            "claude_usage": {
                "five_hour_tokens": 15456373u64,
                "weekly_tokens": 66610005u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64
            }
        });

        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "claude_usage",
                &AgentUsageWidgetSettings {
                    claude_display: WindowedUsageDisplay::Both,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [claude 5h|15.5M|49% wk|66.6M|80%]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "claude_usage",
                &AgentUsageWidgetSettings {
                    claude_display: WindowedUsageDisplay::Token,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [claude 5h|15.5M wk|66.6M]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "claude_usage",
                &AgentUsageWidgetSettings {
                    claude_display: WindowedUsageDisplay::Quota,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [claude 5h|49% wk|80%]"
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
                "updated_at_unix_seconds": 10u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64,
                "five_hour_reset_at_unix_seconds": 9610u64,
                "weekly_reset_at_unix_seconds": 241210u64,
                "five_hour_window_seconds": 18000u64,
                "weekly_window_seconds": 604800u64
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
            " [codex 2h20m/5h 138M 49% · 4d5h/7d 1.34B 80%]"
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
            " [codex 2h20m/5h 138M · 4d5h/7d 1.34B]"
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
            " [codex 2h20m/5h 49% · 4d5h/7d 80%]"
        );
    }

    // Regression: Codex window labels show current window position instead of time remaining until reset.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn codex_window_label_reports_elapsed_position() {
        assert_eq!(
            format_reset_window_label(2 * DAY_SECONDS, 7 * DAY_SECONDS, 7 * HOUR_SECONDS),
            Some("5d7h/7d".to_string())
        );
        assert_eq!(
            format_reset_window_label(5 * HOUR_SECONDS, 5 * HOUR_SECONDS, 10 * MINUTE_SECONDS),
            Some("10m/5h".to_string())
        );
    }

    // Regression: quota-only Codex widgets must remain visible while official quota data is temporarily unavailable.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_codex_usage_quota_mode_renders_partial_token_cache() {
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
                "updated_at_unix_seconds": 10u64,
                "five_hour_tokens": 4015883u64,
                "weekly_tokens": 106335620u64,
                "status": "partial",
                "quota_backoff_until_unix_seconds": 1810u64
            }
        });

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
            " [codex 5h n/a · wk n/a]"
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
            " [go 5h|138M|49% wk|1.34B|80% mo|2.22B|70%]"
        );
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "opencode_go_usage",
                &AgentUsageWidgetSettings {
                    opencode_go_periods: vec![WindowedUsagePeriod::Weekly],
                    opencode_go_display: WindowedUsageDisplay::Both,
                    ..AgentUsageWidgetSettings::default()
                },
            )
            .unwrap(),
            " [go wk|1.34B|80%]"
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

    // Defends: tokenusage JSON shape for active-block, weekly, and official quota facts maps to the compact widget contract.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn tokenusage_json_parsers_read_windows_and_official_quota() {
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
                "secondary_used_percent": 20.0,
                "primary_resets_at": 8_200u64,
                "primary_window_mins": 300u64,
                "secondary_resets_at": 260_200u64,
                "secondary_window_mins": 10_080u64
            },
            "official_claude": {
                "primary_used_percent": 25.0,
                "secondary_used_percent": 35.0
            }
        });

        let codex_quota =
            tokenusage_quota_from_official_json(&official, TokenusageWindowedProvider::Codex);
        let claude_quota =
            tokenusage_quota_from_official_json(&official, TokenusageWindowedProvider::Claude);

        assert_eq!(
            tokenusage_active_block_tokens_from_json(&active),
            Some(138424632)
        );
        assert_eq!(
            tokenusage_weekly_tokens_from_json(&weekly),
            Some(1335519960)
        );
        assert_eq!(codex_quota.five_hour_remaining_percent, Some(49));
        assert_eq!(codex_quota.weekly_remaining_percent, Some(80));
        assert_eq!(codex_quota.five_hour_reset_at_unix_seconds, Some(8_200));
        assert_eq!(codex_quota.weekly_reset_at_unix_seconds, Some(260_200));
        assert_eq!(codex_quota.five_hour_window_seconds, Some(18_000));
        assert_eq!(codex_quota.weekly_window_seconds, Some(604_800));
        assert_eq!(claude_quota.five_hour_remaining_percent, Some(75));
        assert_eq!(claude_quota.weekly_remaining_percent, Some(65));
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
      printf '%s\n' '{"official_codex":{"primary_used_percent":51.0,"secondary_used_percent":20.0,"primary_resets_at":8200,"primary_window_mins":300,"secondary_resets_at":260200,"secondary_window_mins":10080}}'
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
        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
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
            " [codex 3h/5h 138M 49% · 4d/7d 1.34B 80%]"
        );
    }

    // Regression: a partial Codex refresh must not erase a known 5h token count while the official reset window is unchanged.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn codex_usage_refresh_preserves_missing_tokens_for_same_reset_window() {
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
      printf '%s\n' '{"official_codex":{"primary_used_percent":51.0,"secondary_used_percent":20.0,"primary_resets_at":8200,"primary_window_mins":300,"secondary_resets_at":260200,"secondary_window_mins":10080}}'
      exit 0
      ;;
    *)
      exit 65
      ;;
  esac
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
        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 0u64,
                    "five_hour_tokens": 999000u64,
                    "weekly_tokens": 1000000000u64,
                    "five_hour_remaining_percent": 60u64,
                    "weekly_remaining_percent": 50u64,
                    "five_hour_reset_at_unix_seconds": 8200u64,
                    "weekly_reset_at_unix_seconds": 260200u64,
                    "five_hour_window_seconds": 18000u64,
                    "weekly_window_seconds": 604800u64
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();

        let refreshed = refresh_codex_usage_shared_cache(
            &shared_path,
            Some(bin_dir.as_os_str()),
            1_000,
            600,
            1_800,
            Duration::from_secs(1),
        )
        .unwrap();
        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
        hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);

        assert!(refreshed);
        assert_eq!(
            cache
                .get("codex_usage")
                .and_then(|entry| entry.get("five_hour_tokens"))
                .and_then(Value::as_u64),
            Some(999000)
        );
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
            " [codex 3h/5h 999k 49% · 4d/7d 1.34B 80%]"
        );
    }

    // Regression: transient official quota failures must not replace a previously good Codex widget with n/a labels.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn codex_usage_refresh_preserves_previous_quota_during_probe_backoff() {
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
      exit 65
      ;;
    *)
      printf '%s\n' '{"blocks":[{"is_active":true,"totals":{"total_tokens":999000}}]}'
      exit 0
      ;;
  esac
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":1000000}}]}'
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
        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 0u64,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64,
                    "five_hour_reset_at_unix_seconds": 10000u64,
                    "weekly_reset_at_unix_seconds": 260200u64,
                    "five_hour_window_seconds": 18000u64,
                    "weekly_window_seconds": 604800u64,
                    "status": "ok"
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();

        let refreshed = refresh_codex_usage_shared_cache(
            &shared_path,
            Some(bin_dir.as_os_str()),
            1_000,
            600,
            1_800,
            Duration::from_secs(1),
        )
        .unwrap();
        let shared_cache = read_codex_usage_shared_cache_value(&shared_path).unwrap();
        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
        hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);

        assert!(refreshed);
        assert_eq!(
            shared_cache
                .get("codex")
                .and_then(|entry| entry.get("quota_backoff_until_unix_seconds"))
                .and_then(Value::as_u64),
            Some(2_800)
        );
        assert_eq!(
            shared_cache
                .get("codex")
                .and_then(|entry| entry.get("status"))
                .and_then(Value::as_str),
            Some("partial")
        );
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
            " [codex 2h30m/5h 999k 49% · 4d/7d 1M 80%]"
        );
    }

    // Regression: runtime skew must not let old Codex cache writers overwrite the cache file read by a newer schema.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn status_cache_codex_usage_uses_schema_scoped_shared_cache_path() {
        let temp = tempfile::tempdir().unwrap();
        let status_cache_path = temp
            .path()
            .join("state")
            .join("sessions")
            .join("window_a")
            .join("status_bar_cache.json");
        let shared_path =
            codex_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();
        assert_eq!(
            shared_path.file_name().and_then(|name| name.to_str()),
            Some("codex_usage_cache_v2.json")
        );

        write_json_value_atomic(
            &shared_path.with_file_name("codex_usage_cache.json"),
            &json!({
                "schema_version": 1,
                "codex": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64,
                    "status": "ok"
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();

        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
        hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);
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
            ""
        );

        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64,
                    "five_hour_reset_at_unix_seconds": 8_200u64,
                    "weekly_reset_at_unix_seconds": 260_200u64,
                    "five_hour_window_seconds": 18_000u64,
                    "weekly_window_seconds": 604_800u64,
                    "status": "ok"
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();

        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
        hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);
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
            " [codex 3h/5h 138M 49% · 4d/7d 1.34B 80%]"
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
        let now = 2_000_000u64;
        write_opencode_go_usage_test_db(&db_path, now);

        let facts = collect_opencode_go_usage_facts_from_dbs(&[db_path], now);

        assert_eq!(facts.five_hour_tokens, Some(15_000_000));
        assert_eq!(facts.weekly_tokens, Some(100_000_000));
        assert_eq!(facts.monthly_tokens, Some(300_000_000));
        assert_eq!(facts.five_hour_remaining_percent, Some(75));
        assert_eq!(facts.weekly_remaining_percent, Some(50));
        assert_eq!(facts.monthly_remaining_percent, Some(50));
    }

    // Regression: a quiet 5h OpenCode Go window should still render quota instead of disappearing from the combined widget.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn opencode_go_sqlite_reader_keeps_empty_window_quota_facts() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("opencode.db");
        let now = 2_000_000u64;
        let connection = Connection::open(&db_path).unwrap();
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
        let created_at = unix_millis_from_seconds_saturating(now.saturating_sub(6 * 60 * 60));
        connection
            .execute(
                "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES ('within_week', 'session', ?1, ?1, ?2)",
                rusqlite::params![
                    created_at,
                    r#"{"role":"assistant","providerID":"opencode-go","cost":12.0,"tokens":{"total":85000000}}"#
                ],
            )
            .unwrap();

        let facts = collect_opencode_go_usage_facts_from_dbs(&[db_path], now);

        assert_eq!(facts.five_hour_tokens, Some(0));
        assert_eq!(facts.five_hour_remaining_percent, Some(100));
        assert_eq!(facts.weekly_tokens, Some(85_000_000));
        assert_eq!(facts.weekly_remaining_percent, Some(60));
        assert_eq!(facts.monthly_tokens, Some(85_000_000));
        assert_eq!(facts.monthly_remaining_percent, Some(80));

        let cache = json!({
            "schema_version": 1,
            "updated_at_unix_seconds": now,
            "opencode_go_usage": {
                "five_hour_tokens": facts.five_hour_tokens,
                "five_hour_remaining_percent": facts.five_hour_remaining_percent,
                "weekly_tokens": facts.weekly_tokens,
                "weekly_remaining_percent": facts.weekly_remaining_percent,
                "monthly_tokens": facts.monthly_tokens,
                "monthly_remaining_percent": facts.monthly_remaining_percent
            }
        });
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "opencode_go_usage",
                &AgentUsageWidgetSettings::default(),
            )
            .unwrap(),
            " [go 5h|0|100% wk|85M|60% mo|85M|80%]"
        );
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
        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), now);
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

    // Regression: old OpenCode Go shared caches without complete 5h/week/month fields must refresh instead of hiding the 5h window.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn opencode_go_usage_shared_cache_rejects_partial_fresh_shape() {
        let temp = tempfile::tempdir().unwrap();
        let shared_path = temp.path().join("opencode_go_usage_cache.json");

        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
                "opencode_go": {
                    "updated_at_unix_seconds": 1_000u64,
                    "weekly_tokens": 85_000_000u64,
                    "weekly_remaining_percent": 60u64,
                    "monthly_tokens": 85_000_000u64,
                    "monthly_remaining_percent": 80u64,
                    "status": "ok"
                }
            }),
            "opencode_go_usage_cache_test",
        )
        .unwrap();
        assert!(!opencode_go_usage_shared_cache_is_fresh(
            &shared_path,
            1_001,
            600
        ));

        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
                "opencode_go": {
                    "updated_at_unix_seconds": 1_001u64,
                    "five_hour_tokens": 0u64,
                    "five_hour_remaining_percent": 100u64,
                    "weekly_tokens": 85_000_000u64,
                    "weekly_remaining_percent": 60u64,
                    "monthly_tokens": 85_000_000u64,
                    "monthly_remaining_percent": 80u64,
                    "status": "ok"
                }
            }),
            "opencode_go_usage_cache_test",
        )
        .unwrap();
        assert!(opencode_go_usage_shared_cache_is_fresh(
            &shared_path,
            1_002,
            600
        ));
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
                "schema_version": 1,
                "codex": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();
        assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_100, 600));

        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();
        assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_100, 600));

        write_json_value_atomic(
            &shared_path,
            &json!({
                "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
                "codex": {
                    "updated_at_unix_seconds": 1_000u64,
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "five_hour_remaining_percent": 49u64,
                    "weekly_remaining_percent": 80u64,
                    "five_hour_reset_at_unix_seconds": 8_200u64,
                    "weekly_reset_at_unix_seconds": 260_200u64,
                    "five_hour_window_seconds": 18_000u64,
                    "weekly_window_seconds": 604_800u64
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
                    "five_hour_tokens": 138424632u64,
                    "weekly_tokens": 1335519960u64,
                    "error": "quota unavailable",
                    "backoff_until_unix_seconds": 2_000u64
                }
            }),
            "codex_usage_cache_test",
        )
        .unwrap();
        assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_701, 600));
        assert!(!codex_usage_shared_cache_is_backing_off(
            &shared_path,
            1_999
        ));

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

    // Regression: the dedicated Claude refresh writes a shared 5h/week token/quota cache that new windows hydrate before rendering.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn status_cache_claude_usage_refresh_writes_shared_combined_cache() {
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
      printf '%s\n' '{"official_claude":{"primary_used_percent":25.0,"secondary_used_percent":35.0}}'
      ;;
    *)
      printf '%s\n' '{"blocks":[{"is_active":true,"totals":{"total_tokens":15456373}}]}'
      ;;
  esac
  exit 0
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":66610005}}]}'
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
            claude_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();

        let refreshed = refresh_tokenusage_windowed_usage_shared_cache(
            &shared_path,
            TokenusageWindowedProvider::Claude,
            Some(bin_dir.as_os_str()),
            1_000,
            600,
            1_800,
            Duration::from_secs(1),
        );
        let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
        hydrate_status_cache_claude_usage(&mut cache, &status_cache_path);

        assert!(refreshed.unwrap());
        assert_eq!(
            render_status_cache_widget_with_agent_usage_settings(
                &cache,
                "claude_usage",
                &AgentUsageWidgetSettings::default(),
            )
            .unwrap(),
            " [claude 5h|15.5M|75% wk|66.6M|65%]"
        );
    }

    // Regression: logged-out Claude quota probes must back off without stopping cheap local token refreshes.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn tokenusage_windowed_refresh_backs_off_missing_quota_only() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let calls_path = temp.path().join("tu_calls.log");
        let provider = bin_dir.join("tu");
        fs::write(
            &provider,
            format!(
                r#"#!/usr/bin/env sh
printf '%s\n' "$*" >> '{}'
if [ "$1" = "blocks" ] && [ "$2" = "--active" ]; then
  case " $* " in
    *" --official-limits "*)
      printf '%s\n' '{{"official_claude":null}}'
      ;;
    *)
      printf '%s\n' '{{"blocks":[{{"is_active":true,"totals":{{"total_tokens":15456373}}}}]}}'
      ;;
  esac
  exit 0
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{{"weekly":[{{"totals":{{"total_tokens":66610005}}}}]}}'
  exit 0
fi
exit 64
"#,
                calls_path.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&provider).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&provider, permissions).unwrap();
        let shared_path = temp.path().join("claude_usage_cache.json");

        assert!(
            refresh_tokenusage_windowed_usage_shared_cache(
                &shared_path,
                TokenusageWindowedProvider::Claude,
                Some(bin_dir.as_os_str()),
                1_000,
                10,
                1_800,
                Duration::from_secs(1),
            )
            .unwrap()
        );
        assert!(tokenusage_windowed_usage_quota_is_backing_off(
            &shared_path,
            TokenusageWindowedProvider::Claude,
            1_001,
        ));
        assert!(
            refresh_tokenusage_windowed_usage_shared_cache(
                &shared_path,
                TokenusageWindowedProvider::Claude,
                Some(bin_dir.as_os_str()),
                1_010,
                10,
                1_800,
                Duration::from_secs(1),
            )
            .unwrap()
        );

        let calls = fs::read_to_string(calls_path).unwrap();
        assert_eq!(
            calls
                .lines()
                .filter(|line| line.contains("--official-limits"))
                .count(),
            1
        );
        assert_eq!(
            calls
                .lines()
                .filter(|line| line.starts_with("blocks --active --json --offline"))
                .count(),
            2
        );
        assert_eq!(
            calls
                .lines()
                .filter(|line| line.starts_with("weekly --json --offline"))
                .count(),
            2
        );
    }

    // Regression: hung tokenusage providers are killed quickly so the cache producer cannot recreate the CPU-spike failure mode.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn tokenusage_windowed_refresh_times_out_hung_provider() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let provider = bin_dir.join("tu");
        fs::write(&provider, "#!/usr/bin/env sh\nsleep 5\n").unwrap();
        let mut permissions = fs::metadata(&provider).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&provider, permissions).unwrap();
        let started = Instant::now();
        let shared_path = temp.path().join("claude_usage_cache.json");

        let refreshed = refresh_tokenusage_windowed_usage_shared_cache(
            &shared_path,
            TokenusageWindowedProvider::Claude,
            Some(bin_dir.as_os_str()),
            1_000,
            10,
            1_800,
            Duration::from_millis(50),
        )
        .unwrap();

        assert!(refreshed);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(
            read_claude_usage_shared_cache_value(&shared_path)
                .and_then(|cache| cache.pointer("/claude/status").cloned())
                .and_then(|status| status.as_str().map(str::to_string)),
            Some("error".to_string())
        );
    }
}
