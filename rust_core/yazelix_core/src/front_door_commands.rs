//! Rust-owned front-door public commands for `yzx tutor`, `yzx screen`, and `yzx whats_new`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    read_yazelix_version_from_runtime, runtime_dir_from_env, state_dir_from_env,
};
use crate::front_door_render::{
    GameOfLifeCellStyle, play_welcome_style_with_cell_style, run_screen_surface_with_cell_style,
};
use crate::session_facts::compute_session_facts_from_env;
use crate::upgrade_summary::show_current_upgrade_summary;
use std::process::Command;
use std::time::Duration;

const ANSI_RESET: &str = "\u{1b}[0m";
const ANSI_CYAN_BOLD: &str = "\u{1b}[1;36m";
const ANSI_YELLOW_BOLD: &str = "\u{1b}[1;33m";
const ANSI_WHITE: &str = "\u{1b}[37m";

#[derive(Debug, Clone, PartialEq, Eq)]
enum TutorView {
    Overview,
    Begin,
    List,
    Lesson(TutorLesson),
    Helix,
    Nushell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TutorLesson {
    Workspace,
    Discovery,
    ToolTutors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TutorLessonSpec {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
}

const TUTOR_LESSONS: &[TutorLessonSpec] = &[
    TutorLessonSpec {
        id: "workspace",
        title: "Workspace roots and managed panes",
        summary: "Practice the current-tab workspace model, Yazi handoff, and fresh project tabs",
    },
    TutorLessonSpec {
        id: "discovery",
        title: "Command and key discovery",
        summary: "Use the command palette, key tables, and doctor output without memorizing everything",
    },
    TutorLessonSpec {
        id: "tool_tutors",
        title: "Helix and Nushell tutors",
        summary: "Jump from Yazelix-specific guidance into the upstream editor and shell tutors",
    },
];

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
        TutorView::Overview => {
            print!("{}", render_yazelix_tutor_overview());
            Ok(0)
        }
        TutorView::Begin => {
            print!("{}", render_tutor_lesson(TutorLesson::Workspace));
            Ok(0)
        }
        TutorView::List => {
            print!("{}", render_tutor_list());
            Ok(0)
        }
        TutorView::Lesson(lesson) => {
            print!("{}", render_tutor_lesson(lesson));
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
        [] => TutorView::Overview,
        ["begin"] => TutorView::Begin,
        ["list"] => TutorView::List,
        [lesson] if tutor_lesson_from_id(lesson).is_some() => {
            TutorView::Lesson(tutor_lesson_from_id(lesson).unwrap())
        }
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
    let facts = compute_session_facts_from_env()?;
    let raw = facts.game_of_life_cell_style.as_str();
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
    println!("Show the Yazelix guided tutor");
    println!();
    println!("Usage:");
    println!("  yzx tutor");
    println!("  yzx tutor begin");
    println!("  yzx tutor list");
    println!("  yzx tutor workspace");
    println!("  yzx tutor discovery");
    println!("  yzx tutor tool_tutors");
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
    println!("  boids_predator");
    println!("  boids_schools");
    println!("  mandelbrot");
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

fn render_yazelix_tutor_overview() -> String {
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
        "Need a session first? Launch with {} or start in the current terminal with {}.",
        command_label("yzx launch"),
        command_label("yzx enter")
    ));
    lines.push(format!(
        "1. Start the guided flow with {}.",
        command_label("yzx tutor begin")
    ));
    lines.push(format!(
        "2. See every short lesson with {}.",
        command_label("yzx tutor list")
    ));
    lines.push(format!(
        "3. Learn the workspace-critical bindings with {}.",
        command_label("yzx keys")
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

fn tutor_lesson_from_id(id: &str) -> Option<TutorLesson> {
    match id {
        "workspace" => Some(TutorLesson::Workspace),
        "discovery" => Some(TutorLesson::Discovery),
        "tool_tutors" => Some(TutorLesson::ToolTutors),
        _ => None,
    }
}

fn tutor_lesson_spec(lesson: TutorLesson) -> &'static TutorLessonSpec {
    let id = match lesson {
        TutorLesson::Workspace => "workspace",
        TutorLesson::Discovery => "discovery",
        TutorLesson::ToolTutors => "tool_tutors",
    };
    TUTOR_LESSONS
        .iter()
        .find(|spec| spec.id == id)
        .expect("lesson spec exists")
}

fn render_tutor_list() -> String {
    let mut lines = Vec::new();
    lines.push(heading("Yazelix tutor lessons"));
    lines.push(String::new());
    for (index, lesson) in TUTOR_LESSONS.iter().enumerate() {
        lines.push(format!(
            "{}. {} {}",
            index + 1,
            command_label(&format!("yzx tutor {}", lesson.id)),
            lesson.title
        ));
        lines.push(format!("   {}", lesson.summary));
    }
    lines.push(String::new());
    lines.push(format!("Start with {}.", command_label("yzx tutor begin")));
    format!("{}\n", lines.join("\n"))
}

fn render_tutor_lesson(lesson: TutorLesson) -> String {
    match lesson {
        TutorLesson::Workspace => render_workspace_lesson(),
        TutorLesson::Discovery => render_discovery_lesson(),
        TutorLesson::ToolTutors => render_tool_tutors_lesson(),
    }
}

fn render_lesson_header(lesson: TutorLesson) -> Vec<String> {
    let spec = tutor_lesson_spec(lesson);
    vec![
        heading(&format!("Yazelix tutor: {}", spec.title)),
        String::new(),
        spec.summary.to_string(),
        String::new(),
    ]
}

fn render_workspace_lesson() -> String {
    let mut lines = render_lesson_header(TutorLesson::Workspace);
    lines.push(heading("Learn"));
    lines.push("The current tab has a workspace root. Yazelix uses that root for new panes, popup commands, and managed editor/sidebar coordination.".to_string());
    lines.push("Opening a file from Yazi into the managed editor also moves that tab's workspace root to the file's directory.".to_string());
    lines.push(String::new());
    lines.push(heading("Mini quest"));
    lines.push(format!(
        "1. Run {} to open this directory as a fresh workspace tab.",
        command_label("yzx warp .")
    ));
    lines.push(format!(
        "2. Run {} and check the Yazi section before opening a file from the sidebar.",
        command_label("yzx keys yazi")
    ));
    lines.push(format!(
        "3. Run {} and confirm the workspace facts match the tab you expect.",
        command_label("yzx status")
    ));
    lines.push(String::new());
    lines.push(format!(
        "Next lesson: {}.",
        command_label("yzx tutor discovery")
    ));
    format!("{}\n", lines.join("\n"))
}

fn render_discovery_lesson() -> String {
    let mut lines = render_lesson_header(TutorLesson::Discovery);
    lines.push(heading("Learn"));
    lines.push(
        "Use command surfaces when you know what you want, and discovery surfaces when you do not."
            .to_string(),
    );
    lines.push(String::new());
    lines.push(heading("Mini quest"));
    lines.push(format!(
        "1. Run {} for the command reference.",
        command_label("yzx help")
    ));
    lines.push(format!(
        "2. Run {} for keybinding discovery.",
        command_label("yzx keys")
    ));
    lines.push(format!(
        "3. Run {} for fuzzy command discovery, or press {} inside Yazelix.",
        command_label("yzx menu"),
        command_label("Alt+Shift+M")
    ));
    lines.push(format!(
        "4. Run {} when the current runtime or config feels wrong.",
        command_label("yzx doctor")
    ));
    lines.push(String::new());
    lines.push(format!(
        "Next lesson: {}.",
        command_label("yzx tutor tool_tutors")
    ));
    format!("{}\n", lines.join("\n"))
}

fn render_tool_tutors_lesson() -> String {
    let mut lines = render_lesson_header(TutorLesson::ToolTutors);
    lines.push(heading("Learn"));
    lines.push("Yazelix owns the workspace integration. Helix and Nushell still have their own deep learning flows.".to_string());
    lines.push(String::new());
    lines.push(heading("Mini quest"));
    lines.push(format!(
        "1. Run {} to practice Helix inside the editor's own tutor.",
        command_label("yzx tutor hx")
    ));
    lines.push(format!(
        "2. Run {} to practice Nushell in Nushell's own tutor.",
        command_label("yzx tutor nu")
    ));
    lines.push(format!(
        "3. Return to {} when you want the Yazelix workspace path again.",
        command_label("yzx tutor list")
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
        assert_eq!(
            parse_tutor_args(&["begin".into()]).unwrap(),
            TutorArgs {
                view: TutorView::Begin,
                help: false,
            }
        );
        assert_eq!(
            parse_tutor_args(&["list".into()]).unwrap(),
            TutorArgs {
                view: TutorView::List,
                help: false,
            }
        );
        assert_eq!(
            parse_tutor_args(&["workspace".into()]).unwrap(),
            TutorArgs {
                view: TutorView::Lesson(TutorLesson::Workspace),
                help: false,
            }
        );
        assert!(parse_tutor_args(&["weird".into()]).is_err());
    }

    // Defends: the front-door tutor root still prints the managed-workspace guidance instead of drifting through wrapper churn.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn tutor_root_output_keeps_guided_overview_copy() {
        let output = render_yazelix_tutor_overview();
        assert!(output.contains("Yazelix tutor"));
        assert!(output.contains("yzx tutor begin"));
        assert!(output.contains("yzx tutor list"));
        assert!(output.contains("yzx menu"));
        assert!(output.contains("Alt+Shift+M"));
        assert!(output.contains("README.md"));
    }

    // Defends: the guided tutor has concrete lesson entrypoints and a workspace mini quest instead of only a flat overview.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn tutor_lessons_include_list_and_workspace_mini_quest() {
        let list = render_tutor_list();
        assert!(list.contains("yzx tutor workspace"));
        assert!(list.contains("yzx tutor discovery"));
        assert!(list.contains("yzx tutor tool_tutors"));

        let lesson = render_tutor_lesson(TutorLesson::Workspace);
        assert!(lesson.contains("Mini quest"));
        assert!(lesson.contains("yzx warp ."));
        assert!(lesson.contains("yzx keys yazi"));
        assert!(lesson.contains("yzx status"));
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
