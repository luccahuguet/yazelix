use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::runtime_dir_from_env;
use crate::public_command_surface::{YzxCommandMetadata, YzxMenuCategory, yzx_command_metadata};
use crate::terminal_control;
use crossterm::style::Color;
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuMode {
    Direct,
    Popup,
    Pane,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaletteEntry {
    command: String,
    label: String,
}

fn menu_error(class: ErrorClass, code: &'static str, message: impl Into<String>) -> CoreError {
    CoreError::classified(class, code, message, "Run `yzx menu --help`.", json!({}))
}

fn parse_mode(args: &[String]) -> Result<MenuMode, CoreError> {
    let (mut popup, mut pane, mut help) = (false, false, false);
    for arg in args {
        match arg.as_str() {
            "--popup" => popup = true,
            "--pane" => pane = true,
            "--help" | "-h" | "help" => help = true,
            other => {
                return Err(menu_error(
                    ErrorClass::Usage,
                    "unexpected_menu_token",
                    format!("Unexpected argument for yzx menu: {other}"),
                ));
            }
        }
    }

    match (help, popup, pane) {
        (true, _, _) => Ok(MenuMode::Help),
        (_, true, true) => Err(menu_error(
            ErrorClass::Usage,
            "conflicting_menu_modes",
            "Use either `yzx menu --popup` or `yzx menu --pane`, not both.",
        )),
        (_, true, false) => Ok(MenuMode::Popup),
        (_, false, true) => Ok(MenuMode::Pane),
        _ => Ok(MenuMode::Direct),
    }
}

fn print_menu_help() {
    println!("Interactive command palette for Yazelix");
    println!();
    println!("Usage:");
    println!("  yzx menu [--popup | --pane]");
    println!();
    println!("Flags:");
    println!("      --popup  Open menu in a Zellij floating pane");
    println!("      --pane   Run the popup-pane menu UI in the current pane");
}

fn category_name(category: YzxMenuCategory) -> &'static str {
    match category {
        YzxMenuCategory::Config => "config",
        YzxMenuCategory::Help => "help",
        YzxMenuCategory::Session => "session",
        YzxMenuCategory::System => "system",
        YzxMenuCategory::Workspace => "workspace",
    }
}

fn category_color(category: YzxMenuCategory) -> Color {
    match category {
        YzxMenuCategory::Session => Color::Green,
        YzxMenuCategory::Workspace => Color::Cyan,
        YzxMenuCategory::Config => Color::Blue,
        YzxMenuCategory::System => Color::Yellow,
        YzxMenuCategory::Help => Color::Magenta,
    }
}

fn palette_description(command: &YzxCommandMetadata) -> &'static str {
    match command.extra_description {
        Some(value) if !value.trim().is_empty() => value,
        _ => command.description,
    }
}

fn palette_entry(command: &YzxCommandMetadata) -> Option<PaletteEntry> {
    let category = command.menu_category?;
    let tag = terminal_control::styled(
        format!("[{}]", category_name(category)),
        category_color(category),
    );
    let description = palette_description(command).trim();
    let label = if description.is_empty() {
        format!("{}  {tag}", command.name)
    } else {
        format!(
            "{}  {tag}  {}",
            command.name,
            terminal_control::styled(format!("- {description}"), Color::DarkGrey)
        )
    };
    Some(PaletteEntry {
        command: command.name.to_string(),
        label,
    })
}

fn palette_entries() -> Vec<PaletteEntry> {
    let mut entries = yzx_command_metadata()
        .iter()
        .filter_map(palette_entry)
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.command.cmp(&right.command));
    entries
}

fn select_matching_entry<'a>(
    entries: &'a [PaletteEntry],
    selected: &str,
) -> Option<&'a PaletteEntry> {
    let selected = selected.trim();
    entries.iter().find(|entry| {
        selected == entry.command || selected.starts_with(&format!("{}  ", entry.command))
    })
}

fn io_error(
    message: impl Into<String>,
    remediation: &'static str,
    source: std::io::Error,
) -> CoreError {
    CoreError::io("yzx_menu", message, remediation, ".", source)
}

