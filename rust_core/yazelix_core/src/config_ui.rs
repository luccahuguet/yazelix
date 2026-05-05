//! Terminal UI for inspecting and editing the canonical Yazelix config surface.

use crate::active_config_surface::{PrimaryConfigPaths, primary_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{ConfigDiagnostic, ConfigDiagnosticReport, NormalizeConfigRequest};
use crate::settings_jsonc_patch::{
    SettingsJsoncPatchMutation, set_settings_jsonc_value_text, unset_settings_jsonc_value_text,
};
use crate::settings_surface::{SETTINGS_SCHEMA_FILENAME, render_default_settings_jsonc};
use crate::settings_surface::{
    is_settings_config_path, parse_jsonc_value, read_settings_jsonc_value,
};
use crate::user_config_paths::{CURRENT_MANAGED_CONFIG_FILE_NAMES, SETTINGS_CONFIG};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;

const DEFAULT_TABS: &[&str] = &[
    "general", "editor", "terminal", "zellij", "yazi", "cursors", "advanced",
];
const HOME_MANAGER_FILES_MARKER: &str = "-home-manager-files/";
const HEADER_HORIZONTAL_PADDING: u16 = 1;

#[derive(Debug, Clone)]
pub struct ConfigUiRequest {
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_override: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiModel {
    pub active_config_path: PathBuf,
    pub active_config_exists: bool,
    pub config_owner: ConfigUiPathOwner,
    pub config_read_only: bool,
    pub tabs: Vec<String>,
    pub fields: Vec<ConfigUiField>,
    pub sidecars: Vec<ConfigUiSidecar>,
    pub diagnostics: Vec<ConfigUiDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigUiPathOwner {
    Default,
    HomeManager,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigUiValueState {
    Explicit,
    Defaulted,
    Unset,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiField {
    pub path: String,
    pub tab: String,
    pub kind: String,
    pub current_value: String,
    pub default_value: String,
    pub state: ConfigUiValueState,
    pub description: String,
    pub allowed_values: Vec<String>,
    pub validation: String,
    pub rebuild_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiSidecar {
    pub name: String,
    pub path: PathBuf,
    pub present: bool,
    pub owner: ConfigUiPathOwner,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiDiagnostic {
    pub path: String,
    pub status: String,
    pub headline: String,
    pub blocking: bool,
    pub detail_lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct ContractField {
    path: String,
    kind: String,
    default_value: Option<JsonValue>,
    validation: String,
    allowed_values: Vec<String>,
    min: Option<f64>,
    max: Option<f64>,
    rebuild_required: bool,
}

#[derive(Debug, Clone)]
struct SchemaField {
    path: String,
    tab: String,
    kind: String,
    description: String,
    allowed_values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiRowRef {
    Field(usize),
    Sidecar(usize),
    Diagnostic(usize),
}

struct ConfigUiApp {
    request: ConfigUiRequest,
    model: ConfigUiModel,
    selected_tab: usize,
    selected_row: usize,
    search: String,
    search_active: bool,
    edit: Option<ConfigUiEditState>,
    notice: Option<ConfigUiNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigUiEditState {
    field_index: usize,
    input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigUiNotice {
    text: String,
    is_error: bool,
}

pub fn build_config_ui_model(request: &ConfigUiRequest) -> Result<ConfigUiModel, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let schema = read_json_file(
        &paths.settings_schema_path,
        "read_settings_schema",
        "Could not read the Yazelix settings schema",
    )?;
    let tabs = schema_tabs(&schema);
    let top_level_tabs = schema_top_level_tabs(&schema);
    let active_config_path = active_config_path(&paths, request.config_override.as_deref());
    let active_config_exists = path_present(&active_config_path);
    let active_value = if active_config_exists {
        read_active_config_value(&active_config_path)?
    } else {
        JsonValue::Object(JsonMap::new())
    };
    ensure_root_object(&active_config_path, &active_value)?;

    let default_raw = render_default_settings_jsonc(
        &paths.default_config_path,
        &paths.default_cursor_config_path,
    )?;
    let default_value = parse_jsonc_value(&paths.default_config_path, &default_raw)?;
    ensure_root_object(&paths.default_config_path, &default_value)?;

    let contract_fields = load_contract_fields(&paths.contract_path)?;
    let diagnostics = if active_config_exists {
        collect_config_diagnostics(&active_config_path, &paths)?
    } else {
        Vec::new()
    };
    let blocking_paths = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.blocking)
        .map(|diagnostic| diagnostic.path.clone())
        .collect::<BTreeSet<_>>();

    let mut fields = Vec::new();
    for field in contract_fields.values() {
        let tab = tab_for_path(&field.path, &top_level_tabs);
        let current = get_json_path(&active_value, &field.path);
        let default = get_json_path(&default_value, &field.path)
            .cloned()
            .or_else(|| field.default_value.clone());
        fields.push(build_field_row(
            &field.path,
            &tab,
            &field.kind,
            current,
            default.as_ref(),
            field_description(field),
            field.allowed_values.clone(),
            field.validation.clone(),
            field.rebuild_required,
            blocking_paths.contains(&field.path),
        ));
    }

    for schema_field in collect_cursor_schema_fields(&schema) {
        if fields.iter().any(|field| field.path == schema_field.path) {
            continue;
        }
        let current = get_json_path(&active_value, &schema_field.path);
        let default = get_json_path(&default_value, &schema_field.path);
        fields.push(build_field_row(
            &schema_field.path,
            &schema_field.tab,
            &schema_field.kind,
            current,
            default,
            schema_field.description,
            schema_field.allowed_values,
            String::new(),
            false,
            blocking_paths.contains(&schema_field.path),
        ));
    }

    fields.sort_by(|left, right| {
        tab_index(&tabs, &left.tab)
            .cmp(&tab_index(&tabs, &right.tab))
            .then_with(|| left.path.cmp(&right.path))
    });

    Ok(ConfigUiModel {
        active_config_path: active_config_path.clone(),
        active_config_exists,
        config_owner: classify_path_owner(&active_config_path, active_config_exists),
        config_read_only: path_is_read_only(&active_config_path),
        tabs,
        fields,
        sidecars: collect_sidecars(&request.config_dir),
        diagnostics,
    })
}

pub fn run_config_ui(request: ConfigUiRequest) -> Result<i32, CoreError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "config_ui_requires_terminal",
            "`yzx config ui` requires an interactive terminal.",
            "Run `yzx config` for plain text output, or retry from an interactive shell.",
            json!({}),
        ));
    }

    let model = build_config_ui_model(&request)?;
    enable_raw_mode().map_err(terminal_err)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(terminal_err)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(terminal_err)?;
    let result = run_ui_loop(&mut terminal, request, model);
    let cleanup = restore_terminal(&mut terminal);

    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(0),
    }
}

fn run_ui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    request: ConfigUiRequest,
    model: ConfigUiModel,
) -> Result<(), CoreError> {
    let mut app = ConfigUiApp {
        request,
        model,
        selected_tab: 0,
        selected_row: 0,
        search: String::new(),
        search_active: false,
        edit: None,
        notice: None,
    };

    loop {
        app.clamp_selection();
        terminal
            .draw(|frame| draw_config_ui(frame, &mut app))
            .map_err(terminal_err)?;
        if event::poll(Duration::from_millis(200)).map_err(terminal_err)?
            && let Event::Key(key) = event::read().map_err(terminal_err)?
            && key.kind != KeyEventKind::Release
            && app.handle_key(key)
        {
            break;
        }
    }

    Ok(())
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), CoreError> {
    let mut first_error = disable_raw_mode().map_err(terminal_err).err();
    if let Err(error) = execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(terminal_err)
        && first_error.is_none()
    {
        first_error = Some(error);
    }
    if let Err(error) = terminal.show_cursor().map_err(terminal_err)
        && first_error.is_none()
    {
        first_error = Some(error);
    }
    first_error.map_or(Ok(()), Err)
}

fn draw_config_ui(frame: &mut Frame<'_>, app: &mut ConfigUiApp) {
    let area = frame.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(area);

    render_header(frame, app, root[0]);
    render_tabs(frame, app, root[1]);
    render_body(frame, app, root[2]);
    render_footer(frame, app, root[3]);
}

fn render_header(frame: &mut Frame<'_>, app: &ConfigUiApp, area: Rect) {
    let owner = owner_label(app.model.config_owner);
    let write_state = if app.model.config_read_only {
        "read-only"
    } else {
        "writable"
    };
    let source = if app.model.active_config_exists {
        app.model.active_config_path.display().to_string()
    } else {
        format!(
            "{} (missing; showing shipped defaults)",
            app.model.active_config_path.display()
        )
    };
    let warning_count = app
        .model
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.blocking)
        .count();
    let diagnostic_text = if warning_count > 0 {
        warning_count.to_string()
    } else {
        "ok".to_string()
    };
    let diagnostic_style = if warning_count > 0 {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let title = Line::from(vec![Span::styled(
        "Yazelix Config",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    frame.render_widget(Block::default().borders(Borders::BOTTOM), area);
    let horizontal_padding = HEADER_HORIZONTAL_PADDING.min(area.width / 2);
    let content = Rect {
        x: area.x + horizontal_padding,
        y: area.y,
        width: area
            .width
            .saturating_sub(horizontal_padding.saturating_mul(2)),
        height: area.height.saturating_sub(1).max(1),
    };
    let title_width = 15_u16.min(content.width);
    let gap = if content.width > title_width { 1 } else { 0 };
    let title_area = Rect {
        x: content.x,
        y: content.y,
        width: title_width,
        height: 1,
    };
    let metadata_area = Rect {
        x: content.x + title_width + gap,
        y: content.y,
        width: content.width.saturating_sub(title_width + gap),
        height: 1,
    };

    frame.render_widget(Paragraph::new(title).alignment(Alignment::Left), title_area);
    if metadata_area.width > 0 {
        frame.render_widget(
            Paragraph::new(header_metadata_line(
                &source,
                owner,
                write_state,
                &diagnostic_text,
                diagnostic_style,
                metadata_area.width as usize,
            ))
            .alignment(Alignment::Right),
            metadata_area,
        );
    }
}

fn header_metadata_line(
    source: &str,
    owner: &str,
    mode: &str,
    diagnostic: &str,
    diagnostic_style: Style,
    width: usize,
) -> Line<'static> {
    let fixed_width = "path: ".len()
        + "  owner: ".len()
        + owner.len()
        + "  mode: ".len()
        + mode.len()
        + "  diag: ".len()
        + diagnostic.len();
    let path = truncate_start(source, width.saturating_sub(fixed_width));
    Line::from(vec![
        Span::styled("path: ", metadata_key_style()),
        Span::styled(path, metadata_value_style()),
        Span::raw("  "),
        Span::styled("owner: ", metadata_key_style()),
        Span::styled(owner.to_string(), metadata_value_style()),
        Span::raw("  "),
        Span::styled("mode: ", metadata_key_style()),
        Span::styled(mode.to_string(), metadata_value_style()),
        Span::raw("  "),
        Span::styled("diag: ", metadata_key_style()),
        Span::styled(diagnostic.to_string(), diagnostic_style),
    ])
}

fn render_tabs(frame: &mut Frame<'_>, app: &ConfigUiApp, area: Rect) {
    let labels = app
        .model
        .tabs
        .iter()
        .map(|tab| Line::from(Span::raw(tab.clone())))
        .collect::<Vec<_>>();
    frame.render_widget(
        Tabs::new(labels)
            .select(app.selected_tab)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        area,
    );
}

fn render_body(frame: &mut Frame<'_>, app: &mut ConfigUiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(area);
    let rows = app.visible_rows();
    app.clamp_selection_for_len(rows.len());
    render_list(frame, app, chunks[0], &rows);
    render_details(frame, app, chunks[1], rows.get(app.selected_row).copied());
}

fn render_list(frame: &mut Frame<'_>, app: &ConfigUiApp, area: Rect, rows: &[UiRowRef]) {
    let items = rows
        .iter()
        .map(|row| ListItem::new(app.render_row(*row)))
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(app.selected_row));
    }
    let title = if app.search.is_empty() {
        "settings".to_string()
    } else {
        format!("settings filtered by {}", app.search)
    };
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        area,
        &mut state,
    );
}

fn render_details(frame: &mut Frame<'_>, app: &ConfigUiApp, area: Rect, row: Option<UiRowRef>) {
    let lines = match row {
        Some(row) => app.render_details(row),
        None => vec![Line::from(Span::styled(
            "No settings match this tab/search.",
            Style::default().fg(Color::Gray),
        ))],
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title("details").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, app: &ConfigUiApp, area: Rect) {
    if let Some(edit) = &app.edit {
        let field = &app.model.fields[edit.field_index];
        let editing = Line::from(vec![
            Span::styled("editing: ", Style::default().fg(Color::Yellow)),
            Span::raw(field.path.clone()),
            Span::raw(" = "),
            Span::styled(
                format!("{}_", edit.input),
                Style::default().fg(Color::White),
            ),
        ]);
        let status = app
            .notice
            .as_ref()
            .map(|notice| {
                let style = if notice.is_error {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };
                Line::from(Span::styled(
                    truncate(&notice.text, area.width as usize),
                    style,
                ))
            })
            .unwrap_or_else(|| {
                Line::from(vec![
                    Span::raw("Enter save  "),
                    Span::raw("Esc cancel  "),
                    Span::raw("Ctrl+u clear"),
                ])
            });
        frame.render_widget(Paragraph::new(vec![editing, status]), area);
        return;
    }

    let notice = app
        .notice
        .as_ref()
        .map(|notice| {
            let style = if notice.is_error {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };
            Line::from(Span::styled(
                truncate(&notice.text, area.width as usize),
                style,
            ))
        })
        .unwrap_or_else(|| {
            Line::from(vec![
                Span::raw("Enter/e edit  "),
                Span::raw("Space toggle/cycle  "),
                Span::raw("u unset"),
            ])
        });
    let search = if app.search_active {
        format!("search: {}_", app.search)
    } else if app.search.is_empty() {
        "/ search".to_string()
    } else {
        "Esc clears search".to_string()
    };
    let controls = Line::from(vec![
        Span::raw("q quit  "),
        Span::raw("Tab tabs  "),
        Span::raw("j/k move  "),
        Span::styled(search, Style::default().fg(Color::Yellow)),
    ]);
    frame.render_widget(Paragraph::new(vec![notice, controls]), area);
}

impl ConfigUiApp {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.edit.is_some() {
            self.handle_edit_key(key);
            return false;
        }

        if self.search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => self.search_active = false,
                KeyCode::Backspace => {
                    self.search.pop();
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.search.clear();
                }
                KeyCode::Char(ch) => {
                    self.search.push(ch);
                    self.selected_row = 0;
                }
                _ => {}
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('/') => self.search_active = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Enter | KeyCode::Char('e') => self.begin_edit_selected_field(),
            KeyCode::Char(' ') => self.quick_edit_selected_field(),
            KeyCode::Char('u') => self.unset_selected_field(),
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => self.next_tab(),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => self.previous_tab(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            _ => {}
        }

        false
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.edit = None;
                self.notice_info("Edit canceled.");
            }
            KeyCode::Enter => self.save_edit(),
            KeyCode::Backspace => {
                self.notice = None;
                if let Some(edit) = &mut self.edit {
                    edit.input.pop();
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.notice = None;
                if let Some(edit) = &mut self.edit {
                    edit.input.clear();
                }
            }
            KeyCode::Char(ch) => {
                self.notice = None;
                if let Some(edit) = &mut self.edit {
                    edit.input.push(ch);
                }
            }
            _ => {}
        }
    }

    fn selected_field_index(&self) -> Option<usize> {
        let row = self.visible_rows().get(self.selected_row).copied()?;
        match row {
            UiRowRef::Field(index) => Some(index),
            _ => None,
        }
    }

    fn begin_edit_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be edited.");
            return;
        };
        if let Err(error) = self.ensure_editable_config() {
            self.notice_error(error.message());
            return;
        }
        let field = &self.model.fields[field_index];
        self.edit = Some(ConfigUiEditState {
            field_index,
            input: edit_input_for_field(field),
        });
    }

    fn quick_edit_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be edited.");
            return;
        };
        if let Err(error) = self.ensure_editable_config() {
            self.notice_error(error.message());
            return;
        }
        let field = &self.model.fields[field_index];
        let value = if is_bool_field(field) {
            Some(JsonValue::Bool(!field_bool_value(field).unwrap_or(false)))
        } else if is_scalar_enum_field(field) && !field.allowed_values.is_empty() {
            Some(JsonValue::String(next_allowed_value(field)))
        } else {
            None
        };

        if let Some(value) = value {
            self.set_field_value(field_index, value);
        } else {
            self.begin_edit_selected_field();
        }
    }

    fn unset_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be unset.");
            return;
        };
        if let Err(error) = self.ensure_editable_config() {
            self.notice_error(error.message());
            return;
        }
        let path = self.model.fields[field_index].path.clone();
        match self.unset_field_value(&path) {
            Ok(mutation) => {
                if mutation == SettingsJsoncPatchMutation::Unchanged {
                    self.notice_info(format!("{path} was already unset."));
                } else {
                    self.notice_info(format!("Unset {path}."));
                }
            }
            Err(error) => self.notice_error(error.message()),
        }
    }

    fn save_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = self.model.fields[edit.field_index].clone();
        let value = match parse_edit_input(&field, &edit.input) {
            Ok(value) => value,
            Err(message) => {
                self.notice_error(message);
                return;
            }
        };
        self.set_field_value(edit.field_index, value);
        if self
            .notice
            .as_ref()
            .map(|notice| !notice.is_error)
            .unwrap_or(false)
        {
            self.edit = None;
        }
    }

    fn set_field_value(&mut self, field_index: usize, value: JsonValue) {
        let path = self.model.fields[field_index].path.clone();
        match self.write_field_value(&path, &value) {
            Ok(mutation) => {
                if mutation == SettingsJsoncPatchMutation::Unchanged {
                    self.notice_info(format!("{path} was already set."));
                } else {
                    self.notice_info(format!("Saved {path}."));
                }
            }
            Err(error) => self.notice_error(error.message()),
        }
    }

    fn write_field_value(
        &mut self,
        setting_path: &str,
        value: &JsonValue,
    ) -> Result<SettingsJsoncPatchMutation, CoreError> {
        self.ensure_editable_config()?;
        let config_path = self.model.active_config_path.clone();
        let raw = read_settings_for_edit_or_empty(&config_path)?;
        let outcome = set_settings_jsonc_value_text(&config_path, &raw, setting_path, value)?;
        if outcome.changed() {
            validate_patched_settings_for_ui(&self.request, &outcome.text)?;
            write_settings_edit(&config_path, &outcome.text)?;
        }
        self.reload_model_preserving_selection(setting_path)?;
        Ok(outcome.mutation)
    }

    fn unset_field_value(
        &mut self,
        setting_path: &str,
    ) -> Result<SettingsJsoncPatchMutation, CoreError> {
        self.ensure_editable_config()?;
        let config_path = self.model.active_config_path.clone();
        let raw = read_settings_for_edit_or_empty(&config_path)?;
        let outcome = unset_settings_jsonc_value_text(&config_path, &raw, setting_path)?;
        if outcome.changed() {
            validate_patched_settings_for_ui(&self.request, &outcome.text)?;
            write_settings_edit(&config_path, &outcome.text)?;
        }
        self.reload_model_preserving_selection(setting_path)?;
        Ok(outcome.mutation)
    }

    fn reload_model_preserving_selection(&mut self, selected_path: &str) -> Result<(), CoreError> {
        let selected_tab = self
            .model
            .fields
            .iter()
            .find(|field| field.path == selected_path)
            .map(|field| field.tab.clone());
        self.model = build_config_ui_model(&self.request)?;
        if let Some(tab) = selected_tab
            && let Some(tab_index) = self
                .model
                .tabs
                .iter()
                .position(|candidate| candidate == &tab)
        {
            self.selected_tab = tab_index;
        }
        self.selected_row = self
            .visible_rows()
            .iter()
            .position(|row| {
                matches!(
                    row,
                    UiRowRef::Field(index) if self.model.fields[*index].path == selected_path
                )
            })
            .unwrap_or(0);
        Ok(())
    }

    fn ensure_editable_config(&self) -> Result<(), CoreError> {
        if !is_settings_config_path(&self.model.active_config_path) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_config_edit_surface",
                format!(
                    "The config UI can only edit settings.jsonc, but the active config is {}.",
                    self.model.active_config_path.display()
                ),
                "Move this setting to settings.jsonc, or clear YAZELIX_CONFIG_OVERRIDE.",
                json!({ "path": self.model.active_config_path.display().to_string() }),
            ));
        }
        if self.model.config_owner == ConfigUiPathOwner::HomeManager {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "home_manager_owned_config",
                "This settings file is owned by Home Manager.",
                "Edit your Home Manager module options instead, then run home-manager switch.",
                json!({ "path": self.model.active_config_path.display().to_string() }),
            ));
        }
        if self.model.config_read_only {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "read_only_settings_config",
                format!(
                    "The active settings file is read-only: {}.",
                    self.model.active_config_path.display()
                ),
                "Fix file permissions or edit the owning configuration source.",
                json!({ "path": self.model.active_config_path.display().to_string() }),
            ));
        }
        Ok(())
    }

    fn notice_info(&mut self, text: impl Into<String>) {
        self.notice = Some(ConfigUiNotice {
            text: text.into(),
            is_error: false,
        });
    }

    fn notice_error(&mut self, text: impl Into<String>) {
        self.notice = Some(ConfigUiNotice {
            text: text.into(),
            is_error: true,
        });
    }

    fn visible_rows(&self) -> Vec<UiRowRef> {
        let tab = self
            .model
            .tabs
            .get(self.selected_tab)
            .map(String::as_str)
            .unwrap_or("general");
        let mut rows = Vec::new();
        if tab == "advanced" {
            rows.extend(
                self.model
                    .diagnostics
                    .iter()
                    .enumerate()
                    .filter(|(_, diagnostic)| self.matches_diagnostic(diagnostic))
                    .map(|(index, _)| UiRowRef::Diagnostic(index)),
            );
            rows.extend(
                self.model
                    .sidecars
                    .iter()
                    .enumerate()
                    .filter(|(_, sidecar)| self.matches_sidecar(sidecar))
                    .map(|(index, _)| UiRowRef::Sidecar(index)),
            );
            return rows;
        }

        rows.extend(
            self.model
                .fields
                .iter()
                .enumerate()
                .filter(|(_, field)| field.tab == tab && self.matches_field(field))
                .map(|(index, _)| UiRowRef::Field(index)),
        );
        rows
    }

    fn render_row(&self, row: UiRowRef) -> Line<'static> {
        match row {
            UiRowRef::Field(index) => {
                let field = &self.model.fields[index];
                Line::from(vec![
                    Span::styled(
                        fixed_label(state_label(field.state), 9),
                        state_style(field.state),
                    ),
                    Span::styled(truncate(&field.path, 42), config_key_style()),
                    Span::styled(
                        format!(" {}", truncate(&field.current_value, 28)),
                        Style::default().fg(Color::Gray),
                    ),
                ])
            }
            UiRowRef::Sidecar(index) => {
                let sidecar = &self.model.sidecars[index];
                let status = if sidecar.present {
                    "present"
                } else {
                    "missing"
                };
                let style = if sidecar.present {
                    sidecar_status_style(true)
                } else {
                    sidecar_status_style(false)
                };
                Line::from(vec![
                    Span::styled(fixed_label(status, 9), style),
                    Span::styled(sidecar.name.clone(), config_key_style()),
                ])
            }
            UiRowRef::Diagnostic(index) => {
                let diagnostic = &self.model.diagnostics[index];
                let style = if diagnostic.blocking {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Yellow)
                };
                Line::from(vec![
                    Span::styled(fixed_label(&diagnostic.status, 9), style),
                    Span::styled(truncate(&diagnostic.path, 42), config_key_style()),
                ])
            }
        }
    }

    fn render_details(&self, row: UiRowRef) -> Vec<Line<'static>> {
        match row {
            UiRowRef::Field(index) => field_detail_lines(&self.model.fields[index]),
            UiRowRef::Sidecar(index) => sidecar_detail_lines(&self.model.sidecars[index]),
            UiRowRef::Diagnostic(index) => diagnostic_detail_lines(&self.model.diagnostics[index]),
        }
    }

    fn matches_field(&self, field: &ConfigUiField) -> bool {
        self.search_matches([
            field.path.as_str(),
            field.current_value.as_str(),
            field.default_value.as_str(),
            field.description.as_str(),
        ])
    }

    fn matches_sidecar(&self, sidecar: &ConfigUiSidecar) -> bool {
        self.search_matches([
            sidecar.name.as_str(),
            sidecar.path.to_string_lossy().as_ref(),
            owner_label(sidecar.owner),
        ])
    }

    fn matches_diagnostic(&self, diagnostic: &ConfigUiDiagnostic) -> bool {
        self.search_matches([
            diagnostic.path.as_str(),
            diagnostic.status.as_str(),
            diagnostic.headline.as_str(),
        ])
    }

    fn search_matches<'a>(&self, candidates: impl IntoIterator<Item = &'a str>) -> bool {
        if self.search.is_empty() {
            return true;
        }
        let needle = self.search.to_ascii_lowercase();
        candidates
            .into_iter()
            .any(|candidate| candidate.to_ascii_lowercase().contains(&needle))
    }

    fn next_tab(&mut self) {
        if self.model.tabs.is_empty() {
            return;
        }
        self.selected_tab = (self.selected_tab + 1) % self.model.tabs.len();
        self.selected_row = 0;
    }

    fn previous_tab(&mut self) {
        if self.model.tabs.is_empty() {
            return;
        }
        self.selected_tab = if self.selected_tab == 0 {
            self.model.tabs.len() - 1
        } else {
            self.selected_tab - 1
        };
        self.selected_row = 0;
    }

    fn move_down(&mut self) {
        let len = self.visible_rows().len();
        if len > 0 {
            self.selected_row = (self.selected_row + 1).min(len - 1);
        }
    }

    fn move_up(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(1);
    }

    fn clamp_selection(&mut self) {
        if self.selected_tab >= self.model.tabs.len() {
            self.selected_tab = 0;
        }
        self.clamp_selection_for_len(self.visible_rows().len());
    }

    fn clamp_selection_for_len(&mut self, len: usize) {
        if len == 0 {
            self.selected_row = 0;
        } else if self.selected_row >= len {
            self.selected_row = len - 1;
        }
    }
}

