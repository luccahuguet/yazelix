mod cli_render;
mod tutor_document;

use std::{env, process};
use tutor_document::render_tutor_markdown;

const YZX_HELIX: &str = "@yzxHelix@";
const NUSHELL: &str = "@nu@";

const KEY_CONFIG: &str = "Alt Shift K";
const KEY_AGENT: &str = "Alt Shift L";
const KEY_GIT: &str = "Alt Shift J";
const KEY_MENU: &str = "Alt Shift M";
const KEY_FOCUS_LEFT: &str = "Alt h";
const KEY_FOCUS_RIGHT: &str = "Alt l";
const KEY_FULLSCREEN: &str = "Alt Shift F";
const KEY_EDITOR_SIDEBAR_FOCUS: &str = "Ctrl y";
const KEY_REVEAL: &str = "Alt r";
const KEY_SIDEBAR_SWAP: &str = "Alt Shift H";
const KEY_YAZI_POPUP: &str = "Alt Shift Y";
const KEY_NEW_PANE: &str = "Alt m";
const KEY_YAZI_ZOXIDE: &str = "Alt z";
const KEY_TAB_LEFT: &str = "Ctrl Alt h";
const KEY_TAB_RIGHT: &str = "Ctrl Alt l";
const KEY_PANE_DOWN: &str = "Ctrl Alt j";
const KEY_PANE_UP: &str = "Ctrl Alt k";
const KEY_PANE_MODE: &str = "Ctrl p";
const KEY_TAB_MODE: &str = "Ctrl t";
const KEY_RESIZE_MODE: &str = "Ctrl n";
const KEY_QUIT: &str = "Ctrl q";

#[derive(Debug, PartialEq, Eq)]
enum TutorView {
    Overview,
    Begin,
    List,
    Lesson(usize),
    Helix,
    Nushell,
}

struct TutorLesson {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    scope: &'static str,
    outcome: &'static str,
    escape_hatch: &'static str,
    render: fn(usize, &TutorLesson) -> String,
}

const TUTOR_LESSONS: &[TutorLesson] = &[
    TutorLesson {
        id: "workspace",
        title: "Start in the right directory",
        summary: "Choose the workspace root for the current tab or a separate Mars window",
        scope: "Shell and current tab",
        outcome: "You can start Yazelix at the intended project and deliberately retarget the current tab from Yazi.",
        escape_hatch: "Open a shell pane and run `pwd` to check your current directory.",
        render: render_workspace_lesson,
    },
    TutorLesson {
        id: "files",
        title: "Open and reveal files",
        summary: "Use the Yazi sidebar or popup without losing your browsing place",
        scope: "Current tab",
        outcome: "You can browse with Yazi, open a file in the managed editor, and reveal the editor file in Yazi.",
        escape_hatch: "Use `Alt h` or `Alt l` to walk visible panes.",
        render: render_files_lesson,
    },
    TutorLesson {
        id: "panes",
        title: "Focus and arrange panes",
        summary: "Move focus, create space, and reshape the current tab",
        scope: "Current tab",
        outcome: "You can focus, create, fullscreen, and rearrange panes and tabs without changing the workspace root.",
        escape_hatch: "Use `Alt h` or `Alt l` to return focus to a visible pane.",
        render: render_panes_lesson,
    },
    TutorLesson {
        id: "modes",
        title: "Zellij modes and session",
        summary: "Enter the pane, tab, or resize key layer and leave the session",
        scope: "Current Yazelix window",
        outcome: "You can enter and leave each common Zellij mode and quit the session intentionally.",
        escape_hatch: "Press the active mode key again to return to normal mode.",
        render: render_modes_lesson,
    },
    TutorLesson {
        id: "discovery",
        title: "Command and key discovery",
        summary: "Find commands, popups, and the key table without memorizing the map",
        scope: "Current Yazelix window",
        outcome: "You can open the menu, Ratconfig, Git, and agent popups and find the packaged key table.",
        escape_hatch: "Press the same popup key again to focus or close a managed popup; tool-local exits such as `q`, `Esc`, or `:q` stay inside the tool.",
        render: render_discovery_lesson,
    },
    TutorLesson {
        id: "troubleshooting",
        title: "Troubleshooting paths",
        summary: "Get back to a known pane and inspect stale runtime state",
        scope: "Current Yazelix window",
        outcome: "You can recover from lost focus and use status or doctor to inspect runtime and generated state.",
        escape_hatch: "Run `yzx doctor`, then `yzx tutor list` to return to the guided path.",
        render: render_troubleshooting_lesson,
    },
    TutorLesson {
        id: "tool_tutors",
        title: "Editor and Nushell tutors",
        summary: "Switch from Yazelix guidance into the editor and shell tutors",
        scope: "Editor or shell pane",
        outcome: "You can find the managed Helix and Nushell tutors from Yazelix, or continue with a host editor when managed Helix is not included.",
        escape_hatch: "Quit Helix with `:q`; leave a shell prompt with `Ctrl d` or `exit`, then run `yzx tutor list`.",
        render: render_tool_tutors_lesson,
    },
];

