//! Interactive first-run config generator for `yzx onboard`.

use crate::active_config_surface::{
    PrimaryConfigPaths, ensure_managed_toml_tooling_config, primary_config_paths,
    validate_primary_config_surface,
};
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, runtime_dir_from_env};
use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, read};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, disable_raw_mode, enable_raw_mode, is_raw_mode_enabled,
};
use serde_json::json;
use std::fs;
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
struct OnboardCliArgs {
    force: bool,
    dry_run: bool,
    help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Choice {
    label: &'static str,
    value: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SingleQuestion {
    prompt: &'static str,
    choices: Vec<Choice>,
    default_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MultiQuestion {
    prompt: &'static str,
    choices: Vec<Choice>,
    selected: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OnboardAnswers {
    terminal: String,
    shell: String,
    editor_command: String,
    hide_sidebar_on_file_open: bool,
    widget_tray: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptEvent {
    Up,
    Down,
    Toggle,
    Confirm,
    Abort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PromptOutcome {
    Continue,
    SingleSelected(usize),
    MultiSelected(Vec<usize>),
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptState {
    cursor: usize,
    selected: Vec<bool>,
    multi: bool,
}

impl PromptState {
    fn single(default_index: usize, choice_count: usize) -> Self {
        Self {
            cursor: default_index.min(choice_count.saturating_sub(1)),
            selected: vec![false; choice_count],
            multi: false,
        }
    }

    fn multi(selected: Vec<bool>) -> Self {
        Self {
            cursor: 0,
            selected,
            multi: true,
        }
    }

    fn apply(&mut self, event: PromptEvent) -> PromptOutcome {
        if self.selected.is_empty() {
            return PromptOutcome::Aborted;
        }

        match event {
            PromptEvent::Up => {
                self.cursor = if self.cursor == 0 {
                    self.selected.len() - 1
                } else {
                    self.cursor - 1
                };
                PromptOutcome::Continue
            }
            PromptEvent::Down => {
                self.cursor = (self.cursor + 1) % self.selected.len();
                PromptOutcome::Continue
            }
            PromptEvent::Toggle if self.multi => {
                self.selected[self.cursor] = !self.selected[self.cursor];
                PromptOutcome::Continue
            }
            PromptEvent::Toggle => PromptOutcome::SingleSelected(self.cursor),
            PromptEvent::Confirm if self.multi => PromptOutcome::MultiSelected(
                self.selected
                    .iter()
                    .enumerate()
                    .filter_map(|(index, selected)| selected.then_some(index))
                    .collect(),
            ),
            PromptEvent::Confirm => PromptOutcome::SingleSelected(self.cursor),
            PromptEvent::Abort => PromptOutcome::Aborted,
        }
    }
}

struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> Result<Self, CoreError> {
        enable_raw_mode().map_err(|error| {
            CoreError::classified(
                ErrorClass::Runtime,
                "onboard_raw_mode_failed",
                format!("Could not enter terminal raw mode for onboarding: {error}"),
                "Run `yzx onboard` from an interactive terminal, or edit yazelix.toml manually.",
                json!({}),
            )
        })?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub fn run_yzx_onboard(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_onboard_cli_args(args)?;
    if parsed.help {
        print_onboard_help();
        return Ok(0);
    }

    let answers = run_interactive_onboarding()?;
    let generated = build_onboard_config(&answers);
    if parsed.dry_run {
        print!("{generated}");
        return Ok(0);
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = primary_config_paths(&runtime_dir, &config_dir);
    write_onboard_config(&paths, &generated, parsed.force)?;
    println!("Wrote {}", paths.user_config.display());
    println!("Run `yzx doctor` if you want to verify the generated setup.");
    Ok(0)
}

fn parse_onboard_cli_args(args: &[String]) -> Result<OnboardCliArgs, CoreError> {
    let mut out = OnboardCliArgs {
        force: false,
        dry_run: false,
        help: false,
    };
    for token in args {
        match token.as_str() {
            "--force" => out.force = true,
            "--dry-run" => out.dry_run = true,
            "--help" | "-h" | "help" => out.help = true,
            other => {
                return Err(CoreError::classified(
                    ErrorClass::Usage,
                    "unexpected_onboard_token",
                    format!("Unexpected argument for yzx onboard: {other}"),
                    "Run `yzx onboard`, `yzx onboard --dry-run`, or `yzx onboard --force`.",
                    json!({}),
                ));
            }
        }
    }
    Ok(out)
}

fn print_onboard_help() {
    println!("Generate a focused first-run Yazelix config");
    println!();
    println!("Usage:");
    println!("  yzx onboard [--force] [--dry-run]");
    println!();
    println!("Flags:");
    println!("      --force    Overwrite the managed user yazelix.toml if it already exists");
    println!("      --dry-run  Print the generated config instead of writing it");
}

fn run_interactive_onboarding() -> Result<OnboardAnswers, CoreError> {
    println!("Yazelix onboard");
    println!("Use arrow keys to move, Space to toggle multi-select choices, Enter to confirm.");
    println!("Press q or Esc to abort.");
    println!();

    let _raw = RawModeGuard::enable()?;
    let terminal = ask_single(terminal_question())?;
    let shell = ask_single(shell_question())?;
    let editor_command = ask_single(editor_question())?;
    let hide_sidebar_on_file_open = ask_single(sidebar_file_open_question())? == "true";
    let widget_tray = ask_multi(widget_tray_question())?;

    Ok(OnboardAnswers {
        terminal,
        shell,
        editor_command,
        hide_sidebar_on_file_open,
        widget_tray,
    })
}

fn ask_single(question: SingleQuestion) -> Result<String, CoreError> {
    let mut state = PromptState::single(question.default_index, question.choices.len());
    loop {
        render_single_question(&question, &state)?;
        match state.apply(read_prompt_event()?) {
            PromptOutcome::Continue => {}
            PromptOutcome::SingleSelected(index) => {
                finish_prompt_line()?;
                return Ok(question.choices[index].value.to_string());
            }
            PromptOutcome::Aborted => return Err(onboard_aborted_error()),
            PromptOutcome::MultiSelected(_) => unreachable!("single prompt cannot select multi"),
        }
    }
}

fn ask_multi(question: MultiQuestion) -> Result<Vec<String>, CoreError> {
    let mut state = PromptState::multi(question.selected.clone());
    loop {
        render_multi_question(&question, &state)?;
        match state.apply(read_prompt_event()?) {
            PromptOutcome::Continue => {}
            PromptOutcome::MultiSelected(indexes) => {
                finish_prompt_line()?;
                return Ok(indexes
                    .into_iter()
                    .map(|index| question.choices[index].value.to_string())
                    .collect());
            }
            PromptOutcome::Aborted => return Err(onboard_aborted_error()),
            PromptOutcome::SingleSelected(_) => unreachable!("multi prompt cannot select single"),
        }
    }
}

fn read_prompt_event() -> Result<PromptEvent, CoreError> {
    loop {
        match read().map_err(|error| {
            CoreError::classified(
                ErrorClass::Runtime,
                "onboard_key_read_failed",
                format!("Could not read onboarding key input: {error}"),
                "Run `yzx onboard` from an interactive terminal, or edit yazelix.toml manually.",
                json!({}),
            )
        })? {
            Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) => {
                return Ok(match code {
                    KeyCode::Up | KeyCode::Char('k') => PromptEvent::Up,
                    KeyCode::Down | KeyCode::Char('j') => PromptEvent::Down,
                    KeyCode::Char(' ') => PromptEvent::Toggle,
                    KeyCode::Enter => PromptEvent::Confirm,
                    KeyCode::Esc | KeyCode::Char('q') => PromptEvent::Abort,
                    _ => continue,
                });
            }
            _ => {}
        }
    }
}

fn render_single_question(question: &SingleQuestion, state: &PromptState) -> Result<(), CoreError> {
    render_prompt_header(question.prompt)?;
    for (index, choice) in question.choices.iter().enumerate() {
        let marker = if index == state.cursor { ">" } else { " " };
        println!("{marker} {}", choice.label);
    }
    Ok(())
}

fn render_multi_question(question: &MultiQuestion, state: &PromptState) -> Result<(), CoreError> {
    render_prompt_header(question.prompt)?;
    for (index, choice) in question.choices.iter().enumerate() {
        let marker = if index == state.cursor { ">" } else { " " };
        let checked = if state.selected[index] { "[x]" } else { "[ ]" };
        println!("{marker} {checked} {}", choice.label);
    }
    Ok(())
}

fn render_prompt_header(prompt: &str) -> Result<(), CoreError> {
    let mut stdout = io::stdout();
    if is_raw_mode_enabled().unwrap_or(false) {
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).map_err(|error| {
            CoreError::classified(
                ErrorClass::Runtime,
                "onboard_render_failed",
                format!("Could not render onboarding prompt: {error}"),
                "Retry from a normal interactive terminal.",
                json!({}),
            )
        })?;
    }
    println!("{prompt}");
    stdout.flush().map_err(|error| {
        CoreError::io(
            "onboard_render_flush_failed",
            "Could not flush onboarding prompt output",
            "Retry from a normal interactive terminal.",
            "stdout",
            error,
        )
    })
}

fn finish_prompt_line() -> Result<(), CoreError> {
    println!();
    io::stdout().flush().map_err(|error| {
        CoreError::io(
            "onboard_render_flush_failed",
            "Could not flush onboarding prompt output",
            "Retry from a normal interactive terminal.",
            "stdout",
            error,
        )
    })
}

fn onboard_aborted_error() -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "onboard_aborted",
        "Yazelix onboarding was aborted.",
        "Run `yzx onboard` again when you are ready, or edit yazelix.toml manually.",
        json!({}),
    )
}

fn single_question(
    prompt: &'static str,
    default_index: usize,
    choices: &[(&'static str, &'static str)],
) -> SingleQuestion {
    SingleQuestion {
        prompt,
        choices: choices
            .iter()
            .map(|(label, value)| Choice {
                label: *label,
                value: *value,
            })
            .collect(),
        default_index,
    }
}

fn multi_question(
    prompt: &'static str,
    choices: &[(&'static str, &'static str)],
    selected: &[bool],
) -> MultiQuestion {
    MultiQuestion {
        prompt,
        choices: choices
            .iter()
            .map(|(label, value)| Choice {
                label: *label,
                value: *value,
            })
            .collect(),
        selected: selected.to_vec(),
    }
}

fn terminal_question() -> SingleQuestion {
    single_question(
        "Primary terminal emulator",
        0,
        &[
            ("Ghostty (recommended)", "ghostty"),
            ("WezTerm", "wezterm"),
            ("Kitty", "kitty"),
            ("Alacritty", "alacritty"),
            ("Foot", "foot"),
        ],
    )
}

fn shell_question() -> SingleQuestion {
    single_question(
        "Default shell inside Yazelix",
        0,
        &[
            ("Nushell (recommended)", "nu"),
            ("Bash", "bash"),
            ("Zsh", "zsh"),
            ("Fish", "fish"),
        ],
    )
}

fn editor_question() -> SingleQuestion {
    single_question(
        "Editor command",
        0,
        &[
            ("Yazelix Helix (recommended)", ""),
            ("Neovim", "nvim"),
            ("System Helix", "hx"),
            ("Vim", "vim"),
        ],
    )
}

fn sidebar_file_open_question() -> SingleQuestion {
    single_question(
        "Yazi sidebar after opening a file",
        0,
        &[("Keep visible", "false"), ("Hide after file open", "true")],
    )
}

fn widget_tray_question() -> MultiQuestion {
    multi_question(
        "Status-bar widgets",
        &[
            ("Editor", "editor"),
            ("Shell", "shell"),
            ("Terminal", "term"),
            ("Workspace", "workspace"),
            ("Cursor preset", "cursor"),
            ("Claude 5h/week usage and quota", "claude_usage"),
            ("Codex 5h/week reset timing and quota", "codex_usage"),
            (
                "OpenCode Go 5h/week/month usage and quota",
                "opencode_go_usage",
            ),
            ("CPU", "cpu"),
            ("RAM", "ram"),
        ],
        &[
            true, true, true, false, false, false, false, false, true, true,
        ],
    )
}

fn write_onboard_config(
    paths: &PrimaryConfigPaths,
    generated: &str,
    force: bool,
) -> Result<(), CoreError> {
    validate_primary_config_surface(paths)?;
    ensure_managed_toml_tooling_config(
        &paths.runtime_toml_tooling_config,
        &paths.managed_toml_tooling_config,
    )?;

    if paths.user_config.exists() && !force {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "onboard_config_exists",
            format!(
                "Yazelix config already exists at {}.",
                paths.user_config.display()
            ),
            "Run `yzx onboard --force` to overwrite it, or edit the existing yazelix.toml manually.",
            json!({ "path": paths.user_config.display().to_string() }),
        ));
    }

    fs::create_dir_all(&paths.user_config_dir).map_err(|error| {
        CoreError::io(
            "onboard_config_dir_failed",
            "Could not create the Yazelix user config directory",
            "Fix directory permissions, then retry `yzx onboard`.",
            paths.user_config_dir.display().to_string(),
            error,
        )
    })?;
    fs::write(&paths.user_config, generated).map_err(|error| {
        CoreError::io(
            "onboard_config_write_failed",
            "Could not write the generated Yazelix config",
            "Fix file permissions, then retry `yzx onboard`.",
            paths.user_config.display().to_string(),
            error,
        )
    })?;
    Ok(())
}

fn build_onboard_config(answers: &OnboardAnswers) -> String {
    format!(
        r#"# Generated by yzx onboard
# Edit this file directly when your preferences change.

[editor]
command = "{}"
hide_sidebar_on_file_open = {}
sidebar_command = "nu"
sidebar_args = ["__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu"]

[shell]
default_shell = "{}"

[terminal]
terminals = [{}]

[zellij]
widget_tray = [{}]
"#,
        toml_escape_string(&answers.editor_command),
        answers.hide_sidebar_on_file_open,
        toml_escape_string(&answers.shell),
        toml_array_strings(std::slice::from_ref(&answers.terminal)),
        toml_array_strings(&answers.widget_tray),
    )
}

fn toml_array_strings(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", toml_escape_string(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn toml_escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_config_surface::TOML_TOOLING_CONFIG_FILENAME;
    use tempfile::tempdir;

    fn test_paths(root: &std::path::Path) -> PrimaryConfigPaths {
        let runtime = root.join("runtime");
        let config = root.join("config");
        fs::create_dir_all(runtime.join("config_metadata")).unwrap();
        fs::write(runtime.join(TOML_TOOLING_CONFIG_FILENAME), "[format]\n").unwrap();
        fs::write(runtime.join("yazelix_default.toml"), "").unwrap();
        fs::write(
            runtime.join("config_metadata/main_config_contract.toml"),
            "",
        )
        .unwrap();
        primary_config_paths(&runtime, &config)
    }

    // Defends: single-choice onboarding prompts support arrow-style navigation before confirmation.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn prompt_state_selects_single_choice_with_navigation() {
        let mut state = PromptState::single(0, 3);

        assert_eq!(state.apply(PromptEvent::Down), PromptOutcome::Continue);
        assert_eq!(state.apply(PromptEvent::Down), PromptOutcome::Continue);
        assert_eq!(state.apply(PromptEvent::Up), PromptOutcome::Continue);
        assert_eq!(
            state.apply(PromptEvent::Confirm),
            PromptOutcome::SingleSelected(1)
        );
    }

    // Defends: multi-select onboarding prompts support toggling choices before confirmation.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn prompt_state_toggles_multi_select_choices() {
        let mut state = PromptState::multi(vec![true, true, false]);

        assert_eq!(state.apply(PromptEvent::Toggle), PromptOutcome::Continue);
        assert_eq!(state.apply(PromptEvent::Down), PromptOutcome::Continue);
        assert_eq!(state.apply(PromptEvent::Down), PromptOutcome::Continue);
        assert_eq!(state.apply(PromptEvent::Toggle), PromptOutcome::Continue);
        assert_eq!(
            state.apply(PromptEvent::Confirm),
            PromptOutcome::MultiSelected(vec![1, 2])
        );
    }

    // Defends: interrupted onboarding exits before writing a config.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn prompt_state_reports_abort() {
        let mut state = PromptState::single(0, 2);

        assert_eq!(state.apply(PromptEvent::Abort), PromptOutcome::Aborted);
    }

