use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_contract::TerminalCandidate;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::{Path, PathBuf};

use super::process::find_command;

pub(super) const X11_INSTANCE: &str = "yazelix";
pub(super) const WINDOW_CLASS: &str = "com.yazelix.Yazelix";
const SUPPORTED_TERMINALS: &[&str] = &["ghostty", "yzxterm", "wezterm", "ratty", "kitty"];
const DEFAULT_TERMINALS: &[&str] = &["ghostty", "wezterm"];
const NIXGL_WRAPPER_CANDIDATES: &[&[&str]] = &[
    &["libexec", "nixGL"],
    &["libexec", "nixGLDefault"],
    &["libexec", "nixGLMesa"],
    &["libexec", "nixGLIntel"],
    &["bin", "nixGLMesa"],
    &["bin", "nixGLIntel"],
];
const NIX_VULKAN_WRAPPER_CANDIDATES: &[&[&str]] = &[
    &["libexec", "nixVulkanMesa"],
    &["libexec", "nixVulkanIntel"],
    &["bin", "nixVulkanMesa"],
    &["bin", "nixVulkanIntel"],
];
const HOST_NIXGL_COMMANDS: &[&str] = &["nixGL", "nixGLDefault", "nixGLMesa", "nixGLIntel"];
const HOST_NIX_VULKAN_COMMANDS: &[&str] = &["nixVulkanMesa", "nixVulkanIntel"];

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
        .filter(|terminal| find_command(terminal_command_name(terminal)).is_some())
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
        "yzxterm" => root.join("yzxterm").join("config.toml"),
        "ratty" => root.join("ratty").join("ratty.toml"),
        "kitty" => root.join("kitty").join("kitty.conf"),
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
        "yzxterm" => Ok(vec![
            xdg_config_home.join("yazelix-terminal").join("config.toml"),
        ]),
        "ratty" => Ok(vec![xdg_config_home.join("ratty").join("ratty.toml")]),
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
        "yzxterm" => "Yazelix Terminal".to_string(),
        "ratty" => "Ratty".to_string(),
        "kitty" => "Kitty".to_string(),
        other => other.to_string(),
    }
}

pub(super) fn get_working_dir_args(terminal: &str, working_dir: &Path) -> Vec<String> {
    let wd = working_dir.to_string_lossy().into_owned();
    match terminal {
        "ghostty" => vec![format!("--working-directory={wd}")],
        "wezterm" => vec!["--cwd".to_string(), wd],
        "yzxterm" => vec!["--working-dir".to_string(), wd],
        "ratty" => vec![],
        "kitty" => vec![format!("--directory={wd}")],
        _ => vec![],
    }
}

pub(super) fn terminal_command_name(terminal: &str) -> &str {
    match terminal {
        "yzxterm" => "yazelix-terminal-desktop",
        other => other,
    }
}

pub(super) fn current_platform_name() -> String {
    std::env::var("YAZELIX_TEST_OS")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::env::consts::OS.to_string())
}

fn resolve_runtime_or_host_wrapper(
    runtime_dir: &Path,
    runtime_candidates: &[&[&str]],
    host_commands: &[&str],
) -> Option<String> {
    for relative in runtime_candidates {
        let path = runtime_dir.join(relative.iter().collect::<PathBuf>());
        if path.is_file() {
            return Some(path.to_string_lossy().into_owned());
        }
    }

    for command in host_commands {
        if find_command(command).is_some() {
            return Some((*command).to_string());
        }
    }
    None
}

pub(super) fn resolve_nixgl_wrapper(runtime_dir: &Path) -> Option<String> {
    resolve_runtime_or_host_wrapper(runtime_dir, NIXGL_WRAPPER_CANDIDATES, HOST_NIXGL_COMMANDS)
}

fn resolve_nix_vulkan_wrapper(runtime_dir: &Path) -> Option<String> {
    resolve_runtime_or_host_wrapper(
        runtime_dir,
        NIX_VULKAN_WRAPPER_CANDIDATES,
        HOST_NIX_VULKAN_COMMANDS,
    )
}

fn resolve_graphics_wrapper(runtime_dir: &Path, terminal: &str) -> Option<String> {
    if terminal == "ratty" {
        return resolve_nix_vulkan_wrapper(runtime_dir);
    }
    resolve_nixgl_wrapper(runtime_dir)
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
    let graphics_wrapper = resolve_graphics_wrapper(runtime_dir, &terminal.terminal);

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
            let ghostty = maybe_prepend(ghostty, graphics_wrapper);
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
            maybe_prepend(wezterm, graphics_wrapper)
        }
        "yzxterm" => {
            let mut yzxterm = vec![
                terminal.command.clone(),
                "--title-placeholder".to_string(),
                title,
                "--yazelix".to_string(),
            ];
            yzxterm.extend(working_dir_args);
            yzxterm.push("-e".to_string());
            yzxterm.push(startup_script.to_string_lossy().into_owned());
            yzxterm
        }
        "ratty" => {
            let mut ratty = vec![
                terminal.command.clone(),
                "--config-file".to_string(),
                config_string,
                "--title".to_string(),
                title,
            ];
            ratty.extend(working_dir_args);
            ratty.push("-e".to_string());
            ratty.push(startup_script.to_string_lossy().into_owned());
            maybe_prepend(ratty, graphics_wrapper)
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
            maybe_prepend(kitty, graphics_wrapper)
        }
        other => {
            return Err(CoreError::usage(format!("Unknown terminal: {other}")));
        }
    };

    Ok(argv)
}
