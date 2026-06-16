//! Zellij pipe diagnostic commands for `yzx_control`.

use crate::bridge::{CoreError, ErrorClass};
use crate::pane_orchestrator_client::{
    configure_zellij_control_session_env, run_pane_orchestrator_command,
};
use crate::workspace_session::current_tab_workspace_root_from_json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

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
    println!("  yzx_control zellij pipe toggle_sidebar");
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

#[derive(Debug, Deserialize)]
struct ZellijPaneListEntry {
    id: u32,
    #[serde(default)]
    is_plugin: bool,
    #[serde(default)]
    is_focused: bool,
    #[serde(default)]
    exited: bool,
    #[serde(default)]
    title: String,
    tab_id: Option<usize>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct TerminalTitleActivityObservation {
    tab_id: usize,
    pane_id: u32,
    title: String,
    is_focused: bool,
}

fn terminal_title_activity_observations_from_pane_list(
    panes: &[ZellijPaneListEntry],
) -> Vec<TerminalTitleActivityObservation> {
    panes
        .iter()
        .filter(|pane| !pane.is_plugin && !pane.exited)
        .filter_map(|pane| {
            Some(TerminalTitleActivityObservation {
                tab_id: pane.tab_id?,
                pane_id: pane.id,
                title: pane.title.clone(),
                is_focused: pane.is_focused,
            })
        })
        .collect()
}

pub fn run_zellij_refresh_terminal_title_activity(args: &[String]) -> Result<i32, CoreError> {
    if !args.is_empty() {
        return Err(CoreError::usage(
            "zellij refresh-terminal-title-activity accepts no arguments".to_string(),
        ));
    }

    let mut command = Command::new("zellij");
    configure_zellij_control_session_env(&mut command);
    let output = command
        .args(["action", "list-panes", "--json", "--tab", "--state"])
        .output()
        .map_err(|source| {
            CoreError::io(
                "zellij_list_panes_failed",
                "Failed to query Zellij pane titles.",
                "Run this command inside an active Yazelix/Zellij session, then retry.",
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
            "zellij_list_panes_failed",
            format!("Failed to query Zellij pane titles: {details}"),
            "Run this command inside an active Yazelix/Zellij session, then retry.",
            json!({ "command": "zellij action list-panes --json --tab --state" }),
        ));
    }

    let panes: Vec<ZellijPaneListEntry> =
        serde_json::from_slice(&output.stdout).map_err(|source| {
            CoreError::classified(
                ErrorClass::Runtime,
                "zellij_list_panes_invalid_json",
                format!("Zellij returned invalid pane-list JSON: {source}"),
                "Upgrade or restart Zellij, then retry the Yazelix pane activity refresh.",
                json!({ "command": "zellij action list-panes --json --tab --state" }),
            )
        })?;
    let payload = serde_json::to_string(&terminal_title_activity_observations_from_pane_list(
        panes.as_slice(),
    ))
    .map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "terminal_title_activity_payload_serialize_failed",
            format!("Failed to serialize terminal-title activity observations: {source}"),
            "Retry the Yazelix pane activity refresh.",
            json!({}),
        )
    })?;

    let response =
        run_pane_orchestrator_command("reconcile_terminal_title_activity_snapshot", &payload)?;
    if response.trim() != "ok" {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "terminal_title_activity_snapshot_rejected",
            format!("Pane orchestrator rejected terminal-title activity snapshot: {response}"),
            "Run this command inside an active Yazelix/Zellij session with the pane orchestrator loaded, then retry.",
            json!({ "response": response }),
        ));
    }

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

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;

    // Defends: the async terminal-title producer strips Zellij's large pane payload down to stable terminal observations.
    #[test]
    fn reduces_zellij_pane_list_to_terminal_title_activity_observations() {
        let panes = vec![
            ZellijPaneListEntry {
                id: 3,
                is_plugin: true,
                is_focused: false,
                exited: false,
                title: "plugin".to_string(),
                tab_id: Some(0),
            },
            ZellijPaneListEntry {
                id: 7,
                is_plugin: false,
                is_focused: true,
                exited: false,
                title: "codex thinking".to_string(),
                tab_id: Some(2),
            },
            ZellijPaneListEntry {
                id: 8,
                is_plugin: false,
                is_focused: false,
                exited: true,
                title: "old".to_string(),
                tab_id: Some(2),
            },
        ];

        assert_eq!(
            terminal_title_activity_observations_from_pane_list(&panes),
            vec![TerminalTitleActivityObservation {
                tab_id: 2,
                pane_id: 7,
                title: "codex thinking".to_string(),
                is_focused: true,
            }]
        );
    }
}