fn field_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            field.path.clone(),
            config_key_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        detail_line("state", state_label(field.state)),
        detail_line("current", &field.current_value),
        detail_line("default", &field.default_value),
        detail_line("type", &field.kind),
    ];
    if !field.validation.is_empty() {
        lines.push(detail_line("validation", &field.validation));
    }
    if !field.allowed_values.is_empty() {
        lines.push(detail_line("allowed", &field.allowed_values.join(", ")));
    }
    if field.rebuild_required {
        lines.push(detail_line("rebuild", "required"));
    }
    if !field.description.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(field.description.clone()));
    }
    lines
}

fn sidecar_detail_lines(sidecar: &ConfigUiSidecar) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            sidecar.name.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        detail_line("path", &sidecar.path.display().to_string()),
        detail_line(
            "state",
            if sidecar.present {
                "present"
            } else {
                "missing"
            },
        ),
        detail_line("owner", owner_label(sidecar.owner)),
        detail_line(
            "write",
            if sidecar.read_only {
                "read-only"
            } else {
                "writable or absent"
            },
        ),
    ]
}

fn diagnostic_detail_lines(diagnostic: &ConfigUiDiagnostic) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            diagnostic.headline.clone(),
            Style::default()
                .fg(if diagnostic.blocking {
                    Color::Red
                } else {
                    Color::Yellow
                })
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        detail_line("path", &diagnostic.path),
        detail_line("status", &diagnostic.status),
        detail_line("blocking", if diagnostic.blocking { "yes" } else { "no" }),
    ];
    lines.push(Line::from(""));
    for detail in &diagnostic.detail_lines {
        lines.push(Line::from(detail.clone()));
    }
    lines
}

