// Test lane: default
//! Public `yzx cwd` and `yzx reveal` owners for `yzx_control`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, home_dir_from_env,
    load_normalized_config_for_control, runtime_dir_from_env,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CwdArgs {
    target: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RevealArgs {
    target: Option<String>,
    help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkspaceCommandConfig {
    enable_sidebar: bool,
    editor_kind: String,
    ya_command: String,
    home_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SidebarState {
    yazi_id: String,
    cwd: String,
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
    if !config.enable_sidebar {
        println!(
            "📂 Reveal in Yazi only works in sidebar mode. You're currently using no-sidebar mode."
        );
        println!(
            "💡 Tip: Use your editor-local file picker in no-sidebar mode, or enable sidebar mode in yazelix.toml"
        );
        return Ok(0);
    }

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

fn print_cwd_help() {
    println!("Retarget the current Yazelix tab workspace directory");
    println!();
    println!("Usage:");
    println!("  yzx cwd [target]");
    println!();
    println!("Arguments:");
    println!("  target       Directory path or zoxide query for the current tab workspace root");
}

fn print_reveal_help() {
    println!("Reveal a file or directory in the managed Yazi sidebar");
    println!();
    println!("Usage:");
    println!("  yzx reveal <target>");
}

fn load_workspace_command_config() -> Result<WorkspaceCommandConfig, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let normalized =
        load_normalized_config_for_control(&runtime_dir, &config_dir, config_override.as_deref())?;
    let home_dir = home_dir_from_env()?;
    let enable_sidebar = normalized
        .get("enable_sidebar")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let ya_command = normalized
        .get("yazi_ya_command")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("ya")
        .to_string();
    let editor_command = normalized
        .get("editor_command")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let env_editor = env::var("EDITOR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let managed_helix_binary = env::var("YAZELIX_MANAGED_HELIX_BINARY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(WorkspaceCommandConfig {
        enable_sidebar,
        editor_kind: resolve_managed_editor_kind(
            managed_helix_binary.as_deref(),
            editor_command,
            env_editor.as_deref(),
        ),
        ya_command,
        home_dir,
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

fn active_sidebar_state() -> Option<SidebarState> {
    let response = run_pane_orchestrator_command("get_active_tab_session_state", "").ok()?;
    parse_active_sidebar_state(&response)
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

fn sync_sidebar_to_directory(
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

fn command_is_available(command: &str, home_dir: &Path) -> bool {
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
}