fn main() {
    process::exit(match run() {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("{message}");
            64
        }
    });
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let (view, help) = parse_tutor_args(&args)?;
    if help {
        print_tutor_help();
        return Ok(());
    }

    let output = match view {
        TutorView::Overview => render_overview(),
        TutorView::Begin => render_lesson(0),
        TutorView::List => render_lesson_list(),
        TutorView::Lesson(index) => render_lesson(index),
        TutorView::Helix => render_helix_tutor_command(),
        TutorView::Nushell => render_nushell_tutor_command(),
    };
    print!("{output}");
    Ok(())
}

fn parse_tutor_args(args: &[String]) -> Result<(TutorView, bool), String> {
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
        ["hx"] | ["helix"] => TutorView::Helix,
        ["nu"] | ["nushell"] => TutorView::Nushell,
        [lesson] => match lesson_index(lesson) {
            Some(index) => TutorView::Lesson(index),
            None => {
                return Err(format!(
                    "Unknown yzx tutor target: {lesson}. Try `yzx tutor --help`."
                ));
            }
        },
        _ => return Err("Unexpected arguments for yzx tutor.".into()),
    };

    Ok((view, help))
}

fn lesson_index(id: &str) -> Option<usize> {
    TUTOR_LESSONS.iter().position(|lesson| lesson.id == id)
}

fn print_tutor_help() {
    print!(
        "Show the Yazelix guided tutor\n\n\
Usage:\n\
  yzx tutor\n\
  yzx tutor begin\n\
  yzx tutor list\n\
  yzx tutor workspace\n\
  yzx tutor files\n\
  yzx tutor panes\n\
  yzx tutor modes\n\
  yzx tutor discovery\n\
  yzx tutor troubleshooting\n\
  yzx tutor tool_tutors\n\
  yzx tutor hx\n\
  yzx tutor helix\n\
  yzx tutor nu\n\
  yzx tutor nushell\n"
    );
}

fn render_overview() -> String {
    markdown(
        r#"# Yazelix Nova tutor

Start with `yzx tutor begin`.

Use `yzx tutor list` to see every lesson or come back later.

Useful companions:

- `yzx help` shows command syntax.
- `yzx config` opens the Ratconfig UI, including the read-only keys tab.
- `yzx menu` shows the live-filter command palette.
- `yzx doctor` checks config, generated files, packaged tools, and runtime state.
"#,
    )
}

fn render_lesson_list() -> String {
    let mut source = String::from("# Yazelix Nova tutor lessons\n\n");
    for (index, lesson) in TUTOR_LESSONS.iter().enumerate() {
        source.push_str(&format!(
            "{}. `yzx tutor {}` {}  \n",
            index + 1,
            lesson.id,
            lesson.title
        ));
        source.push_str(&format!(
            "   Scope: {}. Goal: {}\n",
            lesson.scope, lesson.summary
        ));
    }
    source.push_str("\nStart with `yzx tutor begin`; revisit this list when coming back later.\n");
    markdown(&source)
}

fn render_lesson(index: usize) -> String {
    let lesson = &TUTOR_LESSONS[index];
    (lesson.render)(index + 1, lesson)
}

fn render_workspace_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Run in shell:** Change to the project directory and run `yzx enter`.
2. **Run in shell:** Use `cd <dir> && yzx launch` when another directory needs its own Mars window.
3. **Inside Yazi:** Press `{yazi_zoxide}` to choose a directory with zoxide, retarget the current tab, and open it in the editor.

## Mental model

The current tab workspace root matters most. Managed panes and popups use that directory until you deliberately choose another one.

