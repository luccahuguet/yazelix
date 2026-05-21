//! Yazi/sidebar command adapter for workspace commands.

use super::{expand_leading_tilde, load_workspace_command_config, resolve_path_like_input};
use crate::bridge::{CoreError, ErrorClass};
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use crate::workspace_session::{SidebarState, parse_active_sidebar_state};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RevealArgs {
    target: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SidebarArgs {
    action: Option<String>,
    help: bool,
}

pub fn run_yzx_reveal(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_reveal_args(args)?;
    if parsed.help {
        print_reveal_help();
        return Ok(0);
    }

    let config = load_workspace_command_config()?;
    if env::var_os("ZELLIJ").is_none() {
        println!("Error: Reveal in Yazi only works inside a Yazelix/Zellij session.");
        return Ok(0);
    }

    if !command_is_available(&config.ya_command, &config.home_dir) {
        println!(
            "Error: The configured Yazi CLI `{}` is not available in this environment.",
            config.ya_command
        );
        return Ok(0);
    }

    let target_path = match resolve_reveal_target_path(
        parsed
            .target
            .as_deref()
            .expect("reveal target required after parse"),
        &config.home_dir,
    ) {
        Ok(path) => path,
        Err(message) => {
            println!("Error: {message}");
            return Ok(0);
        }
    };

    let Some(sidebar_state) = active_sidebar_state() else {
        println!(
            "Error: Managed sidebar Yazi is not available in the current tab. Open the sidebar and try again."
        );
        return Ok(0);
    };

    let target_path_string = target_path.to_string_lossy().to_string();
    let reveal_result = run_ya_emit_to(
        &config.ya_command,
        &config.home_dir,
        &sidebar_state.yazi_id,
        "reveal",
        &[target_path_string.as_str()],
    );
    if let Err(message) = reveal_result {
        println!("Error: Failed to execute yazi/zellij commands: {message}");
        return Ok(0);
    }

    let focus_status = focus_sidebar().unwrap_or_else(|_| "error".to_string());
    if focus_status != "ok" {
        println!(
            "Error: Managed sidebar pane focus failed (status={focus_status}). Ensure the Yazelix pane orchestrator plugin is loaded and the sidebar pane title is 'sidebar'."
        );
    }

    Ok(0)
}

pub fn run_yzx_sidebar(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_sidebar_args(args)?;
    if parsed.help {
        print_sidebar_help();
        return Ok(0);
    }

    match parsed.action.as_deref() {
        Some("yazi") => launch_yazi_sidebar(),
        Some("refresh") => {
            refresh_managed_yazi_sidebar()?;
            Ok(0)
        }
        Some(action) => Err(CoreError::classified(
            ErrorClass::Usage,
            "unknown_sidebar_action",
            format!("Unknown yzx sidebar action: {action}."),
            "Use `yzx sidebar yazi` or `yzx sidebar refresh`.",
            json!({ "action": action }),
        )),
        None => {
            print_sidebar_help();
            Ok(0)
        }
    }
}

fn launch_yazi_sidebar() -> Result<i32, CoreError> {
    let config = load_workspace_command_config()?;
    if !command_is_available(&config.yazi_command, &config.home_dir) {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_yazi_sidebar_command",
            format!(
                "The configured Yazi command `{}` is not available in this environment.",
                config.yazi_command
            ),
            "Install Yazi, fix yazi.command in settings.jsonc, or restart Yazelix so the runtime PATH is active.",
            json!({ "command": config.yazi_command }),
        ));
    }

    let target_dir = consume_bootstrap_sidebar_cwd(&config.home_dir)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| config.home_dir.clone());
    let command_path = resolve_command_path(&config.yazi_command, &config.home_dir);
    let status = Command::new(&command_path)
        .arg(target_dir)
        .status()
        .map_err(|source| {
            CoreError::io(
                "launch_yazi_sidebar",
                "Could not launch the configured Yazi sidebar command",
                "Check yazi.command and the Yazelix runtime PATH, then retry.",
                command_path,
                source,
            )
        })?;
    Ok(status.code().unwrap_or(1))
}

fn consume_bootstrap_sidebar_cwd(home_dir: &Path) -> Option<PathBuf> {
    let cwd_file = env::var("YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())?;
    let cwd_file = PathBuf::from(cwd_file);
    if !cwd_file.is_file() {
        return None;
    }

    let requested = fs::read_to_string(&cwd_file).ok()?;
    let _ = fs::remove_file(&cwd_file);
    let requested = requested.trim();
    if requested.is_empty() {
        return None;
    }

    let expanded = expand_leading_tilde(requested, home_dir);
    let path = PathBuf::from(expanded);
    if !path.exists() {
        return None;
    }
    if path.is_dir() {
        Some(path)
    } else {
        path.parent().map(Path::to_path_buf)
    }
}

fn refresh_managed_yazi_sidebar() -> Result<(), CoreError> {
    let config = load_workspace_command_config()?;
    let Some(sidebar_state) = active_sidebar_state() else {
        return Ok(());
    };
    if !command_is_available(&config.ya_command, &config.home_dir) {
        return Ok(());
    }

    run_ya_emit_to(
        &config.ya_command,
        &config.home_dir,
        &sidebar_state.yazi_id,
        "refresh",
        &[],
    )
    .map_err(|reason| sidebar_refresh_error("sidebar_refresh", &reason))?;
    run_ya_emit_to(
        &config.ya_command,
        &config.home_dir,
        &sidebar_state.yazi_id,
        "plugin",
        &["git", "refresh-sidebar"],
    )
    .map_err(|reason| sidebar_refresh_error("sidebar_git_refresh", &reason))?;

    let sidebar_cwd = sidebar_state.cwd.trim();
    if !sidebar_cwd.is_empty() {
        run_ya_emit_to(
            &config.ya_command,
            &config.home_dir,
            &sidebar_state.yazi_id,
            "plugin",
            &["starship", sidebar_cwd],
        )
        .map_err(|reason| sidebar_refresh_error("sidebar_starship_refresh", &reason))?;
    }

    Ok(())
}

fn sidebar_refresh_error(code: &'static str, reason: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        code,
        format!("Failed to refresh the managed Yazi sidebar: {reason}"),
        "Ensure the managed sidebar Yazi pane is still available, then retry.",
        json!({ "reason": reason }),
    )
}

