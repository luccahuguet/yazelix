//! Rust-owned front-door public commands for `yzx tutor`, `yzx screen`, and `yzx whats_new`.

use crate::action_registry::{
    display_yazi_key, display_zellij_key, yazi_action_by_local_id, zellij_action_by_local_id,
    zellij_native_keybinding_by_local_id,
};
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, load_normalized_config_for_control,
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
use crate::yazi_materialization::resolve_yazi_keybindings;
use crate::zellij_materialization::{
    resolve_custom_popup_keybindings, resolve_zellij_keybindings, resolve_zellij_native_keybindings,
};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::BTreeMap;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TutorView {
    Overview,
    Begin,
    Continue,
    List,
    Lesson(TutorLesson),
    Helix,
    Nushell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TutorLesson {
    Workspace,
    Discovery,
    Troubleshooting,
    ToolTutors,
}

const INFORMATION_POPUP_ID: &str = "zenith";
#[cfg(test)]
const TUTOR_ZELLIJ_ACTION_IDS: &[&str] = &[
    "toggle_editor_sidebar_focus",
    "toggle_editor_right_sidebar_focus",
    "smart_reveal",
    "menu",
    "bottom_popup",
    "top_popup",
    "config",
];
#[cfg(test)]
const TUTOR_ZELLIJ_NATIVE_ACTION_IDS: &[&str] = &["toggle_focus_fullscreen"];
#[cfg(test)]
const TUTOR_YAZI_ACTION_IDS: &[&str] =
    &["open_zoxide_in_editor", "open_directory_as_workspace_pane"];
const TUTOR_CUSTOM_POPUP_IDS: &[&str] = &[INFORMATION_POPUP_ID];

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
        id: "troubleshooting",
        title: "Troubleshooting paths",
        summary: "Get back to a known pane, inspect config, and refresh stale runtime state",
        scope: "Current Yazelix window",
        outcome: "You can recover from lost focus, stuck popups, stale generated state, and unclear config/runtime ownership.",
        escape_hatch: "If a popup or pane still feels wrong, run `yzx doctor` and then `yzx tutor continue` to return to the guided path.",
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

#[derive(Debug, Clone)]
struct TutorKeymap {
    zellij: BTreeMap<String, Vec<String>>,
    zellij_native: BTreeMap<String, Vec<String>>,
    yazi: BTreeMap<String, Vec<String>>,
    custom_popups: BTreeMap<String, Vec<String>>,
}

impl TutorKeymap {
    fn active_from_env() -> Result<Self, CoreError> {
        let runtime_dir = runtime_dir_from_env()?;
        let config_dir = config_dir_from_env()?;
        let config_override = config_override_from_env();
        let config = load_normalized_config_for_control(
            &runtime_dir,
            &config_dir,
            config_override.as_deref(),
        )?;
        Self::from_normalized_config(&config)
    }

    fn from_normalized_config(config: &JsonMap<String, JsonValue>) -> Result<Self, CoreError> {
        let mut custom_popups = BTreeMap::new();
        for popup_id in TUTOR_CUSTOM_POPUP_IDS {
            let keys = resolve_custom_popup_keybindings(config, popup_id)?.unwrap_or_default();
            custom_popups.insert((*popup_id).to_string(), keys);
        }
        Ok(Self {
            zellij: resolve_zellij_keybindings(config)?,
            zellij_native: resolve_zellij_native_keybindings(config)?,
            yazi: resolve_yazi_keybindings(config)?,
            custom_popups,
        })
    }

    fn zellij_keys(&self, local_id: &str) -> &[String] {
        zellij_action_by_local_id(local_id)
            .unwrap_or_else(|| panic!("missing Zellij tutor keybinding action: {local_id}"));
        self.zellij.get(local_id).map(Vec::as_slice).unwrap_or(&[])
    }

    fn zellij_native_keys(&self, local_id: &str) -> &[String] {
        zellij_native_keybinding_by_local_id(local_id)
            .unwrap_or_else(|| panic!("missing native Zellij tutor keybinding action: {local_id}"));
        self.zellij_native
            .get(local_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn yazi_keys(&self, local_id: &str) -> &[String] {
        yazi_action_by_local_id(local_id)
            .unwrap_or_else(|| panic!("missing Yazi tutor keybinding action: {local_id}"));
        self.yazi.get(local_id).map(Vec::as_slice).unwrap_or(&[])
    }

    fn custom_popup_keys(&self, popup_id: &str) -> &[String] {
        self.custom_popups
            .get(popup_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    #[cfg(test)]
    fn defaults() -> Self {
        Self::from_normalized_config(&JsonMap::new()).expect("default tutor keymap is valid")
    }
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
        TutorView::Helix => run_external_command("hx", &["--tutor"], "Helix"),
        TutorView::Nushell => run_external_command("nu", &["-c", "tutor"], "Nushell"),
        TutorView::Overview => {
            let keymap = TutorKeymap::active_from_env()?;
            print!("{}", render_yazelix_tutor_overview(&keymap));
            Ok(0)
        }
        TutorView::Begin => {
            let keymap = TutorKeymap::active_from_env()?;
            print!("{}", render_tutor_lesson(TutorLesson::Workspace, &keymap));
            Ok(0)
        }
        TutorView::Continue => {
            let keymap = TutorKeymap::active_from_env()?;
            print!("{}", render_tutor_continue(&keymap));
            Ok(0)
        }
        TutorView::List => {
            print!("{}", render_tutor_list());
            Ok(0)
        }
        TutorView::Lesson(lesson) => {
            let keymap = TutorKeymap::active_from_env()?;
            print!("{}", render_tutor_lesson(lesson, &keymap));
            Ok(0)
        }
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
        ["continue"] => TutorView::Continue,
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
    println!("  yzx tutor continue");
    println!("  yzx tutor list");
    println!("  yzx tutor workspace");
    println!("  yzx tutor discovery");
    println!("  yzx tutor troubleshooting");
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

fn render_yazelix_tutor_overview(keymap: &TutorKeymap) -> String {
    let menu_key = zellij_key(keymap, "menu");
    let info_key = information_popup_key(keymap);
    let markdown = format!(
        r#"# Yazelix tutor

Yazelix is a managed terminal workspace built around Zellij, Yazi, and Helix.

The current tab workspace root matters most. Managed actions use that directory unless a tool chooses a file or directory.

## Start here

### Outside Yazelix

- From a project directory, run `yzx launch`.
- From the terminal you want to turn into Yazelix, run `yzx enter`.
- After Yazelix opens, run `yzx tutor begin`.

### Inside Yazelix

- Press `{menu_key}` or run `yzx menu` for the command menu.
- Press `{info_key}` for the information popup.
- Run `yzx keys` to see live bindings.
- Run `yzx doctor` when config, panes, runtime state, or generated files look wrong.

### Coming back later

Run `yzx tutor continue` and pick the first lesson you have not practiced.

## Key notation

Yazelix writes key chords as `Alt+Shift+M`, `Ctrl+y`, and `Ctrl+Shift+Y`. `yzx keys` shows the same defaults and any remaps.

## Lessons

1. `yzx tutor workspace` builds the current-tab workspace model.
2. `yzx tutor discovery` shows command, popup, and troubleshooting surfaces.
3. `yzx tutor troubleshooting` covers the fastest ways back to a known state.
4. `yzx tutor tool_tutors` sends you to the Helix and Nushell tutors.

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
        "troubleshooting" => Some(TutorLesson::Troubleshooting),
        "tool_tutors" => Some(TutorLesson::ToolTutors),
        _ => None,
    }
}

fn tutor_lesson_spec(lesson: TutorLesson) -> &'static TutorLessonSpec {
    let id = match lesson {
        TutorLesson::Workspace => "workspace",
        TutorLesson::Discovery => "discovery",
        TutorLesson::Troubleshooting => "troubleshooting",
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
            "   Scope: {}. Goal: {}\n",
            lesson.scope, lesson.summary
        ));
    }
    markdown.push_str("\nStart with `yzx tutor begin`; pick back up with `yzx tutor continue`.\n");
    render_tutor_markdown(&markdown)
}

fn render_tutor_continue(keymap: &TutorKeymap) -> String {
    let menu_key = zellij_key(keymap, "menu");
    let markdown = format!(
        r#"# Continue the Yazelix tutor

Yazelix does not store tutor progress. Pick the first lesson below that you have not practiced.

1. `yzx tutor workspace` if opening projects, Yazi handoff, or sidebar focus still feels unclear.
2. `yzx tutor discovery` if you need command search, key tables, or popup locations.
3. `yzx tutor troubleshooting` if you are lost, a popup is stuck, config changed, or generated state looks stale.
4. `yzx tutor tool_tutors` when the Yazelix workspace path is clear and you want Helix or Nushell practice.

Inside Yazelix, press {menu_key} or run `yzx menu` when you would rather search commands directly.
"#
    );
    render_tutor_markdown(&markdown)
}

fn render_tutor_lesson(lesson: TutorLesson, keymap: &TutorKeymap) -> String {
    match lesson {
        TutorLesson::Workspace => render_workspace_lesson(keymap),
        TutorLesson::Discovery => render_discovery_lesson(keymap),
        TutorLesson::Troubleshooting => render_troubleshooting_lesson(keymap),
        TutorLesson::ToolTutors => render_tool_tutors_lesson(keymap),
    }
}

fn render_workspace_lesson(keymap: &TutorKeymap) -> String {
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
        left_focus = zellij_key(keymap, "toggle_editor_sidebar_focus"),
        right_focus = zellij_key(keymap, "toggle_editor_right_sidebar_focus"),
        zoxide = yazi_key(keymap, "open_zoxide_in_editor"),
        workspace_pane = yazi_key(keymap, "open_directory_as_workspace_pane"),
        reveal = zellij_key(keymap, "smart_reveal"),
        fullscreen = zellij_native_key(keymap, "toggle_focus_fullscreen"),
    );
    render_tutor_markdown(&markdown)
}

fn render_discovery_lesson(keymap: &TutorKeymap) -> String {
    let markdown = format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzx help` when you know the command name and need syntax.
2. **Run in shell or Yazelix:** Use `yzx keys` for live Yazelix, Yazi, Helix, and Nushell bindings.
3. **Inside Yazelix:** Press `{menu}` or run `yzx menu` to open command search.
4. **Inside Yazelix:** Press `{bottom_popup}` for the bottom popup, `{top_popup}` for the top popup, and `{config}` for the config UI.
5. **Inside Yazelix:** Press `{info}` for the keep-alive information popup.
6. **Run in shell or Yazelix:** Use `yzx doctor` when config, runtime, or generated workspace files look wrong.

Next lesson: `yzx tutor troubleshooting`.
"#,
        header = render_lesson_intro(TutorLesson::Discovery),
        menu = zellij_key(keymap, "menu"),
        bottom_popup = zellij_key(keymap, "bottom_popup"),
        top_popup = zellij_key(keymap, "top_popup"),
        config = zellij_key(keymap, "config"),
        info = information_popup_key(keymap),
    );
    render_tutor_markdown(&markdown)
}

fn render_troubleshooting_lesson(keymap: &TutorKeymap) -> String {
    let markdown = format!(
        r#"{header}

## Actions

1. **Inside Yazelix:** Press `{editor_sidebar}` to return to the editor/sidebar loop when focus is lost.
2. **Inside Yazelix:** Press `{menu}` or run `yzx menu` when you know the action but not the command name.
3. **Inside Yazelix:** Press `{config}` for the config UI when a setting needs inspection or a keybinding remap.
4. **Run in shell or Yazelix:** Use `yzx doctor` when config, runtime, generated layouts, or packaged tools look stale.
5. **Run in shell or Yazelix:** Use `yzx keys` to verify the active keymap before editing bindings.
6. **Run in shell or Yazelix:** Use `yzx tutor continue` after the check so the guided path is visible again.

Next lesson: `yzx tutor tool_tutors`.
"#,
        header = render_lesson_intro(TutorLesson::Troubleshooting),
        editor_sidebar = zellij_key(keymap, "toggle_editor_sidebar_focus"),
        menu = zellij_key(keymap, "menu"),
        config = zellij_key(keymap, "config"),
    );
    render_tutor_markdown(&markdown)
}

fn render_tool_tutors_lesson(keymap: &TutorKeymap) -> String {
    let markdown = format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzx tutor hx` to launch `hx --tutor`.
2. **Inside Helix:** Leave the tutor with `:q`; use `{reveal}` in managed Helix sessions when you want Yazi to reveal the current file.
3. **Run in shell or Yazelix:** Use `yzx tutor nu` to launch the Nushell tutor.
4. **Inside Yazelix:** Press `{editor_sidebar}` to return to the editor/sidebar loop; press `{menu}` when you want command search again.
5. **Run in shell:** Use `yzx env` when you want Yazelix tools without opening the workspace UI. Use `yzx env --no-shell` to keep your current shell.
6. **Run in shell or Yazelix:** Return to `yzx tutor list` when you want the Yazelix path.
"#,
        header = render_lesson_intro(TutorLesson::ToolTutors),
        reveal = zellij_key(keymap, "smart_reveal"),
        editor_sidebar = zellij_key(keymap, "toggle_editor_sidebar_focus"),
        menu = zellij_key(keymap, "menu"),
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

fn zellij_key(keymap: &TutorKeymap, local_id: &str) -> String {
    format!(
        "`{}`",
        display_zellij_key_strings(keymap.zellij_keys(local_id))
    )
}

fn zellij_native_key(keymap: &TutorKeymap, local_id: &str) -> String {
    format!(
        "`{}`",
        display_zellij_key_strings(keymap.zellij_native_keys(local_id))
    )
}

fn yazi_key(keymap: &TutorKeymap, local_id: &str) -> String {
    format!("`{}`", display_yazi_key_strings(keymap.yazi_keys(local_id)))
}

fn information_popup_key(keymap: &TutorKeymap) -> String {
    format!(
        "`{}`",
        display_zellij_key_strings(keymap.custom_popup_keys(INFORMATION_POPUP_ID))
    )
}

fn display_zellij_key_strings(keys: &[String]) -> String {
    display_key_strings(keys, display_zellij_key)
}

fn display_yazi_key_strings(keys: &[String]) -> String {
    display_key_strings(keys, display_yazi_key)
}

fn display_key_strings(keys: &[String], display_key: fn(&str) -> String) -> String {
    if keys.is_empty() {
        return "unbound".to_string();
    }
    keys.iter()
        .map(|key| display_key(key))
        .collect::<Vec<_>>()
        .join(" / ")
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
            parse_tutor_args(&["continue".into()]).unwrap(),
            TutorArgs {
                view: TutorView::Continue,
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
        assert_eq!(
            parse_tutor_args(&["troubleshooting".into()]).unwrap(),
            TutorArgs {
                view: TutorView::Lesson(TutorLesson::Troubleshooting),
                help: false,
            }
        );
        assert!(parse_tutor_args(&["weird".into()]).is_err());
    }

    // Defends: the front-door tutor root still prints the managed-workspace guidance instead of drifting through wrapper churn.
    #[test]
    fn tutor_root_output_keeps_guided_overview_copy() {
        let keymap = TutorKeymap::defaults();
        let output = render_yazelix_tutor_overview(&keymap);
        assert!(output.contains("Yazelix tutor"));
        assert!(output.contains("Outside Yazelix"));
        assert!(output.contains("After Yazelix opens"));
        assert!(output.contains("Inside Yazelix"));
        assert!(output.contains("Coming back later"));
        assert!(output.contains("yzx tutor begin"));
        assert!(output.contains("yzx tutor continue"));
        assert!(output.contains("yzx menu"));
        assert!(output.contains("Alt+Shift+M"));
        assert!(output.contains("Alt+Shift+I"));
        assert!(output.contains("Key notation"));
        assert!(!output.contains("Start a session with"));
    }

    // Defends: the guided tutor has indexed lesson entrypoints and structured onboarding metadata instead of only a flat overview.
    #[test]
    fn tutor_lessons_include_indexed_metadata_and_workspace_actions() {
        let list = render_tutor_list();
        assert!(list.contains("1. `yzx tutor workspace`"));
        assert!(list.contains("2. `yzx tutor discovery`"));
        assert!(list.contains("3. `yzx tutor troubleshooting`"));
        assert!(list.contains("4. `yzx tutor tool_tutors`"));
        assert!(list.contains("Scope: Current tab"));
        assert!(list.contains("Goal: Open a project"));
        assert!(!list.contains("Outcome: Open a project"));

        let keymap = TutorKeymap::defaults();
        let lesson = render_tutor_lesson(TutorLesson::Workspace, &keymap);
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
        let keymap = TutorKeymap::defaults();
        let discovery = render_tutor_lesson(TutorLesson::Discovery, &keymap);
        assert!(discovery.contains(key_text(&zellij_key(&keymap, "menu"))));
        assert!(discovery.contains(key_text(&zellij_key(&keymap, "bottom_popup"))));
        assert!(discovery.contains(key_text(&zellij_key(&keymap, "top_popup"))));
        assert!(discovery.contains(key_text(&zellij_key(&keymap, "config"))));
        assert!(discovery.contains(key_text(&information_popup_key(&keymap))));

        let workspace = render_tutor_lesson(TutorLesson::Workspace, &keymap);
        assert!(workspace.contains(key_text(&zellij_key(
            &keymap,
            "toggle_editor_sidebar_focus"
        ))));
        assert!(workspace.contains(key_text(&zellij_key(
            &keymap,
            "toggle_editor_right_sidebar_focus"
        ))));
        assert!(workspace.contains(key_text(&zellij_key(&keymap, "smart_reveal"))));
        assert!(workspace.contains(key_text(&zellij_native_key(
            &keymap,
            "toggle_focus_fullscreen"
        ))));
        assert!(workspace.contains(key_text(&yazi_key(&keymap, "open_zoxide_in_editor"))));
        assert!(workspace.contains(key_text(&yazi_key(
            &keymap,
            "open_directory_as_workspace_pane"
        ))));
    }

    // Defends: tutor keybinding hints follow active settings remaps instead of always printing defaults.
    #[test]
    fn tutor_keybinding_hints_use_active_remaps() {
        let keymap = remapped_tutor_keymap();

        let discovery = render_tutor_lesson(TutorLesson::Discovery, &keymap);
        assert!(discovery.contains("Ctrl+Alt+Shift+M"));
        assert!(discovery.contains("Ctrl+Alt+Shift+J"));
        assert!(discovery.contains("Ctrl+Alt+Shift+K"));
        assert!(discovery.contains("Ctrl+Alt+Shift+C"));
        assert!(discovery.contains("Ctrl+Alt+Shift+I"));
        assert!(!discovery.contains("`Alt+Shift+M`"));
        assert!(!discovery.contains("`Alt+Shift+I`"));

        let workspace = render_tutor_lesson(TutorLesson::Workspace, &keymap);
        assert!(workspace.contains("Ctrl+Alt+Y"));
        assert!(workspace.contains("Ctrl+Alt+Shift+Y"));
        assert!(workspace.contains("Ctrl+Alt+R"));
        assert!(workspace.contains("Ctrl+Alt+F"));
        assert!(workspace.contains("Alt+x"));
        assert!(workspace.contains("Alt+o"));
        assert!(!workspace.contains("`Alt+z`"));
        assert!(!workspace.contains("`Alt+p`"));
    }

    fn remapped_tutor_keymap() -> TutorKeymap {
        let mut config = JsonMap::new();
        config.insert(
            "zellij_keybindings".to_string(),
            serde_json::json!({
                "menu": ["Ctrl Alt Shift M"],
                "bottom_popup": ["Ctrl Alt Shift J"],
                "top_popup": ["Ctrl Alt Shift K"],
                "config": ["Ctrl Alt Shift C"],
                "toggle_editor_sidebar_focus": ["Ctrl Alt Y"],
                "toggle_editor_right_sidebar_focus": ["Ctrl Alt Shift Y"],
                "smart_reveal": ["Ctrl Alt R"]
            }),
        );
        config.insert(
            "zellij_native_keybindings".to_string(),
            serde_json::json!({
                "toggle_focus_fullscreen": ["Ctrl Alt F"]
            }),
        );
        config.insert(
            "yazi_keybindings".to_string(),
            serde_json::json!({
                "open_zoxide_in_editor": ["<A-x>"],
                "open_directory_as_workspace_pane": ["<A-o>"]
            }),
        );
        config.insert(
            "custom_popups".to_string(),
            serde_json::json!([
                {
                    "id": "zenith",
                    "command": ["zenith"],
                    "keybindings": ["Ctrl Alt Shift I"],
                    "keep_alive": true
                }
            ]),
        );
        TutorKeymap::from_normalized_config(&config).unwrap()
    }

    // Defends: every action id named by tutor key hints stays present in the owning registry/resolver.
    #[test]
    fn tutor_keybinding_coverage_ids_are_supported() {
        let keymap = TutorKeymap::defaults();
        for action in TUTOR_ZELLIJ_ACTION_IDS {
            assert!(zellij_action_by_local_id(action).is_some());
            assert!(!keymap.zellij_keys(action).is_empty());
        }
        for action in TUTOR_ZELLIJ_NATIVE_ACTION_IDS {
            assert!(zellij_native_keybinding_by_local_id(action).is_some());
            assert!(!keymap.zellij_native_keys(action).is_empty());
        }
        for action in TUTOR_YAZI_ACTION_IDS {
            assert!(yazi_action_by_local_id(action).is_some());
            assert!(!keymap.yazi_keys(action).is_empty());
        }
        for popup in TUTOR_CUSTOM_POPUP_IDS {
            assert!(!keymap.custom_popup_keys(popup).is_empty());
        }
    }

    // Defends: the stateless continue view points users back into the indexed path without promising stored progress.
    #[test]
    fn tutor_continue_is_stateless_next_step_picker() {
        let keymap = TutorKeymap::defaults();
        let output = render_tutor_continue(&keymap);
        assert!(output.contains("does not store tutor progress"));
        assert!(output.contains("yzx tutor workspace"));
        assert!(output.contains("yzx tutor troubleshooting"));
        assert!(output.contains("Alt+Shift+M"));
    }

    // Defends: the troubleshooting lesson covers lost panes, config/runtime checks, key verification, and returning to the guided path.
    #[test]
    fn tutor_troubleshooting_lesson_covers_common_escape_paths() {
        let keymap = TutorKeymap::defaults();
        let output = render_tutor_lesson(TutorLesson::Troubleshooting, &keymap);
        assert!(output.contains("Troubleshooting paths"));
        assert!(output.contains("yzx doctor"));
        assert!(output.contains("yzx keys"));
        assert!(output.contains("yzx tutor continue"));
        assert!(output.contains("Alt+Shift+M"));
        assert!(output.contains("Alt+Shift+C"));
    }

    // Defends: the tutor teaches the non-UI tool environment only after the main workspace path is clear.
    #[test]
    fn tutor_tool_tutors_mentions_yzx_env_as_optional_plain_shell_path() {
        let keymap = TutorKeymap::defaults();
        let output = render_tutor_lesson(TutorLesson::ToolTutors, &keymap);
        assert!(output.contains("yzx env"));
        assert!(output.contains("yzx env --no-shell"));
        assert!(output.contains("without opening the workspace UI"));
        assert!(!render_yazelix_tutor_overview(&keymap).contains("yzx env"));
    }

    fn key_text(markdown_code: &str) -> &str {
        markdown_code.trim_matches('`')
    }

    // Defends: every bundled tutor Markdown document stays inside the supported terminal-renderer subset.
    #[test]
    fn bundled_tutor_markdown_documents_render_without_panics() {
        let keymap = TutorKeymap::defaults();
        assert!(render_yazelix_tutor_overview(&keymap).contains("Yazelix tutor"));
        assert!(render_tutor_continue(&keymap).contains("yzx tutor troubleshooting"));
        assert!(render_tutor_list().contains("yzx tutor tool_tutors"));
        for lesson in [
            TutorLesson::Workspace,
            TutorLesson::Discovery,
            TutorLesson::Troubleshooting,
            TutorLesson::ToolTutors,
        ] {
            let output = render_tutor_lesson(lesson, &keymap);
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