Next lesson: `yzx tutor files`.
"#,
        header = lesson_intro(index, lesson),
        yazi_zoxide = key(KEY_YAZI_ZOXIDE),
    ))
}

fn render_files_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Inside Yazelix:** Press `{sidebar_focus}` to move between the editor and Yazi sidebar, or `{sidebar_swap}` to hide or show the sidebar.
2. **Inside Yazelix:** Press `{yazi_popup}` to hide or show the full Yazi popup. Its navigation state stays live while hidden.
3. **Inside Yazi:** Press `Enter` to open the selected file in the managed editor.
4. **Inside the editor:** Press `{reveal}` to reveal the current file in Yazi.

## Mental model

The sidebar is the quick companion. The full popup gives Yazi more room and keeps its browsing state while hidden.

Next lesson: `yzx tutor panes`.
"#,
        header = lesson_intro(index, lesson),
        sidebar_focus = key(KEY_EDITOR_SIDEBAR_FOCUS),
        sidebar_swap = key(KEY_SIDEBAR_SWAP),
        yazi_popup = key(KEY_YAZI_POPUP),
        reveal = key(KEY_REVEAL),
    ))
}

fn render_panes_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Inside Yazelix:** Press `{focus_left}` or `{focus_right}` to walk visible panes.
2. **Inside Yazelix:** Press `{new_pane}` for a new stacked pane.
3. **Inside Yazelix:** Press `{fullscreen}` to fullscreen the focused pane.
4. **Inside Yazelix:** Press `{tab_left}` or `{tab_right}` to move the current tab, and `{pane_down}` or `{pane_up}` to move the current pane.

## Mental model

Focusing and rearranging panes changes the view, not the current tab workspace root.

Next lesson: `yzx tutor modes`.
"#,
        header = lesson_intro(index, lesson),
        focus_left = key(KEY_FOCUS_LEFT),
        focus_right = key(KEY_FOCUS_RIGHT),
        new_pane = key(KEY_NEW_PANE),
        fullscreen = key(KEY_FULLSCREEN),
        tab_left = key(KEY_TAB_LEFT),
        tab_right = key(KEY_TAB_RIGHT),
        pane_down = key(KEY_PANE_DOWN),
        pane_up = key(KEY_PANE_UP),
    ))
}

fn render_modes_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Inside Yazelix:** Press `{pane_mode}` for pane mode. Press it again to return to normal mode.
2. **Inside Yazelix:** Press `{tab_mode}` for tab mode. Press it again to return to normal mode.
3. **Inside Yazelix:** Press `{resize_mode}` for resize mode. Press it again to return to normal mode.
4. **Inside Yazelix:** Press `{quit}` to quit the session.

Next lesson: `yzx tutor discovery`.
"#,
        header = lesson_intro(index, lesson),
        pane_mode = key(KEY_PANE_MODE),
        tab_mode = key(KEY_TAB_MODE),
        resize_mode = key(KEY_RESIZE_MODE),
        quit = key(KEY_QUIT),
    ))
}

fn render_discovery_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzx help` when you know the command name and need syntax.
2. **Inside Yazelix:** Press `{menu}` or run `yzx menu` for the live-filter command palette.
3. **Inside Yazelix:** Press `{config}` or run `yzx config` to open Ratconfig; use its `keys` tab when you need the packaged binding table.
4. **Inside Yazelix:** Press `{git}` for the Git popup and `{agent}` for the persistent agent popup.

Next lesson: `yzx tutor troubleshooting`.
"#,
        header = lesson_intro(index, lesson),
        menu = key(KEY_MENU),
        config = key(KEY_CONFIG),
        git = key(KEY_GIT),
        agent = key(KEY_AGENT),
    ))
}

fn render_troubleshooting_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Inside Yazelix:** Press `{focus_left}` or `{focus_right}` to move focus until you reach a known pane.
2. **Inside Yazelix:** Press `{menu}` when you remember the action but not the exact command.
3. **Run in shell or Yazelix:** Use `yzx status` for a compact runtime and config summary.
4. **Run in shell or Yazelix:** Use `yzx doctor` when config, generated layouts, packaged tools, or startup state look stale.

Next lesson: `yzx tutor tool_tutors`.
"#,
        header = lesson_intro(index, lesson),
        focus_left = key(KEY_FOCUS_LEFT),
        focus_right = key(KEY_FOCUS_RIGHT),
        menu = key(KEY_MENU),
    ))
}

fn render_tool_tutors_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzx tutor hx` to print the managed Helix tutor command and package-availability guidance.
2. **Inside Helix:** Leave the tutor with `:q`; use `{reveal}` in managed Helix sessions when you want Yazi to reveal the current file.
3. **Run in shell or Yazelix:** Use `yzx tutor nu` to print the Nushell tutor commands.
4. **Run in shell:** Use `yzx env` for the Yazelix-managed shell and packaged tools without opening the workspace UI.
"#,
        header = lesson_intro(index, lesson),
        reveal = key(KEY_REVEAL),
    ))
}