fn detail_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(fixed_label(label, 11), metadata_key_style()),
        Span::styled(value.to_string(), metadata_value_style()),
    ])
}

fn active_config_path(paths: &PrimaryConfigPaths, config_override: Option<&str>) -> PathBuf {
    match config_override.map(str::trim).filter(|raw| !raw.is_empty()) {
        Some(raw) => PathBuf::from(raw),
        None => paths.user_config.clone(),
    }
}

fn read_active_config_value(path: &Path) -> Result<JsonValue, CoreError> {
    if is_settings_config_path(path) {
        return read_settings_jsonc_value(path);
    }

    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_active_config",
            "Could not read the active Yazelix config",
            "Fix permissions or choose a readable config path, then retry.",
            path.display().to_string(),
            source,
        )
    })?;
    let table = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_toml",
            "Could not parse the active Yazelix config",
            "Fix the TOML syntax in the reported file and retry.",
            path.display().to_string(),
            source,
        )
    })?;
    toml_value_to_json(&TomlValue::Table(table))
}

fn ensure_root_object(path: &Path, value: &JsonValue) -> Result<(), CoreError> {
    if value.is_object() {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Config,
        "settings_jsonc_not_object",
        "Yazelix settings must contain a JSON object.",
        "Replace the settings file with a valid object, then retry.",
        json!({ "path": path.display().to_string() }),
    ))
}

