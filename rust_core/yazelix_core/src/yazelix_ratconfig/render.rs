// Test lane: default
use super::*;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use std::collections::BTreeSet;

const HEADER_HORIZONTAL_PADDING: u16 = 1;
const FIELD_TAKES_EFFECT_COLUMN_WIDTH: usize = 26;
const FIELD_SETTING_COLUMN_WIDTH: usize = 30;
const FIELD_VALUE_COLUMN_WIDTH: usize = 18;
const STATUS_COLUMN_WIDTH: usize = 9;
const STATUS_ITEM_COLUMN_WIDTH: usize = 42;

pub(crate) fn draw_config_ui(frame: &mut Frame<'_>, app: &mut ConfigUiApp) {
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
        .map(|row| ListItem::new(row_line_for_model(&app.model, *row)))
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
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let list_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    frame.render_widget(
        Paragraph::new(list_header_line(app)).alignment(Alignment::Left),
        list_chunks[0],
    );
    if list_chunks[1].height == 0 {
        return;
    }

    frame.render_stateful_widget(
        List::new(items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        list_chunks[1],
        &mut state,
    );
}

fn list_header_line(app: &ConfigUiApp) -> Line<'static> {
    match app.model.tabs.get(app.selected_tab).map(String::as_str) {
        Some("advanced") => status_list_header_line(),
        _ => field_list_header_line(),
    }
}

fn field_list_header_line() -> Line<'static> {
    Line::from(vec![
        Span::styled(
            fixed_label("takes effect", FIELD_TAKES_EFFECT_COLUMN_WIDTH),
            column_header_style(),
        ),
        Span::styled(
            fixed_label("setting", FIELD_SETTING_COLUMN_WIDTH),
            column_header_style(),
        ),
        Span::styled("value", column_header_style()),
    ])
}

fn status_list_header_line() -> Line<'static> {
    Line::from(vec![
        Span::styled(
            fixed_label("status", STATUS_COLUMN_WIDTH),
            column_header_style(),
        ),
        Span::styled(
            fixed_label("item", STATUS_ITEM_COLUMN_WIDTH),
            column_header_style(),
        ),
        Span::styled("detail", column_header_style()),
    ])
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

pub(crate) fn row_line_for_model(model: &ConfigUiModel, row: UiRowRef) -> Line<'static> {
    match row {
        UiRowRef::Field(index) => {
            let field = &model.fields[index];
            Line::from(vec![
                Span::styled(
                    fixed_label(&field.apply_status.summary, FIELD_TAKES_EFFECT_COLUMN_WIDTH),
                    apply_status_style(&field.apply_status),
                ),
                Span::styled(
                    fixed_label(
                        &truncate(&field.path, FIELD_SETTING_COLUMN_WIDTH),
                        FIELD_SETTING_COLUMN_WIDTH,
                    ),
                    field_key_style(field),
                ),
                Span::styled(
                    truncate(&field.current_value, FIELD_VALUE_COLUMN_WIDTH),
                    field_value_style(field),
                ),
            ])
        }
        UiRowRef::Sidecar(index) => {
            let sidecar = &model.sidecars[index];
            let status = if sidecar.present {
                "present"
            } else {
                "missing"
            };
            Line::from(vec![
                Span::styled(
                    fixed_label(status, 9),
                    sidecar_status_style(sidecar.present),
                ),
                Span::styled(sidecar.name.clone(), config_key_style()),
            ])
        }
        UiRowRef::Diagnostic(index) => {
            let diagnostic = &model.diagnostics[index];
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
        UiRowRef::NativeStatus(index) => {
            let status = &model.native_config_statuses[index];
            Line::from(vec![
                Span::styled(fixed_label(&status.status, 24), native_status_style(status)),
                Span::styled(truncate(&status.surface, 36), config_key_style()),
                Span::styled(
                    format!(" {}", truncate(&status.label, 42)),
                    Style::default().fg(Color::Gray),
                ),
            ])
        }
    }
}

fn field_key_style(field: &ConfigUiField) -> Style {
    if field.state == ConfigUiValueState::Invalid {
        state_style(field.state)
    } else {
        config_key_style()
    }
}

