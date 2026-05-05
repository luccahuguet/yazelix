// Test lane: default
//! Public workspace command owners for `yzx_control`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{home_dir_from_env, run_child_in_runtime_env, runtime_dir_from_env};
use crate::session_facts::compute_session_facts_from_env;
use crate::transient_pane_facts::compute_transient_pane_facts_from_env;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CwdArgs {
    target: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct WarpArgs {
    target: Option<String>,
    kill_old_tab: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RevealArgs {
    target: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PopupArgs {
    program: Vec<String>,
    help: bool,
    refresh_sidebar_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceCommandConfig {
    pub(crate) enable_sidebar: bool,
    pub(crate) hide_sidebar_on_file_open: bool,
    pub(crate) editor_kind: String,
    pub(crate) yazi_command: String,
    pub(crate) ya_command: String,
    pub(crate) home_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntegrationFactsData {
    pub enable_sidebar: bool,
    pub hide_sidebar_on_file_open: bool,
    pub managed_editor_kind: String,
    pub yazi_command: String,
    pub ya_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SidebarState {
    pub(crate) yazi_id: String,
    pub(crate) cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkspaceRetargetResult {
    status: String,
    editor_status: String,
    sidebar_state: Option<SidebarState>,
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceRetargetResponse {
    status: String,
    #[serde(default)]
    editor_status: String,
    #[serde(default)]
    sidebar_yazi_id: Option<String>,
    #[serde(default)]
    sidebar_yazi_cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActiveTabSessionStateV1 {
    #[serde(default)]
    sidebar_yazi: Option<SessionSidebarYazi>,
}

#[derive(Debug, Deserialize)]
struct SessionSidebarYazi {
    yazi_id: String,
    cwd: String,
}

pub fn run_yzx_warp(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_warp_args(args)?;
    if parsed.help {
        print_warp_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        println!("❌ yzx warp only works inside Zellij.");
        println!("   Start Yazelix first, then run this command from the tab you want to leave.");
        return Ok(1);
    }

    let old_tab_id = if parsed.kill_old_tab {
        Some(current_zellij_tab_id()?)
    } else {
        None
    };
    let home_dir = home_dir_from_env()?;
    let resolved_target = match resolve_warp_target(parsed.target.as_deref(), &home_dir) {
        Ok(WarpTargetResolution::Target(path)) => path,
        Ok(WarpTargetResolution::Cancelled) => return Ok(0),
        Err(message) => {
            println!("❌ {message}");
            return Ok(1);
        }
    };
    let target_dir = resolve_existing_target_dir(&resolved_target)?;
    let tab_name = workspace_tab_name(&target_dir);

    open_zellij_workspace_tab(&target_dir, &tab_name)?;
    if let Some(tab_id) = old_tab_id {
        close_zellij_tab_by_id(tab_id)?;
    }

    println!("✅ Opened Yazelix workspace tab: {}", target_dir.display());
    println!("   Tab named: {tab_name}");
    if old_tab_id.is_some() {
        println!("   Previous tab closed.");
    }
    Ok(0)
}

pub fn run_yzx_cwd(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_cwd_args(args)?;
    if parsed.help {
        print_cwd_help();
        return Ok(0);
    }

    if env::var_os("ZELLIJ").is_none() {
        println!("❌ yzx cwd only works inside Zellij.");
        println!("   Start Yazelix first, then run this command from the tab you want to update.");
        return Ok(1);
    }

    let config = load_workspace_command_config()?;
    let resolved_target = match resolve_cwd_target(parsed.target.as_deref(), &config.home_dir) {
        Ok(path) => path,
        Err(message) => {
            println!("❌ {message}");
            return Ok(1);
        }
    };
    let target_dir = resolve_existing_target_dir(&resolved_target)?;
    let tab_name = workspace_tab_name(&target_dir);
    let result = match retarget_workspace(&target_dir, &config.editor_kind) {
        Ok(result) => result,
        Err(err) => WorkspaceRetargetResult {
            status: "error".to_string(),
            editor_status: String::new(),
            sidebar_state: None,
            reason: Some(err.message()),
        },
    };

    match result.status.as_str() {
        "ok" => {
            let sidebar_sync_status = if let Some(sidebar_state) = result.sidebar_state.as_ref() {
                sync_sidebar_to_directory(
                    &config.ya_command,
                    &config.home_dir,
                    sidebar_state,
                    &target_dir,
                )
            } else {
                "skipped".to_string()
            };

            println!(
                "✅ Updated current tab workspace directory to: {}",
                target_dir.display()
            );
            println!("   Tab renamed to: {tab_name}");
            println!("   The current pane will switch after this command returns.");
            println!("   Other existing panes keep their current working directories.");
            println!("   New managed actions will use the updated tab directory.");
            if result.editor_status == "ok" {
                println!("   Managed editor cwd synced to the updated directory.");
            }
            if sidebar_sync_status == "ok" {
                println!("   Sidebar Yazi synced to the updated directory.");
            }
            Ok(0)
        }
        "not_ready" => {
            println!("❌ Yazelix tab state is not ready yet.");
            println!(
                "   Wait a moment for the pane orchestrator plugin to finish loading, then try again."
            );
            Ok(1)
        }
        "permissions_denied" => {
            println!(
                "❌ The Yazelix pane orchestrator plugin is missing required Zellij permissions."
            );
            println!("   Run `yzx doctor --fix`, then restart Yazelix.");
            Ok(1)
        }
        _ => {
            let reason = result.reason.as_deref().unwrap_or("unknown error");
            println!("❌ Failed to update the current tab workspace directory: {reason}");
            Ok(1)
        }
    }
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

    if parsed.refresh_sidebar_only {
        refresh_sidebar_after_popup().ok();
        return Ok(0);
    }

    if popup_mode_active() {
        return run_popup_program_in_current_pane(parsed.program);
    }

    if parsed.program.is_empty() {
        let response = run_pane_orchestrator_command("toggle_transient_pane", "popup")?;
        if matches!(response.trim(), "opened" | "focused" | "closed") {
            return Ok(0);
        }

        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "popup_toggle_failed",
            format!("Failed to toggle the Yazelix popup pane: {response}"),
            "Ensure the pane orchestrator plugin is loaded and the current tab is ready, then retry.",
            json!({ "response": response }),
        ));
    }

    let runtime_dir = runtime_dir_from_env()?;
    let popup_program = parsed.program;
    let popup_cwd = current_tab_workspace_root(true).unwrap_or_else(|| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string()
    });

    let payload = json!({
        "kind": "popup",
        "args": popup_program,
        "cwd": popup_cwd,
        "runtime_dir": runtime_dir.to_string_lossy().to_string(),
    })
    .to_string();

    let response = run_pane_orchestrator_command("open_transient_pane", &payload)?;
    if matches!(response.trim(), "ok" | "opened" | "focused") {
        return Ok(0);
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "popup_open_failed",
        format!("Failed to open the Yazelix popup pane: {response}"),
        "Ensure the pane orchestrator plugin is loaded and the current tab is ready, then retry.",
        json!({ "response": response }),
    ))
}

fn popup_mode_active() -> bool {
    matches!(env::var("YAZELIX_POPUP_PANE").as_deref(), Ok("true"))
}

fn run_popup_program_in_current_pane(program_override: Vec<String>) -> Result<i32, CoreError> {
    rename_current_pane("yzx_popup");

    let runtime_env = current_process_runtime_env();
    let popup_program = if program_override.is_empty() {
        compute_transient_pane_facts_from_env()?.popup_program
    } else {
        program_override
    };
    let popup_argv = resolve_popup_runtime_argv(&popup_program, &runtime_env)?;
    let cwd = env::current_dir().map_err(|source| {
        CoreError::io(
            "popup_current_dir",
            "Could not read the current popup working directory.",
            "Reopen the popup from a valid directory, then retry.",
            ".",
            source,
        )
    })?;
    let status = run_child_in_runtime_env(&popup_argv, &runtime_env, &cwd)?;
    if status.success() {
        refresh_sidebar_after_popup().ok();
        close_current_pane();
    }
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

fn refresh_sidebar_after_popup() -> Result<(), CoreError> {
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
    .map_err(|reason| popup_refresh_error("popup_sidebar_refresh", &reason))?;
    run_ya_emit_to(
        &config.ya_command,
        &config.home_dir,
        &sidebar_state.yazi_id,
        "plugin",
        &["git", "refresh-sidebar"],
    )
    .map_err(|reason| popup_refresh_error("popup_sidebar_git_refresh", &reason))?;

    let sidebar_cwd = sidebar_state.cwd.trim();
    if !sidebar_cwd.is_empty() {
        run_ya_emit_to(
            &config.ya_command,
            &config.home_dir,
            &sidebar_state.yazi_id,
            "plugin",
            &["starship", sidebar_cwd],
        )
        .map_err(|reason| popup_refresh_error("popup_sidebar_starship_refresh", &reason))?;
    }

    Ok(())
}

fn popup_refresh_error(code: &'static str, reason: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        code,
        format!("Failed to refresh the managed Yazi sidebar after popup exit: {reason}"),
        "Ensure the managed sidebar Yazi pane is still available, then retry.",
        json!({ "reason": reason }),
    )
}

fn parse_cwd_args(args: &[String]) -> Result<CwdArgs, CoreError> {
    let mut parsed = CwdArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx cwd: {other}. Try `yzx cwd --help`."
                )));
            }
            other => {
                if parsed.target.is_some() {
                    return Err(CoreError::usage(
                        "yzx cwd accepts at most one optional target argument.",
                    ));
                }
                parsed.target = Some(other.to_string());
            }
        }
    }
    Ok(parsed)
}