fn read_json_file(path: &Path, code: &'static str, message: &str) -> Result<JsonValue, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            code,
            message,
            "Reinstall Yazelix so the runtime metadata exists and is readable.",
            path.display().to_string(),
            source,
        )
    })?;
    serde_json::from_str(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_settings_schema_json",
            format!(
                "Could not parse {SETTINGS_SCHEMA_FILENAME} at {}: {source}.",
                path.display()
            ),
            "Reinstall Yazelix so the runtime includes a valid settings schema.",
            json!({ "path": path.display().to_string() }),
        )
    })
}

fn load_contract_fields(path: &Path) -> Result<BTreeMap<String, ContractField>, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_contract",
            "Could not read the Yazelix config contract",
            "Reinstall Yazelix so the runtime includes config_metadata/main_config_contract.toml.",
            path.display().to_string(),
            source,
        )
    })?;
    let contract = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_config_contract",
            "Could not parse the Yazelix config contract",
            "Reinstall Yazelix so the runtime includes a valid config contract.",
            path.display().to_string(),
            source,
        )
    })?;
    let fields_table = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "missing_contract_fields",
                "The Yazelix config contract is missing its fields table.",
                "Reinstall Yazelix so the runtime includes the current config contract.",
                json!({ "path": path.display().to_string() }),
            )
        })?;

    let mut fields = BTreeMap::new();
    for (field_path, value) in fields_table {
        let table = value.as_table().ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_contract_field",
                format!("Config contract field {field_path} must be a TOML table."),
                "Reinstall Yazelix so the runtime includes a valid config contract.",
                json!({ "field": field_path }),
            )
        })?;
        let kind = table
            .get("kind")
            .and_then(TomlValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        let validation = table
            .get("validation")
            .and_then(TomlValue::as_str)
            .unwrap_or("")
            .to_string();
        let allowed_values = string_array(table.get("allowed_values"));
        let min = table.get("min").and_then(toml_number_as_f64);
        let max = table.get("max").and_then(toml_number_as_f64);
        let rebuild_required = table
            .get("rebuild_required")
            .and_then(TomlValue::as_bool)
            .unwrap_or(false);
        let default_value = table.get("default").map(toml_value_to_json).transpose()?;
        fields.insert(
            field_path.clone(),
            ContractField {
                path: field_path.clone(),
                kind,
                default_value,
                validation,
                allowed_values,
                min,
                max,
                rebuild_required,
            },
        );
    }

    Ok(fields)
}