fn field_value_style(field: &ConfigUiField) -> Style {
    if field.state == ConfigUiValueState::Invalid {
        state_style(field.state)
    } else {
        Style::default().fg(Color::Gray)
    }
}

pub(crate) fn default_field_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
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
        detail_line("takes effect", &field.apply_status.label),
        detail_line("after save", &field.apply_status.detail),
    ];
    if !field.validation.is_empty() {
        lines.push(detail_line("validation", &field.validation));
    }
    if !field.allowed_values.is_empty() && !is_scalar_enum_field(field) {
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

pub(crate) fn single_choice_field_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let selected_value = parse_rendered_json_string(&field.current_value)
        .unwrap_or_else(|| field.current_value.clone());
    let mut lines = default_field_detail_lines(field);
    lines.push(Line::from(""));
    append_single_choice_options(&mut lines, field, &selected_value, None);
    lines
}

pub(crate) fn single_choice_detail_lines(
    field: &ConfigUiField,
    edit: &ConfigUiEditState,
) -> Vec<Line<'static>> {
    let selected_value = edit.input.as_str();
    let mut lines = vec![
        Line::from(Span::styled(
            field.path.clone(),
            config_key_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        detail_line("selected", selected_value),
        Line::from(""),
    ];

    append_single_choice_options(
        &mut lines,
        field,
        selected_value,
        Some(
            edit.choice_index
                .min(field.allowed_values.len().saturating_sub(1)),
        ),
    );

    lines
}

fn append_single_choice_options(
    lines: &mut Vec<Line<'static>>,
    field: &ConfigUiField,
    selected_value: &str,
    highlighted_index: Option<usize>,
) {
    for (index, value) in field.allowed_values.iter().enumerate() {
        let highlighted = highlighted_index == Some(index);
        let selected = value == selected_value;
        let selector_style = if highlighted {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let marker_style = if selected {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let value_style = if highlighted {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if selected {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(vec![
            Span::styled(if highlighted { "> " } else { "  " }, selector_style),
            Span::styled(if selected { "(x) " } else { "( ) " }, marker_style),
            Span::styled(value.clone(), value_style),
        ]));
    }
}

pub(crate) fn multi_choice_detail_lines(
    field: &ConfigUiField,
    edit: &ConfigUiEditState,
) -> Vec<Line<'static>> {
    let enabled_values = parse_string_list_values(field, &edit.input).unwrap_or_default();
    let enabled_set = enabled_values.iter().cloned().collect::<BTreeSet<_>>();
    let mut lines = vec![
        Line::from(Span::styled(
            field.path.clone(),
            config_key_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        detail_line(
            "enabled",
            &format!("{}/{}", enabled_set.len(), field.allowed_values.len()),
        ),
        Line::from(""),
    ];

    for (index, value) in field.allowed_values.iter().enumerate() {
        let selected = index
            == edit
                .choice_index
                .min(field.allowed_values.len().saturating_sub(1));
        let enabled = enabled_set.contains(value);
        let selector_style = if selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let marker_style = if enabled {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let value_style = if selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if enabled {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, selector_style),
            Span::styled(if enabled { "[x] " } else { "[ ] " }, marker_style),
            Span::styled(value.clone(), value_style),
        ]));
    }

    lines
}

pub(crate) fn sidecar_detail_lines(sidecar: &ConfigUiSidecar) -> Vec<Line<'static>> {
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

pub(crate) fn diagnostic_detail_lines(diagnostic: &ConfigUiDiagnostic) -> Vec<Line<'static>> {
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

pub(crate) fn native_status_detail_lines(status: &ConfigUiNativeStatus) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            status.label.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        detail_line("surface", &status.surface),
        detail_line("tool", &status.tool),
        detail_line("status", &status.status),
        detail_line("description", &status.description),
        detail_line("allowed action", &status.allowed_action),
    ];
    if let Some(path) = &status.active_path {
        lines.push(detail_line("active path", path));
    }
    if let Some(path) = &status.managed_path {
        lines.push(detail_line("managed path", path));
    }
    if !status.native_paths.is_empty() {
        lines.push(detail_line("native paths", &status.native_paths.join(", ")));
    }
    if let Some(path) = &status.generated_path {
        lines.push(detail_line("generated path", path));
    }
    if let Some(reason) = &status.read_only_reason {
        lines.push(detail_line("read-only", reason));
    }
    lines
}

fn render_footer(frame: &mut Frame<'_>, app: &ConfigUiApp, area: Rect) {
    if let Some(edit) = &app.edit {
        let field = &app.model.fields[edit.field_index];
        let editing = edit_status_line(field, edit);
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
            .unwrap_or_else(|| edit_control_line(field, edit.mode));
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
        .unwrap_or_else(|| normal_control_line(app));
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

fn edit_status_line(field: &ConfigUiField, edit: &ConfigUiEditState) -> Line<'static> {
    let value = match edit.mode {
        ConfigUiEditMode::Text => format!("{}_", edit.input),
        ConfigUiEditMode::Choice if is_scalar_enum_field(field) => {
            single_choice_status_value(field, edit)
        }
        ConfigUiEditMode::Choice => edit.input.clone(),
        ConfigUiEditMode::MultiChoice => multi_choice_status_value(field, edit),
    };
    Line::from(vec![
        Span::styled("editing: ", Style::default().fg(Color::Yellow)),
        Span::styled(field.path.clone(), config_key_style()),
        Span::raw(" = "),
        Span::styled(value, Style::default().fg(Color::White)),
    ])
}

fn normal_control_line(app: &ConfigUiApp) -> Line<'static> {
    match app.selected_field() {
        Some(field) if is_bool_field(field) => Line::from(vec![
            Span::raw("Enter/Space toggle  "),
            Span::raw("e edit  "),
            Span::raw("u unset"),
        ]),
        Some(field) if is_scalar_enum_field(field) => Line::from(vec![
            Span::raw("Enter/e/Space picker  "),
            Span::raw("u unset"),
        ]),
        Some(field) if is_enum_string_list_field(field) => {
            Line::from(vec![Span::raw("Enter/e picker  "), Span::raw("u unset")])
        }
        Some(field) if structured_only_edit_notice(field).is_some() => Line::from(vec![
            Span::raw("structured view only  "),
            Span::raw("u unset"),
        ]),
        Some(_) => Line::from(vec![Span::raw("Enter/e edit  "), Span::raw("u unset")]),
        None => Line::from(Span::raw("Select a setting row to edit")),
    }
}

fn edit_control_line(field: &ConfigUiField, mode: ConfigUiEditMode) -> Line<'static> {
    match mode {
        ConfigUiEditMode::Text => Line::from(vec![
            Span::raw("Enter save  "),
            Span::raw("Esc cancel  "),
            Span::raw("Ctrl+u clear"),
        ]),
        ConfigUiEditMode::Choice if is_scalar_enum_field(field) => Line::from(vec![
            Span::raw("hjkl/Arrows move  "),
            Span::raw("Space select  "),
            Span::raw("Enter save  "),
            Span::raw("Esc cancel"),
        ]),
        ConfigUiEditMode::Choice => Line::from(vec![
            Span::raw("Space toggle  "),
            Span::raw("Enter save  "),
            Span::raw("Esc cancel"),
        ]),
        ConfigUiEditMode::MultiChoice => Line::from(vec![
            Span::raw("hjkl/Arrows move  "),
            Span::raw("Space enable/disable  "),
            Span::raw("Enter save  "),
            Span::raw("Esc cancel"),
        ]),
    }
}

