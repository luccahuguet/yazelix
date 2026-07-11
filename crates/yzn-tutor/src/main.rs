mod cli_render;
mod tutor_document;

use std::{env, process};
use tutor_document::render_tutor_markdown;

const YZN_HELIX: &str = "@yznHelix@";
const NUSHELL: &str = "@nu@";

const KEY_CONFIG: &str = "Alt Shift K";
const KEY_AGENT: &str = "Alt Shift L";
const KEY_GIT: &str = "Alt Shift J";
const KEY_MENU: &str = "Alt Shift M";
const KEY_FOCUS_LEFT: &str = "Alt h";
const KEY_FOCUS_RIGHT: &str = "Alt l";
const KEY_REVEAL: &str = "Alt r";
const KEY_SIDEBAR_SWAP: &str = "Alt Shift h";
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
        title: "Workspace roots and managed panes",
        summary: "Open a project, move between editor, sidebar, popups, and panes",
        scope: "Current tab",
        outcome: "You can start Yazelix in the right directory, open files through Yazi, reveal the editor file, and reshape the active tab without losing the workspace root.",
        escape_hatch: "Use `Alt h` or `Alt l` to walk visible panes, then rerun `yzn tutor list`.",
        render: render_workspace_lesson,
    },
    TutorLesson {
        id: "discovery",
        title: "Command and key discovery",
        summary: "Find commands, popups, key tables, and runtime checks without memorizing the map",
        scope: "Current Yazelix window",
        outcome: "You can open the menu popup, inspect config and runtime state, find the key table, and run doctor when generated files or tools look wrong.",
        escape_hatch: "Press the same popup key again to focus or close a managed popup; tool-local exits such as `q`, `Esc`, or `:q` stay inside the tool.",
        render: render_discovery_lesson,
    },
    TutorLesson {
        id: "troubleshooting",
        title: "Troubleshooting paths",
        summary: "Get back to a known pane, inspect config, and refresh stale runtime state",
        scope: "Current Yazelix window",
        outcome: "You can recover from lost focus, loud popups, stale generated state, and unclear config or runtime ownership.",
        escape_hatch: "Run `yzn doctor`, then `yzn tutor list` to return to the guided path.",
        render: render_troubleshooting_lesson,
    },
    TutorLesson {
        id: "tool_tutors",
        title: "Helix and Nushell tutors",
        summary: "Switch from Yazelix guidance into the editor and shell tutors",
        scope: "Editor or shell pane",
        outcome: "You can find the native Helix and Nushell tutors from Yazelix without making `yzn tutor` own those tools.",
        escape_hatch: "Quit Helix with `:q`; leave a shell prompt with `Ctrl d` or `exit`, then run `yzn tutor list`.",
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
                    "Unknown yzn tutor target: {lesson}. Try `yzn tutor --help`."
                ));
            }
        },
        _ => return Err("Unexpected arguments for yzn tutor.".into()),
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
  yzn tutor\n\
  yzn tutor begin\n\
  yzn tutor list\n\
  yzn tutor workspace\n\
  yzn tutor discovery\n\
  yzn tutor troubleshooting\n\
  yzn tutor tool_tutors\n\
  yzn tutor hx\n\
  yzn tutor helix\n\
  yzn tutor nu\n\
  yzn tutor nushell\n"
    );
}

fn render_overview() -> String {
    markdown(
        r#"# Yazelix Nova tutor

Start with `yzn tutor begin`.

Use `yzn tutor list` to see every lesson or come back later.

Useful companions:

- `yzn help` shows command syntax.
- `yzn config` opens the Ratconfig UI, including the read-only keys tab.
- `yzn menu` shows the live-filter command palette.
- `yzn doctor` checks config, generated files, packaged tools, and runtime state.
"#,
    )
}