fn collect_config_diagnostics(
    config_path: &Path,
    paths: &PrimaryConfigPaths,
) -> Result<Vec<ConfigUiDiagnostic>, CoreError> {
    let request = NormalizeConfigRequest {
        config_path: config_path.to_path_buf(),
        default_config_path: paths.default_config_path.clone(),
        contract_path: paths.contract_path.clone(),
        include_missing: false,
    };

    match crate::config_normalize::normalize_config(&request) {
        Ok(data) => Ok(map_diagnostics(
            data.diagnostic_report.doctor_diagnostics.as_slice(),
        )),
        Err(error) if error.code() == "unsupported_config" => {
            let report = serde_json::from_value::<ConfigDiagnosticReport>(error.details())
                .map_err(|source| {
                    CoreError::classified(
                        ErrorClass::Internal,
                        "invalid_config_ui_diagnostic_report",
                        format!("Could not decode config diagnostics for the config UI: {source}"),
                        "Rebuild or reinstall Yazelix so the Rust config helpers agree.",
                        json!({ "config_path": config_path.display().to_string() }),
                    )
                })?;
            Ok(map_diagnostics(report.doctor_diagnostics.as_slice()))
        }
        Err(error) => Err(error),
    }
}

fn map_diagnostics(diagnostics: &[ConfigDiagnostic]) -> Vec<ConfigUiDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| ConfigUiDiagnostic {
            path: diagnostic.path.clone(),
            status: diagnostic.status.clone(),
            headline: diagnostic.headline.clone(),
            blocking: diagnostic.blocking,
            detail_lines: diagnostic.detail_lines.clone(),
        })
        .collect()
}

fn schema_tabs(schema: &JsonValue) -> Vec<String> {
    let mut tabs = schema
        .get("x-yazelix")
        .and_then(|value| value.get("tabs"))
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        tabs = DEFAULT_TABS.iter().map(|tab| (*tab).to_string()).collect();
    }
    if !tabs.iter().any(|tab| tab == "advanced") {
        tabs.push("advanced".to_string());
    }
    tabs
}

fn schema_top_level_tabs(schema: &JsonValue) -> BTreeMap<String, String> {
    schema
        .get("properties")
        .and_then(JsonValue::as_object)
        .into_iter()
        .flat_map(|properties| properties.iter())
        .filter_map(|(name, value)| {
            let tab = value
                .get("x-yazelix")
                .and_then(|metadata| metadata.get("tab"))
                .and_then(JsonValue::as_str)?;
            Some((name.clone(), tab.to_string()))
        })
        .collect()
}

fn collect_cursor_schema_fields(schema: &JsonValue) -> Vec<SchemaField> {
    let mut fields = Vec::new();
    let Some(cursors) = schema
        .get("properties")
        .and_then(|properties| properties.get("cursors"))
    else {
        return fields;
    };
    collect_schema_fields(cursors, "cursors", "cursors", &mut fields);
    fields
}

fn collect_schema_fields(schema: &JsonValue, path: &str, tab: &str, out: &mut Vec<SchemaField>) {
    let kind = schema_type(schema);
    if kind == "object" {
        let Some(properties) = schema.get("properties").and_then(JsonValue::as_object) else {
            out.push(schema_field(schema, path, tab, kind));
            return;
        };
        for (name, property) in properties {
            collect_schema_fields(property, &format!("{path}.{name}"), tab, out);
        }
        return;
    }

    if kind == "array"
        && let Some(items) = schema.get("items")
        && items.get("type").and_then(JsonValue::as_str) == Some("object")
    {
        out.push(schema_field(schema, path, tab, kind));
        return;
    }

    out.push(schema_field(schema, path, tab, kind));
}

fn schema_field(schema: &JsonValue, path: &str, tab: &str, kind: String) -> SchemaField {
    SchemaField {
        path: path.to_string(),
        tab: tab.to_string(),
        kind,
        description: schema
            .get("description")
            .and_then(JsonValue::as_str)
            .unwrap_or("")
            .to_string(),
        allowed_values: schema_enum_values(schema),
    }
}

