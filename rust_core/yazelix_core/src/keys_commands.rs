// Test lane: default
//! `yzx keys*` family implemented in Rust for `yzx_control`.

use crate::bridge::CoreError;

const ANSI_RESET: &str = "\u{1b}[0m";
const ANSI_CYAN_BOLD: &str = "\u{1b}[1;36m";
const ANSI_YELLOW_BOLD: &str = "\u{1b}[1;33m";
const ANSI_WHITE: &str = "\u{1b}[37m";

const ROOT_ALIAS_TOKENS: &[&str] = &["yzx"];
const YAZI_ALIAS_TOKENS: &[&str] = &["yazi"];
const HELIX_ALIAS_TOKENS: &[&str] = &["hx", "helix"];
const NUSHELL_ALIAS_TOKENS: &[&str] = &["nu", "nushell"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeysView {
    Yazelix,
    Yazi,
    Helix,
    Nushell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeysArgs {
    view: KeysView,
    help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Column<'a> {
    heading: &'a str,
    width: usize,
}

struct TableRow {
    cells: Vec<String>,
}

pub fn run_yzx_keys(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_keys_args(args)?;
    if parsed.help {
        print_keys_help();
        return Ok(0);
    }

    let output = match parsed.view {
        KeysView::Yazelix => render_yazelix_keys(),
        KeysView::Yazi => render_yazi_keys(),
        KeysView::Helix => render_helix_keys(),
        KeysView::Nushell => render_nushell_keys(),
    };
    print!("{output}");
    Ok(0)
}

fn parse_keys_args(args: &[String]) -> Result<KeysArgs, CoreError> {
    let mut help = false;
    let mut tokens = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other => tokens.push(other),
        }
    }

    let view = match tokens.as_slice() {
        [] => KeysView::Yazelix,
        [token] if ROOT_ALIAS_TOKENS.contains(token) => KeysView::Yazelix,
        [token] if YAZI_ALIAS_TOKENS.contains(token) => KeysView::Yazi,
        [token] if HELIX_ALIAS_TOKENS.contains(token) => KeysView::Helix,
        [token] if NUSHELL_ALIAS_TOKENS.contains(token) => KeysView::Nushell,
        [other] => {
            return Err(CoreError::usage(format!(
                "Unknown yzx keys target: {other}. Try `yzx keys --help`."
            )));
        }
        _ => {
            return Err(CoreError::usage(
                "Unexpected arguments for yzx keys. Try `yzx keys --help`.",
            ));
        }
    };

    Ok(KeysArgs { view, help })
}

fn print_keys_help() {
    println!("Show Yazelix-owned keybindings and remaps");
    println!();
    println!("Usage:");
    println!("  yzx keys");
    println!("  yzx keys yzx");
    println!("  yzx keys yazi");
    println!("  yzx keys hx");
    println!("  yzx keys helix");
    println!("  yzx keys nu");
    println!("  yzx keys nushell");
}

fn heading(text: &str) -> String {
    format!("{ANSI_CYAN_BOLD}{text}{ANSI_RESET}")
}

fn label(text: &str) -> String {
    format!("{ANSI_YELLOW_BOLD}{text}{ANSI_RESET}")
}

