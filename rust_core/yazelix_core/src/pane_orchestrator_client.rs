//! Shared pane-orchestrator pipe transport for Yazelix command adapters.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::env;
use std::ffi::OsString;
use std::process::Command;

pub(crate) const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";
pub(crate) const YZPP_PLUGIN_ALIAS: &str = "yzpp";
const YAZELIX_ZELLIJ_SESSION_NAME_ENV: &str = "YAZELIX_ZELLIJ_SESSION_NAME";
const ZELLIJ_SESSION_NAME_ENV: &str = "ZELLIJ_SESSION_NAME";

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
    let mut command = Command::new("zellij");
    configure_zellij_control_session_env(&mut command);
    let output = command
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

pub(crate) fn configure_zellij_control_session_env(command: &mut Command) {
    if let Some(session_name) = zellij_session_name_override_for_control_command(
        env::var_os(ZELLIJ_SESSION_NAME_ENV),
        env::var_os(YAZELIX_ZELLIJ_SESSION_NAME_ENV),
    ) {
        command.env(ZELLIJ_SESSION_NAME_ENV, session_name);
    }
}

fn zellij_session_name_override_for_control_command(
    current_session_name: Option<OsString>,
    saved_session_name: Option<OsString>,
) -> Option<OsString> {
    if current_session_name.is_some_and(|value| !value.is_empty()) {
        return None;
    }

    saved_session_name.filter(|value| !value.is_empty())
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Regression: managed Yazi blanks ZELLIJ_SESSION_NAME for graphics detection but saves the real session for Yazelix control subprocesses.
    #[test]
    fn zellij_control_restores_saved_session_when_current_session_is_empty() {
        assert_eq!(
            zellij_session_name_override_for_control_command(
                Some(OsString::from("")),
                Some(OsString::from("real-session")),
            ),
            Some(OsString::from("real-session"))
        );
        assert_eq!(
            zellij_session_name_override_for_control_command(
                None,
                Some(OsString::from("real-session")),
            ),
            Some(OsString::from("real-session"))
        );
    }

    // Invariant: callers already attached to a concrete Zellij session keep their native session env unchanged.
    #[test]
    fn zellij_control_keeps_existing_nonempty_session() {
        assert_eq!(
            zellij_session_name_override_for_control_command(
                Some(OsString::from("current-session")),
                Some(OsString::from("saved-session")),
            ),
            None
        );
        assert_eq!(
            zellij_session_name_override_for_control_command(
                Some(OsString::from("")),
                Some(OsString::from("")),
            ),
            None
        );
    }
}
