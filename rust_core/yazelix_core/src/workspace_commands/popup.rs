//! yzpp-backed workspace popup command adapter.

use crate::bridge::{CoreError, ErrorClass};
use crate::compute_runtime_env;
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, load_normalized_config_for_control,
    run_child_in_runtime_env, runtime_dir_from_env, runtime_env_request,
};
use crate::pane_orchestrator_client::{
    YZPP_PLUGIN_ALIAS, run_pane_orchestrator_command, run_zellij_plugin_command,
};
use crate::popup_runtime_command::popup_command_argv_for_yazelix_runtime;
use crate::popup_session_facts::compute_popup_session_facts_from_env;
use crate::workspace_session::{
    current_tab_workspace_root_from_json, sidebar_focused_cwd_from_json,
};
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
        return Err(CoreError::usage(
            "yzx popup expects an explicit program. Use `yzx popup <program> [args...]`, or configure a persistent popup through zellij.custom_popups.",
        ));
    }

    let runtime_dir = runtime_dir_from_env()?;
    let runtime_env = current_process_runtime_env();
    let popup_argv = resolve_popup_runtime_argv(&parsed.program, &runtime_env)?;
    let popup_facts = compute_popup_session_facts_from_env()?;
    let popup_cwd = current_tab_workspace_root(true).unwrap_or_else(|| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string()
    });
    let yzx_cli = runtime_dir.join("shells").join("posix").join("yzx_cli.sh");
    let yzx_cli_text = yzx_cli.to_string_lossy().to_string();
    let command_marker = popup_argv[0].clone();
    let popup_command = popup_command_argv_for_yazelix_runtime(&popup_argv, &yzx_cli_text);

    let payload = json!({
        "action": "open",
        "spec": {
            "id": "popup",
            "pane_title": "yzx_popup",
            "command_marker": command_marker,
            "command": popup_command,
            "cwd": popup_cwd,
            "width_percent": popup_facts.popup_width_percent,
            "height_percent": popup_facts.popup_height_percent,
            "on_close": {
                "command": [
                    yzx_cli_text,
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

pub fn run_yzx_popup_run(args: &[String]) -> Result<i32, CoreError> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h" | "help") {
        print_popup_run_help();
        return Ok(0);
    }
    if args.is_empty() {
        return Err(CoreError::usage(
            "yzx popup_run expects a command to run inside a Yazelix popup pane.",
        ));
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let normalized = load_normalized_config_for_control(
        &runtime_dir,
        &config_dir,
        config_override_from_env().as_deref(),
    )?;
    let req = runtime_env_request(runtime_dir, &normalized)?;
    let data = compute_runtime_env(&req)?;
    let cwd = popup_run_cwd().unwrap_or_else(|| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string()
    });
    let status = run_child_in_runtime_env(args, &data.runtime_env, &PathBuf::from(cwd))?;
    Ok(status.code().unwrap_or(1))
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
    popup_argv: &[String],
    runtime_env: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<String>, CoreError> {
    if popup_argv.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "popup_command_missing",
            "No popup command was provided.",
            "Pass an explicit command to `yzx popup`, or configure a persistent popup through zellij.custom_popups.",
            json!({}),
        ));
    }

    let command = popup_argv[0].trim();
    let tail = popup_argv[1..].to_vec();
    if command.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "popup_command_empty",
            "Popup program command cannot be empty.",
            "Pass a real executable to `yzx popup`, or configure a persistent popup through zellij.custom_popups.",
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
                    "The configured Yazelix editor could not be resolved for `yzx popup editor`.",
                    "Set editor.command in config.toml or set EDITOR inside the Yazelix runtime.",
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

fn popup_run_cwd() -> Option<String> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "").ok()?;
    sidebar_focused_cwd_from_json(&response)
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
    println!("Open an explicit command in a transient Yazelix popup pane");
    println!();
    println!("Usage:");
    println!("  yzx popup <program> [args...]");
}

fn print_popup_run_help() {
    println!("Run an internal Yazelix popup command with context-aware cwd");
    println!();
    println!("Usage:");
    println!("  yzx popup_run <program> [args...]");
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
