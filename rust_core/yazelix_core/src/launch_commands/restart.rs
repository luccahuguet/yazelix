use super::config_override::{
    config_override_extra_env, prepare_session_config_override, resolve_cli_config_override,
};
use super::create_sidebar_bootstrap_file;
use super::process::{command_output_with_overrides, print_completed_output};
use super::{RUNTIME_RELAUNCH_CLEARED_ENV_KEYS, current_runtime_yzx_launcher};
use crate::bridge::CoreError;
use crate::control_plane::{config_override_from_env, runtime_dir_from_env};
use crate::sidebar_bootstrap::SIDEBAR_BOOTSTRAP_CWD_ENV;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RestartArgs {
    pub(super) config: Option<String>,
    pub(super) with_overrides: Vec<String>,
    pub(super) skip_welcome: bool,
    pub(super) help: bool,
}

pub(super) fn run_restart(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_restart_args(args)?;
    if parsed.help {
        print_restart_help();
        return Ok(0);
    }

    let session_to_kill = current_zellij_session();
    let restart_file = create_sidebar_bootstrap_file(
        "restart",
        &std::env::current_dir().map_err(|source| {
            CoreError::io(
                "restart_cwd",
                "Could not read the current working directory.",
                "cd into a valid directory, then retry.",
                ".",
                source,
            )
        })?,
    )?;

    let is_yzxterm = std::env::var_os("YAZELIX_TERMINAL").is_some();
    if is_yzxterm {
        println!("🔄 Restarting Yazelix...");
    } else {
        println!("🔄 Restarting Yazelix (opening new window)...");
    }

    let runtime_dir = runtime_dir_from_env()?;
    let inherited_config_override = config_override_from_env();
    let config_override = prepare_session_config_override(
        parsed
            .config
            .as_deref()
            .or(inherited_config_override.as_deref()),
        &parsed.with_overrides,
    )?;
    let launcher = current_runtime_yzx_launcher(&runtime_dir);
    let mut restart_extra_env = vec![
        (
            SIDEBAR_BOOTSTRAP_CWD_ENV.to_string(),
            Some(restart_file.to_string_lossy().into_owned()),
        ),
        (
            "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT".to_string(),
            Some("1".to_string()),
        ),
    ];
    if parsed.skip_welcome {
        restart_extra_env.push((
            "YAZELIX_STARTUP_PROFILE_SKIP_WELCOME".to_string(),
            Some("true".to_string()),
        ));
    }
    restart_extra_env.extend(config_override_extra_env(config_override.as_deref()));

    let output = command_output_with_overrides(
        &[
            launcher.to_string_lossy().into_owned(),
            "launch".to_string(),
        ],
        None,
        &std::env::current_dir().map_err(|source| {
            CoreError::io(
                "restart_cwd",
                "Could not read the current working directory.",
                "cd into a valid directory, then retry.",
                ".",
                source,
            )
        })?,
        RUNTIME_RELAUNCH_CLEARED_ENV_KEYS,
        &restart_extra_env,
        "restart_launch",
        "Retry the restart from a working Yazelix install, or relaunch manually with `yzx launch`.",
    )?;
    if !output.status.success() {
        print_completed_output(&output);
        eprintln!("❌ Failed to relaunch Yazelix through the current runtime launcher.");
        return Ok(output.status.code().unwrap_or(1));
    }

    thread::sleep(Duration::from_secs(1));
    kill_zellij_session(session_to_kill.as_deref());
    Ok(0)
}

pub(super) fn parse_restart_args(args: &[String]) -> Result<RestartArgs, CoreError> {
    let mut parsed = RestartArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "help" => parsed.help = true,
            "--skip" | "-s" => parsed.skip_welcome = true,
            "--config" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx restart --config. Try `yzx restart --help`.",
                    )
                })?;
                if parsed.config.is_some() {
                    return Err(CoreError::usage(
                        "yzx restart accepts at most one --config override.",
                    ));
                }
                parsed.config = Some(resolve_cli_config_override(value)?);
            }
            "--with" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx restart --with. Try `yzx restart --help`.",
                    )
                })?;
                parsed.with_overrides.push(value.clone());
            }
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx restart: {other}. Try `yzx restart --help`."
                )));
            }
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown yzx restart argument: {other}. Try `yzx restart --help`."
                )));
            }
        }
        index += 1;
    }
    Ok(parsed)
}

fn print_restart_help() {
    println!("Restart the current Yazelix window");
    println!();
    println!("Usage:");
    println!("  yzx restart [-s | --skip] [--config <file>] [--with key=value]");
    println!();
    println!("Options:");
    println!("  -s, --skip    Skip the welcome screen for the restarted window");
    println!("  --config      Use an alternate complete settings.jsonc for the restarted window");
    println!("  --with        Apply one session-only settings override, repeatable");
}

fn current_zellij_session() -> Option<String> {
    if let Ok(session) = std::env::var("ZELLIJ_SESSION_NAME") {
        let trimmed = session.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let output = Command::new("zellij").arg("list-sessions").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if !line.contains("current") {
            continue;
        }
        let cleaned_line = strip_ansi(line);
        let clean = cleaned_line.trim_start_matches('>').trim();
        let token = clean
            .split_whitespace()
            .find(|token| !token.is_empty())
            .map(str::to_string);
        if token.is_some() {
            return token;
        }
    }
    None
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn kill_zellij_session(session_name: Option<&str>) {
    let Some(session_name) = session_name.map(str::trim).filter(|name| !name.is_empty()) else {
        println!("⚠️  No Zellij session detected to close");
        return;
    };
    println!("Killing Zellij session: {session_name}");
    let _ = Command::new("zellij")
        .args(["kill-session", session_name])
        .status();
}
