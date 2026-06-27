use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_contract::TerminalCandidate;
use crate::terminal_variant::terminal_window_title;
use std::path::{Path, PathBuf};

pub(super) fn generated_terminal_config_path(state_dir: &Path, terminal: &str) -> PathBuf {
    state_dir
        .join("configs")
        .join("terminal_emulators")
        .join(terminal)
        .join("config.toml")
}

pub(super) fn xdg_config_home_for_user(home_dir: &Path) -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| home_dir.join(".config"))
}

pub(super) fn select_existing_user_terminal_config_path(
    terminal: &str,
    candidates: &[PathBuf],
) -> Result<PathBuf, String> {
    if let Some(path) = candidates.iter().find(|path| path.exists()) {
        return Ok(path.clone());
    }

    let checked = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "terminal.config_mode = user requires a real {terminal} user config at one of: {checked}"
    ))
}

pub(super) fn resolve_terminal_config_path(
    home_dir: &Path,
    state_dir: &Path,
    mode: &str,
    terminal: &str,
) -> Result<PathBuf, String> {
    if terminal != "mars" {
        return Err(format!(
            "Yazelix only launches the packaged Mars terminal; configure host terminal '{terminal}' to run `yzx enter`."
        ));
    }

    match mode {
        "yazelix" => Ok(generated_terminal_config_path(state_dir, terminal)),
        "user" => select_existing_user_terminal_config_path(
            terminal,
            &[xdg_config_home_for_user(home_dir)
                .join("mars")
                .join("config.toml")],
        ),
        other => Err(format!(
            "Unsupported terminal.config_mode '{other}'. Expected 'yazelix' or 'user'."
        )),
    }
}

pub(super) fn current_platform_name() -> String {
    std::env::var("YAZELIX_TEST_OS")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::env::consts::OS.to_string())
}

pub(super) fn build_launch_command_argv(
    runtime_dir: &Path,
    terminal: &TerminalCandidate,
    _config_path: &Path,
    working_dir: &Path,
    session_name: Option<&str>,
) -> Result<Vec<String>, CoreError> {
    if terminal.terminal != "mars" {
        return Err(CoreError::usage(format!(
            "Yazelix only launches the packaged Mars terminal; configure host terminal '{}' to run `yzx enter`.",
            terminal.terminal
        )));
    }

    let startup_script = runtime_dir
        .join("shells")
        .join("posix")
        .join("start_yazelix.sh");
    if !startup_script.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_startup_script",
            format!(
                "Missing Yazelix startup script at {}.",
                startup_script.display()
            ),
            "Restore shells/posix/start_yazelix.sh or reinstall Yazelix.",
            serde_json::json!({}),
        ));
    }

    Ok(vec![
        terminal.command.clone(),
        "--title-placeholder".to_string(),
        terminal_window_title(&terminal.terminal, session_name),
        "--working-dir".to_string(),
        working_dir.to_string_lossy().into_owned(),
        "-e".to_string(),
        startup_script.to_string_lossy().into_owned(),
    ])
}