fn parse_warp_args(args: &[String]) -> Result<WarpArgs, CoreError> {
    let mut parsed = WarpArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            "-k" | "--kill" => parsed.kill_old_tab = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx warp: {other}. Try `yzx warp --help`."
                )));
            }
            other => {
                if parsed.target.is_some() {
                    return Err(CoreError::usage(
                        "yzx warp accepts at most one optional target argument.",
                    ));
                }
                parsed.target = Some(other.to_string());
            }
        }
    }
    Ok(parsed)
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

fn parse_popup_args(args: &[String]) -> Result<PopupArgs, CoreError> {
    let mut parsed = PopupArgs::default();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            "--refresh-sidebar-only" => parsed.refresh_sidebar_only = true,
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

fn print_cwd_help() {
    println!("Retarget the current Yazelix tab workspace directory");
    println!();
    println!("Usage:");
    println!("  yzx cwd [target]");
    println!();
    println!("Arguments:");
    println!("  target       Directory path or zoxide query for the current tab workspace root");
}

fn print_warp_help() {
    println!("Open a Yazelix workspace in a new Zellij tab");
    println!();
    println!("Usage:");
    println!("  yzx warp [target] [--kill]");
    println!();
    println!("Arguments:");
    println!("  target       Directory path or zoxide query for the new workspace tab");
    println!();
    println!("Flags:");
    println!("  -k, --kill   Close the previous tab after the new workspace tab opens");
}

fn print_reveal_help() {
    println!("Reveal a file or directory in the managed Yazi sidebar");
    println!();
    println!("Usage:");
    println!("  yzx reveal <target>");
}

fn print_popup_help() {
    println!("Open or toggle the configured Yazelix popup program in Zellij");
    println!();
    println!("Usage:");
    println!("  yzx popup [program...]");
}

fn rename_current_pane(title: &str) {
    let _ = Command::new("zellij")
        .args(["action", "rename-pane", title])
        .output();
}

fn close_current_pane() {
    let _ = Command::new("zellij")
        .args(["action", "close-pane"])
        .output();
}

pub(crate) fn load_workspace_command_config() -> Result<WorkspaceCommandConfig, CoreError> {
    let facts = compute_session_facts_from_env()?;
    let home_dir = home_dir_from_env()?;
    let env_editor = env::var("EDITOR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let managed_helix_binary = env::var("YAZELIX_MANAGED_HELIX_BINARY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(WorkspaceCommandConfig {
        enable_sidebar: facts.enable_sidebar,
        hide_sidebar_on_file_open: facts.hide_sidebar_on_file_open,
        editor_kind: resolve_managed_editor_kind(
            managed_helix_binary.as_deref(),
            facts.editor_command.as_deref(),
            env_editor.as_deref(),
        ),
        yazi_command: facts.yazi_command,
        ya_command: facts.ya_command,
        home_dir,
    })
}

pub fn compute_integration_facts_from_env() -> Result<IntegrationFactsData, CoreError> {
    let config = load_workspace_command_config()?;
    Ok(IntegrationFactsData {
        enable_sidebar: config.enable_sidebar,
        hide_sidebar_on_file_open: config.hide_sidebar_on_file_open,
        managed_editor_kind: config.editor_kind,
        yazi_command: config.yazi_command,
        ya_command: config.ya_command,
    })
}

fn resolve_managed_editor_kind(
    managed_helix_binary: Option<&str>,
    config_editor: Option<&str>,
    env_editor: Option<&str>,
) -> String {
    if managed_helix_binary.is_some() {
        return "helix".to_string();
    }

    let editor = config_editor.or(env_editor).unwrap_or("");
    let normalized = editor.trim();
    let basename = Path::new(normalized)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if normalized.ends_with("/hx")
        || normalized == "hx"
        || normalized.ends_with("/helix")
        || normalized == "helix"
        || basename == "yazelix_hx.sh"
    {
        "helix".to_string()
    } else if normalized.ends_with("/nvim")
        || normalized == "nvim"
        || normalized.ends_with("/neovim")
        || normalized == "neovim"
    {
        "neovim".to_string()
    } else {
        String::new()
    }
}

fn resolve_cwd_target(target: Option<&str>, home_dir: &Path) -> Result<PathBuf, String> {
    let current_dir =
        env::current_dir().map_err(|err| format!("Could not read the current directory: {err}"))?;
    let requested_owned = target
        .map(str::to_string)
        .unwrap_or_else(|| current_dir.to_string_lossy().to_string());
    let requested = requested_owned.as_str();

    if command_is_available("zoxide", home_dir) {
        if let Ok(output) = Command::new("zoxide")
            .args(["query", "--", requested])
            .output()
        {
            if output.status.success() {
                let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !resolved.is_empty() {
                    return Ok(PathBuf::from(resolved));
                }
            }
        }
    }

    let requested_path = resolve_path_like_input(requested, &current_dir, home_dir);
    if requested_path.exists() {
        return Ok(requested_path);
    }

    if command_is_available("zoxide", home_dir) {
        Err(format!(
            "Could not resolve '{requested}' with zoxide or as an existing path."
        ))
    } else {
        Err(format!(
            "zoxide is not available and '{requested}' is not an existing path."
        ))
    }
}

fn resolve_existing_target_dir(target_path: &Path) -> Result<PathBuf, CoreError> {
    if !target_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "missing_workspace_target",
            format!("Path does not exist: {}", target_path.display()),
            "Choose an existing directory or file path, then retry.",
            json!({ "path": target_path.display().to_string() }),
        ));
    }

    if target_path.is_dir() {
        Ok(target_path.to_path_buf())
    } else {
        Ok(target_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| target_path.to_path_buf()))
    }
}