pub(crate) fn state_label(state: ConfigUiValueState) -> &'static str {
    match state {
        ConfigUiValueState::Explicit => "explicit",
        ConfigUiValueState::Defaulted => "default",
        ConfigUiValueState::Unset => "unset",
        ConfigUiValueState::Invalid => "invalid",
    }
}

pub(crate) fn state_style(state: ConfigUiValueState) -> Style {
    match state {
        ConfigUiValueState::Explicit => Style::default().fg(Color::Green),
        ConfigUiValueState::Defaulted => Style::default().fg(Color::Cyan),
        ConfigUiValueState::Unset => Style::default().fg(Color::Yellow),
        ConfigUiValueState::Invalid => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    }
}

pub(crate) fn apply_status_style(status: &ConfigUiApplyStatus) -> Style {
    if status.pending {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    }
}

pub(crate) fn sidecar_status_style(present: bool) -> Style {
    if present {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    }
}

pub(crate) fn native_status_style(status: &ConfigUiNativeStatus) -> Style {
    match status.severity.as_str() {
        "error" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "warning" => Style::default().fg(Color::Yellow),
        "ok" => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::Cyan),
    }
}

pub(crate) fn column_header_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn metadata_key_style() -> Style {
    Style::default().fg(Color::LightBlue)
}

