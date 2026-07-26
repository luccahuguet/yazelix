use std::{env, io};

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
use ratconfig::{ConfigUiApp, ConfigUiFieldId, ConfigUiIntent, ConfigUiKey, draw_config_ui};

use crate::{
    common::*,
    file_actions::{
        AppearanceProjection, MarsAppearanceProjection, ZellijAppearanceProjection,
        edit_text_externally, open_file_action, write_config_ui,
    },
    model::build_model,
    paths::{ConfigPaths, ensure_config_sources},
};

const RESET_TERMINAL_BACKGROUND: &str = "\x1b]111\x07";

pub(crate) fn run_ui() -> Result<()> {
    let paths = ensure_config_sources()?;
    let mut app = ConfigUiApp::try_new(build_model(&paths)?).map_err(error)?;
    let mut session = TerminalSession::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mars_included = env::var("YAZELIX_MARS_INCLUDED").as_deref() == Ok("1");

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
                apply_field_write(&mut app, &paths, field, Some(&value), mars_included)?;
            }
            ConfigUiIntent::UnsetField { field } => {
                apply_field_write(&mut app, &paths, field, None, mars_included)?;
            }
        }
    }

    Ok(())
}

fn apply_field_write(
    app: &mut ConfigUiApp,
    paths: &ConfigPaths,
    field: ConfigUiFieldId,
    value: Option<&serde_json::Value>,
    mars_included: bool,
) -> Result<()> {
    let reset = value.is_none();
    match write_config_ui(
        paths,
        &field.source_id,
        &field.path,
        value,
        mars_included,
        true,
    ) {
        Ok(projection) => reload_after_successful_write(
            app,
            build_model(paths)?,
            &field,
            write_notice(&field.path, projection, reset),
        ),
        Err(write_error) => {
            reload_after_failed_write(app, build_model(paths)?, write_error.to_string())
        }
    }
}

fn write_notice(field_path: &str, projection: Option<AppearanceProjection>, reset: bool) -> String {
    let action = if reset { "Now inheriting" } else { "Saved" };
    let Some(projection) = projection else {
        return format!("{action} {field_path}.");
    };
    let mut updates = Vec::new();
    match projection.mars {
        Some(MarsAppearanceProjection::Config) => {
            updates.push("Mars config is synchronized");
        }
        Some(MarsAppearanceProjection::Environment(_)) => {
            updates.push("Mars will apply it on the next launch");
        }
        None => {}
    }
    updates.push(match projection.zellij {
        ZellijAppearanceProjection::Live => "Zellij and the bar switched",
        ZellijAppearanceProjection::NextLaunch => {
            "Zellij and the bar will apply it on the next managed launch"
        }
    });
    format!("{action} {field_path}; {}.", updates.join("; "))
}

pub(crate) fn reload_after_failed_write(
    app: &mut ConfigUiApp,
    model: ratconfig::ConfigUiModel,
    message: String,
) -> Result<()> {
    app.notice_error(message);
    app.replace_model(model).map_err(error)
}

pub(crate) fn reload_after_successful_write(
    app: &mut ConfigUiApp,
    model: ratconfig::ConfigUiModel,
    field: &ratconfig::ConfigUiFieldId,
    message: String,
) -> Result<()> {
    app.replace_model_after_success(model, field)
        .map_err(error)?;
    app.notice_info(message);
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
