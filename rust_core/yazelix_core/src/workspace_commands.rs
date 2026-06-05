// Test lane: default
//! Public workspace command owners for `yzx_control`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::home_dir_from_env;
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use crate::session_facts::compute_session_facts_from_env;
use crate::workspace_session::{
    IntegrationFactsData, WorkspaceRetargetResult, parse_workspace_retarget_response,
    resolve_managed_editor_kind, workspace_dir_for_target, workspace_retarget_payload,
    workspace_tab_name,
};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

mod popup;
mod yazi_sidebar;

pub use popup::{run_yzx_popup, run_yzx_popup_run};
pub(crate) use yazi_sidebar::sync_sidebar_to_directory;
pub use yazi_sidebar::{run_yzx_reveal, run_yzx_sidebar};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CwdArgs {
    target: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceCommandConfig {
    pub(crate) hide_sidebar_on_file_open: bool,
    pub(crate) editor_kind: String,
    pub(crate) yazi_command: String,
    pub(crate) ya_command: String,
    pub(crate) home_dir: PathBuf,
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

fn print_cwd_help() {
    println!("Retarget the current Yazelix tab workspace directory");
    println!();
    println!("Usage:");
    println!("  yzx cwd [target]");
    println!();
    println!("Arguments:");
    println!("  target       Directory path or zoxide query for the current tab workspace root");
}

pub(crate) fn load_workspace_command_config() -> Result<WorkspaceCommandConfig, CoreError> {
    let facts = compute_session_facts_from_env()?;
    let home_dir = home_dir_from_env()?;

    Ok(WorkspaceCommandConfig {
        hide_sidebar_on_file_open: facts.hide_sidebar_on_file_open,
        editor_kind: resolve_managed_editor_kind(facts.editor_command.as_deref()),
        yazi_command: facts.yazi_command,
        ya_command: facts.ya_command,
        home_dir,
    })
}

pub fn compute_integration_facts_from_env() -> Result<IntegrationFactsData, CoreError> {
    let config = load_workspace_command_config()?;
    Ok(IntegrationFactsData {
        hide_sidebar_on_file_open: config.hide_sidebar_on_file_open,
        managed_editor_kind: config.editor_kind,
        yazi_command: config.yazi_command,
        ya_command: config.ya_command,
    })
}

fn resolve_cwd_target(target: Option<&str>, home_dir: &Path) -> Result<PathBuf, String> {
    let current_dir =
        env::current_dir().map_err(|err| format!("Could not read the current directory: {err}"))?;
    let requested_owned = target
        .map(str::to_string)
        .unwrap_or_else(|| current_dir.to_string_lossy().to_string());
    let requested = requested_owned.as_str();

    if yazi_sidebar::command_is_available("zoxide", home_dir) {
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

    if yazi_sidebar::command_is_available("zoxide", home_dir) {
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

    Ok(workspace_dir_for_target(target_path))
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

fn retarget_workspace(
    workspace_root: &Path,
    editor_kind: &str,
) -> Result<WorkspaceRetargetResult, CoreError> {
    let payload = workspace_retarget_payload(workspace_root, true, Some(editor_kind), None);
    let response = run_pane_orchestrator_command("retarget_workspace", &payload)?;
    Ok(parse_workspace_retarget_response(&response))
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;

    // Defends: the workspace owner keeps Helix wrapper detection so managed-editor cwd retargeting survives the public Rust owner cut.
    #[test]
    fn resolves_managed_editor_kind_for_supported_variants() {
        assert_eq!(resolve_managed_editor_kind(None), "helix");
        assert_eq!(resolve_managed_editor_kind(Some("hx")), "helix");
        assert_eq!(
            resolve_managed_editor_kind(Some("/tmp/yazelix_hx.sh")),
            "helix"
        );
        assert_eq!(resolve_managed_editor_kind(Some("nvim")), "neovim");
        assert_eq!(resolve_managed_editor_kind(Some("vim")), "");
    }
}