pub(crate) fn metadata_value_style() -> Style {
    Style::default().fg(Color::White)
}

pub(crate) fn config_key_style() -> Style {
    Style::default().fg(Color::LightCyan)
}

pub(crate) fn fixed_label(value: &str, width: usize) -> String {
    let label = format!("{value:<width$}");
    if label.ends_with(' ') {
        label
    } else {
        format!("{label} ")
    }
}

pub(crate) fn detail_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(fixed_label(label, 11), metadata_key_style()),
        Span::styled(value.to_string(), metadata_value_style()),
    ])
}

pub(crate) fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    value
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>()
        + "..."
}

pub(crate) fn truncate_start(value: &str, limit: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn rendered_cells(line: &Line<'_>) -> Vec<String> {
        line.spans
            .iter()
            .map(|span| span.content.trim().to_string())
            .collect()
    }

    fn rendered_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn test_model(state: ConfigUiValueState) -> ConfigUiModel {
        ConfigUiModel {
            active_config_path: PathBuf::from("/home/lucca/.config/yazelix/settings.jsonc"),
            cursor_config_path: PathBuf::from(
                "/home/lucca/.config/yazelix_ghostty_cursors/settings.jsonc",
            ),
            default_cursor_config_path: PathBuf::from(
                "/runtime/yazelix_ghostty_cursors_default.toml",
            ),
            active_config_exists: true,
            config_owner: ConfigUiPathOwner::User,
            config_read_only: false,
            tabs: vec!["general".to_string()],
            fields: vec![ConfigUiField {
                path: "core.debug_mode".to_string(),
                tab: "general".to_string(),
                kind: "bool".to_string(),
                current_value: "false".to_string(),
                edit_value: "false".to_string(),
                default_value: "false".to_string(),
                state,
                description: String::new(),
                allowed_values: Vec::new(),
                validation: String::new(),
                rebuild_required: false,
                apply_status: ConfigUiApplyStatus {
                    summary: "after Yazelix restart".to_string(),
                    label: "after Yazelix restart".to_string(),
                    detail: "Restart Yazelix after saving".to_string(),
                    pending: true,
                },
                edit_behavior: ConfigUiEditBehavior::Default,
            }],
            sidecars: Vec::new(),
            native_config_statuses: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    // Defends: settings rows expose apply/setting/value without repeating complete-config explicit state.
    #[test]
    fn field_row_omits_explicit_state_column() {
        let model = test_model(ConfigUiValueState::Explicit);
        let line = row_line_for_model(&model, UiRowRef::Field(0));

        assert_eq!(
            rendered_cells(&line),
            vec!["after Yazelix restart", "core.debug_mode", "false"]
        );
        assert!(!rendered_text(&line).contains("explicit"));
    }

    // Defends: removing the state column still leaves visible names for the remaining settings columns.
    #[test]
    fn field_header_names_remaining_columns() {
        assert_eq!(
            rendered_cells(&field_list_header_line()),
            vec!["takes effect", "setting", "value"]
        );
    }

    // Defends: invalid field rows remain visibly exceptional even without the normal state column.
    #[test]
    fn invalid_field_row_keeps_error_style_on_setting_and_value() {
        let model = test_model(ConfigUiValueState::Invalid);
        let line = row_line_for_model(&model, UiRowRef::Field(0));

        assert_eq!(
            line.spans[1].style,
            state_style(ConfigUiValueState::Invalid)
        );
        assert_eq!(
            line.spans[2].style,
            state_style(ConfigUiValueState::Invalid)
        );
    }
}
