//! Rust-owned front-door public commands for `yzx tutor`, `yzx screen`, and `yzx whats_new`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, load_normalized_config_for_control,
    read_yazelix_version_from_runtime, runtime_dir_from_env, state_dir_from_env,
};
use crate::front_door_render::{
    GameOfLifeCellStyle, play_welcome_style_with_cell_style, run_screen_surface_with_cell_style,
};
use crate::upgrade_summary::show_current_upgrade_summary;
use std::process::Command;
use std::time::Duration;

const ANSI_RESET: &str = "\u{1b}[0m";
const ANSI_CYAN_BOLD: &str = "\u{1b}[1;36m";
const ANSI_YELLOW_BOLD: &str = "\u{1b}[1;33m";
const ANSI_WHITE: &str = "\u{1b}[37m";

#[derive(Debug, Clone, PartialEq, Eq)]
enum TutorView {
    Yazelix,
    Helix,
    Nushell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TutorArgs {
    view: TutorView,
    help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScreenArgs {
    style: Option<String>,
    help: bool,
    internal_welcome: bool,
    duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WhatsNewArgs {
    help: bool,
}

pub fn run_yzx_tutor(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_tutor_args(args)?;
    if parsed.help {
        print_tutor_help();
        return Ok(0);
    }

    match parsed.view {
        TutorView::Yazelix => {
            print!("{}", render_yazelix_tutor());
            Ok(0)
        }
        TutorView::Helix => run_external_command("hx", &["--tutor"], "Helix"),
        TutorView::Nushell => run_external_command("nu", &["-c", "tutor"], "Nushell"),
    }
}

pub fn run_yzx_screen(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_screen_args(args)?;
    if parsed.help {
        print_screen_help();
        return Ok(0);
    }
    if parsed.internal_welcome {
        let style = parsed.style.as_deref().unwrap_or("logo");
        return run_internal_welcome_screen(style, Duration::from_millis(parsed.duration_ms));
    }
    run_screen_surface_with_cell_style(
        parsed.style.as_deref(),
        configured_game_of_life_cell_style()?,
    )
}

pub fn run_internal_welcome_screen(style: &str, duration: Duration) -> Result<i32, CoreError> {
    play_welcome_style_with_cell_style(style, duration, configured_game_of_life_cell_style()?)?;
    Ok(0)
}

pub fn run_yzx_whats_new(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_whats_new_args(args)?;
    if parsed.help {
        print_whats_new_help();
        return Ok(0);
    }

    let runtime_dir = runtime_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let version = read_yazelix_version_from_runtime(&runtime_dir)?;
    let report = show_current_upgrade_summary(&runtime_dir, &state_dir, &version, true)?;
    println!("{}", report.report.output);
    Ok(0)
}

fn parse_tutor_args(args: &[String]) -> Result<TutorArgs, CoreError> {
    let mut help = false;
    let mut tokens = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other => tokens.push(other),
        }
    }

    let view = match tokens.as_slice() {
        [] => TutorView::Yazelix,
        ["hx"] | ["helix"] => TutorView::Helix,
        ["nu"] | ["nushell"] => TutorView::Nushell,
        [other] => {
            return Err(CoreError::usage(format!(
                "Unknown yzx tutor target: {other}. Try `yzx tutor --help`."
            )));
        }
        _ => return Err(CoreError::usage("Unexpected arguments for yzx tutor.")),
    };

    Ok(TutorArgs { view, help })
}

fn parse_screen_args(args: &[String]) -> Result<ScreenArgs, CoreError> {
    let mut help = false;
    let mut style = None;
    let mut internal_welcome = false;
    let mut duration_ms = 1000u64;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            "--internal-welcome" => internal_welcome = true,
            "--duration-ms" => {
                let Some(raw) = iter.next() else {
                    return Err(CoreError::usage("Missing value after --duration-ms."));
                };
                duration_ms = raw.parse::<u64>().map_err(|_| {
                    CoreError::usage(format!("Invalid --duration-ms value `{raw}`."))
                })?;
            }
            other if style.is_none() => style = Some(other.to_string()),
            _ => {
                return Err(CoreError::usage(
                    "Unexpected arguments for yzx screen. Try `yzx screen --help`.",
                ));
            }
        }
    }
    Ok(ScreenArgs {
        style,
        help,
        internal_welcome,
        duration_ms,
    })
}

