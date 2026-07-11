//! Interactive first-run config generator for `yzx onboard`.

use crate::active_config_surface::{
    PrimaryConfigPaths, primary_config_paths, validate_primary_config_surface,
};
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, runtime_dir_from_env};
use crate::settings_surface::{parse_config_value, render_config_value, render_default_config};
use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, read};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, disable_raw_mode, enable_raw_mode, is_raw_mode_enabled,
};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
struct OnboardCliArgs {
    force: bool,
    dry_run: bool,
    help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Choice {
    label: String,
    value: String,
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
                "Run `yzx onboard` from an interactive terminal, or edit config.toml manually.",
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

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = primary_config_paths(&runtime_dir, &config_dir);
    let answers = run_interactive_onboarding(&paths.contract_path)?;
    let generated = build_onboard_config(&answers, &paths.default_config_path)?;
    if parsed.dry_run {
        print!("{generated}");
        return Ok(0);
    }

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
    println!("      --force    Overwrite the managed user config.toml if it already exists");
    println!("      --dry-run  Print the generated config instead of writing it");
}

fn run_interactive_onboarding(contract_path: &Path) -> Result<OnboardAnswers, CoreError> {
    println!("Yazelix onboard");
    println!("Use arrow keys to move, Space to toggle multi-select choices, Enter to confirm.");
    println!("Press q or Esc to abort.");
    println!();

    let _raw = RawModeGuard::enable()?;
    let shell = ask_single(shell_question())?;
    let editor_command = ask_single(editor_question())?;
    let hide_sidebar_on_file_open = ask_single(sidebar_file_open_question())? == "true";
    let widget_tray = ask_multi(widget_tray_question(contract_path)?)?;

    Ok(OnboardAnswers {
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
                return Ok(question.choices[index].value.clone());
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
                    .map(|index| question.choices[index].value.clone())
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
                "Run `yzx onboard` from an interactive terminal, or edit config.toml manually.",
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
        "Run `yzx onboard` again when you are ready, or edit config.toml manually.",
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
                label: (*label).to_string(),
                value: (*value).to_string(),
            })
            .collect(),
        default_index,
    }
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

fn widget_tray_question(contract_path: &Path) -> Result<MultiQuestion, CoreError> {
    let (allowed_values, default_values) = widget_tray_contract_values(contract_path)?;
    let choices = allowed_values
        .iter()
        .map(|value| {
            widget_tray_label(value).map(|label| Choice {
                label: label.to_string(),
                value: value.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let selected = allowed_values
        .iter()
        .map(|value| default_values.contains(value))
        .collect();

    Ok(MultiQuestion {
        prompt: "Status-bar widgets",
        choices,
        selected,
    })
}

fn widget_tray_label(value: &str) -> Result<&'static str, CoreError> {
    match value {
        "editor" => Ok("Editor"),
        "session" => Ok("Session"),
        "shell" => Ok("Shell"),
        "term" => Ok("Terminal"),
        "workspace" => Ok("Workspace"),
        "claude_usage" => Ok("Claude 5h/week usage and quota"),
        "codex_usage" => Ok("Codex 5h/week reset timing and quota"),
        "opencode_go_usage" => Ok("OpenCode Go 5h/week/month usage and quota"),
        "cpu" => Ok("CPU"),
        "ram" => Ok("RAM"),
        other => Err(CoreError::classified(
            ErrorClass::Internal,
            "missing_onboard_widget_label",
            format!("Status-bar widget {other} has no onboarding label."),
            "Add an onboarding label for every zellij.widget_tray allowed value.",
            json!({ "widget": other }),
        )),
    }
}

fn widget_tray_contract_values(
    contract_path: &Path,
) -> Result<(Vec<String>, Vec<String>), CoreError> {
    let raw = fs::read_to_string(contract_path).map_err(|error| {
        CoreError::io(
            "read_onboard_widget_contract",
            "Could not read the Yazelix config contract for onboarding",
            "Reinstall Yazelix so config_metadata/main_config_contract.toml is available.",
            contract_path.display().to_string(),
            error,
        )
    })?;
    let contract = ::toml::from_str::<::toml::Table>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_onboard_widget_contract",
            "Could not parse the Yazelix config contract for onboarding.",
            "Reinstall Yazelix so config_metadata/main_config_contract.toml is valid.",
            json!({ "path": contract_path.display().to_string(), "source": source.to_string() }),
        )
    })?;
    let field = contract
        .get("fields")
        .and_then(::toml::Value::as_table)
        .and_then(|fields| fields.get("zellij.widget_tray"))
        .and_then(::toml::Value::as_table)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "missing_onboard_widget_contract_field",
                "The Yazelix config contract is missing zellij.widget_tray.",
                "Reinstall Yazelix so config_metadata/main_config_contract.toml is current.",
                json!({ "path": contract_path.display().to_string() }),
            )
        })?;
    let allowed_values = widget_tray_contract_string_array(field, "allowed_values", contract_path)?;
    let default_values = widget_tray_contract_string_array(field, "default", contract_path)?;
    for value in &default_values {
        if !allowed_values.contains(value) {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "invalid_onboard_widget_contract_default",
                format!("zellij.widget_tray default contains unsupported value: {value}."),
                "Fix config_metadata/main_config_contract.toml, then retry.",
                json!({ "path": contract_path.display().to_string(), "widget": value }),
            ));
        }
    }

    Ok((allowed_values, default_values))
}

