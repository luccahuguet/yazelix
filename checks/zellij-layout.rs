use std::{collections::BTreeSet, env, fs, process::ExitCode};

const HOME_TAB_MARKER: &str = "\u{f015}";

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    let [_, layout_path, swap_path] = args.as_slice() else {
        eprintln!("usage: zellij-layout <layout.kdl> <layout.swap.kdl>");
        return ExitCode::FAILURE;
    };

    let layout = read(layout_path);
    let templates = layout
        .lines()
        .filter_map(tab_template)
        .collect::<BTreeSet<_>>();
    let home_tab = format!("tab name=\"{HOME_TAB_MARKER}\"");
    let mut ok = true;
    for (block, needle, message) in [
        (
            "tab_template name=\"ui\"",
            "children",
            "missing content delimiter in swap UI template",
        ),
        (
            "default_tab_template",
            "plugin location=\"radar\"",
            "missing Radar sidebar in default tab template",
        ),
        (
            "new_tab_template",
            "plugin location=\"radar\"",
            "missing Radar sidebar in new tab template",
        ),
        (
            &home_tab,
            "pane name=\"yazi_picker\" command=",
            "missing tiled Yazi picker in startup tab",
        ),
        (
            "new_tab_template",
            "pane name=\"yazi_picker\" command=",
            "missing tiled Yazi picker in new tab template",
        ),
    ] {
        if !block_contains(&layout, block, needle) {
            eprintln!("{layout_path}: {message}");
            ok = false;
        }
    }
    if layout.matches(r#"plugin location="radar""#).count() != 2
        || layout
            .matches(r#"pane name="sidebar" size=32 borderless=false {"#)
            .count()
            != 2
    {
        eprintln!(
            "{layout_path}: startup and new-tab templates must each own one framed sidebar pane"
        );
        ok = false;
    }
    if layout
        .matches(r#"pane name="yazi_picker" command="#)
        .count()
        != 2
        || layout.matches(r#"args "--yzx-startup-picker""#).count() != 2
        || layout.matches("focus=true close_on_exit=true").count() != 2
        || layout.contains("floating_panes {")
        || layout.contains(r#"pane name="editor" command="#)
        || layout.contains(r#"args "--yzx-managed-pane""#)
    {
        eprintln!(
            "{layout_path}: startup and new-tab templates must each open one focused tiled Yazi picker without a prestarted editor"
        );
        ok = false;
    }
    if !layout_order_is_valid(&layout) {
        eprintln!(
            "{layout_path}: startup tab must follow default_tab_template and precede new_tab_template"
        );
        ok = false;
    }
    if !layout.lines().any(|line| {
        line.trim()
            .starts_with(&format!(r#"tab name="{HOME_TAB_MARKER}""#))
    }) {
        eprintln!("{layout_path}: startup tab must use the Yazelix home tab marker");
        ok = false;
    }
    if !layout
        .lines()
        .any(|line| line.trim() == r#"new_tab_template cwd="$HOME" {"#)
    {
        eprintln!("{layout_path}: new tabs must open in home to match the home marker");
        ok = false;
    }
    if !bar_layout_is_valid(&layout) {
        eprintln!(
            "{layout_path}: top bars must use the rendered yzx Yazelix bar widgets and bottom bars must keep native status-bar"
        );
        ok = false;
    }

    let swap = read(swap_path);
    if swap.matches(r#"plugin location="radar""#).count() != 2
        || !swap.contains(r#"pane name="sidebar" size=32 borderless=false {"#)
        || !swap.contains(r#"pane name="sidebar" size=1 borderless=false {"#)
    {
        eprintln!("{swap_path}: the open and collapsed Radar sidebar states must use pane frames");
        ok = false;
    }
    if !matches!(
        (
            swap.find(r#"swap_tiled_layout name="single_open""#),
            swap.find(r#"swap_tiled_layout name="single_closed""#),
        ),
        (Some(open), Some(closed)) if open < closed
    ) {
        eprintln!(
            "{swap_path}: open layout must precede collapsed layout so the startup picker preserves the visible sidebar"
        );
        ok = false;
    }
    let mut depth = 0i32;

    for (index, line) in swap.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("swap_tiled_layout ") {
            depth = 1;
            continue;
        }
        if depth == 1 && trimmed.ends_with('{') {
            let name = trimmed.split_whitespace().next().unwrap_or_default();
            if !templates.contains(name) {
                eprintln!("{swap_path}:{}: missing tab_template {name}", index + 1);
                ok = false;
            }
        }
        if depth > 0 {
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        }
    }

    ExitCode::from((!ok) as u8)
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("could not read {}: {}", path, error))
}

fn tab_template(line: &str) -> Option<String> {
    line.trim()
        .strip_prefix("tab_template name=\"")?
        .split('"')
        .next()
        .map(str::to_owned)
}

fn block_contains(text: &str, block_name: &str, needle: &str) -> bool {
    let mut depth = 0i32;

    for line in text.lines() {
        let trimmed = line.trim();
        if depth == 0 && trimmed.starts_with(block_name) && trimmed.ends_with('{') {
            depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if trimmed.contains(needle) {
                return true;
            }
            continue;
        }

        if depth > 0 {
            if trimmed.contains(needle) {
                return true;
            }
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        }
    }

    false
}

fn layout_order_is_valid(layout: &str) -> bool {
    let mut default = None;
    let mut tab = None;
    let mut new = None;
    let mut depth = 0i32;

    for (index, line) in layout.lines().enumerate() {
        let trimmed = line.trim();
        if depth == 1 {
            if trimmed.starts_with("default_tab_template") {
                default = Some(index);
            } else if trimmed == "tab" || trimmed.starts_with("tab ") {
                tab = Some(index);
            } else if trimmed.starts_with("new_tab_template") {
                new = Some(index);
            }
        }
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
    }

    matches!((default, tab, new), (Some(default), Some(tab), Some(new)) if default < tab && tab < new)
}

fn bar_layout_is_valid(layout: &str) -> bool {
    let bars = layout.matches("share/nova_bar/zjstatus.wasm").count();
    let native_status_bars = layout.matches(r#"plugin location="status-bar""#).count();
    let tab_only_bars = layout.matches(r#"format_left   "{tabs}""#).count();
    let segmented_bars = layout
        .lines()
        .filter(|line| line.trim().starts_with("format_right_separator "))
        .count();
    let version_widgets = layout
        .lines()
        .filter(|line| {
            line.trim().starts_with("format_right ") && line.contains("{command_version}")
        })
        .count();
    bars == 3
        && native_status_bars == 3
        && tab_only_bars == 3
        && segmented_bars == bars
        && version_widgets == bars
        && rendered_bar_widgets_are_valid(layout)
        && !layout.contains(r#"YZX " // {datetime}"#)
        && !layout.contains("NOVA ")
        && !layout.contains("{mode}")
        && !layout.contains("mode_normal")
        && !layout.contains(r#"plugin location="tab-bar""#)
}

fn rendered_bar_widgets_are_valid(layout: &str) -> bool {
    [
        " hx",
        "❯nu",
        "{command_term}",
        "{command_codex_usage}",
        "{command_cpu}",
        "{command_ram}",
        r#"command_term_command ""#,
        r#"command_codex_usage_command ""#,
        r#"command_cpu_command ""#,
        r#"command_ram_command ""#,
        "--display quota --periods 5h,week",
        r#"--runtime-dir /nix/store/"#,
    ]
    .into_iter()
    .all(|needle| layout.contains(needle))
}
