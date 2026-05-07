//! Shared pane-orchestrator pipe transport for Yazelix command adapters.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::process::Command;

pub(crate) const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";
pub(crate) const YZPP_PLUGIN_ALIAS: &str = "yzpp";

pub(crate) fn run_pane_orchestrator_command(
    command_name: &str,
    payload: &str,
) -> Result<String, CoreError> {
    run_zellij_plugin_command_with_error(
        PANE_ORCHESTRATOR_PLUGIN_ALIAS,
        command_name,
        payload,
        "pane_orchestrator_pipe_failed",
        "Yazelix pane-orchestrator",
        "Run this command inside an active Yazelix/Zellij session with the pane orchestrator loaded, then retry.",
    )
}

pub(crate) fn run_zellij_plugin_command(
    plugin_alias: &str,
    command_name: &str,
    payload: &str,
) -> Result<String, CoreError> {
    run_zellij_plugin_command_with_error(
        plugin_alias,
        command_name,
        payload,
        "zellij_plugin_pipe_failed",
        "Zellij plugin",
        "Run this command inside an active Yazelix/Zellij session with the required plugin loaded, then retry.",
    )
}

fn run_zellij_plugin_command_with_error(
    plugin_alias: &str,
    command_name: &str,
    payload: &str,
    error_code: &'static str,
    command_label: &'static str,
    recovery: &'static str,
) -> Result<String, CoreError> {
    let output = Command::new("zellij")
        .args([
            "action",
            "pipe",
            "--plugin",
            plugin_alias,
            "--name",
            command_name,
            "--",
            payload,
        ])
        .output()
        .map_err(|source| {
            CoreError::io(
                error_code,
                format!("Failed to run the {command_label} command `{command_name}`."),
                recovery,
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
            error_code,
            format!("{command_label} pipe failed for `{plugin_alias}` `{command_name}`: {details}"),
            recovery,
            json!({ "plugin": plugin_alias, "command": command_name }),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