fn render_helix_tutor_command() -> String {
    markdown(&format!(
        r#"# Helix tutor

Packages with managed Helix provide this tutor command:

- `{yzx_helix} --tutor`

When you are already inside `yzx env` or a managed Yazelix shell, the short form is:

- `yzx-hx --tutor`

If your selected package omits managed Helix, use your host editor's own tutor instead.
"#,
        yzx_helix = YZX_HELIX,
    ))
}

fn render_nushell_tutor_command() -> String {
    markdown(&format!(
        r#"# Nushell tutor

Inside any Nushell prompt, run:

- `tutor begin`

From a regular shell, run the packaged Nushell command:

- `{nu} -c 'tutor begin'`
"#,
        nu = NUSHELL,
    ))
}

fn lesson_intro(index: usize, lesson: &TutorLesson) -> String {
    format!(
        r#"# {index}. {title}

{summary}

**Scope:** {scope}

**Outcome:** {outcome}

**Escape hatch:** {escape_hatch}
"#,
        title = lesson.title,
        summary = lesson.summary,
        scope = lesson.scope,
        outcome = lesson.outcome,
        escape_hatch = lesson.escape_hatch,
    )
}

fn markdown(source: &str) -> String {
    render_tutor_markdown(source).expect("bundled tutor Markdown is supported")
}

