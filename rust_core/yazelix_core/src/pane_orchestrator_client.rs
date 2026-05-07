//! Shared pane-orchestrator pipe transport for Yazelix command adapters.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::process::Command;

pub(crate) const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";

pub(crate) fn run_pane_orchestrator_command(
    command_name: &str,
    payload: &str,
) -> Result<String, CoreError> {
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
                format!("Failed to run the Yazelix pane-orchestrator command `{command_name}`."),
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