fn select_with_fzf(entries: &[PaletteEntry]) -> Result<Option<String>, CoreError> {
    let mut child = Command::new("fzf")
        .args([
            "--ansi",
            "--border",
            "rounded",
            "--header",
            "  Yazelix Command Palette",
            "--prompt",
            "  yzx> ",
            "--pointer",
            "\u{25b8}",
            "--layout",
            "reverse",
            "--cycle",
            "--color",
            "border:blue,header:bold:blue,prompt:bold:yellow,pointer:bold:cyan,hl:bold:magenta,hl+:bold:magenta,info:dim",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|source| {
            io_error(
                "Failed to launch fzf for the Yazelix command palette.",
                "Install fzf or run a non-menu yzx command directly.",
                source,
            )
        })?;

    let input = entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    child
        .stdin
        .take()
        .ok_or_else(|| {
            menu_error(
                ErrorClass::Internal,
                "missing_fzf_stdin",
                "Failed to open fzf stdin.",
            )
        })?
        .write_all(input.as_bytes())
        .map_err(|source| {
            io_error(
                "Failed to write command palette entries to fzf.",
                "Retry the command palette or run the command directly.",
                source,
            )
        })?;

    let output = child.wait_with_output().map_err(|source| {
        io_error(
            "Failed to read fzf command palette selection.",
            "Retry the command palette or run the command directly.",
            source,
        )
    })?;
    Ok(output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string()))
}

fn yzx_cli_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("shells").join("posix").join("yzx_cli.sh")
}

fn command_tail(command: &str) -> Vec<String> {
    command
        .strip_prefix("yzx ")
        .unwrap_or(command)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

fn run_menu_action(runtime_dir: &Path, command: &str) -> Result<i32, CoreError> {
    let status = Command::new("sh")
        .arg(yzx_cli_path(runtime_dir))
        .args(command_tail(command))
        .status()
        .map_err(|source| {
            io_error(
                format!("Failed to run selected Yazelix command: {command}"),
                "Run the selected command directly.",
                source,
            )
        })?;
    Ok(status.code().unwrap_or(1))
}

fn should_pause_in_popup(command: &str) -> bool {
    !(command.starts_with("yzx launch")
        || command.starts_with("yzx enter")
        || command.starts_with("yzx env")
        || command.starts_with("yzx restart"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupDecision {
    Menu,
    Close,
    Continue,
}

fn popup_key_decision(key: crossterm::event::KeyCode) -> PopupDecision {
    match key {
        crossterm::event::KeyCode::Backspace => PopupDecision::Menu,
        crossterm::event::KeyCode::Enter => PopupDecision::Close,
        _ => PopupDecision::Continue,
    }
}

fn popup_post_action_decision() -> Result<PopupDecision, CoreError> {
    println!();
    println!("Backspace: return to menu | Enter: close");
    crossterm::terminal::enable_raw_mode().map_err(|source| {
        io_error(
            "Failed to enable raw mode for the menu popup prompt.",
            "Close the popup and retry the command.",
            source,
        )
    })?;

    let result = loop {
        match crossterm::event::read() {
            Ok(crossterm::event::Event::Key(event)) => match popup_key_decision(event.code) {
                PopupDecision::Continue => {}
                decision => break Ok(decision),
            },
            Ok(_) => {}
            Err(source) => {
                break Err(io_error(
                    "Failed to read the menu popup prompt key.",
                    "Close the popup and retry the command.",
                    source,
                ));
            }
        }
    };
    let _ = crossterm::terminal::disable_raw_mode();
    result
}

fn clear_screen() {
    let _ = terminal_control::clear_screen_now();
}

fn selected_entry<'a>(
    entries: &'a [PaletteEntry],
    selected: Option<String>,
) -> Result<Option<&'a PaletteEntry>, CoreError> {
    let Some(selected) = selected else {
        return Ok(None);
    };
    select_matching_entry(entries, &selected)
        .map(Some)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Internal,
                "unknown_menu_selection",
                "fzf returned a command palette entry Yazelix did not recognize.",
                "Retry the command palette.",
                json!({ "selection": selected }),
            )
        })
}

fn run_popup_palette(runtime_dir: &Path, entries: &[PaletteEntry]) -> Result<i32, CoreError> {
    loop {
        let Some(entry) = selected_entry(entries, select_with_fzf(entries)?)? else {
            return Ok(0);
        };
        let code = run_menu_action(runtime_dir, &entry.command)?;
        if should_pause_in_popup(&entry.command)
            && popup_post_action_decision()? == PopupDecision::Menu
        {
            clear_screen();
            continue;
        }
        return Ok(code);
    }
}