fn parse_reveal_args(args: &[String]) -> Result<RevealArgs, CoreError> {
    let mut parsed = RevealArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx reveal: {other}. Try `yzx reveal --help`."
                )));
            }
            other => {
                if parsed.target.is_some() {
                    return Err(CoreError::usage(
                        "yzx reveal requires exactly one file or directory target.",
                    ));
                }
                parsed.target = Some(other.to_string());
            }
        }
    }

    if !parsed.help && parsed.target.is_none() {
        return Err(CoreError::usage(
            "yzx reveal requires a file or directory target. Try `yzx reveal --help`.",
        ));
    }

    Ok(parsed)
}

fn print_reveal_help() {
    println!("Reveal a file or directory in the managed Yazi sidebar");
    println!();
    println!("Usage:");
    println!("  yzx reveal <target>");
}

fn parse_sidebar_args(args: &[String]) -> Result<SidebarArgs, CoreError> {
    let mut parsed = SidebarArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            value if parsed.action.is_none() => parsed.action = Some(value.to_string()),
            other => {
                return Err(CoreError::classified(
                    ErrorClass::Usage,
                    "unknown_sidebar_argument",
                    format!("Unknown argument for yzx sidebar: {other}."),
                    "Use `yzx sidebar yazi` or `yzx sidebar refresh`.",
                    json!({ "argument": other }),
                ));
            }
        }
    }
    Ok(parsed)
}

fn print_sidebar_help() {
    println!("Manage the Yazelix sidebar");
    println!();
    println!("Usage:");
    println!("  yzx sidebar yazi");
    println!("  yzx sidebar refresh");
}

fn resolve_reveal_target_path(target: &str, home_dir: &Path) -> Result<PathBuf, String> {
    let current_dir =
        env::current_dir().map_err(|err| format!("Could not read the current directory: {err}"))?;
    let full_path = resolve_path_like_input(target, &current_dir, home_dir);

    if !full_path.exists() {
        return Err(format!(
            "Resolved path '{}' does not exist.",
            full_path.display()
        ));
    }

    Ok(full_path)
}

fn active_sidebar_state() -> Option<SidebarState> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "").ok()?;
    parse_active_sidebar_state(&response)
}

fn focus_sidebar() -> Result<String, CoreError> {
    let response = run_pane_orchestrator_command("focus_sidebar", "")?;
    Ok(match response.trim() {
        "ok" | "opened" | "focused" | "focused_sidebar" | "opened_sidebar" => "ok".to_string(),
        other => other.to_string(),
    })
}

pub(crate) fn sync_sidebar_to_directory(
    ya_command: &str,
    home_dir: &Path,
    sidebar_state: &SidebarState,
    target_dir: &Path,
) -> String {
    if !command_is_available(ya_command, home_dir) {
        return "skipped".to_string();
    }
    let target = if target_dir.is_dir() {
        target_dir
    } else {
        target_dir.parent().unwrap_or(target_dir)
    };
    let target_string = target.to_string_lossy().to_string();
    match run_ya_emit_to(
        ya_command,
        home_dir,
        &sidebar_state.yazi_id,
        "cd",
        &[target_string.as_str()],
    ) {
        Ok(()) => "ok".to_string(),
        Err(_) => "error".to_string(),
    }
}

fn run_ya_emit_to(
    ya_command: &str,
    home_dir: &Path,
    yazi_id: &str,
    action: &str,
    args: &[&str],
) -> Result<(), String> {
    let command_path = resolve_command_path(ya_command, home_dir);
    let output = Command::new(&command_path)
        .arg("emit-to")
        .arg(yazi_id)
        .arg(action)
        .args(args)
        .output()
        .map_err(|err| err.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stderr.is_empty() {
        Err(stderr)
    } else if !stdout.is_empty() {
        Err(stdout)
    } else {
        Err(format!("exit code {}", output.status.code().unwrap_or(1)))
    }
}

pub(super) fn command_is_available(command: &str, home_dir: &Path) -> bool {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.contains('/') || trimmed.starts_with('~') {
        return PathBuf::from(expand_leading_tilde(trimmed, home_dir)).exists();
    }

    find_external_command(trimmed).is_some()
}

fn resolve_command_path(command: &str, home_dir: &Path) -> String {
    let trimmed = command.trim();
    if trimmed.contains('/') || trimmed.starts_with('~') {
        expand_leading_tilde(trimmed, home_dir)
    } else {
        trimmed.to_string()
    }
}

fn find_external_command(command_name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|entry| entry.join(command_name))
        .find(|candidate| candidate.is_file())
}