fn schema_type(schema: &JsonValue) -> String {
    schema
        .get("type")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn schema_enum_values(schema: &JsonValue) -> Vec<String> {
    schema
        .get("enum")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn tab_for_path(path: &str, top_level_tabs: &BTreeMap<String, String>) -> String {
    let root = path.split('.').next().unwrap_or("");
    top_level_tabs.get(root).cloned().unwrap_or_else(|| {
        match root {
            "core" => "general",
            "helix" | "editor" => "editor",
            "shell" | "terminal" => "terminal",
            "zellij" => "zellij",
            "yazi" => "yazi",
            "cursors" => "cursors",
            _ => "advanced",
        }
        .to_string()
    })
}

fn build_field_row(
    path: &str,
    tab: &str,
    kind: &str,
    current: Option<&JsonValue>,
    default: Option<&JsonValue>,
    description: String,
    allowed_values: Vec<String>,
    validation: String,
    rebuild_required: bool,
    has_blocking_diagnostic: bool,
) -> ConfigUiField {
    let state = if has_blocking_diagnostic {
        ConfigUiValueState::Invalid
    } else if current.is_some() {
        ConfigUiValueState::Explicit
    } else if default.is_some() {
        ConfigUiValueState::Defaulted
    } else {
        ConfigUiValueState::Unset
    };
    ConfigUiField {
        path: path.to_string(),
        tab: tab.to_string(),
        kind: kind.to_string(),
        current_value: current
            .or(default)
            .map(render_json_value)
            .unwrap_or_else(|| "not set".to_string()),
        default_value: default
            .map(render_json_value)
            .unwrap_or_else(|| "no default".to_string()),
        state,
        description,
        allowed_values,
        validation,
        rebuild_required,
    }
}

fn field_description(field: &ContractField) -> String {
    let mut parts = Vec::new();
    if !field.validation.is_empty() {
        parts.push(format!("validation: {}", field.validation));
    }
    if let (Some(min), Some(max)) = (field.min, field.max) {
        parts.push(format!("range: {min}..{max}"));
    }
    if field.rebuild_required {
        parts.push("takes effect after runtime rebuild or rematerialization".to_string());
    }
    parts.join("; ")
}

fn collect_sidecars(config_dir: &Path) -> Vec<ConfigUiSidecar> {
    CURRENT_MANAGED_CONFIG_FILE_NAMES
        .iter()
        .filter(|name| **name != SETTINGS_CONFIG)
        .map(|name| {
            let path = config_dir.join(name);
            let present = fs::symlink_metadata(&path).is_ok();
            ConfigUiSidecar {
                name: (*name).to_string(),
                owner: classify_path_owner(&path, present),
                read_only: path_is_read_only(&path),
                path,
                present,
            }
        })
        .collect()
}

fn classify_path_owner(path: &Path, present: bool) -> ConfigUiPathOwner {
    if !present {
        return ConfigUiPathOwner::Default;
    }
    if fs::read_link(path)
        .ok()
        .map(|target| target.to_string_lossy().contains(HOME_MANAGER_FILES_MARKER))
        .unwrap_or(false)
    {
        return ConfigUiPathOwner::HomeManager;
    }
    ConfigUiPathOwner::User
}

fn path_is_read_only(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().readonly())
        .unwrap_or(false)
}

fn path_present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn get_json_path<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let mut current = value;
    for part in path.split('.') {
        current = current.as_object()?.get(part)?;
    }
    Some(current)
}

fn render_json_value(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::String(value) => format!("{value:?}"),
        JsonValue::Array(values) => {
            if values.len() <= 4 {
                serde_json::to_string(values)
                    .unwrap_or_else(|_| format!("[{} items]", values.len()))
            } else {
                format!("[{} items]", values.len())
            }
        }
        JsonValue::Object(object) => format!("{{{} keys}}", object.len()),
    }
}

fn edit_input_for_field(field: &ConfigUiField) -> String {
    if field.current_value == "not set" {
        return String::new();
    }
    if is_string_field(field) || is_scalar_enum_field(field) {
        return parse_rendered_json_string(&field.current_value)
            .unwrap_or_else(|| field.current_value.clone());
    }
    field.current_value.clone()
}

fn parse_edit_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let trimmed = input.trim();
    match field.kind.as_str() {
        "bool" | "boolean" => parse_bool_input(field, trimmed),
        "int" | "integer" => parse_i64_input(field, trimmed),
        "float" | "number" => parse_f64_input(field, trimmed),
        "string" => parse_string_field_input(field, input),
        "string_list" => parse_string_list_input(field, trimmed),
        "array" => parse_json_input(field, trimmed, "JSON array").and_then(|value| {
            if value.is_array() {
                Ok(value)
            } else {
                Err(format!("{} must be a JSON array.", field.path))
            }
        }),
        "object" => parse_json_input(field, trimmed, "JSON object").and_then(|value| {
            if value.is_object() {
                Ok(value)
            } else {
                Err(format!("{} must be a JSON object.", field.path))
            }
        }),
        _ => parse_json_input(field, trimmed, "JSON value"),
    }
}

fn parse_bool_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    match input {
        "true" => Ok(JsonValue::Bool(true)),
        "false" => Ok(JsonValue::Bool(false)),
        _ => Err(format!("{} must be true or false.", field.path)),
    }
}

fn parse_i64_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let value = input
        .parse::<i64>()
        .map_err(|_| format!("{} must be an integer.", field.path))?;
    Ok(JsonValue::Number(value.into()))
}

fn parse_f64_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let value = input
        .parse::<f64>()
        .map_err(|_| format!("{} must be a number.", field.path))?;
    let number = serde_json::Number::from_f64(value)
        .ok_or_else(|| format!("{} must be a finite number.", field.path))?;
    Ok(JsonValue::Number(number))
}

fn parse_string_field_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let value = parse_string_input(input)
        .map_err(|message| format!("{} must be a string: {message}.", field.path))?;
    ensure_allowed_value(field, &value)?;
    Ok(JsonValue::String(value))
}

fn parse_string_list_input(field: &ConfigUiField, input: &str) -> Result<JsonValue, String> {
    let value = parse_json_input(field, input, "JSON string array")?;
    let array = value
        .as_array()
        .ok_or_else(|| format!("{} must be a JSON string array.", field.path))?;
    let mut strings = Vec::with_capacity(array.len());
    for value in array {
        let Some(value) = value.as_str() else {
            return Err(format!("{} must contain only strings.", field.path));
        };
        ensure_allowed_value(field, value)?;
        strings.push(JsonValue::String(value.to_string()));
    }
    Ok(JsonValue::Array(strings))
}

fn parse_json_input(
    field: &ConfigUiField,
    input: &str,
    expected: &str,
) -> Result<JsonValue, String> {
    serde_json::from_str::<JsonValue>(input)
        .map_err(|source| format!("{} must be a valid {expected}: {source}.", field.path))
}

fn parse_string_input(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.starts_with('"') {
        serde_json::from_str::<String>(trimmed).map_err(|source| source.to_string())
    } else {
        Ok(input.to_string())
    }
}

fn parse_rendered_json_string(value: &str) -> Option<String> {
    serde_json::from_str::<String>(value).ok()
}

fn ensure_allowed_value(field: &ConfigUiField, value: &str) -> Result<(), String> {
    if field.allowed_values.is_empty()
        || field.allowed_values.iter().any(|allowed| allowed == value)
    {
        return Ok(());
    }
    Err(format!(
        "{} must be one of: {}.",
        field.path,
        field.allowed_values.join(", ")
    ))
}

fn is_bool_field(field: &ConfigUiField) -> bool {
    matches!(field.kind.as_str(), "bool" | "boolean")
}

fn is_string_field(field: &ConfigUiField) -> bool {
    field.kind == "string"
}

fn is_scalar_enum_field(field: &ConfigUiField) -> bool {
    is_string_field(field) && !field.allowed_values.is_empty()
}