fn run_direct_palette(runtime_dir: &Path, entries: &[PaletteEntry]) -> Result<i32, CoreError> {
    let Some(entry) = selected_entry(entries, select_with_fzf(entries)?)? else {
        return Ok(0);
    };
    run_menu_action(runtime_dir, &entry.command)
}

fn in_zellij() -> bool {
    std::env::var("ZELLIJ")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn open_popup() -> Result<i32, CoreError> {
    if !in_zellij() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "not_in_zellij",
            "Not in a Zellij session; run `yzx menu` directly or start Yazelix/Zellij first.",
            "Start Yazelix or run `yzx menu` without --popup.",
            json!({}),
        ));
    }

    let output = Command::new("zellij")
        .args([
            "action", "pipe", "--plugin", "yzpp", "--name", "toggle", "--", "menu",
        ])
        .output()
        .map_err(|source| {
            io_error(
                "Failed to open the Yazelix menu popup pane.",
                "Check that zellij is available in the active Yazelix runtime.",
                source,
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() && matches!(stdout.as_str(), "ok" | "opened" | "focused" | "closed")
    {
        return Ok(0);
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "menu_popup_pipe_failed",
        stderr
            .is_empty()
            .then(|| format!("Failed to open the Yazelix menu popup pane: {stdout}"))
            .unwrap_or(stderr),
        "Check that the Yazelix popup plugin is loaded, then retry.",
        json!({ "response": stdout }),
    ))
}

pub fn run_yzx_menu(args: &[String]) -> Result<i32, CoreError> {
    match parse_mode(args)? {
        MenuMode::Help => {
            print_menu_help();
            Ok(0)
        }
        MenuMode::Popup => open_popup(),
        mode => {
            let runtime_dir = runtime_dir_from_env()?;
            let entries = palette_entries();
            match mode {
                MenuMode::Pane => run_popup_palette(&runtime_dir, &entries),
                MenuMode::Direct => run_direct_palette(&runtime_dir, &entries),
                MenuMode::Help | MenuMode::Popup => unreachable!(),
            }
        }
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: Rust-owned menu entries still derive visibility from shared command metadata.
    #[test]
    fn palette_entries_include_metadata_categories_and_exclude_menu_itself() {
        let entries = palette_entries();
        assert!(entries.iter().any(|entry| entry.command == "yzx status"));
        assert!(entries.iter().any(|entry| entry.label.contains("[system]")));
        assert!(!entries.iter().any(|entry| entry.command == "yzx menu"));
    }

    // Defends: fzf label matching remains stable when labels include ANSI category styling.
    #[test]
    fn selected_entry_matches_command_prefix_before_ansi_label_content() {
        let styled_system = terminal_control::styled("[system]", Color::Yellow);
        let entries = vec![PaletteEntry {
            command: "yzx status".to_string(),
            label: format!("yzx status  {styled_system}"),
        }];
        assert_eq!(
            select_matching_entry(&entries, &format!("yzx status  {styled_system}"))
                .unwrap()
                .command,
            "yzx status"
        );
    }

    // Defends: selected commands dispatch through argv, not a shell string.
    #[test]
    fn command_tail_strips_yzx_prefix_without_shell_parsing() {
        assert_eq!(
            command_tail("yzx update home_manager"),
            ["update", "home_manager"]
        );
    }

    // Defends: popup mode keeps the old Backspace/Enter post-action contract.
    #[test]
    fn popup_key_decision_matches_old_menu_contract() {
        assert_eq!(
            popup_key_decision(crossterm::event::KeyCode::Backspace),
            PopupDecision::Menu
        );
        assert_eq!(
            popup_key_decision(crossterm::event::KeyCode::Enter),
            PopupDecision::Close
        );
        assert_eq!(
            popup_key_decision(crossterm::event::KeyCode::Char('x')),
            PopupDecision::Continue
        );
    }

    // Defends: menu argument parsing rejects mutually exclusive presentation modes.
    #[test]
    fn parse_rejects_conflicting_menu_modes() {
        let err = parse_mode(&["--popup".to_string(), "--pane".to_string()]).unwrap_err();
        assert!(matches!(err.class(), ErrorClass::Usage));
        assert_eq!(err.code(), "conflicting_menu_modes");
    }
}
