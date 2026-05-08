use super::*;
use crate::config_ui::ConfigUiApp;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};

const HEADER_HORIZONTAL_PADDING: u16 = 1;

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
            Span::raw("Enter/Space toggle/cycle  "),
            Span::raw("e edit  "),
            Span::raw("u unset"),
        ]),
        Some(field) if is_scalar_enum_field(field) => Line::from(vec![
            Span::raw("Enter/e picker  "),
            Span::raw("Space cycle  "),
            Span::raw("u unset"),
        ]),
        Some(field) if is_enum_string_list_field(field) => {
            Line::from(vec![Span::raw("Enter/e picker  "), Span::raw("u unset")])
        }
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

pub(crate) fn metadata_key_style() -> Style {
    Style::default().fg(Color::LightBlue)
}

pub(crate) fn metadata_value_style() -> Style {
    Style::default().fg(Color::White)
}

pub(crate) fn config_key_style() -> Style {
    Style::default().fg(Color::LightCyan)
}

pub(crate) fn owner_label(owner: ConfigUiPathOwner) -> &'static str {
    match owner {
        ConfigUiPathOwner::Default => "default",
        ConfigUiPathOwner::HomeManager => "home-manager",
        ConfigUiPathOwner::User => "user",
    }
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
