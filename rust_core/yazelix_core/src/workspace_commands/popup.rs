//! yzpp-backed workspace popup command adapter.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::runtime_dir_from_env;
use crate::pane_orchestrator_client::{
    YZPP_PLUGIN_ALIAS, run_pane_orchestrator_command, run_zellij_plugin_command,
};
use crate::popup_session_facts::compute_popup_session_facts_from_env;
use crate::workspace_session::current_tab_workspace_root_from_json;
use serde_json::{Value, json};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PopupArgs {
    program: Vec<String>,
    help: bool,
}

pub fn run_yzx_popup(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_popup_args(args)?;
    if parsed.help {
        print_popup_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "popup_outside_zellij",
            "yzx popup only works inside Zellij. Start Yazelix first, then run it from the tab where you want the popup.",
            "Run this command from inside an active Yazelix/Zellij session.",
            json!({}),
        ));
    }

    if parsed.program.is_empty() {
        let response = run_zellij_plugin_command(YZPP_PLUGIN_ALIAS, "toggle", "popup")?;
        if matches!(response.trim(), "opened" | "focused" | "closed") {
            return Ok(0);
        }

        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "popup_toggle_failed",
            format!("Failed to toggle the Yazelix popup pane: {response}"),
            "Ensure the yzpp plugin is loaded and the current tab is ready, then retry.",
            json!({ "response": response }),
        ));
    }

    let runtime_dir = runtime_dir_from_env()?;
    let runtime_env = current_process_runtime_env();
    let popup_program = resolve_popup_runtime_argv(&parsed.program, &runtime_env)?;
    let popup_facts = compute_popup_session_facts_from_env()?;
    let popup_cwd = current_tab_workspace_root(true).unwrap_or_else(|| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string()
    });
    let yzx_cli = runtime_dir.join("shells").join("posix").join("yzx_cli.sh");

    let payload = json!({
        "action": "open",
        "spec": {
            "id": "popup",
            "pane_title": "yzx_popup",
            "command_marker": popup_program[0],
            "command": popup_program,
            "cwd": popup_cwd,
            "width_percent": popup_facts.popup_width_percent,
            "height_percent": popup_facts.popup_height_percent,
            "on_close": {
                "command": [
                    yzx_cli.to_string_lossy().to_string(),
                    "sidebar",
                    "refresh"
                ]
            }
        },
        "args": [],
    })
    .to_string();

    let response = run_zellij_plugin_command(YZPP_PLUGIN_ALIAS, "transient_popup", &payload)?;
    if matches!(response.trim(), "ok" | "opened" | "focused") {
        return Ok(0);
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "popup_open_failed",
        format!("Failed to open the Yazelix popup pane: {response}"),
        "Ensure the yzpp plugin is loaded and the current tab is ready, then retry.",
        json!({ "response": response }),
    ))
}

fn current_process_runtime_env() -> serde_json::Map<String, serde_json::Value> {
    [
        "PATH",
        "EDITOR",
        "VISUAL",
        "YAZELIX_RUNTIME_DIR",
        "YAZELIX_SESSION_CONFIG_PATH",
        "YAZELIX_SESSION_FACTS_PATH",
        "IN_YAZELIX_SHELL",
        "ZELLIJ_DEFAULT_LAYOUT",
        "YAZI_CONFIG_HOME",
        "YAZELIX_MANAGED_HELIX_BINARY",
        "HELIX_RUNTIME",
    ]
    .into_iter()
    .filter_map(|key| {
        env::var(key)
            .ok()
            .map(|value| (key.to_string(), Value::String(value)))
    })
    .collect()
}

fn resolve_popup_runtime_argv(
    popup_program: &[String],
    runtime_env: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<String>, CoreError> {
    if popup_program.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "popup_program_empty",
            "No popup program was configured for Yazelix.",
            "Set zellij.popup_program in settings.jsonc or pass an explicit program to `yzx popup`.",
            json!({}),
        ));
    }

    let command = popup_program[0].trim();
    let tail = popup_program[1..].to_vec();
    if command.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "popup_command_empty",
            "Popup program command cannot be empty.",
            "Set popup_program to a real executable or pass an explicit program to `yzx popup`.",
            json!({}),
        ));
    }

    let resolved_command = if command == "editor" {
        runtime_env
            .get("EDITOR")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Config,
                    "popup_editor_unresolved",
                    "The configured Yazelix editor could not be resolved for popup_program = [\"editor\"].",
                    "Set editor.command in settings.jsonc or set EDITOR inside the Yazelix runtime.",
                    json!({}),
                )
            })?
            .to_string()
    } else {
        command.to_string()
    };

    Ok(std::iter::once(resolved_command).chain(tail).collect())
}

fn current_tab_workspace_root(include_bootstrap: bool) -> Option<String> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "").ok()?;
    current_tab_workspace_root_from_json(&response, include_bootstrap)
}

fn parse_popup_args(args: &[String]) -> Result<PopupArgs, CoreError> {
    let mut parsed = PopupArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx popup: {other}. Try `yzx popup --help`."
                )));
            }
            other => parsed.program.push(other.to_string()),
        }
    }
    Ok(parsed)
}

fn print_popup_help() {
    println!("Open or toggle the configured Yazelix popup program in Zellij");
    println!();
    println!("Usage:");
    println!("  yzx popup [program...]");
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use serde_json::Value;

    // Regression: popup pane execution resolves the editor alias from the Rust-owned runtime env instead of reviving a Nu popup wrapper.
    #[test]
    fn popup_runtime_argv_resolves_editor_alias_from_runtime_env() {
        let runtime_env = serde_json::Map::from_iter([(
            "EDITOR".to_string(),
            Value::String("/tmp/yazelix_hx.sh".to_string()),
        )]);

        let argv = resolve_popup_runtime_argv(
            &["editor".to_string(), "README.md".to_string()],
            &runtime_env,
        )
        .expect("popup argv");

        assert_eq!(
            argv,
            vec!["/tmp/yazelix_hx.sh".to_string(), "README.md".to_string()]
        );
    }
}