fn render_lesson_list() -> String {
    let mut source = String::from("# Yazelix Nova tutor lessons\n\n");
    for (index, lesson) in TUTOR_LESSONS.iter().enumerate() {
        source.push_str(&format!(
            "{}. `yzn tutor {}` {}  \n",
            index + 1,
            lesson.id,
            lesson.title
        ));
        source.push_str(&format!(
            "   Scope: {}. Goal: {}\n",
            lesson.scope, lesson.summary
        ));
    }
    source.push_str("\nStart with `yzn tutor begin`; revisit this list when coming back later.\n");
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

1. **Run in shell:** Start in the project directory with `yzn enter`; use `cd <dir> && yzn launch` when you need a separate Mars window for another directory.
2. **Inside Yazelix:** Press `{focus_left}` or `{focus_right}` to walk visible panes; press `{sidebar_swap}` when you need the Yazi sidebar layout.
3. **Inside Yazi:** Press `Enter` to open the selected file in the managed editor.
4. **Inside Yazi:** Press `{yazi_zoxide}` to jump with zoxide and open the target in the editor.
5. **Inside Yazelix:** Press `{new_pane}` for a new stacked pane, `{pane_mode}` for pane mode, `{tab_mode}` for tab mode, `{resize_mode}` for resize mode, and `{quit}` for quit mode.
6. **Inside Yazelix:** Press `{tab_left}` or `{tab_right}` to move the current tab; press `{pane_down}` or `{pane_up}` to move the current pane.
7. **Inside the editor:** Press `{reveal}` to reveal the current file in Yazi.

## Mental model

The current tab workspace root matters most. Managed actions use that directory unless a tool chooses a file or directory. Yazelix keeps the editor, Yazi sidebar, popups, shell panes, and agent pane connected around that root.

Next lesson: `yzn tutor discovery`.
"#,
        header = lesson_intro(index, lesson),
        focus_left = key(KEY_FOCUS_LEFT),
        focus_right = key(KEY_FOCUS_RIGHT),
        sidebar_swap = key(KEY_SIDEBAR_SWAP),
        yazi_zoxide = key(KEY_YAZI_ZOXIDE),
        new_pane = key(KEY_NEW_PANE),
        pane_mode = key(KEY_PANE_MODE),
        tab_mode = key(KEY_TAB_MODE),
        resize_mode = key(KEY_RESIZE_MODE),
        quit = key(KEY_QUIT),
        tab_left = key(KEY_TAB_LEFT),
        tab_right = key(KEY_TAB_RIGHT),
        pane_down = key(KEY_PANE_DOWN),
        pane_up = key(KEY_PANE_UP),
        reveal = key(KEY_REVEAL),
    ))
}

