use std::io;

use crossterm::{
    cursor,
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind,
        KeyModifiers,
    },
    execute,
    style::Print,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use ratconfig::{ConfigUiApp, ConfigUiIntent, ConfigUiKey, draw_config_ui};

use crate::{
    common::*,
    file_actions::{
        edit_text_externally, open_file_action, write_source_default, write_source_field,
    },
    model::build_model,
    paths::ensure_config_sources,
};

const RESET_TERMINAL_BACKGROUND: &str = "\x1b]111\x07";

pub(crate) fn run_ui() -> Result<()> {
    let paths = ensure_config_sources()?;
    let mut app = ConfigUiApp::try_new(build_model(&paths)?).map_err(error)?;
    let mut session = TerminalSession::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        terminal.draw(|frame| draw_config_ui(frame, &mut app))?;
        let Some(key) = config_event(event::read()?) else {
            continue;
        };
        match app.handle_key(key) {
            ConfigUiIntent::Exit => break,
            ConfigUiIntent::None => {}
            ConfigUiIntent::EditTextExternally { field, input } => {
                let result = session.suspend(|| edit_text_externally(&field.path, &input))?;
                terminal.clear()?;
                match result {
                    Ok(edited) => {
                        if let Err(message) = app.apply_external_text_edit(&field, edited) {
                            app.notice_error(message);
                        }
                    }
                    Err(error) => app.notice_error(error.to_string()),
                }
            }
            ConfigUiIntent::OpenFile {
                source_id,
                action_id,
                path,
                create_if_missing,
                ..
            } => {
                if let Some(guidance) = paths.home_manager_guidance(&path) {
                    app.notice_error(guidance);
                    continue;
                }
                let result = session.suspend(|| {
                    open_file_action(&paths, &source_id, &action_id, &path, create_if_missing)
                })?;
                terminal.clear()?;
                app.replace_model(build_model(&paths)?).map_err(error)?;
                match result {
                    Ok(()) => app.notice_info(format!("Opened {}.", path.display())),
                    Err(error) => app.notice_error(error.to_string()),
                }
            }
            ConfigUiIntent::SetField { field, value } => {
                if let Err(source) =
                    write_source_field(&paths, &field.source_id, &field.path, &value)
                {
                    app.notice_error(source.to_string());
                    app.replace_model(build_model(&paths)?).map_err(error)?;
                    continue;
                }
                app.replace_model_after_success(build_model(&paths)?, &field)
                    .map_err(error)?;
                app.notice_info(format!("Saved {}.", field.path));
            }
            ConfigUiIntent::UnsetField { field } => {
                if let Err(source) = write_source_default(&paths, &field.source_id, &field.path) {
                    app.notice_error(source.to_string());
                    app.replace_model(build_model(&paths)?).map_err(error)?;
                    continue;
                }
                app.replace_model_after_success(build_model(&paths)?, &field)
                    .map_err(error)?;
                app.notice_info(format!("Removed override for {}.", field.path));
            }
        }
    }

    Ok(())
}

pub(crate) struct TerminalSession;
impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let session = Self;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableBracketedPaste,
            cursor::Hide
        )?;
        Ok(session)
    }

    fn suspend<T>(&mut self, action: impl FnOnce() -> Result<T>) -> Result<Result<T>> {
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            DisableBracketedPaste,
            cursor::Show,
            LeaveAlternateScreen
        )?;
        let result = action();
        enable_raw_mode()?;
        execute!(io::stdout(), Print(RESET_TERMINAL_BACKGROUND))?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableBracketedPaste,
            cursor::Hide
        )?;
        Ok(result)
    }
}
impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            DisableBracketedPaste,
            cursor::Show,
            LeaveAlternateScreen
        );
    }
}
pub(crate) fn config_event(event: Event) -> Option<ConfigUiKey> {
    let key = match event {
        Event::Key(key) => key,
        Event::Paste(text) => return Some(ConfigUiKey::Paste(text)),
        _ => return None,
    };
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let unsupported =
        KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META;
    match key.code {
        KeyCode::Esc => Some(ConfigUiKey::Esc),
        KeyCode::Enter => Some(ConfigUiKey::Enter),
        KeyCode::Backspace => Some(ConfigUiKey::Backspace),
        KeyCode::Delete => Some(ConfigUiKey::Delete),
        KeyCode::Home => Some(ConfigUiKey::Home),
        KeyCode::End => Some(ConfigUiKey::End),
        KeyCode::Tab => Some(ConfigUiKey::Tab),
        KeyCode::BackTab => Some(ConfigUiKey::BackTab),
        KeyCode::Up => Some(ConfigUiKey::Up),
        KeyCode::Down => Some(ConfigUiKey::Down),
        KeyCode::Left => Some(ConfigUiKey::Left),
        KeyCode::Right => Some(ConfigUiKey::Right),
        KeyCode::Char(_) if key.modifiers.intersects(unsupported) => None,
        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ConfigUiKey::Ctrl(ch))
        }
        KeyCode::Char(ch) => Some(ConfigUiKey::Char(ch)),
        _ => None,
    }
}
