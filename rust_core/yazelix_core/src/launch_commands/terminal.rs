use crate::atomic_fs::is_executable_file;
use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_contract::{TerminalCandidate, resolve_runtime_nixgl_wrapper};
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

/// User config file candidates per terminal, used by `terminal.config_mode = user`.
fn user_terminal_config_candidates(home_dir: &Path, terminal: &str) -> Vec<PathBuf> {
    let config_home = xdg_config_home_for_user(home_dir);
    match terminal {
        "mars" => vec![config_home.join("mars").join("config.toml")],
        "kitty" => vec![config_home.join("kitty").join("kitty.conf")],
        "ghostty" => vec![config_home.join("ghostty").join("config")],
        _ => Vec::new(),
    }
}

pub(super) fn resolve_terminal_config_path(
    home_dir: &Path,
    state_dir: &Path,
    mode: &str,
    terminal: &str,
) -> Result<PathBuf, String> {
    if !crate::terminal_variant::SUPPORTED_TERMINALS.contains(&terminal) {
        return Err(format!(
            "Yazelix does not launch terminal '{terminal}'; configure it as a host terminal to run `yzx enter`."
        ));
    }

    match mode {
        // kitty and ghostty have no Yazelix-generated config yet; they launch with
        // their own defaults (or the user's config, picked up by the terminal
        // itself), so the generated path is advisory for them.
        "yazelix" => Ok(generated_terminal_config_path(state_dir, terminal)),
        "user" => {
            let candidates = user_terminal_config_candidates(home_dir, terminal);
            match select_existing_user_terminal_config_path(terminal, &candidates) {
                Ok(path) => Ok(path),
                // Only mars hard-requires a config file to launch correctly;
                // kitty/ghostty fall back to their built-in defaults.
                Err(_) if terminal != "mars" => {
                    Ok(generated_terminal_config_path(state_dir, terminal))
                }
                Err(err) => Err(err),
            }
        }
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
    let terminal_command = resolve_runtime_terminal_command(runtime_dir, &terminal.command);
    let title = terminal_window_title(&terminal.terminal, session_name);
    let working_dir = working_dir.to_string_lossy().into_owned();
    let startup_script = startup_script.to_string_lossy().into_owned();

    let argv = match terminal.terminal.as_str() {
        "mars" => vec![
            terminal_command,
            "--title-placeholder".to_string(),
            title,
            "--working-dir".to_string(),
            working_dir,
            "-e".to_string(),
            startup_script,
        ],
        "kitty" => vec![
            terminal_command,
            "--title".to_string(),
            title,
            "--directory".to_string(),
            working_dir,
            startup_script,
        ],
        "ghostty" => vec![
            terminal_command,
            format!("--title={title}"),
            format!("--working-directory={working_dir}"),
            "-e".to_string(),
            startup_script,
        ],
        other => {
            return Err(CoreError::usage(format!(
                "Yazelix does not launch terminal '{other}'; configure it as a host terminal to run `yzx enter`."
            )));
        }
    };

    apply_runtime_graphics_wrapper(runtime_dir, &terminal.terminal, argv)
}

fn apply_runtime_graphics_wrapper(
    runtime_dir: &Path,
    terminal: &str,
    argv: Vec<String>,
) -> Result<Vec<String>, CoreError> {
    if current_platform_name() != "linux" || terminal != "kitty" {
        return Ok(argv);
    }

    let wrapper = resolve_runtime_nixgl_wrapper(runtime_dir).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "missing_runtime_graphics_wrapper",
            format!(
                "The packaged Kitty runtime is missing its Linux graphics wrapper under {}.",
                runtime_dir.display()
            ),
            "Reinstall or update Yazelix so the active package runtime ships libexec/nixGLMesa.",
            serde_json::json!({
                "runtime_dir": runtime_dir,
                "terminal": terminal,
            }),
        )
    })?;

    let mut wrapped = Vec::with_capacity(argv.len() + 1);
    wrapped.push(wrapper.to_string_lossy().into_owned());
    wrapped.extend(argv);
    Ok(wrapped)
}

fn resolve_runtime_terminal_command(runtime_dir: &Path, command: &str) -> String {
    if command.contains(std::path::MAIN_SEPARATOR) {
        return command.to_string();
    }

    for dir in ["toolbin", "bin"] {
        let candidate = runtime_dir.join(dir).join(command);
        if is_executable_file(&candidate) {
            return candidate.to_string_lossy().into_owned();
        }
    }

    command.to_string()
}