fn accent_cmd(text: &str) -> String {
    format!("{ANSI_WHITE}{text}{ANSI_RESET}")
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let current_width = display_width(&current);
        let word_width = display_width(word);
        let needed = if current.is_empty() {
            word_width
        } else {
            current_width + 1 + word_width
        };

        if !current.is_empty() && needed > width {
            lines.push(current);
            current = word.to_string();
            continue;
        }

        if current.is_empty() {
            current.push_str(word);
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if current.is_empty() {
        lines.push(String::new());
    } else {
        lines.push(current);
    }

    lines
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

fn pad(text: &str, width: usize) -> String {
    let pad_len = width.saturating_sub(display_width(text));
    format!("{text}{}", " ".repeat(pad_len))
}

fn border(left: char, fill: char, join: char, right: char, widths: &[usize]) -> String {
    let mut out = String::new();
    out.push(left);
    for (index, width) in widths.iter().enumerate() {
        out.push_str(&fill.to_string().repeat(width + 2));
        if index + 1 == widths.len() {
            out.push(right);
        } else {
            out.push(join);
        }
    }
    out
}

fn render_table(columns: &[Column<'_>], rows: &[TableRow]) -> String {
    let mut widths = Vec::with_capacity(columns.len() + 1);
    widths.push(3);
    widths.extend(columns.iter().map(|column| column.width));

    let mut lines = Vec::new();
    lines.push(border('╭', '─', '┬', '╮', &widths));
    let mut header_cells = vec![pad("#", widths[0])];
    for (column, width) in columns.iter().zip(widths.iter().skip(1)) {
        header_cells.push(pad(column.heading, *width));
    }
    lines.push(format!("│ {} │", header_cells.join(" │ ")));
    lines.push(border('├', '─', '┼', '┤', &widths));

    for (index, row) in rows.iter().enumerate() {
        let wrapped_cells: Vec<Vec<String>> = row
            .cells
            .iter()
            .zip(columns.iter())
            .map(|(cell, column)| wrap_text(cell, column.width))
            .collect();
        let line_count = wrapped_cells.iter().map(Vec::len).max().unwrap_or(1);

        for line_index in 0..line_count {
            let mut cells = Vec::with_capacity(columns.len() + 1);
            let index_text = if line_index == 0 {
                index.to_string()
            } else {
                String::new()
            };
            cells.push(pad(&index_text, widths[0]));
            for (wrapped, width) in wrapped_cells.iter().zip(widths.iter().skip(1)) {
                let text = wrapped.get(line_index).map(String::as_str).unwrap_or("");
                cells.push(pad(text, *width));
            }
            lines.push(format!("│ {} │", cells.join(" │ ")));
        }
    }

    lines.push(border('╰', '─', '┴', '╯', &widths));
    lines.join("\n")
}

fn table_row(cells: &[&str]) -> TableRow {
    TableRow {
        cells: cells.iter().map(|cell| (*cell).to_string()).collect(),
    }
}

fn root_workspace_rows() -> Vec<TableRow> {
    vec![
        table_row(&[
            "Ctrl+y",
            "Toggle focus between the managed editor and sidebar",
        ]),
        table_row(&["Alt+y", "Toggle the sidebar open/closed"]),
        table_row(&["Alt+[ / Alt+]", "Switch between Yazelix layout families"]),
        table_row(&[
            "Alt+m",
            "Open a new terminal in the current tab workspace root",
        ]),
        table_row(&[
            "Alt+p",
            "In Yazi, open the selected directory in a new pane and make it the tab workspace root",
        ]),
        table_row(&[
            "Alt+z",
            "In Yazi, open a Zoxide picker and retarget the managed editor/workspace to the selected directory",
        ]),
    ]
}

fn root_command_rows() -> Vec<TableRow> {
    vec![
        table_row(&["Alt+t", "Toggle the configured managed popup program"]),
        table_row(&["Alt+Shift+M", "Open the yzx command palette popup"]),
    ]
}

fn root_tab_rows() -> Vec<TableRow> {
    vec![
        table_row(&["Alt+1..9", "Go directly to tab 1-9"]),
        table_row(&[
            "Alt+h / Alt+l",
            "Walk left/right across visible panes, falling back to previous/next tab",
        ]),
        table_row(&["Alt+w / Alt+q", "Walk next/previous tab"]),
        table_row(&["Alt+Shift+H / Alt+Shift+L", "Move current tab left/right"]),
        table_row(&["Alt+Shift+F", "Toggle pane fullscreen"]),
    ]
}

fn render_yazelix_keys() -> String {
    let workspace = render_table(
        &[
            Column {
                heading: "keybinding",
                width: 29,
            },
            Column {
                heading: "action",
                width: 56,
            },
        ],
        &root_workspace_rows(),
    );
    let command_access = render_table(
        &[
            Column {
                heading: "keybinding",
                width: 12,
            },
            Column {
                heading: "action",
                width: 43,
            },
        ],
        &root_command_rows(),
    );
    let tabs = render_table(
        &[
            Column {
                heading: "keybinding",
                width: 27,
            },
            Column {
                heading: "action",
                width: 46,
            },
        ],
        &root_tab_rows(),
    );

    [
        heading("Yazelix keybindings"),
        String::new(),
        heading("Workspace actions"),
        workspace,
        String::new(),
        heading("Command access"),
        command_access,
        String::new(),
        heading("Tab and pane movement"),
        tabs,
        String::new(),
        heading("More"),
        format!("{} {}", label("Yazi:"), accent_cmd("yzx keys yazi")),
        format!("{} {}", label("Helix:"), accent_cmd("yzx keys hx")),
        format!("{} {}", label("Nushell:"), accent_cmd("yzx keys nu")),
        String::new(),
    ]
    .join("\n")
}

fn yazi_rows() -> Vec<TableRow> {
    vec![
        table_row(&[
            "Open in editor",
            "`Enter`",
            "Uses Yazelix's configured editor opener",
        ]),
        table_row(&["Built-in open", "`o`", "Uses Yazi's built-in open action"]),
        table_row(&[
            "Open with",
            "`O`",
            "Shows Yazi's open menu — includes Reveal to open in system file manager",
        ]),
        table_row(&[
            "Yazelix workspace",
            "`Alt+p`",
            "Open the selected directory in a new pane and make it the tab workspace root",
        ]),
        table_row(&[
            "Native zoxide jump",
            "`Z`",
            "Use Yazi's built-in Zoxide jump and stay inside Yazi",
        ]),
        table_row(&[
            "Direct-open zoxide jump",
            "`Alt+z`",
            "Use Yazelix's bundled zoxide jump to retarget the managed editor/workspace immediately",
        ]),
        table_row(&[
            "Open key help",
            "Focus the Yazi pane and press `~`",
            "Shows Yazi's keybindings and commands",
        ]),
        table_row(&[
            "Optional",
            "Press `Alt+Shift+F` first",
            "Fullscreen the pane for easier reading",
        ]),
    ]
}

fn render_yazi_keys() -> String {
    let table = render_table(
        &[
            Column {
                heading: "step",
                width: 25,
            },
            Column {
                heading: "action",
                width: 35,
            },
            Column {
                heading: "notes",
                width: 18,
            },
        ],
        &yazi_rows(),
    );

    [
        heading("Yazi keybindings"),
        String::new(),
        table,
        String::new(),
        format!(
            "{} {}",
            label("For Yazelix-specific bindings:"),
            accent_cmd("yzx keys")
        ),
        String::new(),
    ]
    .join("\n")
}

fn helix_topic_rows() -> Vec<TableRow> {
    vec![
        table_row(&["Browse commands", "Press `<space>?`"]),
        table_row(&[
            "Full keymap docs",
            "https://docs.helix-editor.com/master/keymap.html",
        ]),
    ]
}

fn helix_caveat_rows() -> Vec<TableRow> {
    vec![table_row(&[
        "No default Helix-local Yazi binding in Yazelix",
        "Use Zellij-level `Ctrl+y` and `Alt+y` for managed workspace navigation",
    ])]
}

fn render_helix_keys() -> String {
    let topics = render_table(
        &[
            Column {
                heading: "topic",
                width: 16,
            },
            Column {
                heading: "how",
                width: 48,
            },
        ],
        &helix_topic_rows(),
    );
    let caveat = render_table(
        &[
            Column {
                heading: "caveat",
                width: 48,
            },
            Column {
                heading: "details",
                width: 25,
            },
        ],
        &helix_caveat_rows(),
    );

    [
        heading("Helix keybindings"),
        String::new(),
        topics,
        String::new(),
        caveat,
        String::new(),
        format!(
            "{} {}",
            label("For Yazelix-specific bindings:"),
            accent_cmd("yzx keys")
        ),
        String::new(),
    ]
    .join("\n")
}

fn nushell_rows() -> Vec<TableRow> {
    vec![
        table_row(&["Ctrl+r", "Search shell history", ""]),
        table_row(&[
            "Ctrl+f",
            "Complete the current history hint",
            "Different from Tab completion",
        ]),
        table_row(&["Ctrl+o", "Open the current command in your editor", ""]),
        table_row(&["Alt+Enter", "Insert a newline without executing", ""]),
    ]
}

fn render_nushell_keys() -> String {
    let table = render_table(
        &[
            Column {
                heading: "keybinding",
                width: 10,
            },
            Column {
                heading: "action",
                width: 39,
            },
            Column {
                heading: "notes",
                width: 19,
            },
        ],
        &nushell_rows(),
    );

    [
        heading("Nushell keybindings"),
        String::new(),
        table,
        String::new(),
        heading("More"),
        format!(
            "{} run {} inside Nushell",
            label("Guided intro:"),
            label("`tutor`")
        ),
        format!(
            "{} {}",
            label("Full reference:"),
            accent_cmd("https://www.nushell.sh/book/line_editor.html")
        ),
        format!(
            "{} {}",
            label("For Yazelix-specific bindings:"),
            accent_cmd("yzx keys")
        ),
        String::new(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the Rust-owned `yzx keys` family keeps the full alias set instead of collapsing discoverability leaves during the owner cut.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parses_keys_alias_family() {
        assert_eq!(
            parse_keys_args(&[]).unwrap(),
            KeysArgs {
                view: KeysView::Yazelix,
                help: false,
            }
        );
        assert_eq!(
            parse_keys_args(&["yzx".into()]).unwrap(),
            KeysArgs {
                view: KeysView::Yazelix,
                help: false,
            }
        );
        assert_eq!(
            parse_keys_args(&["yazi".into()]).unwrap(),
            KeysArgs {
                view: KeysView::Yazi,
                help: false,
            }
        );
        assert_eq!(
            parse_keys_args(&["hx".into()]).unwrap(),
            KeysArgs {
                view: KeysView::Helix,
                help: false,
            }
        );
        assert_eq!(
            parse_keys_args(&["helix".into()]).unwrap(),
            KeysArgs {
                view: KeysView::Helix,
                help: false,
            }
        );
        assert_eq!(
            parse_keys_args(&["nu".into()]).unwrap(),
            KeysArgs {
                view: KeysView::Nushell,
                help: false,
            }
        );
        assert_eq!(
            parse_keys_args(&["nushell".into()]).unwrap(),
            KeysArgs {
                view: KeysView::Nushell,
                help: false,
            }
        );
    }

    // Regression: the Rust owner cut must preserve the table-style discoverability surface instead of flattening the keys help into plain paragraphs.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_table_style_root_discoverability_surface() {
        let rendered = render_yazelix_keys();

        assert!(rendered.contains("Yazelix keybindings"));
        assert!(rendered.contains("Workspace actions"));
        assert!(rendered.contains("Command access"));
        assert!(rendered.contains("Tab and pane movement"));
        assert!(rendered.contains("╭"));
        assert!(rendered.contains("Alt+Shift+M"));
        assert!(rendered.contains("yzx keys yazi"));
    }
}