fn parse_whats_new_args(args: &[String]) -> Result<WhatsNewArgs, CoreError> {
    let mut help = false;
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx whats_new: {other}. Try `yzx whats_new --help`."
                )));
            }
        }
    }
    Ok(WhatsNewArgs { help })
}

fn configured_game_of_life_cell_style() -> Result<GameOfLifeCellStyle, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let normalized =
        load_normalized_config_for_control(&runtime_dir, &config_dir, config_override.as_deref())?;
    let raw = normalized
        .get("game_of_life_cell_style")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("full_block");
    GameOfLifeCellStyle::parse(raw).map_err(|err| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_game_of_life_cell_style",
            format!("Invalid Game of Life cell style `{}`.", err.normalized()),
            "Use `full_block` or `dotted`.",
            serde_json::json!({ "style": err.normalized() }),
        )
    })
}

fn print_tutor_help() {
    println!("Show the Yazelix guided overview");
    println!();
    println!("Usage:");
    println!("  yzx tutor");
    println!("  yzx tutor hx");
    println!("  yzx tutor helix");
    println!("  yzx tutor nu");
    println!("  yzx tutor nushell");
}

fn print_screen_help() {
    println!("Show an animated Yazelix full-terminal screen");
    println!();
    println!("Usage:");
    println!("  yzx screen [STYLE]");
    println!();
    println!("Styles:");
    println!("  logo");
    println!("  boids");
    println!("  mandelbrot");
    println!("  mandelbrot_seahorse");
    println!("  game_of_life_gliders");
    println!("  game_of_life_oscillators");
    println!("  game_of_life_bloom");
    println!("  random");
    println!();
    println!("Notes:");
    println!("  `static` remains startup-only and is rejected here");
    println!("  Press any key to exit");
}

fn print_whats_new_help() {
    println!("Show the current Yazelix upgrade summary");
    println!();
    println!("Usage:");
    println!("  yzx whats_new");
}

fn heading(text: &str) -> String {
    format!("{ANSI_CYAN_BOLD}{text}{ANSI_RESET}")
}

fn accent(text: &str) -> String {
    format!("{ANSI_YELLOW_BOLD}{text}{ANSI_RESET}")
}

fn command_label(text: &str) -> String {
    format!("{ANSI_WHITE}{text}{ANSI_RESET}")
}

