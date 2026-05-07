use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_contract::TerminalCandidate;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::{Path, PathBuf};

use super::find_command;

pub(super) const X11_INSTANCE: &str = "yazelix";
pub(super) const WINDOW_CLASS: &str = "com.yazelix.Yazelix";
const SUPPORTED_TERMINALS: &[&str] = &["ghostty", "wezterm", "kitty", "alacritty", "foot"];
const DEFAULT_TERMINALS: &[&str] = &["ghostty", "wezterm"];
pub(super) fn normalized_configured_terminals(config: &JsonMap<String, JsonValue>) -> Vec<String> {
    let raw = match config.get("terminals") {
        Some(JsonValue::Array(items)) => items
            .iter()
            .filter_map(JsonValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase())
            .collect::<Vec<_>>(),
        _ => DEFAULT_TERMINALS
            .iter()
            .map(|terminal| (*terminal).to_string())
            .collect(),
    };

    let mut out = Vec::new();
    for terminal in raw {
        if !SUPPORTED_TERMINALS.contains(&terminal.as_str()) {
            continue;
        }
        if !out.contains(&terminal) {
            out.push(terminal);
        }
    }
    out
}

pub(super) fn print_empty_terminal_error() -> Result<(), CoreError> {
    let available = SUPPORTED_TERMINALS
        .iter()
        .filter(|terminal| find_command(terminal).is_some())
        .copied()
        .collect::<Vec<_>>();
    let available_text = if available.is_empty() {
        "none detected".to_string()
    } else {
        available.join(", ")
    };
    eprintln!("Error: terminal.terminals must include at least one terminal");
    eprintln!("Detected terminals: {available_text}");
    eprintln!("Set terminal.terminals in ~/.config/yazelix/settings.jsonc");
    Ok(())
}

pub(super) fn generated_terminal_config_path(state_dir: &Path, terminal: &str) -> PathBuf {
    let root = state_dir.join("configs").join("terminal_emulators");
    match terminal {
        "ghostty" => root.join("ghostty").join("config"),
        "wezterm" => root.join("wezterm").join(".wezterm.lua"),
        "kitty" => root.join("kitty").join("kitty.conf"),
        "alacritty" => root.join("alacritty").join("alacritty.toml"),
        "foot" => root.join("foot").join("foot.ini"),
        other => root.join(other),
    }
}

pub(super) fn xdg_config_home_for_user(home_dir: &Path) -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| home_dir.join(".config"))
}

pub(super) fn ghostty_user_config_candidates(
    home_dir: &Path,
    xdg_config_home: &Path,
    platform: &str,
) -> Vec<PathBuf> {
    let xdg_ghostty = xdg_config_home.join("ghostty");
    let mut candidates = vec![
        xdg_ghostty.join("config.ghostty"),
        xdg_ghostty.join("config"),
    ];

    if matches!(platform, "macos" | "darwin") {
        let app_support = home_dir
            .join("Library")
            .join("Application Support")
            .join("com.mitchellh.ghostty");
        candidates.push(app_support.join("config.ghostty"));
        candidates.push(app_support.join("config"));
    }

    candidates
}

pub(super) fn user_terminal_config_candidates_for_platform(
    home_dir: &Path,
    terminal: &str,
    xdg_config_home: &Path,
    platform: &str,
) -> Result<Vec<PathBuf>, String> {
    match terminal {
        "ghostty" => Ok(ghostty_user_config_candidates(
            home_dir,
            xdg_config_home,
            platform,
        )),
        "kitty" => Ok(vec![
            home_dir.join(".config").join("kitty").join("kitty.conf"),
        ]),
        "wezterm" => Ok(vec![
            home_dir.join(".wezterm.lua"),
            home_dir.join(".config").join("wezterm").join("wezterm.lua"),
        ]),
        "alacritty" => Ok(vec![
            home_dir
                .join(".config")
                .join("alacritty")
                .join("alacritty.toml"),
        ]),
        "foot" => Ok(vec![home_dir.join(".config").join("foot").join("foot.ini")]),
        other => Err(format!("Unsupported terminal config lookup: {other}")),
    }
}

pub(super) fn user_terminal_config_path(
    home_dir: &Path,
    terminal: &str,
) -> Result<PathBuf, String> {
    let candidates = user_terminal_config_candidates_for_platform(
        home_dir,
        terminal,
        &xdg_config_home_for_user(home_dir),
        &current_platform_name(),
    )?;
    select_existing_user_terminal_config_path(terminal, &candidates)
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
    match mode {
        "yazelix" => Ok(generated_terminal_config_path(state_dir, terminal)),
        "user" => user_terminal_config_path(home_dir, terminal),
        other => Err(format!(
            "Unsupported terminal.config_mode '{other}'. Expected 'yazelix' or 'user'."
        )),
    }
}

pub(super) fn terminal_display_name(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "kitty" => "Kitty".to_string(),
        "alacritty" => "Alacritty".to_string(),
        "foot" => "Foot".to_string(),
        other => other.to_string(),
    }
}