fn workspace_tab_name(workspace_root: &Path) -> String {
    workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("unnamed")
        .to_string()
}

enum WarpTargetResolution {
    Target(PathBuf),
    Cancelled,
}

fn resolve_warp_target(
    target: Option<&str>,
    home_dir: &Path,
) -> Result<WarpTargetResolution, String> {
    if let Some(target) = target {
        return resolve_cwd_target(Some(target), home_dir).map(WarpTargetResolution::Target);
    }

    if !command_is_available("zoxide", home_dir) {
        return Err(
            "zoxide is required for interactive yzx warp. Pass a directory path explicitly."
                .to_string(),
        );
    }

    let current_dir =
        env::current_dir().map_err(|err| format!("Could not read the current directory: {err}"))?;
    let output = Command::new("zoxide")
        .args(["query", "-i", "--exclude"])
        .arg(&current_dir)
        .env("SHELL", "sh")
        .env("CLICOLOR", "1")
        .env("CLICOLOR_FORCE", "1")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| format!("Failed to launch zoxide: {err}"))?;

    if output.status.success() {
        let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if resolved.is_empty() {
            return Ok(WarpTargetResolution::Cancelled);
        }
        return Ok(WarpTargetResolution::Target(PathBuf::from(resolved)));
    }

    if output.status.code() == Some(130) {
        return Ok(WarpTargetResolution::Cancelled);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err(format!(
            "zoxide exited with code {}.",
            output.status.code().unwrap_or(1)
        ))
    } else {
        Err(stderr.trim_start_matches("zoxide: ").to_string())
    }
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