    // Defends: onboarding emits a valid readable main yazelix.toml using current supported config fields only.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn onboard_config_is_valid_current_main_config() {
        let config = build_onboard_config(&OnboardAnswers {
            terminal: "wezterm".into(),
            shell: "bash".into(),
            editor_command: "nvim".into(),
            hide_sidebar_on_file_open: true,
            widget_tray: vec!["editor".into(), "cpu".into()],
        });
        let parsed: toml::Value = toml::from_str(&config).unwrap();

        assert_eq!(parsed["editor"]["command"].as_str(), Some("nvim"));
        assert_eq!(
            parsed["editor"]["hide_sidebar_on_file_open"].as_bool(),
            Some(true)
        );
        assert_eq!(parsed["editor"]["sidebar_command"].as_str(), Some("nu"));
        assert_eq!(
            parsed["editor"]["sidebar_args"].as_array().unwrap()[0].as_str(),
            Some("__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu")
        );
        assert_eq!(parsed["shell"]["default_shell"].as_str(), Some("bash"));
        assert_eq!(
            parsed["terminal"]["terminals"].as_array().unwrap()[0].as_str(),
            Some("wezterm")
        );
    }

    // Defends: onboarding writes only the supported main config surface and refuses accidental overwrite by default.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn write_onboard_config_writes_main_config_only_and_requires_force() {
        let tmp = tempdir().unwrap();
        let paths = test_paths(tmp.path());
        let config = build_onboard_config(&OnboardAnswers {
            terminal: "ghostty".into(),
            shell: "nu".into(),
            editor_command: String::new(),
            hide_sidebar_on_file_open: false,
            widget_tray: vec!["editor".into(), "shell".into()],
        });

        write_onboard_config(&paths, &config, false).unwrap();
        assert!(paths.user_config.is_file());
        assert!(!paths.user_config_dir.join("yazelix_packs.toml").exists());
        assert!(write_onboard_config(&paths, &config, false).is_err());
        write_onboard_config(&paths, &config, true).unwrap();
    }
}