pub(super) fn get_working_dir_args(terminal: &str, working_dir: &Path) -> Vec<String> {
    let wd = working_dir.to_string_lossy().into_owned();
    match terminal {
        "ghostty" => vec![format!("--working-directory={wd}")],
        "wezterm" => vec!["--cwd".to_string(), wd],
        "kitty" => vec![format!("--directory={wd}")],
        "alacritty" => vec!["--working-directory".to_string(), wd],
        "foot" => vec![format!("--working-directory={wd}")],
        _ => vec![],
    }
}

pub(super) fn current_platform_name() -> String {
    std::env::var("YAZELIX_TEST_OS")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::env::consts::OS.to_string())
}

pub(super) fn resolve_nixgl_wrapper(runtime_dir: &Path) -> Option<String> {
    for relative in [
        ["libexec", "nixGL"].as_slice(),
        ["libexec", "nixGLDefault"].as_slice(),
        ["libexec", "nixGLMesa"].as_slice(),
        ["libexec", "nixGLIntel"].as_slice(),
        ["bin", "nixGLMesa"].as_slice(),
        ["bin", "nixGLIntel"].as_slice(),
    ] {
        let path = runtime_dir.join(relative.iter().collect::<PathBuf>());
        if path.is_file() {
            return Some(path.to_string_lossy().into_owned());
        }
    }

    for command in ["nixGL", "nixGLDefault", "nixGLMesa", "nixGLIntel"] {
        if find_command(command).is_some() {
            return Some(command.to_string());
        }
    }
    None
}

pub(super) fn maybe_prepend(argv: Vec<String>, wrapper: Option<String>) -> Vec<String> {
    if let Some(wrapper) = wrapper.filter(|value| !value.trim().is_empty()) {
        let mut out = vec![wrapper];
        out.extend(argv);
        out
    } else {
        argv
    }
}

pub(super) fn build_launch_command_argv(
    runtime_dir: &Path,
    terminal: &TerminalCandidate,
    config_path: &Path,
    working_dir: &Path,
) -> Result<Vec<String>, CoreError> {
    let working_dir_args = get_working_dir_args(&terminal.terminal, working_dir);
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

    let title = format!("Yazelix - {}", terminal_display_name(&terminal.terminal));
    let config_string = config_path.to_string_lossy().into_owned();
    let nixgl = resolve_nixgl_wrapper(runtime_dir);

    let argv = match terminal.terminal.as_str() {
        "ghostty" => {
            let mut ghostty = if current_platform_name() == "macos" {
                vec![
                    terminal.command.clone(),
                    "--config-default-files=false".to_string(),
                    format!("--config-file={config_string}"),
                    format!("--title={title}"),
                ]
            } else {
                vec![
                    terminal.command.clone(),
                    "--config-default-files=false".to_string(),
                    format!("--config-file={config_string}"),
                    "--gtk-single-instance=false".to_string(),
                    format!("--class={WINDOW_CLASS}"),
                    format!("--x11-instance-name={X11_INSTANCE}"),
                    format!("--title={title}"),
                ]
            };
            ghostty.extend(working_dir_args);
            ghostty.push("-e".to_string());
            ghostty.push(startup_script.to_string_lossy().into_owned());
            let ghostty = maybe_prepend(ghostty, nixgl);
            let ghostty_wrapper = runtime_dir
                .join("shells")
                .join("posix")
                .join("yazelix_ghostty.sh");
            maybe_prepend(
                ghostty,
                ghostty_wrapper
                    .is_file()
                    .then(|| ghostty_wrapper.to_string_lossy().into_owned()),
            )
        }
        "wezterm" => {
            let mut wezterm = vec![
                terminal.command.clone(),
                "--config-file".to_string(),
                config_string,
                "start".to_string(),
                format!("--class={WINDOW_CLASS}"),
            ];
            wezterm.extend(working_dir_args);
            wezterm.push("--".to_string());
            wezterm.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(wezterm, nixgl)
        }
        "kitty" => {
            let mut kitty = vec![
                terminal.command.clone(),
                format!("--config={config_string}"),
                format!("--class={WINDOW_CLASS}"),
                format!("--title={title}"),
            ];
            kitty.extend(working_dir_args);
            kitty.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(kitty, nixgl)
        }
        "alacritty" => {
            let mut alacritty = vec![
                terminal.command.clone(),
                "--config-file".to_string(),
                config_string,
                "--class".to_string(),
                WINDOW_CLASS.to_string(),
                "--title".to_string(),
                title,
            ];
            alacritty.extend(working_dir_args);
            alacritty.push("-e".to_string());
            alacritty.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(alacritty, nixgl)
        }
        "foot" => {
            let mut foot = vec![
                terminal.command.clone(),
                "--config".to_string(),
                config_string,
                "--app-id".to_string(),
                WINDOW_CLASS.to_string(),
            ];
            foot.extend(working_dir_args);
            foot.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(foot, nixgl)
        }
        other => {
            return Err(CoreError::usage(format!("Unknown terminal: {other}")));
        }
    };

    Ok(argv)
}