fn field_bool_value(field: &ConfigUiField) -> Option<bool> {
    match field.current_value.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn field_string_value(field: &ConfigUiField) -> Option<String> {
    parse_rendered_json_string(&field.current_value).or_else(|| {
        if field.current_value == "not set" {
            None
        } else {
            Some(field.current_value.clone())
        }
    })
}

fn next_allowed_value(field: &ConfigUiField) -> String {
    let current = field_string_value(field);
    let next_index = current
        .as_deref()
        .and_then(|value| {
            field
                .allowed_values
                .iter()
                .position(|candidate| candidate == value)
        })
        .map(|index| (index + 1) % field.allowed_values.len())
        .unwrap_or(0);
    field.allowed_values[next_index].clone()
}

fn read_settings_for_edit_or_empty(path: &Path) -> Result<String, CoreError> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(raw),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok("{}\n".to_string()),
        Err(source) => Err(CoreError::io(
            "read_settings_jsonc_for_edit",
            "Could not read Yazelix settings.jsonc for editing",
            "Fix permissions or restore the settings file, then retry.",
            path.display().to_string(),
            source,
        )),
    }
}

fn write_settings_edit(path: &Path, raw: &str) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "create_settings_jsonc_parent",
                "Could not create the Yazelix config directory",
                "Fix permissions for the config directory, then retry.",
                parent.display().to_string(),
                source,
            )
        })?;
    }
    fs::write(path, raw).map_err(|source| {
        CoreError::io(
            "write_settings_jsonc_edit",
            "Could not write Yazelix settings.jsonc",
            "Fix permissions for the settings file, then retry.",
            path.display().to_string(),
            source,
        )
    })
}

fn validate_patched_settings_for_ui(request: &ConfigUiRequest, raw: &str) -> Result<(), CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let temp_dir = std::env::temp_dir().join(format!(
        "yazelix_config_ui_settings_check_{}_{}",
        std::process::id(),
        monotonic_suffix()
    ));
    fs::create_dir_all(&temp_dir).map_err(|source| {
        CoreError::io(
            "create_settings_validation_temp_dir",
            "Could not create a temporary directory to validate settings.jsonc",
            "Check the system temporary directory permissions, then retry.",
            temp_dir.display().to_string(),
            source,
        )
    })?;
    let temp_config = temp_dir.join(SETTINGS_CONFIG);
    let result = (|| {
        fs::write(&temp_config, raw).map_err(|source| {
            CoreError::io(
                "write_settings_validation_temp_config",
                "Could not write a temporary settings.jsonc validation file",
                "Check the system temporary directory permissions, then retry.",
                temp_config.display().to_string(),
                source,
            )
        })?;
        crate::config_normalize::normalize_config(&NormalizeConfigRequest {
            config_path: temp_config,
            default_config_path: paths.default_config_path.clone(),
            contract_path: paths.contract_path.clone(),
            include_missing: true,
        })?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn monotonic_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn toml_value_to_json(value: &TomlValue) -> Result<JsonValue, CoreError> {
    match value {
        TomlValue::String(value) => Ok(JsonValue::String(value.clone())),
        TomlValue::Integer(value) => Ok(JsonValue::Number((*value).into())),
        TomlValue::Float(value) => serde_json::Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Config,
                    "non_finite_toml_float",
                    "Could not convert a TOML float to JSON.",
                    "Use a finite number in the settings input.",
                    json!({ "value": value.to_string() }),
                )
            }),
        TomlValue::Boolean(value) => Ok(JsonValue::Bool(*value)),
        TomlValue::Datetime(value) => Ok(JsonValue::String(value.to_string())),
        TomlValue::Array(values) => values
            .iter()
            .map(toml_value_to_json)
            .collect::<Result<Vec<_>, _>>()
            .map(JsonValue::Array),
        TomlValue::Table(table) => {
            let mut object = JsonMap::new();
            for (key, value) in table {
                object.insert(key.clone(), toml_value_to_json(value)?);
            }
            Ok(JsonValue::Object(object))
        }
    }
}

fn string_array(value: Option<&TomlValue>) -> Vec<String> {
    value
        .and_then(TomlValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(TomlValue::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn toml_number_as_f64(value: &TomlValue) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}

fn state_label(state: ConfigUiValueState) -> &'static str {
    match state {
        ConfigUiValueState::Explicit => "explicit",
        ConfigUiValueState::Defaulted => "default",
        ConfigUiValueState::Unset => "unset",
        ConfigUiValueState::Invalid => "invalid",
    }
}

fn state_style(state: ConfigUiValueState) -> Style {
    match state {
        ConfigUiValueState::Explicit => Style::default().fg(Color::Green),
        ConfigUiValueState::Defaulted => Style::default().fg(Color::Cyan),
        ConfigUiValueState::Unset => Style::default().fg(Color::Yellow),
        ConfigUiValueState::Invalid => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    }
}

fn sidecar_status_style(present: bool) -> Style {
    if present {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    }
}

fn metadata_key_style() -> Style {
    Style::default().fg(Color::LightBlue)
}

fn metadata_value_style() -> Style {
    Style::default().fg(Color::White)
}

fn config_key_style() -> Style {
    Style::default().fg(Color::LightCyan)
}

fn owner_label(owner: ConfigUiPathOwner) -> &'static str {
    match owner {
        ConfigUiPathOwner::Default => "default",
        ConfigUiPathOwner::HomeManager => "home-manager",
        ConfigUiPathOwner::User => "user",
    }
}

fn fixed_label(value: &str, width: usize) -> String {
    let label = format!("{value:<width$}");
    if label.ends_with(' ') {
        label
    } else {
        format!("{label} ")
    }
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    value
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>()
        + "..."
}

fn truncate_start(value: &str, limit: usize) -> String {
    let len = value.chars().count();
    if len <= limit {
        return value.to_string();
    }
    if limit <= 3 {
        return ".".repeat(limit);
    }
    let tail = value
        .chars()
        .skip(len.saturating_sub(limit - 3))
        .collect::<String>();
    format!("...{tail}")
}

fn tab_index(tabs: &[String], tab: &str) -> usize {
    tabs.iter()
        .position(|candidate| candidate == tab)
        .unwrap_or(tabs.len())
}

