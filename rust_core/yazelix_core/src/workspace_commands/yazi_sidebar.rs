//! Yazi/sidebar command adapter for workspace commands.

use super::{expand_leading_tilde, load_workspace_command_config, resolve_path_like_input};
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::state_dir_from_env;
use crate::pane_orchestrator_client::run_pane_orchestrator_command;
use crate::sidebar_bootstrap::{SIDEBAR_BOOTSTRAP_CWD_ENV, is_sidebar_bootstrap_file};
use crate::workspace_session::{SidebarState, parse_active_sidebar_state};
use serde_json::json;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ZELLIJ_KITTY_PASSTHROUGH_FEATURE: &str = "zellij_kitty_passthrough";
const YAZELIX_RUNTIME_DIR_ENV: &str = "YAZELIX_RUNTIME_DIR";
const YAZELIX_ZELLIJ_SESSION_NAME_ENV: &str = "YAZELIX_ZELLIJ_SESSION_NAME";
const ZELLIJ_SESSION_NAME_ENV: &str = "ZELLIJ_SESSION_NAME";
const KITTY_WINDOW_ID_ENV: &str = "KITTY_WINDOW_ID";

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
            "Install Yazi, fix yazi.command in config.toml, or restart Yazelix so the runtime PATH is active.",
            json!({ "command": config.yazi_command }),
        ));
    }

    let target_dir = consume_bootstrap_sidebar_cwd(&config.home_dir)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| config.home_dir.clone());
    let command_path = resolve_command_path(&config.yazi_command, &config.home_dir);
    let runtime_dir = sidebar_runtime_dir();
    let mut command = Command::new(&command_path);
    configure_yazi_runtime_env(&mut command, runtime_dir.as_deref());
    configure_yazi_graphics_env(&mut command, runtime_dir.as_deref());
    let status = command.arg(target_dir).status().map_err(|source| {
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

fn configure_yazi_runtime_env(command: &mut Command, runtime_dir: Option<&Path>) {
    if let Some(runtime_dir) = runtime_dir {
        command.env(YAZELIX_RUNTIME_DIR_ENV, runtime_dir);
    }
}

fn configure_yazi_graphics_env(command: &mut Command, runtime_dir: Option<&Path>) {
    configure_yazi_graphics_env_with_session(
        command,
        runtime_dir,
        zellij_control_session_name_from_env(),
    );
}

fn configure_yazi_graphics_env_with_session(
    command: &mut Command,
    runtime_dir: Option<&Path>,
    session_name: Option<OsString>,
) {
    if runtime_dir.is_some_and(runtime_has_zellij_kitty_passthrough) {
        configure_yazi_zellij_control_session_env(command, session_name);
        command.env(ZELLIJ_SESSION_NAME_ENV, "");
        command.env(KITTY_WINDOW_ID_ENV, "1");
    }
}

fn configure_yazi_zellij_control_session_env(
    command: &mut Command,
    session_name: Option<OsString>,
) {
    if let Some(session_name) = session_name.filter(|value| !value.is_empty()) {
        command.env(YAZELIX_ZELLIJ_SESSION_NAME_ENV, session_name);
    }
}

fn zellij_control_session_name_from_env() -> Option<OsString> {
    env::var_os(ZELLIJ_SESSION_NAME_ENV)
        .filter(|value| !value.is_empty())
        .or_else(|| env::var_os(YAZELIX_ZELLIJ_SESSION_NAME_ENV).filter(|value| !value.is_empty()))
}

fn sidebar_runtime_dir() -> Option<PathBuf> {
    env::var_os(YAZELIX_RUNTIME_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(infer_runtime_dir_from_current_exe)
}

fn infer_runtime_dir_from_current_exe() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    runtime_dir_from_helper_exe(&exe)
}

fn runtime_dir_from_helper_exe(exe: &Path) -> Option<PathBuf> {
    let file_name = exe.file_name()?.to_str()?;
    if file_name != "yzx" && file_name != "yzx_control" {
        return None;
    }

    let libexec_dir = exe.parent()?;
    if libexec_dir.file_name()?.to_str()? != "libexec" {
        return None;
    }

    libexec_dir.parent().map(Path::to_path_buf)
}

fn runtime_has_zellij_kitty_passthrough(runtime_dir: &Path) -> bool {
    runtime_dir
        .join("runtime_features")
        .join(ZELLIJ_KITTY_PASSTHROUGH_FEATURE)
        .is_file()
}

fn consume_bootstrap_sidebar_cwd(home_dir: &Path) -> Option<PathBuf> {
    let cwd_file = env::var(SIDEBAR_BOOTSTRAP_CWD_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())?;
    let cwd_file = PathBuf::from(cwd_file);
    let state_dir = state_dir_from_env().ok()?;
    consume_bootstrap_sidebar_cwd_file(home_dir, &state_dir, &cwd_file)
}

fn consume_bootstrap_sidebar_cwd_file(
    home_dir: &Path,
    state_dir: &Path,
    cwd_file: &Path,
) -> Option<PathBuf> {
    if !is_sidebar_bootstrap_file(state_dir, cwd_file) {
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

#[cfg(test)]
mod tests {
    // Test lane: default

    use super::*;
    use crate::sidebar_bootstrap::sidebar_bootstrap_owner_dir;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    fn command_envs(command: &Command) -> BTreeMap<String, Option<String>> {
        command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect()
    }

    // Regression: upstream Yazi can use Kitty graphics under Yazelix's Zellij bridge when the launch env hides Zellij from Yazi only.
    #[test]
    fn yazi_launch_env_enables_kitty_adapter_for_passthrough_runtime() {
        let tmp = TempDir::new().expect("tmp");
        fs::create_dir_all(tmp.path().join("runtime_features")).expect("runtime features");
        fs::write(
            tmp.path()
                .join("runtime_features")
                .join(ZELLIJ_KITTY_PASSTHROUGH_FEATURE),
            "",
        )
        .expect("feature marker");

        let mut command = Command::new("yazi");
        configure_yazi_graphics_env_with_session(
            &mut command,
            Some(tmp.path()),
            Some(OsString::from("real-zellij-session")),
        );

        let envs = command_envs(&command);
        assert_eq!(
            envs.get(ZELLIJ_SESSION_NAME_ENV),
            Some(&Some(String::new()))
        );
        assert_eq!(
            envs.get(YAZELIX_ZELLIJ_SESSION_NAME_ENV),
            Some(&Some("real-zellij-session".to_string()))
        );
        assert_eq!(envs.get(KITTY_WINDOW_ID_ENV), Some(&Some("1".to_string())));
    }

    // Regression: managed Yazi plugins must keep access to the current runtime even when Zellij starts panes without the full Yazelix env.
    #[test]
    fn yazi_launch_env_sets_runtime_dir_for_plugin_commands() {
        let tmp = TempDir::new().expect("tmp");
        let mut command = Command::new("yazi");
        configure_yazi_runtime_env(&mut command, Some(tmp.path()));

        let envs = command_envs(&command);
        assert_eq!(
            envs.get(YAZELIX_RUNTIME_DIR_ENV),
            Some(&Some(tmp.path().to_string_lossy().into_owned()))
        );
    }

    // Defends: packaged libexec helpers can recover their runtime root without trusting inherited pane env.
    #[test]
    fn runtime_dir_inference_accepts_packaged_libexec_helpers() {
        let runtime = PathBuf::from("/nix/store/example-yazelix");
        assert_eq!(
            runtime_dir_from_helper_exe(&runtime.join("libexec").join("yzx_control")),
            Some(runtime.clone())
        );
        assert_eq!(
            runtime_dir_from_helper_exe(&runtime.join("libexec").join("yzx")),
            Some(runtime)
        );
        assert_eq!(
            runtime_dir_from_helper_exe(Path::new("/tmp/yazelix/target/debug/yzx_control")),
            None
        );
    }

    // Invariant: runtimes without the explicit Zellij Kitty bridge marker keep upstream Yazi's normal Zellij adapter filtering.
    #[test]
    fn yazi_launch_env_leaves_plain_zellij_runtime_unchanged() {
        let tmp = TempDir::new().expect("tmp");
        let mut command = Command::new("yazi");
        configure_yazi_graphics_env(&mut command, Some(tmp.path()));

        let envs = command_envs(&command);
        assert!(!envs.contains_key(ZELLIJ_SESSION_NAME_ENV));
        assert!(!envs.contains_key(KITTY_WINDOW_ID_ENV));
    }

    // Defends: sidebar startup consumes and deletes only Yazelix-owned one-shot cwd files.
    #[test]
    fn consume_bootstrap_sidebar_cwd_reads_owned_file_once() {
        let tmp = TempDir::new().expect("tmp");
        let home = tmp.path().join("home");
        let project = tmp.path().join("project");
        let bootstrap_dir = sidebar_bootstrap_owner_dir(tmp.path(), "enter");
        let bootstrap_file = bootstrap_dir.join("cwd.tmp");
        fs::create_dir_all(&home).expect("home");
        fs::create_dir_all(&project).expect("project");
        fs::create_dir_all(&bootstrap_dir).expect("bootstrap dir");
        fs::write(&bootstrap_file, project.to_string_lossy().as_ref()).expect("bootstrap file");

        assert_eq!(
            consume_bootstrap_sidebar_cwd_file(&home, tmp.path(), &bootstrap_file),
            Some(project)
        );
        assert!(!bootstrap_file.exists());
    }

    // Defends: arbitrary inherited env paths are ignored without deleting user-owned files.
    #[test]
    fn consume_bootstrap_sidebar_cwd_ignores_unowned_file() {
        let tmp = TempDir::new().expect("tmp");
        let home = tmp.path().join("home");
        let project = tmp.path().join("project");
        let unowned_file = tmp.path().join("outside.tmp");
        fs::create_dir_all(&home).expect("home");
        fs::create_dir_all(&project).expect("project");
        fs::write(&unowned_file, project.to_string_lossy().as_ref()).expect("unowned file");

        assert_eq!(
            consume_bootstrap_sidebar_cwd_file(&home, tmp.path(), &unowned_file),
            None
        );
        assert!(unowned_file.exists());
    }
}
