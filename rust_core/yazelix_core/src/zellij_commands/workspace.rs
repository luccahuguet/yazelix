//! Workspace/editor/session Zellij commands for `yzx_control`.

use super::status::{decode_status_bus_snapshot, nested_bool, nested_str};
use crate::bridge::{CoreError, ErrorClass};
use crate::compute_runtime_env;
use crate::control_plane::{
    home_dir_from_env, json_map_to_child_env, runtime_dir_from_env, runtime_env_request,
};
use crate::helix_bridge_client::{HelixBridgeActionTarget, send_helix_bridge_action_to_target};
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use crate::session_facts::compute_session_facts_from_env;
use crate::workspace_commands::{compute_integration_facts_from_env, sync_sidebar_to_directory};
use crate::workspace_session::{
    SidebarYaziRegistration, WorkspaceRetargetResult, managed_editor_open_payload,
    open_terminal_in_cwd_payload, parse_workspace_retarget_response, workspace_dir_for_target,
    workspace_retarget_payload, workspace_tab_name,
};
use serde_json::{Value, json};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const EDITOR_PANE_CREATE_LAYOUT_SETTLE_MS: u64 = 80;
const OPEN_FILE_ORCHESTRATOR_RETRY_DELAYS_MS: &[u64] = &[50, 100, 200];
const EDITOR_PANE_NAME: &str = "editor";
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
enum ManagedEditorPaneTarget {
    Ready(String),
    Missing,
    NotReady,
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

fn resolve_existing_target(
    target_path: &str,
    cwd_code: &str,
    missing_code: &str,
    missing_suggestion: &str,
) -> Result<PathBuf, CoreError> {
    let path = PathBuf::from(target_path);
    let expanded = if path.is_absolute() {
        path
    } else {
        env::current_dir()
            .map_err(|source| {
                CoreError::io(
                    cwd_code,
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
            missing_code,
            format!("Path does not exist: {}", canonical.display()),
            missing_suggestion,
            json!({ "path": canonical.display().to_string() }),
        ));
    }

    Ok(canonical)
}

fn resolve_target_dir(target_path: &str) -> Result<PathBuf, CoreError> {
    resolve_existing_target(
        target_path,
        "retarget_cwd",
        "missing_workspace_target",
        "Choose an existing directory or file path, then retry.",
    )
    .map(|path| workspace_dir_for_target(&path))
}

fn resolve_existing_target_path(target_path: &str) -> Result<PathBuf, CoreError> {
    resolve_existing_target(
        target_path,
        "editor_target_cwd",
        "missing_editor_target",
        "Choose an existing file or directory path, then retry.",
    )
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
    let target_dir = workspace_dir_for_target(target_path);

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
    let target_dir = workspace_dir_for_target(target_path);
    retarget_workspace_dir_without_focused_cd(&target_dir, editor_kind)
}

fn retarget_workspace_dir_without_focused_cd(
    target_dir: &Path,
    editor_kind: Option<&str>,
) -> Result<WorkspaceRetargetResult, CoreError> {
    let sidebar_yazi = current_sidebar_yazi_registration();
    let payload = workspace_retarget_payload(target_dir, false, editor_kind, sidebar_yazi.as_ref());
    let response = run_pane_orchestrator_command("retarget_workspace", &payload)?;
    Ok(parse_workspace_retarget_response(&response))
}

fn current_sidebar_yazi_registration() -> Option<SidebarYaziRegistration> {
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

    Some(SidebarYaziRegistration {
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
    if let Some(helix_external) = facts.helix_external {
        normalized.insert("helix_external".to_string(), helix_external.as_json());
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
    if editor_kind == "helix" {
        return open_files_in_helix_bridge_managed_editor(file_paths, working_dir);
    }

    let payload = managed_editor_open_payload(editor_kind, file_paths, working_dir);

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

fn open_files_in_helix_bridge_managed_editor(
    file_paths: &[PathBuf],
    working_dir: &Path,
) -> Result<ManagedEditorOpenStatus, CoreError> {
    for retry_index in 0..=OPEN_FILE_ORCHESTRATOR_RETRY_DELAYS_MS.len() {
        match active_managed_editor_pane_target() {
            Ok(ManagedEditorPaneTarget::Ready(zellij_pane_id)) => {
                focus_managed_editor_pane()?;
                let payload = json!({
                    "working_dir": working_dir.display().to_string(),
                    "file_paths": file_paths
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>(),
                    "focus": true,
                });
                send_helix_bridge_action_to_target(
                    HelixBridgeActionTarget {
                        session_id: None,
                        instance_id: None,
                        zellij_pane_id: Some(zellij_pane_id),
                    },
                    "helix.open_files",
                    payload,
                    5_000,
                )?;
                return Ok(ManagedEditorOpenStatus::Ok);
            }
            Ok(ManagedEditorPaneTarget::Missing) => return Ok(ManagedEditorOpenStatus::Missing),
            Ok(ManagedEditorPaneTarget::NotReady) => {}
            Err(error) if is_transient_orchestrator_pipe_error(&error) => {}
            Err(error) => return Err(error),
        }

        if let Some(delay_ms) = OPEN_FILE_ORCHESTRATOR_RETRY_DELAYS_MS.get(retry_index) {
            thread::sleep(Duration::from_millis(*delay_ms));
        }
    }

    Ok(ManagedEditorOpenStatus::NotReady)
}

fn open_directory_in_helix_bridge_managed_editor(
    working_dir: &Path,
) -> Result<ManagedEditorOpenStatus, CoreError> {
    let status = set_helix_bridge_managed_editor_cwd(working_dir)?;
    if status == ManagedEditorOpenStatus::Ok {
        focus_managed_editor_pane()?;
    }
    Ok(status)
}

fn active_managed_editor_pane_target() -> Result<ManagedEditorPaneTarget, CoreError> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "")?;
    match response.trim() {
        "not_ready" => return Ok(ManagedEditorPaneTarget::NotReady),
        "missing" => return Ok(ManagedEditorPaneTarget::Missing),
        "permissions_denied" => {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "managed_editor_state_permissions_denied",
                "Pane orchestrator permissions are not granted for managed editor state.",
                "Run `yzx doctor --fix`, restart Yazelix, and retry.",
                json!({}),
            ));
        }
        _ => {}
    }

    let state = decode_status_bus_snapshot(&response)?;
    Ok(
        match nested_str(&state, &["managed_panes", "editor_pane_id"]) {
            Some(pane_id) => ManagedEditorPaneTarget::Ready(pane_id.to_string()),
            None => ManagedEditorPaneTarget::Missing,
        },
    )
}

fn focus_managed_editor_pane() -> Result<(), CoreError> {
    let response = run_pane_orchestrator_command("focus_editor", "")?;
    match response.trim() {
        "ok" | "focused" => Ok(()),
        "missing" => Err(CoreError::classified(
            ErrorClass::Runtime,
            "managed_editor_focus_missing",
            "The managed editor pane disappeared before Yazelix could focus it.",
            "Retry after the Yazelix layout settles.",
            json!({ "response": response }),
        )),
        other => Err(CoreError::classified(
            ErrorClass::Runtime,
            "managed_editor_focus_failed",
            format!("Could not focus the managed editor pane: {other}"),
            "Ensure the pane orchestrator plugin is loaded, then retry.",
            json!({ "response": response }),
        )),
    }
}

fn set_helix_bridge_managed_editor_cwd(
    working_dir: &Path,
) -> Result<ManagedEditorOpenStatus, CoreError> {
    match active_managed_editor_pane_target()? {
        ManagedEditorPaneTarget::Ready(zellij_pane_id) => {
            let payload = json!({
                "working_dir": working_dir.display().to_string(),
            });
            send_helix_bridge_action_to_target(
                HelixBridgeActionTarget {
                    session_id: None,
                    instance_id: None,
                    zellij_pane_id: Some(zellij_pane_id),
                },
                "helix.set_cwd",
                payload,
                5_000,
            )?;
            Ok(ManagedEditorOpenStatus::Ok)
        }
        ManagedEditorPaneTarget::Missing => Ok(ManagedEditorOpenStatus::Missing),
        ManagedEditorPaneTarget::NotReady => Ok(ManagedEditorOpenStatus::NotReady),
    }
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

    let payload = workspace_retarget_payload(&target_dir, false, parsed.editor.as_deref(), None);

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
    let editor_working_dir = if primary_target_path.is_dir() {
        primary_target_path.to_path_buf()
    } else {
        resolve_editor_working_dir(primary_target_path)
    };
    let mut created_editor_pane = false;

    if integration_facts.hide_sidebar_on_file_open {
        hide_sidebar_if_visible()?;
    }

    if editor_kind == "helix" || editor_kind == "neovim" {
        let open_status =
            if editor_kind == "helix" && target_paths.len() == 1 && primary_target_path.is_dir() {
                open_directory_in_helix_bridge_managed_editor(&editor_working_dir)?
            } else {
                open_files_in_managed_editor(&editor_kind, &target_paths, &editor_working_dir)?
            };
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
        if retarget_result.status() == "ok" {
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

    let bridge_updates_helix_cwd = editor_kind == "helix";
    let retarget_result = retarget_workspace_without_focused_cd(
        &target_dir,
        if bridge_updates_helix_cwd {
            None
        } else {
            Some(editor_kind.as_str())
        },
    )?;
    let mut created_editor_pane = false;
    let status = retarget_result.status();
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

    let editor_status = if bridge_updates_helix_cwd {
        set_helix_bridge_managed_editor_cwd(&target_dir)?
    } else {
        match retarget_result.editor_status.as_str() {
            "missing" => ManagedEditorOpenStatus::Missing,
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
            _ => ManagedEditorOpenStatus::Ok,
        }
    };

    match editor_status {
        ManagedEditorOpenStatus::Missing | ManagedEditorOpenStatus::NotReady => {
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
        ManagedEditorOpenStatus::Ok => {}
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

    let payload = open_terminal_in_cwd_payload(&target_dir);

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

    let retarget_payload = workspace_retarget_payload(&target_dir, false, None, None);

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
