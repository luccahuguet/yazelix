use std::io;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
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

pub(crate) fn run_ui() -> Result<()> {
    let paths = ensure_config_sources()?;
    let mut app = ConfigUiApp::new(build_model(&paths)?);
    let mut session = TerminalSession::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        terminal.draw(|frame| draw_config_ui(frame, &mut app))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        let Some(key) = config_key(key) else {
            continue;
        };
        match app.handle_key(key) {
            ConfigUiIntent::Exit => break,
            ConfigUiIntent::None => {}
            ConfigUiIntent::BeginEdit { field_index, .. } => app.begin_edit_field(field_index),
            ConfigUiIntent::EditTextExternally {
                field_index, input, ..
            } => {
                let result = session.suspend(|| edit_text_externally(&input))?;
                match result {
                    Ok(edited) => {
                        if let Err(message) = app.apply_external_text_edit(field_index, edited) {
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
                let result = session.suspend(|| {
                    open_file_action(&paths, &source_id, &action_id, &path, create_if_missing)
                })?;
                app.model = build_model(&paths)?;
                match result {
                    Ok(()) => app.notice_info(format!("Opened {}.", path.display())),
                    Err(error) => app.notice_error(error.to_string()),
                }
            }
            ConfigUiIntent::SetField {
                field_index,
                source_id,
                path: field_path,
                value,
            } => {
                if let Err(error) = write_source_field(&paths, &source_id, &field_path, &value) {
                    app.notice_error(error.to_string());
                    app.model = build_model(&paths)?;
                    continue;
                }
                app.model = build_model(&paths)?;
                app.notice_info(format!("Saved {field_path}."));
                app.finish_successful_set_field(field_index, &value);
            }
            ConfigUiIntent::UnsetField {
                field_index,
                source_id,
                path: field_path,
            } => {
                if let Err(error) = write_source_default(&paths, &source_id, &field_path) {
                    app.notice_error(error.to_string());
                    app.model = build_model(&paths)?;
                    continue;
                }
                app.model = build_model(&paths)?;
                app.notice_info(format!("Restored default for {field_path}."));
                app.finish_successful_unset_field(field_index);
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
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(session)
    }

    fn suspend<T>(&mut self, action: impl FnOnce() -> Result<T>) -> Result<Result<T>> {
        disable_raw_mode()?;
        execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
        let result = action();
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(result)
    }
}
impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);
    }
}
pub(crate) fn config_key(key: KeyEvent) -> Option<ConfigUiKey> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let unsupported =
        KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META;
    match key.code {
        KeyCode::Esc => Some(ConfigUiKey::Esc),
        KeyCode::Enter => Some(ConfigUiKey::Enter),
        KeyCode::Backspace => Some(ConfigUiKey::Backspace),
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
