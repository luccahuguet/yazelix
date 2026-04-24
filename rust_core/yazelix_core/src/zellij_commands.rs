// Test lane: default
//! Zellij integration commands for `yzx_control`.
//!
//! These are thin wrappers around `zellij action pipe --plugin yazelix_pane_orchestrator`
//! that replace the Nushell `zellij.nu` pipe functions.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";

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
                    iter
                        .next()
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
        CoreError::usage("zellij pipe requires a command name. Try `yzx_control zellij pipe --help`.".to_string())
    })?;

    let payload = parsed.payload.as_deref().unwrap_or("");
    let response = run_pane_orchestrator_command(&command, payload)?;
    println!("{}", response);
    Ok(0)
}

fn parse_zellij_get_workspace_root_args(args: &[String]) -> Result<ZellijGetWorkspaceRootArgs, CoreError> {
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
                    iter
                        .next()
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
    println!("This is the internal variant of `yzx cwd` that does not cd the focused pane.");
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
        _ => {
            match serde_json::from_str::<serde_json::Value>(trimmed) {
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
            }
        }
    }
}

pub fn run_zellij_retarget(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_zellij_retarget_args(args)?;
    if parsed.help {
        print_zellij_retarget_help();
        return Ok(0);
    }

    let target = parsed.target.ok_or_else(|| {
        CoreError::usage("zellij retarget requires a target path. Try `yzx_control zellij retarget --help`.".to_string())
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

    let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("error");
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
            eprintln!("   Wait a moment for the pane orchestrator plugin to finish loading, then try again.");
            Ok(1)
        }
        "permissions_denied" => {
            eprintln!("❌ The Yazelix pane orchestrator plugin is missing required Zellij permissions.");
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
            format!("Pane orchestrator failed to open directory pane in '{}': {}", target_dir.display(), response),
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
            eprintln!("⚠️  Terminal pane opened, but workspace retarget failed: {}", retarget_response);
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
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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
        assert_eq!(sidebar.get("yazi_id").and_then(|v| v.as_str()), Some("yazi-123"));
        assert_eq!(sidebar.get("cwd").and_then(|v| v.as_str()), Some("/home/sidebar"));
    }

    // Defends: retarget response parsing handles simple error strings.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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
}
