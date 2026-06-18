//! Rust-owned front-door public commands for `yzx tutor`, `yzx screen`, and `yzx whats_new`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    read_runtime_identity_from_runtime, read_yazelix_version_from_runtime, runtime_dir_from_env,
    state_dir_from_env,
};
use crate::front_door_render::{
    GameOfLifeCellStyle, play_welcome_style_with_runtime_dir, run_screen_surface_with_runtime_dir,
};
use crate::require_runtime_component_enabled;
use crate::session_facts::compute_session_facts_from_env;
use crate::tutor_document;
use crate::upgrade_summary::{RuntimeSnapshotContext, show_known_changes_since_installed_runtime};
use std::process::Command;
use std::time::Duration;

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

const TUTOR_OVERVIEW_MARKDOWN: &str = r#"# Yazelix tutor

Yazelix is a managed terminal workspace built around Zellij, Yazi, and Helix.

The important unit is the current tab workspace root: managed actions use that directory unless a tool is doing something more specific.

## Start here

Need a session first? Launch with `yzx launch` or start in the current terminal with `yzx enter`.

1. Start the guided flow with `yzx tutor begin`.
2. See every short lesson with `yzx tutor list`.
3. Learn the workspace-critical bindings with `yzx keys`.
4. Use `yzx menu` for fuzzy command discovery (or `Alt+Shift+M` inside Yazelix) and `yzx doctor` when behavior looks wrong.

## Mental model

**Managed panes:** Yazelix treats the editor/sidebar flow as a coordinated workspace, not just a pile of unrelated panes.

**Directory flow:** The current tab root drives new panes, popup commands, and workspace-aware actions.

**Discoverability:** `yzx help` is the command reference, `yzx keys` is the keybinding surface, and `yzx tutor` is the guided overview.

## Next steps

**Helix tutor:** `yzx tutor hx`

**Nushell tutor:** `yzx tutor nu`

**Command reference:** `yzx help`

**Project overview:** `README.md`
"#;

const WORKSPACE_LESSON_MARKDOWN: &str = r#"# Yazelix tutor: Workspace roots and managed panes

Practice the current-tab workspace model, Yazi handoff, and fresh project tabs.

## Learn

Start Yazelix from the folder you want to work in. That folder becomes the current tab workspace root for new panes, popup commands, and managed editor/sidebar coordination.

Opening a file from Yazi into the managed editor also moves that tab's workspace root to the file's directory.

## Mini quest

1. If this session is not in the folder you want, leave it and start again from that folder with `yzx enter`, or use `yzx launch --path <dir>`.
2. Run `yzx keys` and find the workspace actions.
3. Use the managed Yazi sidebar to open a file in the editor.

Next lesson: `yzx tutor discovery`.
"#;

const DISCOVERY_LESSON_MARKDOWN: &str = r#"# Yazelix tutor: Command and key discovery

Use the command palette, key tables, and doctor output without memorizing everything.

## Learn

Use command surfaces when you know what you want, and discovery surfaces when you do not.

## Mini quest

1. Run `yzx help` for the command reference.
2. Run `yzx keys` for keybinding discovery.
3. Run `yzx menu` for fuzzy command discovery, or press `Alt+Shift+M` inside Yazelix.
4. Run `yzx doctor` when the current runtime or config feels wrong.

Next lesson: `yzx tutor tool_tutors`.
"#;

const TOOL_TUTORS_LESSON_MARKDOWN: &str = r#"# Yazelix tutor: Helix and Nushell tutors

Jump from Yazelix-specific guidance into the upstream editor and shell tutors.

## Learn

Yazelix owns the workspace integration. Helix and Nushell still have their own deep learning flows.

## Mini quest

1. Run `yzx tutor hx` to practice Helix inside the editor's own tutor.
2. Run `yzx tutor nu` to practice Nushell in Nushell's own tutor.
3. Return to `yzx tutor list` when you want the Yazelix workspace path again.
"#;

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
    let runtime_dir = runtime_dir_from_env()?;
    require_runtime_component_enabled(&runtime_dir, "screen", "yzx screen")?;
    if parsed.internal_welcome {
        let style = parsed.style.as_deref().unwrap_or("logo");
        return run_internal_welcome_screen(
            style,
            Duration::from_millis(parsed.duration_ms),
            &runtime_dir,
        );
    }
    run_screen_surface_with_runtime_dir(
        parsed.style.as_deref(),
        configured_game_of_life_cell_style()?,
        &runtime_dir,
    )
}