fn key(value: &str) -> String {
    format!("`{value}`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tutor_args_preserves_alias_surface() {
        assert_eq!(parse_tutor_args(&[]).unwrap(), (TutorView::Overview, false));
        assert_eq!(
            parse_tutor_args(&["begin".into()]).unwrap(),
            (TutorView::Begin, false)
        );
        assert_eq!(
            parse_tutor_args(&["list".into()]).unwrap(),
            (TutorView::List, false)
        );
        assert_eq!(
            parse_tutor_args(&["workspace".into()]).unwrap(),
            (TutorView::Lesson(lesson_index("workspace").unwrap()), false)
        );
        assert_eq!(
            parse_tutor_args(&["files".into()]).unwrap(),
            (TutorView::Lesson(lesson_index("files").unwrap()), false)
        );
        assert_eq!(
            parse_tutor_args(&["helix".into()]).unwrap(),
            (TutorView::Helix, false)
        );
        assert_eq!(
            parse_tutor_args(&["nushell".into()]).unwrap(),
            (TutorView::Nushell, false)
        );
        assert!(parse_tutor_args(&["continue".into()]).is_err());
        assert!(parse_tutor_args(&["workspace".into(), "extra".into()]).is_err());
    }

    #[test]
    fn tutor_root_output_stays_minimal() {
        let output = render_overview();
        assert!(output.contains("Yazelix Nova tutor"));
        assert!(output.contains("yzx tutor begin"));
        assert!(output.contains("yzx tutor list"));
        assert!(output.contains("yzx help"));
        assert!(output.contains("yzx config"));
        assert!(output.contains("yzx menu"));
        assert!(output.contains("yzx doctor"));
        assert!(!output.contains("yzx tutor continue"));
        assert!(!output.contains("Mental model"));
    }

    #[test]
    fn lesson_list_includes_indexed_lessons() {
        let output = render_lesson_list();
        assert!(output.contains("1. "));
        assert!(output.contains("2. "));
        assert!(output.contains("3. "));
        assert!(output.contains("4. "));
        assert!(output.contains("5. "));
        assert!(output.contains("6. "));
        assert!(output.contains("7. "));
        assert!(output.contains("yzx tutor workspace"));
        assert!(output.contains("yzx tutor files"));
        assert!(output.contains("yzx tutor panes"));
        assert!(output.contains("yzx tutor modes"));
        assert!(output.contains("yzx tutor discovery"));
        assert!(output.contains("yzx tutor troubleshooting"));
        assert!(output.contains("yzx tutor tool_tutors"));
        assert!(output.contains("yzx tutor begin"));
    }

    #[test]
    fn lessons_teach_current_next_surface() {
        let workspace = render_lesson(lesson_index("workspace").unwrap());
        for expected in [
            "yzx enter",
            "cd <dir> && yzx launch",
            KEY_YAZI_ZOXIDE,
            "current tab workspace root matters most",
        ] {
            assert!(workspace.contains(expected), "missing {expected}");
        }
        assert!(!workspace.contains("launch --path"));

        let files = render_lesson(lesson_index("files").unwrap());
        for expected in [
            KEY_EDITOR_SIDEBAR_FOCUS,
            KEY_SIDEBAR_SWAP,
            KEY_YAZI_POPUP,
            KEY_REVEAL,
        ] {
            assert!(files.contains(expected), "missing {expected}");
        }

        let panes = render_lesson(lesson_index("panes").unwrap());
        for expected in [
            KEY_FOCUS_LEFT,
            KEY_FOCUS_RIGHT,
            KEY_FULLSCREEN,
            KEY_NEW_PANE,
            KEY_TAB_LEFT,
            KEY_TAB_RIGHT,
            KEY_PANE_DOWN,
            KEY_PANE_UP,
        ] {
            assert!(panes.contains(expected), "missing {expected}");
        }

        let modes = render_lesson(lesson_index("modes").unwrap());
        for expected in [KEY_PANE_MODE, KEY_TAB_MODE, KEY_RESIZE_MODE, KEY_QUIT] {
            assert!(modes.contains(expected), "missing {expected}");
        }

        let discovery = render_lesson(lesson_index("discovery").unwrap());
        for expected in [
            "yzx help",
            "yzx menu",
            "yzx config",
            "keys",
            KEY_MENU,
            KEY_CONFIG,
            KEY_GIT,
            KEY_AGENT,
        ] {
            assert!(discovery.contains(expected), "missing {expected}");
        }

        let troubleshooting = render_lesson(lesson_index("troubleshooting").unwrap());
        assert!(troubleshooting.contains("yzx status"));
        assert!(troubleshooting.contains("yzx doctor"));
        assert!(troubleshooting.contains("yzx tutor list"));
        assert!(!troubleshooting.contains("yzx tutor continue"));
    }

    #[test]
    fn tool_tutors_print_commands_instead_of_claiming_to_run_them() {
        let tool_lesson = render_lesson(lesson_index("tool_tutors").unwrap());
        assert!(tool_lesson.contains("print the managed Helix tutor command"));
        assert!(tool_lesson.contains("print the Nushell tutor commands"));
        assert!(tool_lesson.contains("yzx env"));
        assert!(!tool_lesson.contains("yzx env --no-shell"));

        let helix = render_helix_tutor_command();
        assert!(helix.contains(&format!("{YZX_HELIX} --tutor")));
        assert!(helix.contains("yzx-hx --tutor"));
        assert!(helix.contains("package omits managed Helix"));
        assert!(!helix.contains("launch"));

        let nu = render_nushell_tutor_command();
        assert!(nu.contains("tutor begin"));
        assert!(nu.contains(&format!("{NUSHELL} -c 'tutor begin'")));
        assert!(!nu.contains("exec"));
    }

    #[test]
    fn bundled_tutor_markdown_documents_render() {
        assert!(render_overview().contains("Yazelix Nova tutor"));
        assert!(render_lesson_list().contains("yzx tutor tool_tutors"));
        assert!(render_helix_tutor_command().contains("--tutor"));
        assert!(render_nushell_tutor_command().contains("tutor begin"));
        for index in 0..TUTOR_LESSONS.len() {
            let output = render_lesson(index);
            assert!(output.contains("Outcome:"));
            assert!(output.contains("Escape hatch:"));
        }
    }

    #[test]
    fn lessons_keep_action_lists_short() {
        for (index, lesson) in TUTOR_LESSONS.iter().enumerate() {
            let output = render_lesson(index);
            let action_count = output
                .lines()
                .skip_while(|line| !line.contains("Actions"))
                .skip(1)
                .filter(|line| {
                    line.split_once(". ")
                        .is_some_and(|(number, _)| number.parse::<usize>().is_ok())
                })
                .count();
            assert!(
                (1..=4).contains(&action_count),
                "lesson {} has {action_count} actions",
                lesson.id
            );
        }
    }
}