fn render_discovery_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzn help` when you know the command name and need syntax.
2. **Inside Yazelix:** Press `{menu}` or run `yzn menu` for the live-filter command palette.
3. **Inside Yazelix:** Press `{config}` or run `yzn config` to open Ratconfig; use its `keys` tab when you need the packaged binding table.
4. **Inside Yazelix:** Press `{git}` for the Git popup and `{agent}` for the persistent agent popup.
5. **Run in shell or Yazelix:** Use `yzn status` for a compact runtime/config summary.
6. **Run in shell or Yazelix:** Use `yzn doctor` when config, runtime, generated layouts, or packaged tools look wrong.

Next lesson: `yzn tutor troubleshooting`.
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
3. **Inside Yazelix:** Press `{config}` to inspect a setting; the config popup is managed and can be toggled with the same key.
4. **Run in shell or Yazelix:** Use `yzn doctor` when config, generated layouts, packaged tools, or startup state look stale.
5. **Run in shell or Yazelix:** Use `yzn config`, then the `keys` tab, to verify the packaged bindings before editing native files.
6. **Run in shell or Yazelix:** Use `yzn tutor list` after the check so the guided path is visible again.

Next lesson: `yzn tutor tool_tutors`.
"#,
        header = lesson_intro(index, lesson),
        focus_left = key(KEY_FOCUS_LEFT),
        focus_right = key(KEY_FOCUS_RIGHT),
        menu = key(KEY_MENU),
        config = key(KEY_CONFIG),
    ))
}

fn render_tool_tutors_lesson(index: usize, lesson: &TutorLesson) -> String {
    markdown(&format!(
        r#"{header}

## Actions

1. **Run in shell or Yazelix:** Use `yzn tutor hx` to print the packaged Helix tutor command.
2. **Inside Helix:** Leave the tutor with `:q`; use `{reveal}` in managed Helix sessions when you want Yazi to reveal the current file.
3. **Run in shell or Yazelix:** Use `yzn tutor nu` to print the Nushell tutor commands.
4. **Inside Yazelix:** Press `{focus_left}` or `{focus_right}` to return to a known pane; press `{menu}` when you want the command reference again.
5. **Run in shell:** Use `yzn env` when you want the Yazelix-managed shell and packaged tools without opening the workspace UI.
6. **Run in shell or Yazelix:** Return to `yzn tutor list` when you want the Yazelix path.
"#,
        header = lesson_intro(index, lesson),
        reveal = key(KEY_REVEAL),
        focus_left = key(KEY_FOCUS_LEFT),
        focus_right = key(KEY_FOCUS_RIGHT),
        menu = key(KEY_MENU),
    ))
}

fn render_helix_tutor_command() -> String {
    markdown(&format!(
        r#"# Helix tutor

Run the packaged Helix tutor command:

- `{yzn_helix} --tutor`

When you are already inside `yzn env` or a managed Yazelix shell, the short form is:

- `yzn-hx --tutor`
"#,
        yzn_helix = YZN_HELIX,
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
        assert!(output.contains("yzn tutor begin"));
        assert!(output.contains("yzn tutor list"));
        assert!(output.contains("yzn help"));
        assert!(output.contains("yzn config"));
        assert!(output.contains("yzn menu"));
        assert!(output.contains("yzn doctor"));
        assert!(!output.contains("yzn tutor continue"));
        assert!(!output.contains("Mental model"));
    }

    #[test]
    fn lesson_list_includes_indexed_lessons() {
        let output = render_lesson_list();
        assert!(output.contains("1. "));
        assert!(output.contains("2. "));
        assert!(output.contains("3. "));
        assert!(output.contains("4. "));
        assert!(output.contains("yzn tutor workspace"));
        assert!(output.contains("yzn tutor discovery"));
        assert!(output.contains("yzn tutor troubleshooting"));
        assert!(output.contains("yzn tutor tool_tutors"));
        assert!(output.contains("yzn tutor begin"));
    }

    #[test]
    fn lessons_teach_current_next_surface() {
        let workspace = render_lesson(lesson_index("workspace").unwrap());
        for expected in [
            "yzn enter",
            "cd <dir> && yzn launch",
            KEY_FOCUS_LEFT,
            KEY_FOCUS_RIGHT,
            KEY_SIDEBAR_SWAP,
            KEY_YAZI_ZOXIDE,
            KEY_NEW_PANE,
            KEY_REVEAL,
            "current tab workspace root matters most",
        ] {
            assert!(workspace.contains(expected), "missing {expected}");
        }
        assert!(!workspace.contains("launch --path"));

        let discovery = render_lesson(lesson_index("discovery").unwrap());
        for expected in [
            "yzn help",
            "yzn menu",
            "yzn config",
            "keys",
            KEY_MENU,
            KEY_CONFIG,
            KEY_GIT,
            KEY_AGENT,
            "yzn status",
            "yzn doctor",
        ] {
            assert!(discovery.contains(expected), "missing {expected}");
        }

        let troubleshooting = render_lesson(lesson_index("troubleshooting").unwrap());
        assert!(troubleshooting.contains("yzn doctor"));
        assert!(troubleshooting.contains("yzn tutor list"));
        assert!(!troubleshooting.contains("yzn tutor continue"));
    }

    #[test]
    fn tool_tutors_print_commands_instead_of_claiming_to_run_them() {
        let tool_lesson = render_lesson(lesson_index("tool_tutors").unwrap());
        assert!(tool_lesson.contains("print the packaged Helix tutor command"));
        assert!(tool_lesson.contains("print the Nushell tutor commands"));
        assert!(tool_lesson.contains("yzn env"));
        assert!(!tool_lesson.contains("yzn env --no-shell"));

        let helix = render_helix_tutor_command();
        assert!(helix.contains(&format!("{YZN_HELIX} --tutor")));
        assert!(helix.contains("yzn-hx --tutor"));
        assert!(!helix.contains("launch"));

        let nu = render_nushell_tutor_command();
        assert!(nu.contains("tutor begin"));
        assert!(nu.contains(&format!("{NUSHELL} -c 'tutor begin'")));
        assert!(!nu.contains("exec"));
    }

    #[test]
    fn bundled_tutor_markdown_documents_render() {
        assert!(render_overview().contains("Yazelix Nova tutor"));
        assert!(render_lesson_list().contains("yzn tutor tool_tutors"));
        assert!(render_helix_tutor_command().contains("--tutor"));
        assert!(render_nushell_tutor_command().contains("tutor begin"));
        for index in 0..TUTOR_LESSONS.len() {
            let output = render_lesson(index);
            assert!(output.contains("Outcome:"));
            assert!(output.contains("Escape hatch:"));
        }
    }
}