fn resolve_path_like_input(raw: &str, current_dir: &Path, home_dir: &Path) -> PathBuf {
    let expanded = expand_leading_tilde(raw, home_dir);
    let path = PathBuf::from(expanded);
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

fn expand_leading_tilde(raw: &str, home_dir: &Path) -> String {
    if raw == "~" {
        return home_dir.to_string_lossy().to_string();
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return home_dir.join(rest).to_string_lossy().to_string();
    }
    raw.to_string()
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

fn parse_workspace_retarget_response(raw: &str) -> WorkspaceRetargetResult {
    match raw.trim() {
        "missing" | "not_ready" | "permissions_denied" | "invalid_payload" => {
            WorkspaceRetargetResult {
                status: raw.trim().to_string(),
                editor_status: String::new(),
                sidebar_state: None,
                reason: None,
            }
        }
        other => match serde_json::from_str::<WorkspaceRetargetResponse>(other) {
            Ok(parsed) => {
                let sidebar_state = parsed
                    .sidebar_yazi_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|yazi_id| SidebarState {
                        yazi_id: yazi_id.to_string(),
                        cwd: parsed
                            .sidebar_yazi_cwd
                            .as_deref()
                            .map(str::trim)
                            .unwrap_or("")
                            .to_string(),
                    });
                WorkspaceRetargetResult {
                    status: parsed.status,
                    editor_status: parsed.editor_status,
                    sidebar_state,
                    reason: None,
                }
            }
            Err(_) => WorkspaceRetargetResult {
                status: "error".to_string(),
                editor_status: String::new(),
                sidebar_state: None,
                reason: Some(other.to_string()),
            },
        },
    }
}

fn retarget_workspace(
    workspace_root: &Path,
    editor_kind: &str,
) -> Result<WorkspaceRetargetResult, CoreError> {
    let payload = json!({
        "workspace_root": workspace_root.display().to_string(),
        "cd_focused_pane": true,
        "editor": if editor_kind.trim().is_empty() {
            Value::Null
        } else {
            Value::String(editor_kind.to_string())
        },
    })
    .to_string();
    let response = run_pane_orchestrator_command("retarget_workspace", &payload)?;
    Ok(parse_workspace_retarget_response(&response))
}

pub(crate) fn active_sidebar_state() -> Option<SidebarState> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "").ok()?;
    parse_active_sidebar_state(&response)
}