fn render_yazelix_tutor() -> String {
    let mut lines = Vec::new();
    lines.push(heading("Yazelix tutor"));
    lines.push(String::new());
    lines.push(
        "Yazelix is a managed terminal workspace built around Zellij, Yazi, and Helix.".to_string(),
    );
    lines.push("The important unit is the current tab workspace root: managed actions use that directory unless a tool is doing something more specific.".to_string());
    lines.push(String::new());
    lines.push(heading("Start here"));
    lines.push(format!(
        "1. Launch a session with {} or start it in the current terminal with {}.",
        command_label("yzx launch"),
        command_label("yzx enter")
    ));
    lines.push(format!(
        "2. Learn the workspace-critical bindings with {}.",
        command_label("yzx keys")
    ));
    lines.push(format!(
        "3. Use {} when you want to retarget the current tab workspace root manually. Opening a file from Yazi into the managed editor also moves the workspace root to that file's directory.",
        command_label("yzx cwd <dir>")
    ));
    lines.push(format!(
        "4. Use {} for fuzzy command discovery (or {} inside Yazelix) and {} when behavior looks wrong.",
        command_label("yzx menu"),
        command_label("Alt+Shift+M"),
        command_label("yzx doctor")
    ));
    lines.push(String::new());
    lines.push(heading("Mental model"));
    lines.push(format!(
        "{} Yazelix treats the editor/sidebar flow as a coordinated workspace, not just a pile of unrelated panes.",
        accent("Managed panes:")
    ));
    lines.push(format!(
        "{} The current tab root drives new panes, popup commands, and workspace-aware actions.",
        accent("Directory flow:")
    ));
    lines.push(format!(
        "{} {} is the command reference, {} is the keybinding surface, and {} is the guided overview.",
        accent("Discoverability:"),
        command_label("yzx help"),
        command_label("yzx keys"),
        command_label("yzx tutor")
    ));
    lines.push(String::new());
    lines.push(heading("Next steps"));
    lines.push(format!(
        "{} {}",
        accent("Helix tutor:"),
        command_label("yzx tutor hx")
    ));
    lines.push(format!(
        "{} {}",
        accent("Nushell tutor:"),
        command_label("yzx tutor nu")
    ));
    lines.push(format!(
        "{} {}",
        accent("Command reference:"),
        command_label("yzx help")
    ));
    lines.push(format!(
        "{} {}",
        accent("Project overview:"),
        command_label("README.md")
    ));
    format!("{}\n", lines.join("\n"))
}

fn run_external_command(command: &str, args: &[&str], label: &str) -> Result<i32, CoreError> {
    let status = Command::new(command).args(args).status().map_err(|_| {
        let remediation = match command {
            "hx" => "Install Helix in the active Yazelix environment, then retry `yzx tutor hx`.",
            "nu" => "Install Nushell in the active Yazelix environment, then retry `yzx tutor nu`.",
            _ => "Install the required command, then retry.",
        };
        CoreError::classified(
            ErrorClass::Runtime,
            "missing_tutor_command",
            format!("Required command not found for {label}: {command}"),
            remediation,
            serde_json::json!({ "command": command }),
        )
    })?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default
    // Defends: the Rust tutor parser preserves root/help/alias parity instead of reviving ambiguous Nu routing for the public front-door family.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parse_tutor_args_preserves_alias_parity() {
        assert_eq!(
            parse_tutor_args(&["helix".into()]).unwrap(),
            TutorArgs {
                view: TutorView::Helix,
                help: false,
            }
        );
        assert_eq!(
            parse_tutor_args(&["nushell".into()]).unwrap(),
            TutorArgs {
                view: TutorView::Nushell,
                help: false,
            }
        );
        assert!(parse_tutor_args(&["weird".into()]).is_err());
    }

    // Defends: the front-door tutor root still prints the managed-workspace guidance instead of drifting through wrapper churn.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn tutor_root_output_keeps_guided_overview_copy() {
        let output = render_yazelix_tutor();
        assert!(output.contains("Yazelix tutor"));
        assert!(output.contains("yzx menu"));
        assert!(output.contains("Alt+Shift+M"));
        assert!(output.contains("README.md"));
    }

    // Defends: the Rust-owned `yzx screen` parser keeps the public one-style surface while reserving the welcome-only internal flags for startup callers.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parse_screen_args_keeps_internal_welcome_flags_out_of_the_public_surface() {
        assert_eq!(
            parse_screen_args(&["logo".into()]).unwrap(),
            ScreenArgs {
                style: Some("logo".to_string()),
                help: false,
                internal_welcome: false,
                duration_ms: 1000,
            }
        );
        assert_eq!(
            parse_screen_args(&[
                "--internal-welcome".into(),
                "--duration-ms".into(),
                "750".into(),
                "boids".into(),
            ])
            .unwrap(),
            ScreenArgs {
                style: Some("boids".to_string()),
                help: false,
                internal_welcome: true,
                duration_ms: 750,
            }
        );
        assert!(parse_screen_args(&["logo".into(), "extra".into()]).is_err());
    }
}