fn terminal_err(source: io::Error) -> CoreError {
    CoreError::io(
        "config_ui_terminal",
        "Could not run the Yazelix config UI terminal session",
        "Retry from a healthy interactive terminal, or run `yzx config` for plain text output.",
        ".",
        source,
    )
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME;
    use ratatui::backend::TestBackend;
    use tempfile::tempdir;

    fn buffer_line(buffer: &ratatui::buffer::Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .filter_map(|x| buffer.cell((x, y)))
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    fn buffer_text_fg(buffer: &ratatui::buffer::Buffer, y: u16, text: &str) -> Color {
        let line = buffer_line(buffer, y);
        let x = line.find(text).expect("text in buffer") as u16;
        buffer.cell((x, y)).expect("cell").fg
    }

    fn test_field(
        path: &str,
        kind: &str,
        current_value: &str,
        allowed_values: &[&str],
    ) -> ConfigUiField {
        ConfigUiField {
            path: path.to_string(),
            tab: "general".to_string(),
            kind: kind.to_string(),
            current_value: current_value.to_string(),
            default_value: "no default".to_string(),
            state: ConfigUiValueState::Explicit,
            description: String::new(),
            allowed_values: allowed_values
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            validation: String::new(),
            rebuild_required: false,
        }
    }

    fn write_runtime_layout(runtime: &Path) {
        fs::create_dir_all(runtime.join("config_metadata")).expect("metadata dir");
        fs::write(
            runtime
                .join("config_metadata")
                .join("main_config_contract.toml"),
            include_str!("../../../config_metadata/main_config_contract.toml"),
        )
        .expect("main config contract");
        fs::write(
            runtime
                .join("config_metadata")
                .join("yazelix_settings.schema.json"),
            include_str!("../../../config_metadata/yazelix_settings.schema.json"),
        )
        .expect("settings schema");
        fs::write(
            runtime.join("yazelix_default.toml"),
            include_str!("../../../yazelix_default.toml"),
        )
        .expect("main defaults");
        fs::write(
            runtime.join(DEFAULT_CURSOR_CONFIG_FILENAME),
            include_str!("../../../yazelix_cursors_default.toml"),
        )
        .expect("cursor defaults");
    }

    fn test_request(runtime: &Path, config: &Path) -> ConfigUiRequest {
        ConfigUiRequest {
            runtime_dir: runtime.to_path_buf(),
            config_dir: config.to_path_buf(),
            config_override: None,
        }
    }

    // Regression: diagnostic statuses longer than their nominal column width still need a separator before the path.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn fixed_label_keeps_separator_after_long_values() {
        assert_eq!(fixed_label("default", 9), "default  ");
        assert_eq!(fixed_label("missing_field", 9), "missing_field ");
    }

    // Regression: missing optional sidecars stay readable instead of rendering as low-contrast dark gray.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn missing_sidecar_status_uses_readable_warning_color() {
        assert_eq!(sidecar_status_style(false).fg, Some(Color::Yellow));
        assert_eq!(sidecar_status_style(true).fg, Some(Color::Green));
    }

    // Regression: the config UI header keeps metadata structured on the title line and labels the config path.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn header_uses_structured_title_metadata_and_path_label() {
        let app = ConfigUiApp {
            request: ConfigUiRequest {
                runtime_dir: PathBuf::from("/runtime"),
                config_dir: PathBuf::from("/home/lucca/.config/yazelix"),
                config_override: None,
            },
            model: ConfigUiModel {
                active_config_path: PathBuf::from("/home/lucca/.config/yazelix/settings.jsonc"),
                active_config_exists: true,
                config_owner: ConfigUiPathOwner::User,
                config_read_only: false,
                tabs: Vec::new(),
                fields: Vec::new(),
                sidecars: Vec::new(),
                diagnostics: Vec::new(),
            },
            selected_tab: 0,
            selected_row: 0,
            search: String::new(),
            search_active: false,
            edit: None,
            notice: None,
        };
        let backend = TestBackend::new(120, 2);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| render_header(frame, &app, frame.area()))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let title = buffer_line(buffer, 0);
        assert!(title.starts_with(" Yazelix Config"));
        assert!(title.contains("Yazelix Config"));
        assert!(title.contains("path: /home/lucca/.config/yazelix/settings.jsonc"));
        assert!(title.contains("owner: user"));
        assert!(title.contains("mode: writable"));
        assert!(title.contains("diag: ok"));
        assert_eq!(buffer_text_fg(buffer, 0, "path:"), Color::LightBlue);
        assert_eq!(buffer_text_fg(buffer, 0, "owner:"), Color::LightBlue);
        assert!(!buffer_line(buffer, 1).contains("path:"));
    }

    // Regression: the tabs row should not reserve a blank divider row between tabs and the settings body.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn tabs_render_as_single_unbordered_row() {
        let app = ConfigUiApp {
            request: ConfigUiRequest {
                runtime_dir: PathBuf::from("/runtime"),
                config_dir: PathBuf::from("/home/lucca/.config/yazelix"),
                config_override: None,
            },
            model: ConfigUiModel {
                active_config_path: PathBuf::from("/home/lucca/.config/yazelix/settings.jsonc"),
                active_config_exists: true,
                config_owner: ConfigUiPathOwner::User,
                config_read_only: false,
                tabs: vec![
                    "general".to_string(),
                    "editor".to_string(),
                    "terminal".to_string(),
                ],
                fields: Vec::new(),
                sidecars: Vec::new(),
                diagnostics: Vec::new(),
            },
            selected_tab: 0,
            selected_row: 0,
            search: String::new(),
            search_active: false,
            edit: None,
            notice: None,
        };
        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| render_tabs(frame, &app, frame.area()))
            .expect("draw");

        let line = buffer_line(terminal.backend().buffer(), 0);
        assert!(line.contains("general"));
        assert!(line.contains("editor"));
        assert!(!line.contains("─"));
    }

    // Regression: config UI setting keys should read as navigable keys, not unstyled body text.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn setting_keys_use_dedicated_key_color() {
        let field = test_field("editor.hide_sidebar_on_file_open", "bool", "true", &[]);
        let lines = field_detail_lines(&field);

        assert_eq!(lines[0].spans[0].style.fg, Some(Color::LightCyan));
        assert_eq!(
            detail_line("current", "true").spans[0].style.fg,
            Some(Color::LightBlue)
        );
    }

    // Defends: the editable config UI interprets typed values from the field contract instead of guessing strings for every setting.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parse_edit_input_uses_field_type_and_allowed_values() {
        let bool_field = test_field("editor.hide_sidebar_on_file_open", "bool", "false", &[]);
        assert_eq!(
            parse_edit_input(&bool_field, "true").expect("bool"),
            json!(true)
        );
        assert!(parse_edit_input(&bool_field, "yes").is_err());

        let enum_field = test_field(
            "zellij.tab_label_mode",
            "string",
            "\"short\"",
            &["short", "compact", "full"],
        );
        assert_eq!(
            parse_edit_input(&enum_field, "compact").expect("enum"),
            json!("compact")
        );
        assert!(parse_edit_input(&enum_field, "wide").is_err());

        let list_field = test_field("yazi.plugins", "string_list", "[\"git\"]", &["git", "ouch"]);
        assert_eq!(
            parse_edit_input(&list_field, r#"["git","ouch"]"#).expect("list"),
            json!(["git", "ouch"])
        );
        assert!(parse_edit_input(&list_field, r#"["unknown"]"#).is_err());
    }

    // Defends: keyboard-oriented quick edits produce deterministic toggles/cycles from the value shown in the UI.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn quick_edit_helpers_toggle_bool_and_cycle_enum() {
        let bool_field = test_field("core.debug_mode", "bool", "true", &[]);
        assert_eq!(field_bool_value(&bool_field), Some(true));

        let enum_field = test_field(
            "zellij.tab_label_mode",
            "string",
            "\"compact\"",
            &["short", "compact", "full"],
        );
        assert_eq!(edit_input_for_field(&enum_field), "compact");
        assert_eq!(next_allowed_value(&enum_field), "full");
    }

    // Defends: UI edits use the same comment-preserving settings.jsonc patcher and validation path as `yzx config set`.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn write_field_value_patches_settings_jsonc_and_reloads_model() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        fs::write(
            &settings_path,
            r#"{
  // keep this comment
  "editor": { "hide_sidebar_on_file_open": false }
}
"#,
        )
        .expect("settings");
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = ConfigUiApp {
            request,
            model,
            selected_tab: 0,
            selected_row: 0,
            search: String::new(),
            search_active: false,
            edit: None,
            notice: None,
        };

        let mutation = app
            .write_field_value("editor.hide_sidebar_on_file_open", &json!(true))
            .expect("write");

        assert_eq!(mutation, SettingsJsoncPatchMutation::Replaced);
        let raw = fs::read_to_string(&settings_path).expect("settings raw");
        assert!(raw.contains("// keep this comment"));
        let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
        assert_eq!(
            get_json_path(&value, "editor.hide_sidebar_on_file_open"),
            Some(&json!(true))
        );
        let field = app
            .model
            .fields
            .iter()
            .find(|field| field.path == "editor.hide_sidebar_on_file_open")
            .expect("field");
        assert_eq!(field.state, ConfigUiValueState::Explicit);
        assert_eq!(field.current_value, "true");
    }
}