fn current_tab_workspace_root(include_bootstrap: bool) -> Option<String> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "").ok()?;
    current_tab_workspace_root_from_json(&response, include_bootstrap)
}

fn current_tab_workspace_root_from_json(raw: &str, include_bootstrap: bool) -> Option<String> {
    let parsed = serde_json::from_str::<serde_json::Value>(raw).ok()?;
    let workspace = parsed.get("workspace")?;
    let root = workspace.get("root")?.as_str()?.trim();
    if root.is_empty() {
        return None;
    }
    let source = workspace
        .get("source")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if !include_bootstrap && source == "bootstrap" {
        return None;
    }
    Some(root.to_string())
}

fn parse_active_sidebar_state(raw: &str) -> Option<SidebarState> {
    let parsed = serde_json::from_str::<ActiveTabSessionStateV1>(raw).ok()?;
    let sidebar = parsed.sidebar_yazi?;
    let yazi_id = sidebar.yazi_id.trim();
    let cwd = sidebar.cwd.trim();
    if yazi_id.is_empty() || cwd.is_empty() {
        return None;
    }

    Some(SidebarState {
        yazi_id: yazi_id.to_string(),
        cwd: cwd.to_string(),
    })
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

pub(crate) fn run_ya_emit_to(
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

pub(crate) fn command_is_available(command: &str, home_dir: &Path) -> bool {
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

fn zellij_new_tab_args(
    target_dir: &Path,
    tab_name: &str,
    layout_path: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        "action".to_string(),
        "new-tab".to_string(),
        "--cwd".to_string(),
        target_dir.display().to_string(),
        "--name".to_string(),
        tab_name.to_string(),
    ];
    if let Some(layout) = layout_path
        .map(str::trim)
        .filter(|layout| !layout.is_empty())
    {
        args.push("--layout".to_string());
        args.push(layout.to_string());
    }
    args
}

fn resolve_new_tab_layout_arg(
    raw_layout: Option<&str>,
    state_dir: Option<&Path>,
) -> Option<String> {
    let layout = raw_layout?.trim();
    if layout.is_empty() {
        return None;
    }

    if layout.contains('/') || layout.ends_with(".kdl") {
        return Some(layout.to_string());
    }

    let generated_layout = state_dir.map(|dir| {
        dir.join("configs")
            .join("zellij")
            .join("layouts")
            .join(format!("{layout}.kdl"))
    });
    if let Some(path) = generated_layout.as_ref().filter(|path| path.exists()) {
        return Some(path.to_string_lossy().into_owned());
    }

    Some(layout.to_string())
}

fn active_tab_id_from_json(raw: &str) -> Option<u64> {
    serde_json::from_str::<Value>(raw)
        .ok()?
        .get("tab_id")?
        .as_u64()
}

fn current_zellij_tab_id() -> Result<u64, CoreError> {
    let output = Command::new("zellij")
        .args(["action", "current-tab-info", "--json"])
        .output()
        .map_err(|source| {
            CoreError::io(
                "zellij_current_tab_failed",
                "Failed to read the current Zellij tab id.",
                "Run this command inside an active Yazelix/Zellij session, then retry.",
                "zellij",
                source,
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "zellij_current_tab_failed",
            format!("Failed to read the current Zellij tab id: {stderr}"),
            "Run this command inside an active Yazelix/Zellij session, then retry.",
            json!({ "stderr": stderr }),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    active_tab_id_from_json(&stdout).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "zellij_current_tab_unparseable",
            "Zellij returned current-tab-info JSON without a tab_id.",
            "Upgrade Zellij or run `yzx warp` without --kill.",
            json!({ "stdout": stdout.to_string() }),
        )
    })
}

fn open_zellij_workspace_tab(target_dir: &Path, tab_name: &str) -> Result<(), CoreError> {
    let layout_from_env = env::var("ZELLIJ_DEFAULT_LAYOUT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let state_dir = env::var_os("YAZELIX_STATE_DIR").map(PathBuf::from);
    let layout_arg = resolve_new_tab_layout_arg(layout_from_env.as_deref(), state_dir.as_deref());
    let args = zellij_new_tab_args(target_dir, tab_name, layout_arg.as_deref());
    let output = Command::new("zellij")
        .args(&args)
        .output()
        .map_err(|source| {
            CoreError::io(
                "zellij_new_tab_failed",
                "Failed to open a new Yazelix workspace tab.",
                "Run this command inside an active Yazelix/Zellij session, then retry.",
                "zellij",
                source,
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "zellij_new_tab_failed",
        format!("Failed to open a new Yazelix workspace tab: {stderr}"),
        "Ensure the current Zellij session can create new tabs, then retry.",
        json!({ "cwd": target_dir.display().to_string(), "stderr": stderr }),
    ))
}

fn close_zellij_tab_by_id(tab_id: u64) -> Result<(), CoreError> {
    let output = Command::new("zellij")
        .args(["action", "close-tab-by-id", &tab_id.to_string()])
        .output()
        .map_err(|source| {
            CoreError::io(
                "zellij_close_tab_failed",
                "Failed to close the previous Zellij tab.",
                "Close the previous tab manually if the new workspace tab opened.",
                "zellij",
                source,
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "zellij_close_tab_failed",
        format!("Failed to close the previous Zellij tab: {stderr}"),
        "Close the previous tab manually if the new workspace tab opened.",
        json!({ "tab_id": tab_id, "stderr": stderr }),
    ))
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;

    // Defends: the Rust workspace retarget owner keeps plugin-owned sidebar state in the single retarget response instead of reviving separate cache reads.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parses_workspace_retarget_response_with_sidebar_state() {
        let parsed = parse_workspace_retarget_response(
            r#"{"status":"ok","editor_status":"ok","sidebar_yazi_id":"plugin-sidebar-yazi-123","sidebar_yazi_cwd":"/home/sidebar"}"#,
        );

        assert_eq!(parsed.status, "ok");
        assert_eq!(parsed.editor_status, "ok");
        assert_eq!(
            parsed.sidebar_state,
            Some(SidebarState {
                yazi_id: "plugin-sidebar-yazi-123".into(),
                cwd: "/home/sidebar".into(),
            })
        );
    }

    // Defends: the workspace owner keeps Helix wrapper detection so managed-editor cwd retargeting survives the public Rust owner cut.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn resolves_managed_editor_kind_for_supported_variants() {
        assert_eq!(
            resolve_managed_editor_kind(Some("/nix/store/helix"), None, None),
            "helix"
        );
        assert_eq!(resolve_managed_editor_kind(None, Some("hx"), None), "helix");
        assert_eq!(
            resolve_managed_editor_kind(None, Some("/tmp/yazelix_hx.sh"), None),
            "helix"
        );
        assert_eq!(
            resolve_managed_editor_kind(None, Some("nvim"), None),
            "neovim"
        );
        assert_eq!(resolve_managed_editor_kind(None, None, Some("vim")), "");
    }

    // Regression: reveal must keep using the pane-orchestrator session snapshot as the only live sidebar identity source.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parses_active_sidebar_state_from_session_snapshot() {
        let state = parse_active_sidebar_state(
            r#"{"schema_version":1,"active_tab_position":0,"focus_context":"sidebar","managed_panes":{"editor_pane_id":null,"sidebar_pane_id":"terminal:0"},"layout":{"active_swap_layout_name":null,"sidebar_collapsed":false},"sidebar_yazi":{"yazi_id":"plugin-yazi-id","cwd":"/home/plugin"}}"#,
        );

        assert_eq!(
            state,
            Some(SidebarState {
                yazi_id: "plugin-yazi-id".into(),
                cwd: "/home/plugin".into(),
            })
        );
    }

    // Defends: popup routing keeps using the pane-orchestrator workspace snapshot instead of reviving a second Nu-owned workspace cache.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parses_workspace_root_from_session_snapshot() {
        let root = current_tab_workspace_root_from_json(
            r#"{"workspace":{"root":"/tmp/demo","source":"plugin"}}"#,
            false,
        );
        assert_eq!(root.as_deref(), Some("/tmp/demo"));
    }

    // Regression: popup pane execution resolves the editor alias from the Rust-owned runtime env instead of reviving a Nu popup wrapper.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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

    // Defends: yzx warp keeps its kill-old-tab behavior explicit instead of overloading positional project queries.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parses_warp_target_and_kill_flag() {
        let parsed =
            parse_warp_args(&["project".to_string(), "--kill".to_string()]).expect("warp args");

        assert_eq!(
            parsed,
            WarpArgs {
                target: Some("project".to_string()),
                kill_old_tab: true,
                help: false,
            }
        );
    }

    // Defends: yzx warp opens a fresh tab through structured Zellij argv with cwd/name/layout separated from user input.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn builds_structured_zellij_new_tab_args() {
        let args = zellij_new_tab_args(Path::new("/tmp/demo"), "demo", Some("yzx_side_closed"));

        assert_eq!(
            args,
            vec![
                "action",
                "new-tab",
                "--cwd",
                "/tmp/demo",
                "--name",
                "demo",
                "--layout",
                "yzx_side_closed",
            ]
        );
    }

    // Regression: existing Yazelix windows expose `ZELLIJ_DEFAULT_LAYOUT` as a layout name, but `zellij action new-tab --layout` needs a resolvable layout path.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn warp_resolves_generated_layout_name_to_state_path() {
        let state = tempfile::tempdir().expect("state dir");
        let layouts = state.path().join("configs").join("zellij").join("layouts");
        std::fs::create_dir_all(&layouts).expect("layouts dir");
        std::fs::write(layouts.join("yzx_side.kdl"), "layout { pane }\n").expect("layout");

        let resolved = resolve_new_tab_layout_arg(Some("yzx_side"), Some(state.path()));

        assert_eq!(
            resolved,
            Some(
                state
                    .path()
                    .join("configs/zellij/layouts/yzx_side.kdl")
                    .to_string_lossy()
                    .into_owned()
            )
        );
    }

    // Defends: explicit custom layout paths survive yzx warp unchanged instead of being rewritten as managed Yazelix layouts.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn warp_preserves_explicit_layout_paths() {
        let resolved = resolve_new_tab_layout_arg(Some("/tmp/custom.kdl"), None);

        assert_eq!(resolved, Some("/tmp/custom.kdl".to_string()));
    }

    // Defends: yzx warp --kill closes the original stable tab id rather than whichever tab is focused after new-tab.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parses_active_zellij_tab_id_from_current_tab_info_json() {
        let raw = r#"{"position":0,"name":"project","active":true,"tab_id":42}"#;

        assert_eq!(active_tab_id_from_json(raw), Some(42));
    }
}
