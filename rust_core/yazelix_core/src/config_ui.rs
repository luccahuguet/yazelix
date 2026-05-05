//! Read-only terminal UI for inspecting the canonical Yazelix config surface.

use crate::active_config_surface::{PrimaryConfigPaths, primary_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{ConfigDiagnostic, ConfigDiagnosticReport, NormalizeConfigRequest};
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
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::Duration;
use toml::Value as TomlValue;

const DEFAULT_TABS: &[&str] = &[
    "general", "editor", "terminal", "zellij", "yazi", "cursors", "advanced",
];
const HOME_MANAGER_FILES_MARKER: &str = "-home-manager-files/";

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
    model: ConfigUiModel,
    selected_tab: usize,
    selected_row: usize,
    search: String,
    search_active: bool,
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
    let result = run_ui_loop(&mut terminal, model);
    let cleanup = restore_terminal(&mut terminal);

    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(0),
    }
}

fn run_ui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    model: ConfigUiModel,
) -> Result<(), CoreError> {
    let mut app = ConfigUiApp {
        model,
        selected_tab: 0,
        selected_row: 0,
        search: String::new(),
        search_active: false,
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
            Constraint::Length(3),
            Constraint::Length(3),
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
    let diagnostics = if warning_count > 0 {
        Span::styled(
            format!("  blocking issues: {warning_count}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("  diagnostics: ok", Style::default().fg(Color::Green))
    };

    let line = Line::from(vec![
        Span::styled(
            "Yazelix Config",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  owner: {owner}  {write_state}")),
        diagnostics,
    ]);
    let path = Line::from(vec![Span::styled(source, Style::default().fg(Color::Gray))]);
    frame.render_widget(
        Paragraph::new(vec![line, path]).block(Block::default().borders(Borders::BOTTOM)),
        area,
    );
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
            )
            .block(Block::default().borders(Borders::BOTTOM)),
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
    let search = if app.search_active {
        format!("search: {}_", app.search)
    } else if app.search.is_empty() {
        "/ search".to_string()
    } else {
        "Esc clears search".to_string()
    };
    let line = Line::from(vec![
        Span::raw("q quit  "),
        Span::raw("Tab tabs  "),
        Span::raw("j/k move  "),
        Span::styled(search, Style::default().fg(Color::Yellow)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

impl ConfigUiApp {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
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
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => self.next_tab(),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => self.previous_tab(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            _ => {}
        }

        false
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
                    Span::raw(truncate(&field.path, 42)),
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
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                Line::from(vec![
                    Span::styled(fixed_label(status, 9), style),
                    Span::raw(sidecar.name.clone()),
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
                    Span::raw(truncate(&diagnostic.path, 42)),
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
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
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
        Span::styled(fixed_label(label, 11), Style::default().fg(Color::Gray)),
        Span::raw(value.to_string()),
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
        include_missing: true,
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

fn owner_label(owner: ConfigUiPathOwner) -> &'static str {
    match owner {
        ConfigUiPathOwner::Default => "default",
        ConfigUiPathOwner::HomeManager => "home-manager",
        ConfigUiPathOwner::User => "user",
    }
}

fn fixed_label(value: &str, width: usize) -> String {
    format!("{value:<width$}")
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
