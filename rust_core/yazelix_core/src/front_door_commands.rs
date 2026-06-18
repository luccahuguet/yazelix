//! Rust-owned front-door public commands for `yzx tutor`, `yzx screen`, and `yzx whats_new`.

use crate::action_registry::{
    DEFAULT_INFORMATION_POPUP_KEYS, display_yazi_keys, display_zellij_keys,
    yazi_action_by_local_id, zellij_action_by_local_id, zellij_native_keybinding_by_local_id,
};
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
    scope: &'static str,
    outcome: &'static str,
    escape_hatch: &'static str,
}

const TUTOR_LESSONS: &[TutorLessonSpec] = &[
    TutorLessonSpec {
        id: "workspace",
        title: "Workspace roots and managed panes",
        summary: "Open a project, move between editor and sidebars, and keep the tab root clear",
        scope: "Current tab",
        outcome: "You can start Yazelix in the right directory, open files through Yazi, reveal the editor file, and move between the managed editor and sidebars.",
        escape_hatch: "Use the editor/sidebar focus keys to get back to a known pane, then rerun `yzx tutor list`.",
    },
    TutorLessonSpec {
        id: "discovery",
        title: "Command and key discovery",
        summary: "Find commands, popups, key tables, and runtime checks without memorizing the map",
        scope: "Current Yazelix window",
        outcome: "You can open the command menu, inspect useful popups, check the live key table, and run doctor when the runtime feels wrong.",
        escape_hatch: "Use the same popup key to focus or close a managed popup. Tool-local exits such as `q` or `Esc` stay inside the tool.",
    },
    TutorLessonSpec {
        id: "tool_tutors",
        title: "Helix and Nushell tutors",
        summary: "Switch from Yazelix guidance into the editor and shell tutors",
        scope: "Editor or shell pane",
        outcome: "You can start the Helix and Nushell tutors from Yazelix, then return to the Yazelix lesson list.",
        escape_hatch: "Quit Helix with `:q`; leave a shell prompt with `Ctrl+d` or `exit`, then run `yzx tutor list`.",
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
    let menu_key = zellij_key("menu");
    let info_key = information_popup_key();
    let markdown = format!(
        r#"# Yazelix tutor

Yazelix is a managed terminal workspace built around Zellij, Yazi, and Helix.

The current tab workspace root matters most. Managed actions use that directory unless a tool chooses a file or directory.

## Start here

1. **Run in shell:** Start a session with `yzx launch` or enter the current terminal with `yzx enter`.
2. **Run in shell or Yazelix:** Start the guided path with `yzx tutor begin`.
3. **Run in shell or Yazelix:** List the lessons with `yzx tutor list`.
4. **Inside Yazelix:** Press `{menu_key}` or run `yzx menu` for the command menu; press `{info_key}` for the information popup.
5. **Run in shell or Yazelix:** Use `yzx keys` for live bindings and `yzx doctor` when behavior looks wrong.

## Key notation

Yazelix writes key chords as `Alt+Shift+M`, `Ctrl+y`, and `Ctrl+Shift+Y`. `yzx keys` shows the same defaults and any remaps.

## Lessons

1. `yzx tutor workspace` builds the current-tab workspace model.
2. `yzx tutor discovery` shows command, popup, and recovery surfaces.
3. `yzx tutor tool_tutors` sends you to the Helix and Nushell tutors.

## Mental model

**Managed panes:** Yazelix keeps editor, sidebars, popups, and agent panes connected.

**Directory flow:** The tab root drives new panes, popup commands, and workspace-aware actions.

**Discovery:** `yzx help` gives command syntax. `yzx keys` gives bindings. `yzx tutor` gives the guided path.
"#
    );
    render_tutor_markdown(&markdown)
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

fn tutor_lesson_index(lesson: TutorLesson) -> usize {
    let spec = tutor_lesson_spec(lesson);
    TUTOR_LESSONS
        .iter()
        .position(|candidate| candidate.id == spec.id)
        .expect("lesson spec is indexed")
        + 1
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
        markdown.push_str(&format!(
            "   Scope: {}. Outcome: {}\n",
            lesson.scope, lesson.summary
        ));
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
    let markdown = format!(
        r#"{header}

## Actions

1. **Run in shell:** Start in the project directory with `yzx enter`; use `yzx launch --path <dir>` when you need a separate window.
2. **Inside Yazelix:** Press `{left_focus}` to switch between the editor and left sidebar; press `{right_focus}` for the right agent sidebar.
3. **Inside Yazi:** Press `Enter` to open the selected file in the managed editor.
4. **Inside Yazi:** Press `{zoxide}` to jump with zoxide and open the target in the editor; press `{workspace_pane}` to open the selected directory in a workspace terminal pane.
5. **Inside the editor:** Press `{reveal}` to reveal the current file in Yazi; press `{fullscreen}` when one pane needs the whole screen.

Next lesson: `yzx tutor discovery`.
"#,
        header = render_lesson_intro(TutorLesson::Workspace),
        left_focus = zellij_key("toggle_editor_sidebar_focus"),
        right_focus = zellij_key("toggle_editor_right_sidebar_focus"),
        zoxide = yazi_key("open_zoxide_in_editor"),
        workspace_pane = yazi_key("open_directory_as_workspace_pane"),
        reveal = zellij_key("smart_reveal"),
        fullscreen = zellij_native_key("toggle_focus_fullscreen"),
    );
    render_tutor_markdown(&markdown)
}

fn render_discovery_lesson() -> String {
    let markdown = format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzx help` when you know the command name and need syntax.
2. **Run in shell or Yazelix:** Use `yzx keys` for live Yazelix, Yazi, Helix, and Nushell bindings.
3. **Inside Yazelix:** Press `{menu}` or run `yzx menu` to open command search.
4. **Inside Yazelix:** Press `{bottom_popup}` for the bottom popup, `{top_popup}` for the top popup, and `{config}` for the config UI.
5. **Inside Yazelix:** Press `{info}` for the keep-alive information popup.
6. **Run in shell or Yazelix:** Use `yzx doctor` when config, runtime, or generated workspace files look wrong.

Next lesson: `yzx tutor tool_tutors`.
"#,
        header = render_lesson_intro(TutorLesson::Discovery),
        menu = zellij_key("menu"),
        bottom_popup = zellij_key("bottom_popup"),
        top_popup = zellij_key("top_popup"),
        config = zellij_key("config"),
        info = information_popup_key(),
    );
    render_tutor_markdown(&markdown)
}

fn render_tool_tutors_lesson() -> String {
    let markdown = format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzx tutor hx` to launch `hx --tutor`.
2. **Inside Helix:** Leave the tutor with `:q`; use `{reveal}` in managed Helix sessions when you want Yazi to reveal the current file.
3. **Run in shell or Yazelix:** Use `yzx tutor nu` to launch the Nushell tutor.
4. **Inside Yazelix:** Press `{editor_sidebar}` to return to the editor/sidebar loop; press `{menu}` when you want command search again.
5. **Run in shell or Yazelix:** Return to `yzx tutor list` when you want the Yazelix path.
"#,
        header = render_lesson_intro(TutorLesson::ToolTutors),
        reveal = zellij_key("smart_reveal"),
        editor_sidebar = zellij_key("toggle_editor_sidebar_focus"),
        menu = zellij_key("menu"),
    );
    render_tutor_markdown(&markdown)
}

fn render_tutor_markdown(markdown: &str) -> String {
    tutor_document::render_tutor_markdown(markdown).expect("bundled tutor Markdown is supported")
}

fn render_lesson_intro(lesson: TutorLesson) -> String {
    let spec = tutor_lesson_spec(lesson);
    format!(
        r#"# {}. {}

{}

**Scope:** {}

**Outcome:** {}

**Escape hatch:** {}
"#,
        tutor_lesson_index(lesson),
        spec.title,
        spec.summary,
        spec.scope,
        spec.outcome,
        spec.escape_hatch
    )
}

fn zellij_key(local_id: &str) -> String {
    let action = zellij_action_by_local_id(local_id)
        .unwrap_or_else(|| panic!("missing Zellij tutor keybinding action: {local_id}"));
    format!("`{}`", display_zellij_keys(action.action.default_keys))
}

fn zellij_native_key(local_id: &str) -> String {
    let action = zellij_native_keybinding_by_local_id(local_id)
        .unwrap_or_else(|| panic!("missing native Zellij tutor keybinding action: {local_id}"));
    format!("`{}`", display_zellij_keys(action.action.default_keys))
}

fn yazi_key(local_id: &str) -> String {
    let action = yazi_action_by_local_id(local_id)
        .unwrap_or_else(|| panic!("missing Yazi tutor keybinding action: {local_id}"));
    format!("`{}`", display_yazi_keys(action.action.default_keys))
}

fn information_popup_key() -> String {
    format!("`{}`", display_zellij_keys(DEFAULT_INFORMATION_POPUP_KEYS))
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
        assert!(output.contains("Alt+Shift+I"));
        assert!(output.contains("Key notation"));
    }

    // Defends: the guided tutor has indexed lesson entrypoints and structured onboarding metadata instead of only a flat overview.
    #[test]
    fn tutor_lessons_include_indexed_metadata_and_workspace_actions() {
        let list = render_tutor_list();
        assert!(list.contains("1. `yzx tutor workspace`"));
        assert!(list.contains("2. `yzx tutor discovery`"));
        assert!(list.contains("3. `yzx tutor tool_tutors`"));
        assert!(list.contains("Scope: Current tab"));
        assert!(list.contains("Outcome: Open a project"));

        let lesson = render_tutor_lesson(TutorLesson::Workspace);
        assert!(lesson.contains("Scope: Current tab"));
        assert!(lesson.contains("Outcome:"));
        assert!(lesson.contains("Escape hatch:"));
        assert!(lesson.contains("Run in shell:"));
        assert!(lesson.contains("Inside Yazelix:"));
        assert!(lesson.contains("Inside Yazi:"));
        assert!(lesson.contains("yzx enter"));
        assert!(lesson.contains("yzx launch --path <dir>"));
        assert!(lesson.contains("Ctrl+y"));
        assert!(lesson.contains("Ctrl+Shift+Y"));
        assert!(lesson.contains("Alt+z"));
        assert!(lesson.contains("Alt+p"));
        assert!(lesson.contains("Alt+r"));
        assert!(lesson.contains("Alt+Shift+F"));
        assert!(!lesson.contains("yzx cwd"));
    }

    // Defends: tutor keybinding hints come from the same registry/default constants as `yzx keys`.
    #[test]
    fn tutor_keybinding_hints_use_registry_defaults() {
        let discovery = render_tutor_lesson(TutorLesson::Discovery);
        assert!(discovery.contains(key_text(&zellij_key("menu"))));
        assert!(discovery.contains(key_text(&zellij_key("bottom_popup"))));
        assert!(discovery.contains(key_text(&zellij_key("top_popup"))));
        assert!(discovery.contains(key_text(&zellij_key("config"))));
        assert!(discovery.contains(key_text(&information_popup_key())));

        let workspace = render_tutor_lesson(TutorLesson::Workspace);
        assert!(workspace.contains(key_text(&zellij_key("toggle_editor_sidebar_focus"))));
        assert!(workspace.contains(key_text(&zellij_key("toggle_editor_right_sidebar_focus"))));
        assert!(workspace.contains(key_text(&zellij_key("smart_reveal"))));
        assert!(workspace.contains(key_text(&zellij_native_key("toggle_focus_fullscreen"))));
        assert!(workspace.contains(key_text(&yazi_key("open_zoxide_in_editor"))));
        assert!(workspace.contains(key_text(&yazi_key("open_directory_as_workspace_pane"))));
    }

    fn key_text(markdown_code: &str) -> &str {
        markdown_code.trim_matches('`')
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
            assert!(output.contains("Outcome:"));
            assert!(output.contains("Escape hatch:"));
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