pub fn run_internal_welcome_screen(
    style: &str,
    duration: Duration,
    runtime_dir: &std::path::Path,
) -> Result<i32, CoreError> {
    play_welcome_style_with_runtime_dir(
        style,
        duration,
        configured_game_of_life_cell_style()?,
        runtime_dir,
    )?;
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
    let runtime_identity = read_runtime_identity_from_runtime(&runtime_dir)?;
    let snapshot = RuntimeSnapshotContext::from_runtime_identity(&runtime_identity);
    let report = show_known_changes_since_installed_runtime(
        &runtime_dir,
        &state_dir,
        &version,
        Some(&snapshot),
        true,
    )?;
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
    println!("Show Yazelix changes since the installed runtime");
    println!();
    println!("Usage:");
    println!("  yzx whats_new");
}

fn command_label(text: &str) -> String {
    tutor_document::render_code_label(text)
}

fn render_yazelix_tutor_overview() -> String {
    render_tutor_markdown(TUTOR_OVERVIEW_MARKDOWN)
}

fn tutor_lesson_from_id(id: &str) -> Option<TutorLesson> {
    match id {
        "workspace" => Some(TutorLesson::Workspace),
        "discovery" => Some(TutorLesson::Discovery),
        "tool_tutors" => Some(TutorLesson::ToolTutors),
        _ => None,
    }
}

fn render_tutor_list() -> String {
    let mut markdown = String::from("# Yazelix tutor lessons\n\n");
    for (index, lesson) in TUTOR_LESSONS.iter().enumerate() {
        markdown.push_str(&format!(
            "{}. `yzx tutor {}` {}  \n",
            index + 1,
            lesson.id,
            lesson.title
        ));
        markdown.push_str(&format!("   {}\n", lesson.summary));
    }
    markdown.push_str("\nStart with `yzx tutor begin`.\n");
    render_tutor_markdown(&markdown)
}

fn render_tutor_lesson(lesson: TutorLesson) -> String {
    match lesson {
        TutorLesson::Workspace => render_workspace_lesson(),
        TutorLesson::Discovery => render_discovery_lesson(),
        TutorLesson::ToolTutors => render_tool_tutors_lesson(),
    }
}

fn render_workspace_lesson() -> String {
    render_tutor_markdown(WORKSPACE_LESSON_MARKDOWN)
}

fn render_discovery_lesson() -> String {
    render_tutor_markdown(DISCOVERY_LESSON_MARKDOWN)
}

fn render_tool_tutors_lesson() -> String {
    render_tutor_markdown(TOOL_TUTORS_LESSON_MARKDOWN)
}

fn render_tutor_markdown(markdown: &str) -> String {
    tutor_document::render_tutor_markdown(markdown).expect("bundled tutor Markdown is supported")
}

fn run_external_command(command: &str, args: &[&str], label: &str) -> Result<i32, CoreError> {
    let status = Command::new(command).args(args).status().map_err(|_| {
        let remediation = match command {
            "hx" => format!(
                "Install Helix in the active Yazelix environment, then retry {}.",
                command_label("yzx tutor hx")
            ),
            "nu" => format!(
                "Install Nushell in the active Yazelix environment, then retry {}.",
                command_label("yzx tutor nu")
            ),
            _ => "Install the required command, then retry.".to_string(),
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
    #[test]
    fn tutor_lessons_include_list_and_workspace_mini_quest() {
        let list = render_tutor_list();
        assert!(list.contains("yzx tutor workspace"));
        assert!(list.contains("yzx tutor discovery"));
        assert!(list.contains("yzx tutor tool_tutors"));

        let lesson = render_tutor_lesson(TutorLesson::Workspace);
        assert!(lesson.contains("Mini quest"));
        assert!(lesson.contains("yzx enter"));
        assert!(lesson.contains("yzx launch --path <dir>"));
        assert!(lesson.contains("yzx keys"));
        assert!(!lesson.contains("yzx cwd"));
    }

    // Defends: every bundled tutor Markdown document stays inside the supported terminal-renderer subset.
    #[test]
    fn bundled_tutor_markdown_documents_render_without_panics() {
        assert!(render_yazelix_tutor_overview().contains("Yazelix tutor"));
        assert!(render_tutor_list().contains("yzx tutor tool_tutors"));
        for lesson in [
            TutorLesson::Workspace,
            TutorLesson::Discovery,
            TutorLesson::ToolTutors,
        ] {
            let output = render_tutor_lesson(lesson);
            assert!(output.contains("Mini quest"));
        }
    }

    // Defends: the Rust-owned `yzx screen` parser keeps the public one-style surface while reserving the welcome-only internal flags for startup callers.
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