fn widget_tray_contract_string_array(
    field: &::toml::Table,
    key: &str,
    contract_path: &Path,
) -> Result<Vec<String>, CoreError> {
    field
        .get(key)
        .and_then(::toml::Value::as_array)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "invalid_onboard_widget_contract_field",
                format!("zellij.widget_tray is missing a {key} array."),
                "Fix config_metadata/main_config_contract.toml, then retry.",
                json!({ "path": contract_path.display().to_string(), "key": key }),
            )
        })?
        .iter()
        .map(|value| {
            value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Runtime,
                    "invalid_onboard_widget_contract_value",
                    format!("zellij.widget_tray {key} contains a non-string value."),
                    "Fix config_metadata/main_config_contract.toml, then retry.",
                    json!({
                        "path": contract_path.display().to_string(),
                        "key": key,
                        "value": value.to_string()
                    }),
                )
            })
        })
        .collect()
}

fn write_onboard_config(
    paths: &PrimaryConfigPaths,
    generated: &str,
    force: bool,
) -> Result<(), CoreError> {
    validate_primary_config_surface(paths)?;

    if paths.user_config.exists() && !force {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "onboard_config_exists",
            format!(
                "Yazelix config already exists at {}.",
                paths.user_config.display()
            ),
            "Run `yzx onboard --force` to overwrite it, or edit the existing config.toml manually.",
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

fn build_onboard_config(
    answers: &OnboardAnswers,
    default_main_config: &Path,
) -> Result<String, CoreError> {
    let default_toml = render_default_config(default_main_config)?;
    let mut settings = parse_config_value(Path::new("config.toml"), &default_toml)?;

    set_settings_field(
        &mut settings,
        "editor",
        "command",
        JsonValue::String(answers.editor_command.clone()),
    )?;
    set_settings_field(
        &mut settings,
        "editor",
        "hide_sidebar_on_file_open",
        JsonValue::Bool(answers.hide_sidebar_on_file_open),
    )?;
    set_settings_field(
        &mut settings,
        "shell",
        "default_shell",
        JsonValue::String(answers.shell.clone()),
    )?;
    set_settings_field(
        &mut settings,
        "zellij",
        "widget_tray",
        JsonValue::Array(
            answers
                .widget_tray
                .iter()
                .cloned()
                .map(JsonValue::String)
                .collect(),
        ),
    )?;

    render_config_value(&settings)
}

fn set_settings_field(
    settings: &mut JsonValue,
    section: &str,
    key: &str,
    value: JsonValue,
) -> Result<(), CoreError> {
    let root = settings
        .as_object_mut()
        .ok_or_else(onboard_settings_shape_error)?;
    let section = root
        .entry(section.to_string())
        .or_insert_with(|| JsonValue::Object(JsonMap::new()))
        .as_object_mut()
        .ok_or_else(onboard_settings_shape_error)?;
    section.insert(key.to_string(), value);
    Ok(())
}

fn onboard_settings_shape_error() -> CoreError {
    CoreError::classified(
        ErrorClass::Internal,
        "onboard_settings_shape",
        "Could not apply onboarding answers to the default settings shape.",
        "Report this as a Yazelix internal error.",
        json!({}),
    )
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_paths(root: &std::path::Path) -> PrimaryConfigPaths {
        let runtime = root.join("runtime");
        let config = root.join("config");
        fs::create_dir_all(runtime.join("config_metadata")).unwrap();
        fs::write(runtime.join("config_default.toml"), "").unwrap();
        fs::write(
            runtime.join("config_metadata/main_config_contract.toml"),
            "",
        )
        .unwrap();
        primary_config_paths(&runtime, &config)
    }

    fn repo_contract_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("config_metadata/main_config_contract.toml")
    }

    // Defends: single-choice onboarding prompts support arrow-style navigation before confirmation.
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
    #[test]
    fn prompt_state_reports_abort() {
        let mut state = PromptState::single(0, 2);

        assert_eq!(state.apply(PromptEvent::Abort), PromptOutcome::Aborted);
    }

    // Defends: onboarding keeps Helix selection simple; custom forks use helix.external instead of a duplicate editor choice.
    #[test]
    fn editor_question_does_not_offer_system_helix_mode() {
        let question = editor_question();
        let labels = question
            .choices
            .iter()
            .map(|choice| choice.label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(labels, vec!["Yazelix Helix (recommended)", "Neovim", "Vim"]);
    }

    // Regression: first-run onboarding derives the status-widget choices from the config contract and cannot retain retired values.
    #[test]
    fn widget_tray_question_matches_contract_and_excludes_retired_cursor_widget() {
        let contract_path = repo_contract_path();
        let (allowed, defaults) = widget_tray_contract_values(&contract_path).unwrap();
        let question = widget_tray_question(&contract_path).unwrap();
        let values = question
            .choices
            .iter()
            .map(|choice| choice.value.clone())
            .collect::<Vec<_>>();
        let selected = question
            .choices
            .iter()
            .zip(question.selected.iter())
            .filter_map(|(choice, selected)| selected.then_some(choice.value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(values, allowed);
        assert_eq!(selected, defaults);
        assert_eq!(question.selected.len(), question.choices.len());
        assert!(!values.contains(&"cursor".to_string()));
    }

    // Defends: onboarding emits valid config.toml with current supported main config fields.
    #[test]
    fn onboard_config_is_valid_current_main_config() {
        let tmp = tempdir().unwrap();
        let paths = test_paths(tmp.path());
        fs::write(
            &paths.default_config_path,
            include_str!("../../../config_default.toml"),
        )
        .unwrap();
        let config = build_onboard_config(
            &OnboardAnswers {
                shell: "bash".into(),
                editor_command: "nvim".into(),
                hide_sidebar_on_file_open: true,
                widget_tray: vec!["editor".into(), "cpu".into()],
            },
            &paths.default_config_path,
        )
        .unwrap();
        let parsed = parse_config_value(Path::new("config.toml"), &config).unwrap();

        assert_eq!(parsed["editor"]["command"].as_str(), Some("nvim"));
        assert_eq!(
            parsed["editor"]["hide_sidebar_on_file_open"].as_bool(),
            Some(true)
        );
        assert_eq!(
            parsed["workspace"]["left_sidebar"]["command"].as_str(),
            Some("yzx")
        );
        assert_eq!(
            parsed["workspace"]["left_sidebar"]["args"]
                .as_array()
                .unwrap()[0]
                .as_str(),
            Some("sidebar")
        );
        assert_eq!(
            parsed["workspace"]["left_sidebar"]["args"]
                .as_array()
                .unwrap()[1]
                .as_str(),
            Some("yazi")
        );
        assert_eq!(parsed["shell"]["default_shell"].as_str(), Some("bash"));
        assert!(parsed["terminal"].get("terminals").is_none());
        assert!(parsed.get("cursors").is_none());
    }

    // Defends: onboarding writes only the supported main config surface and refuses accidental overwrite by default.
    #[test]
    fn write_onboard_config_writes_main_config_only_and_requires_force() {
        let tmp = tempdir().unwrap();
        let paths = test_paths(tmp.path());
        fs::write(
            &paths.default_config_path,
            include_str!("../../../config_default.toml"),
        )
        .unwrap();
        let config = build_onboard_config(
            &OnboardAnswers {
                shell: "nu".into(),
                editor_command: String::new(),
                hide_sidebar_on_file_open: false,
                widget_tray: vec!["editor".into(), "shell".into()],
            },
            &paths.default_config_path,
        )
        .unwrap();

        write_onboard_config(&paths, &config, false).unwrap();
        assert!(paths.user_config.is_file());
        assert!(!paths.user_config_dir.join("yazelix_packs.toml").exists());
        assert!(write_onboard_config(&paths, &config, false).is_err());
        write_onboard_config(&paths, &config, true).unwrap();
    }
}
