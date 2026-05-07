//! Terminal UI for inspecting and editing the canonical Yazelix config surface.

mod apply_adapter;

use crate::action_registry::ZELLIJ_ACTIONS;
use crate::active_config_surface::{PrimaryConfigPaths, primary_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_apply::ConfigEditApplyStatus;
use crate::config_normalize::{ConfigDiagnostic, ConfigDiagnosticReport, NormalizeConfigRequest};
use crate::control_plane::{home_dir_from_env, state_dir_from_env};
use crate::native_config_status::{
    NativeConfigStatusEntry, NativeConfigStatusRequest, classify_native_config_statuses,
    current_platform_name, path_owned_by_home_manager, status_code_for_entry,
    xdg_config_home_from_env,
};
use crate::runtime_apply_mode::RuntimeApplyMode;
use crate::settings_jsonc_patch::{
    SettingsJsoncPatchMutation, set_settings_jsonc_value_text, unset_settings_jsonc_value_text,
};
use crate::settings_surface::{SETTINGS_SCHEMA_FILENAME, render_default_settings_jsonc};
use crate::settings_surface::{
    is_settings_config_path, parse_jsonc_value, read_settings_jsonc_value,
};
use crate::user_config_paths::{CURRENT_MANAGED_CONFIG_FILE_NAMES, SETTINGS_CONFIG};
use crate::yazelix_ratconfig::{draw_config_ui, *};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;
use yazelix_cursors::{CursorRegistry, render_cursor_settings_jsonc};

pub use crate::yazelix_ratconfig::{
    ConfigUiApplyStatus, ConfigUiDiagnostic, ConfigUiField, ConfigUiModel, ConfigUiPathOwner,
    ConfigUiRequest, ConfigUiSidecar, ConfigUiValueState,
};
use apply_adapter::apply_after_field_write;

const DEFAULT_TABS: &[&str] = &[
    "general",
    "workspace",
    "editor",
    "terminal",
    "appearance",
    "cursors",
    "status_bar",
    "file_manager",
    "keybindings",
    "shell",
    "advanced",
];
const CONFIG_UI_METADATA_FILENAME: &str = "config_ui_metadata.toml";
pub(crate) const HEADER_HORIZONTAL_PADDING: u16 = 1;
const ZELLIJ_KEYBINDINGS_FIELD_PATH: &str = "zellij.keybindings";

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
    apply_mode: RuntimeApplyMode,
}

#[derive(Debug, Clone)]
struct FieldUiMetadata {
    tab: String,
    help: String,
}

#[derive(Debug, Clone)]
struct ConfigUiMetadata {
    tabs: Vec<String>,
    fields: BTreeMap<String, FieldUiMetadata>,
}

#[derive(Debug, Clone)]
struct SchemaField {
    path: String,
    kind: String,
    allowed_values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiRowRef {
    Field(usize),
    Sidecar(usize),
    NativeStatus(usize),
    Diagnostic(usize),
}

pub(crate) struct ConfigUiApp {
    pub(crate) request: ConfigUiRequest,
    pub(crate) model: ConfigUiModel,
    pub(crate) selected_tab: usize,
    pub(crate) selected_row: usize,
    pub(crate) search: String,
    pub(crate) search_active: bool,
    pub(crate) edit: Option<ConfigUiEditState>,
    pub(crate) notice: Option<ConfigUiNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigUiNotice {
    pub(crate) text: String,
    pub(crate) is_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigUiWriteOutcome {
    mutation: SettingsJsoncPatchMutation,
    apply_status: Option<ConfigEditApplyStatus>,
    apply_error: Option<String>,
}

#[derive(Debug, Clone)]
struct ConfigUiEditTarget {
    path: PathBuf,
    path_in_file: String,
    kind: ConfigUiEditTargetKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigUiEditTargetKind {
    Main,
    Cursors,
}

pub fn build_config_ui_model(request: &ConfigUiRequest) -> Result<ConfigUiModel, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let schema = read_json_file(
        &paths.settings_schema_path,
        "read_settings_schema",
        "Could not read the Yazelix settings schema",
    )?;
    let schema_tab_order = schema_tabs(&schema);
    let ui_metadata =
        load_config_ui_metadata(&config_ui_metadata_path(&paths.settings_schema_path))?;
    ensure_ui_metadata_tabs_match_schema(&ui_metadata.tabs, &schema_tab_order)?;
    let tabs = ui_metadata.tabs.clone();
    let active_config_path = active_config_path(&paths, request.config_override.as_deref());
    let active_config_exists = path_present(&active_config_path);
    let config_owner = classify_path_owner(&active_config_path, active_config_exists);
    let active_main_value = if active_config_exists {
        read_active_config_value(&active_config_path)?
    } else {
        JsonValue::Object(JsonMap::new())
    };
    ensure_root_object(&active_config_path, &active_main_value)?;
    let active_value = compose_config_ui_value(
        active_main_value,
        read_cursor_config_value(&paths.user_cursor_config)?,
    )?;

    let default_raw = render_default_settings_jsonc(
        &paths.default_config_path,
        &paths.default_cursor_config_path,
    )?;
    let default_main_value = parse_jsonc_value(&paths.default_config_path, &default_raw)?;
    let default_value = compose_config_ui_value(
        default_main_value,
        read_default_cursor_config_value(&paths.default_cursor_config_path)?,
    )?;
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
        let metadata = field_ui_metadata(&ui_metadata, &field.path)?;
        let current = get_json_path(&active_value, &field.path);
        let default = get_json_path(&default_value, &field.path)
            .cloned()
            .or_else(|| field.default_value.clone());
        let apply_mode = if config_owner == ConfigUiPathOwner::HomeManager {
            RuntimeApplyMode::PackageHomeManagerActivation
        } else {
            field.apply_mode
        };
        fields.push(build_field_row(
            &field.path,
            &metadata.tab,
            &field.kind,
            current,
            default.as_ref(),
            field_description(field, metadata),
            field.allowed_values.clone(),
            field.validation.clone(),
            field.rebuild_required,
            apply_mode,
            blocking_paths.contains(&field.path),
        ));
    }

    for schema_field in collect_cursor_schema_fields(&schema) {
        if fields.iter().any(|field| field.path == schema_field.path) {
            continue;
        }
        let metadata = field_ui_metadata(&ui_metadata, &schema_field.path)?;
        let current = get_json_path(&active_value, &schema_field.path);
        let default = get_json_path(&default_value, &schema_field.path);
        fields.push(build_field_row(
            &schema_field.path,
            &metadata.tab,
            &schema_field.kind,
            current,
            default,
            metadata.help.clone(),
            schema_field.allowed_values,
            String::new(),
            false,
            RuntimeApplyMode::ShellTerminalRestart,
            blocking_paths.contains(&schema_field.path),
        ));
    }

    fields.sort_by(|left, right| {
        tab_index(&tabs, &left.tab)
            .cmp(&tab_index(&tabs, &right.tab))
            .then_with(|| left.path.cmp(&right.path))
    });

    let home_dir = home_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let native_config_statuses = classify_native_config_statuses(&NativeConfigStatusRequest {
        xdg_config_home: xdg_config_home_from_env(&home_dir),
        home_dir,
        config_dir: request.config_dir.clone(),
        state_dir,
        platform: current_platform_name(),
        terminal_config_mode: effective_string_config(
            &active_value,
            &default_value,
            "terminal.config_mode",
            "yazelix",
        ),
        selected_terminals: effective_string_list_config(
            &active_value,
            &default_value,
            "terminal.terminals",
            &["ghostty", "wezterm"],
        ),
        settings_home_manager_read_only: config_owner == ConfigUiPathOwner::HomeManager,
    });

    Ok(ConfigUiModel {
        active_config_path: active_config_path.clone(),
        cursor_config_path: paths.user_cursor_config.clone(),
        default_cursor_config_path: paths.default_cursor_config_path.clone(),
        active_config_exists,
        config_owner,
        config_read_only: path_is_read_only(&active_config_path),
        tabs,
        fields,
        sidecars: collect_sidecars(&request.config_dir),
        native_config_statuses,
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
            KeyCode::Enter => self.activate_selected_field(),
            KeyCode::Char('e') => self.begin_edit_selected_field(),
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
        if let Some(mode) = self.edit.as_ref().map(|edit| edit.mode) {
            match mode {
                ConfigUiEditMode::Choice => {
                    self.handle_choice_edit_key(key);
                    return;
                }
                ConfigUiEditMode::MultiChoice => {
                    self.handle_multi_choice_edit_key(key);
                    return;
                }
                ConfigUiEditMode::Text => {}
            }
        }

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

    fn handle_choice_edit_key(&mut self, key: KeyEvent) {
        let scalar_enum = self
            .edit
            .as_ref()
            .and_then(|edit| self.model.fields.get(edit.field_index))
            .is_some_and(is_scalar_enum_field);
        match key.code {
            KeyCode::Esc => {
                self.edit = None;
                self.notice_info("Edit canceled.");
            }
            KeyCode::Enter if scalar_enum => {
                self.select_single_choice_edit();
                self.save_edit();
            }
            KeyCode::Enter => self.save_edit(),
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h')
                if scalar_enum =>
            {
                self.notice = None;
                self.move_single_choice_edit(-1);
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l')
                if scalar_enum =>
            {
                self.notice = None;
                self.move_single_choice_edit(1);
            }
            KeyCode::Char(' ') if scalar_enum => {
                self.notice = None;
                self.select_single_choice_edit();
            }
            KeyCode::Up | KeyCode::Right | KeyCode::Down | KeyCode::Left | KeyCode::Char(' ') => {
                self.notice = None;
                self.cycle_choice_edit();
            }
            _ => {}
        }
    }

    fn handle_multi_choice_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.edit = None;
                self.notice_info("Edit canceled.");
            }
            KeyCode::Enter => self.save_edit(),
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h') => {
                self.notice = None;
                self.move_multi_choice_edit(-1);
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l') => {
                self.notice = None;
                self.move_multi_choice_edit(1);
            }
            KeyCode::Char(' ') => {
                self.notice = None;
                self.toggle_multi_choice_edit();
            }
            _ => {}
        }
    }

    fn cycle_choice_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let next = if is_bool_field(field) {
            if edit.input.trim() == "true" {
                "false".to_string()
            } else {
                "true".to_string()
            }
        } else if is_scalar_enum_field(field) && !field.allowed_values.is_empty() {
            next_allowed_value_from(&field.allowed_values, Some(edit.input.as_str()))
        } else {
            return;
        };
        if let Some(edit) = &mut self.edit {
            edit.input = next;
        }
    }

    fn move_single_choice_edit(&mut self, delta: isize) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let len = field.allowed_values.len();
        if len == 0 {
            return;
        }
        let index = edit.choice_index.min(len - 1);
        let next = if delta < 0 {
            index.checked_sub(1).unwrap_or(len - 1)
        } else {
            (index + 1) % len
        };
        if let Some(edit) = &mut self.edit {
            edit.choice_index = next;
        }
    }

    fn select_single_choice_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let Some(value) = field.allowed_values.get(edit.choice_index) else {
            return;
        };
        if let Some(edit) = &mut self.edit {
            edit.input = value.clone();
        }
    }

    fn move_multi_choice_edit(&mut self, delta: isize) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let len = field.allowed_values.len();
        if len == 0 {
            return;
        }
        let index = edit.choice_index.min(len - 1);
        let next = if delta < 0 {
            index.checked_sub(1).unwrap_or(len - 1)
        } else {
            (index + 1) % len
        };
        if let Some(edit) = &mut self.edit {
            edit.choice_index = next;
        }
    }

    fn toggle_multi_choice_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let next = match toggled_string_list_input(field, &edit.input, edit.choice_index) {
            Ok(next) => next,
            Err(message) => {
                self.notice_error(message);
                return;
            }
        };
        if let Some(edit) = &mut self.edit {
            edit.input = next;
        }
    }

    fn selected_field_index(&self) -> Option<usize> {
        let row = self.visible_rows().get(self.selected_row).copied()?;
        match row {
            UiRowRef::Field(index) => Some(index),
            _ => None,
        }
    }

    pub(crate) fn selected_field(&self) -> Option<&ConfigUiField> {
        self.selected_field_index()
            .and_then(|index| self.model.fields.get(index))
    }

    fn activate_selected_field(&mut self) {
        if self.selected_field().is_some_and(is_bool_field) {
            self.quick_edit_selected_field();
        } else {
            self.begin_edit_selected_field();
        }
    }

    fn begin_edit_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be edited.");
            return;
        };
        let field = &self.model.fields[field_index];
        if let Err(error) = self.ensure_editable_config(&field.path) {
            self.notice_error(error.message());
            return;
        }
        let input = edit_input_for_field(field);
        self.edit = Some(ConfigUiEditState {
            field_index,
            choice_index: initial_edit_choice_index(field, &input),
            input,
            mode: edit_mode_for_field(field),
        });
    }

    fn quick_edit_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be edited.");
            return;
        };
        let field = &self.model.fields[field_index];
        if let Err(error) = self.ensure_editable_config(&field.path) {
            self.notice_error(error.message());
            return;
        }
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
        let path = self.model.fields[field_index].path.clone();
        if let Err(error) = self.ensure_editable_config(&path) {
            self.notice_error(error.message());
            return;
        }
        match self.unset_field_value(&path) {
            Ok(outcome) => {
                if outcome.mutation == SettingsJsoncPatchMutation::Unchanged {
                    self.notice_info(format!("{path} was already unset."));
                } else {
                    self.notice_info(write_notice_text("Unset", &path, &outcome));
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
            Ok(outcome) => {
                if outcome.mutation == SettingsJsoncPatchMutation::Unchanged {
                    self.notice_info(format!("{path} was already set."));
                } else {
                    self.notice_info(write_notice_text("Saved", &path, &outcome));
                }
            }
            Err(error) => self.notice_error(error.message()),
        }
    }

    fn write_field_value(
        &mut self,
        setting_path: &str,
        value: &JsonValue,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        self.ensure_editable_config(setting_path)?;
        let target = self.edit_target(setting_path);
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome =
            set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, value)?;
        let mut apply_status = None;
        let mut apply_error = None;
        if outcome.changed() {
            self.validate_patched_edit_target(&target, &outcome.text)?;
            write_settings_edit(&target.path, &outcome.text)?;
            match apply_after_field_write(&self.request, &self.model, setting_path) {
                Ok(status) => apply_status = Some(status),
                Err(error) => apply_error = Some(apply_error_notice(&error)),
            }
        }
        self.reload_model_preserving_selection(setting_path)?;
        Ok(ConfigUiWriteOutcome {
            mutation: outcome.mutation,
            apply_status,
            apply_error,
        })
    }

    fn unset_field_value(&mut self, setting_path: &str) -> Result<ConfigUiWriteOutcome, CoreError> {
        self.ensure_editable_config(setting_path)?;
        let target = self.edit_target(setting_path);
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome = unset_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file)?;
        let mut apply_status = None;
        let mut apply_error = None;
        if outcome.changed() {
            self.validate_patched_edit_target(&target, &outcome.text)?;
            write_settings_edit(&target.path, &outcome.text)?;
            match apply_after_field_write(&self.request, &self.model, setting_path) {
                Ok(status) => apply_status = Some(status),
                Err(error) => apply_error = Some(apply_error_notice(&error)),
            }
        }
        self.reload_model_preserving_selection(setting_path)?;
        Ok(ConfigUiWriteOutcome {
            mutation: outcome.mutation,
            apply_status,
            apply_error,
        })
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

    fn edit_target(&self, setting_path: &str) -> ConfigUiEditTarget {
        if let Some(cursor_path) = setting_path.strip_prefix("cursors.") {
            ConfigUiEditTarget {
                path: self.model.cursor_config_path.clone(),
                path_in_file: cursor_path.to_string(),
                kind: ConfigUiEditTargetKind::Cursors,
            }
        } else {
            ConfigUiEditTarget {
                path: self.model.active_config_path.clone(),
                path_in_file: setting_path.to_string(),
                kind: ConfigUiEditTargetKind::Main,
            }
        }
    }

    fn read_edit_target_or_default(
        &self,
        target: &ConfigUiEditTarget,
    ) -> Result<String, CoreError> {
        if target.path.exists() {
            return read_settings_for_edit_or_empty(&target.path);
        }
        match target.kind {
            ConfigUiEditTargetKind::Main => read_settings_for_edit_or_empty(&target.path),
            ConfigUiEditTargetKind::Cursors => {
                let raw =
                    fs::read_to_string(&self.model.default_cursor_config_path).map_err(|source| {
                        CoreError::io(
                            "read_default_cursor_config_for_ui_edit",
                            "Could not read the default Yazelix cursor settings",
                            "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml.",
                            self.model.default_cursor_config_path.display().to_string(),
                            source,
                        )
                    })?;
                let registry =
                    CursorRegistry::parse_str(&self.model.default_cursor_config_path, &raw)?;
                Ok(render_cursor_settings_jsonc(&registry))
            }
        }
    }

    fn validate_patched_edit_target(
        &self,
        target: &ConfigUiEditTarget,
        text: &str,
    ) -> Result<(), CoreError> {
        match target.kind {
            ConfigUiEditTargetKind::Main => validate_patched_settings_for_ui(&self.request, text),
            ConfigUiEditTargetKind::Cursors => {
                let value = parse_jsonc_value(&target.path, text)?;
                CursorRegistry::parse_json_value(&target.path, value)?;
                Ok(())
            }
        }
    }

    fn ensure_editable_config(&self, setting_path: &str) -> Result<(), CoreError> {
        let target = self.edit_target(setting_path);
        if !is_settings_config_path(&target.path) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_config_edit_surface",
                format!(
                    "The config UI can only edit settings.jsonc, but the active config is {}.",
                    target.path.display()
                ),
                "Move this setting to settings.jsonc, or clear YAZELIX_CONFIG_OVERRIDE.",
                json!({ "path": target.path.display().to_string() }),
            ));
        }
        let target_exists = path_present(&target.path);
        if classify_path_owner(&target.path, target_exists) == ConfigUiPathOwner::HomeManager {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "home_manager_owned_config",
                "This settings file is owned by Home Manager.",
                "Edit your Home Manager module options instead, then run home-manager switch.",
                json!({ "path": target.path.display().to_string() }),
            ));
        }
        if path_is_read_only(&target.path) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "read_only_settings_config",
                format!(
                    "The active settings file is read-only: {}.",
                    target.path.display()
                ),
                "Fix file permissions or edit the owning configuration source.",
                json!({ "path": target.path.display().to_string() }),
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

    pub(crate) fn visible_rows(&self) -> Vec<UiRowRef> {
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
            rows.extend(
                self.model
                    .native_config_statuses
                    .iter()
                    .enumerate()
                    .filter(|(_, status)| self.matches_native_status(status))
                    .map(|(index, _)| UiRowRef::NativeStatus(index)),
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

    pub(crate) fn render_row(&self, row: UiRowRef) -> Line<'static> {
        match row {
            UiRowRef::Field(index) => {
                let field = &self.model.fields[index];
                Line::from(vec![
                    Span::styled(
                        fixed_label(state_label(field.state), 9),
                        state_style(field.state),
                    ),
                    Span::styled(
                        fixed_label(&field.apply_status.summary, 13),
                        apply_status_style(&field.apply_status),
                    ),
                    Span::styled(truncate(&field.path, 34), config_key_style()),
                    Span::styled(
                        format!(" {}", truncate(&field.current_value, 22)),
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
            UiRowRef::NativeStatus(index) => {
                let status = &self.model.native_config_statuses[index];
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

    pub(crate) fn render_details(&self, row: UiRowRef) -> Vec<Line<'static>> {
        match row {
            UiRowRef::Field(index) => {
                let field = &self.model.fields[index];
                if let Some(edit) = &self.edit
                    && edit.field_index == index
                    && edit.mode == ConfigUiEditMode::Choice
                    && is_scalar_enum_field(field)
                {
                    return single_choice_detail_lines(field, edit);
                }
                if let Some(edit) = &self.edit
                    && edit.field_index == index
                    && edit.mode == ConfigUiEditMode::MultiChoice
                {
                    return multi_choice_detail_lines(field, edit);
                }
                field_detail_lines(field)
            }
            UiRowRef::Sidecar(index) => sidecar_detail_lines(&self.model.sidecars[index]),
            UiRowRef::Diagnostic(index) => diagnostic_detail_lines(&self.model.diagnostics[index]),
            UiRowRef::NativeStatus(index) => {
                native_status_detail_lines(&self.model.native_config_statuses[index])
            }
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

    fn matches_native_status(&self, status: &NativeConfigStatusEntry) -> bool {
        self.search_matches([
            status.surface.as_str(),
            status.tool.as_str(),
            status.status.as_str(),
            status.label.as_str(),
            status.description.as_str(),
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

    pub(crate) fn clamp_selection_for_len(&mut self, len: usize) {
        if len == 0 {
            self.selected_row = 0;
        } else if self.selected_row >= len {
            self.selected_row = len - 1;
        }
    }
}

fn field_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    if field.path == ZELLIJ_KEYBINDINGS_FIELD_PATH {
        return zellij_keybinding_detail_lines(field);
    }

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
        detail_line("apply", &field.apply_status.label),
        detail_line("active", &field.apply_status.detail),
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

fn write_notice_text(verb: &str, path: &str, outcome: &ConfigUiWriteOutcome) -> String {
    let mut text = format!("{verb} {path}.");
    if let Some(error) = &outcome.apply_error {
        text.push(' ');
        text.push_str(error);
        return text;
    }
    if let Some(refresh) = outcome
        .apply_status
        .as_ref()
        .and_then(|status| status.generated_refresh.as_ref())
    {
        text.push(' ');
        text.push_str(&refresh.message);
        text.push(' ');
        text.push_str(&refresh.remediation);
    }
    if let Some(refresh) = outcome
        .apply_status
        .as_ref()
        .and_then(|status| status.pane_orchestrator_refresh.as_ref())
    {
        text.push(' ');
        text.push_str(&refresh.message);
        text.push(' ');
        text.push_str(&refresh.remediation);
    }
    text
}

fn apply_error_notice(error: &CoreError) -> String {
    let remediation = error.remediation();
    if remediation.trim().is_empty() {
        format!("Apply pending: {}", error.message())
    } else {
        format!("Apply pending: {} {}", error.message(), remediation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ZellijKeybindingObject {
    entries: BTreeMap<String, Vec<String>>,
    malformed_entries: Vec<String>,
    malformed_object: Option<String>,
}

fn zellij_keybinding_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let object = zellij_keybinding_object_for_field(field);
    let supported_actions = ZELLIJ_ACTIONS
        .iter()
        .map(|spec| spec.action.local_id)
        .collect::<BTreeSet<_>>();
    let unsupported_entries = object
        .entries
        .keys()
        .filter(|action| !supported_actions.contains(action.as_str()))
        .cloned()
        .collect::<Vec<_>>();

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
        detail_line("apply", &field.apply_status.label),
        detail_line("active", &field.apply_status.detail),
    ];
    if !field.validation.is_empty() {
        lines.push(detail_line("validation", &field.validation));
    }
    if field.rebuild_required {
        lines.push(detail_line("rebuild", "required"));
    }
    if let Some(message) = object.malformed_object {
        lines.push(detail_line("invalid", &message));
    }
    if !object.malformed_entries.is_empty() {
        lines.push(detail_line("invalid", &object.malformed_entries.join("; ")));
    }
    if !unsupported_entries.is_empty() {
        lines.push(detail_line("unsupported", &unsupported_entries.join(", ")));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Yazelix Zellij actions",
        metadata_key_style().add_modifier(Modifier::BOLD),
    )));

    for spec in ZELLIJ_ACTIONS {
        let action = &spec.action;
        let default_keys = action.default_keys;
        let explicit_keys = object.entries.get(action.local_id);
        let current_label = explicit_keys
            .map(|keys| zellij_keybinding_keys_label(keys.as_slice()))
            .unwrap_or_else(|| zellij_keybinding_keys_label(default_keys));
        let source_label = if let Some(keys) = explicit_keys {
            if keys.is_empty() {
                "disabled"
            } else {
                "remapped"
            }
        } else {
            "default"
        };

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            action.label.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(detail_line("action", action.id));
        lines.push(detail_line(
            "current",
            &format!("{current_label} ({source_label})"),
        ));
        lines.push(detail_line(
            "default",
            &zellij_keybinding_keys_label(default_keys),
        ));
        lines.push(detail_line("mode", spec.mode));
        lines.push(detail_line("backend", action.backend.as_str()));
        if action.disable_policy.empty_binding_list_allowed() {
            lines.push(detail_line("disable", "empty list disables this action"));
        } else {
            lines.push(detail_line("disable", "binding required"));
        }
    }

    if !field.description.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(field.description.clone()));
    }

    lines
}

fn zellij_keybinding_object_for_field(field: &ConfigUiField) -> ZellijKeybindingObject {
    if !matches!(
        field.state,
        ConfigUiValueState::Explicit | ConfigUiValueState::Invalid
    ) {
        return ZellijKeybindingObject {
            entries: BTreeMap::new(),
            malformed_entries: Vec::new(),
            malformed_object: None,
        };
    }

    let value = match serde_json::from_str::<JsonValue>(&field.edit_value) {
        Ok(value) => value,
        Err(source) => {
            return ZellijKeybindingObject {
                entries: BTreeMap::new(),
                malformed_entries: Vec::new(),
                malformed_object: Some(format!("not valid JSON: {source}")),
            };
        }
    };
    let Some(object) = value.as_object() else {
        return ZellijKeybindingObject {
            entries: BTreeMap::new(),
            malformed_entries: Vec::new(),
            malformed_object: Some("must be a JSON object".to_string()),
        };
    };

    let mut entries = BTreeMap::new();
    let mut malformed_entries = Vec::new();
    for (action, raw_keys) in object {
        let Some(values) = raw_keys.as_array() else {
            malformed_entries.push(format!("{action}: not a list"));
            continue;
        };
        let mut keys = Vec::with_capacity(values.len());
        let mut invalid = false;
        for value in values {
            let Some(key) = value.as_str() else {
                invalid = true;
                break;
            };
            keys.push(key.to_string());
        }
        if invalid {
            malformed_entries.push(format!("{action}: contains a non-string key"));
        } else {
            entries.insert(action.clone(), keys);
        }
    }

    ZellijKeybindingObject {
        entries,
        malformed_entries,
        malformed_object: None,
    }
}

fn zellij_keybinding_keys_label(keys: &[impl AsRef<str>]) -> String {
    if keys.is_empty() {
        "disabled".to_string()
    } else {
        keys.iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn single_choice_detail_lines(
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

    for (index, value) in field.allowed_values.iter().enumerate() {
        let highlighted = index
            == edit
                .choice_index
                .min(field.allowed_values.len().saturating_sub(1));
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

    lines
}

fn multi_choice_detail_lines(
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

fn native_status_detail_lines(status: &NativeConfigStatusEntry) -> Vec<Line<'static>> {
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

fn compose_config_ui_value(
    mut main_value: JsonValue,
    cursor_value: JsonValue,
) -> Result<JsonValue, CoreError> {
    let Some(object) = main_value.as_object_mut() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "settings_jsonc_not_object",
            "Yazelix settings must contain a JSON object.",
            "Replace the settings file with a valid object, then retry.",
            json!({}),
        ));
    };
    object.insert("cursors".to_string(), cursor_value);
    Ok(main_value)
}

fn read_cursor_config_value(path: &Path) -> Result<JsonValue, CoreError> {
    if !path.exists() {
        return Ok(JsonValue::Object(JsonMap::new()));
    }
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_cursor_config",
            "Could not read the Yazelix cursor settings",
            "Fix permissions for ~/.config/yazelix_cursors/settings.jsonc, then retry.",
            path.display().to_string(),
            source,
        )
    })?;
    parse_jsonc_value(path, &raw)
}

fn read_default_cursor_config_value(path: &Path) -> Result<JsonValue, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_default_cursor_config",
            "Could not read the default Yazelix cursor settings",
            "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml.",
            path.display().to_string(),
            source,
        )
    })?;
    let registry = CursorRegistry::parse_str(path, &raw)?;
    let rendered = render_cursor_settings_jsonc(&registry);
    parse_jsonc_value(path, &rendered)
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
        let apply_mode = table
            .get("apply_mode")
            .and_then(TomlValue::as_str)
            .ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Config,
                    "missing_apply_mode",
                    format!("Config contract field {field_path} is missing apply_mode."),
                    "Reinstall Yazelix so the runtime includes the current config contract.",
                    json!({ "field": field_path }),
                )
            })?
            .parse::<RuntimeApplyMode>()
            .map_err(|message| {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_apply_mode",
                    format!("Config contract field {field_path} has {message}."),
                    "Reinstall Yazelix so the runtime includes a valid config contract.",
                    json!({ "field": field_path }),
                )
            })?;
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
                apply_mode,
            },
        );
    }

    Ok(fields)
}

fn config_ui_metadata_path(settings_schema_path: &Path) -> PathBuf {
    settings_schema_path.with_file_name(CONFIG_UI_METADATA_FILENAME)
}

fn load_config_ui_metadata(path: &Path) -> Result<ConfigUiMetadata, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_metadata",
            "Could not read the Yazelix config UI metadata",
            "Reinstall Yazelix so the runtime includes config_metadata/config_ui_metadata.toml.",
            path.display().to_string(),
            source,
        )
    })?;
    let metadata = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_config_ui_metadata",
            "Could not parse the Yazelix config UI metadata",
            "Reinstall Yazelix so the runtime includes a valid config UI metadata file.",
            path.display().to_string(),
            source,
        )
    })?;

    let tabs = metadata
        .get("tab_order")
        .and_then(TomlValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(TomlValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_config_ui_tabs",
            "The Yazelix config UI metadata is missing tab_order.",
            "Reinstall Yazelix so the runtime includes current config UI metadata.",
            json!({ "path": path.display().to_string() }),
        ));
    }

    let fields_table = metadata
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "missing_config_ui_fields",
                "The Yazelix config UI metadata is missing its fields table.",
                "Reinstall Yazelix so the runtime includes current config UI metadata.",
                json!({ "path": path.display().to_string() }),
            )
        })?;

    let mut fields = BTreeMap::new();
    for (field_path, value) in fields_table {
        let table = value.as_table().ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_config_ui_field",
                format!("Config UI metadata field {field_path} must be a TOML table."),
                "Reinstall Yazelix so the runtime includes valid config UI metadata.",
                json!({ "field": field_path }),
            )
        })?;
        fields.insert(
            field_path.clone(),
            FieldUiMetadata {
                tab: required_toml_string(table, field_path, "tab")?,
                help: required_toml_string(table, field_path, "help")?,
            },
        );
    }

    Ok(ConfigUiMetadata { tabs, fields })
}

fn required_toml_string(
    table: &toml::Table,
    field_path: &str,
    key: &str,
) -> Result<String, CoreError> {
    table
        .get(key)
        .and_then(TomlValue::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_config_ui_field_metadata",
                format!("Config UI metadata field {field_path} is missing {key}."),
                "Reinstall Yazelix so the runtime includes complete config UI metadata.",
                json!({ "field": field_path, "key": key }),
            )
        })
}

fn ensure_ui_metadata_tabs_match_schema(
    metadata_tabs: &[String],
    schema_tabs: &[String],
) -> Result<(), CoreError> {
    if metadata_tabs == schema_tabs {
        return Ok(());
    }

    Err(CoreError::classified(
        ErrorClass::Config,
        "config_ui_tab_order_mismatch",
        "The Yazelix config UI metadata tab order does not match the settings schema.",
        "Reinstall Yazelix so config_metadata/config_ui_metadata.toml and yazelix_settings.schema.json come from the same version.",
        json!({
            "metadata_tabs": metadata_tabs,
            "schema_tabs": schema_tabs,
        }),
    ))
}

fn field_ui_metadata<'a>(
    metadata: &'a ConfigUiMetadata,
    path: &str,
) -> Result<&'a FieldUiMetadata, CoreError> {
    metadata.fields.get(path).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Config,
            "missing_config_ui_field_metadata",
            format!("The Yazelix config UI metadata is missing field {path}."),
            "Reinstall Yazelix so the config UI metadata covers the current settings surface.",
            json!({ "field": path }),
        )
    })
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

fn collect_cursor_schema_fields(schema: &JsonValue) -> Vec<SchemaField> {
    let mut fields = Vec::new();
    let Some(cursors) = schema
        .get("properties")
        .and_then(|properties| properties.get("cursors"))
    else {
        return fields;
    };
    collect_schema_fields(cursors, "cursors", &mut fields);
    fields
}

fn collect_schema_fields(schema: &JsonValue, path: &str, out: &mut Vec<SchemaField>) {
    let kind = schema_type(schema);
    if kind == "object" {
        let Some(properties) = schema.get("properties").and_then(JsonValue::as_object) else {
            out.push(schema_field(schema, path, kind));
            return;
        };
        for (name, property) in properties {
            collect_schema_fields(property, &format!("{path}.{name}"), out);
        }
        return;
    }

    if kind == "array"
        && let Some(items) = schema.get("items")
        && items.get("type").and_then(JsonValue::as_str) == Some("object")
    {
        out.push(schema_field(schema, path, kind));
        return;
    }

    out.push(schema_field(schema, path, kind));
}

fn schema_field(schema: &JsonValue, path: &str, kind: String) -> SchemaField {
    SchemaField {
        path: path.to_string(),
        kind,
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
    apply_mode: RuntimeApplyMode,
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
        edit_value: current
            .or(default)
            .map(render_json_edit_value)
            .unwrap_or_default(),
        default_value: default
            .map(render_json_value)
            .unwrap_or_else(|| "no default".to_string()),
        state,
        description,
        allowed_values,
        validation,
        rebuild_required,
        apply_mode,
        apply_status: apply_status_for_mode(apply_mode),
    }
}

fn apply_status_for_mode(apply_mode: RuntimeApplyMode) -> ConfigUiApplyStatus {
    let (summary, detail, pending) = match apply_mode {
        RuntimeApplyMode::Live => ("active", "Saved values are active in this process.", false),
        RuntimeApplyMode::LiveWithPaneRefresh => (
            "pane refresh",
            "Saved changes require a Yazelix-owned pane or plugin refresh before running panes use them.",
            true,
        ),
        RuntimeApplyMode::GeneratedRuntimeRefresh => (
            "gen refresh",
            "Saved changes regenerate managed runtime config; running tools must be restarted or reopened.",
            true,
        ),
        RuntimeApplyMode::TabSessionRestart => (
            "tab restart",
            "Saved changes become active after a fresh Yazelix tab or session starts.",
            true,
        ),
        RuntimeApplyMode::ShellTerminalRestart => (
            "shell restart",
            "Saved changes become active in newly launched terminal or shell processes.",
            true,
        ),
        RuntimeApplyMode::PackageHomeManagerActivation => (
            "HM activate",
            "Edit the Home Manager source and run home-manager switch before the runtime can use this value.",
            true,
        ),
        RuntimeApplyMode::NeverLive => (
            "not live",
            "This setting is a native/import/generated ownership boundary and is not live-applicable.",
            true,
        ),
    };
    ConfigUiApplyStatus {
        summary: summary.to_string(),
        label: apply_mode.label().to_string(),
        detail: detail.to_string(),
        pending,
    }
}

fn field_description(field: &ContractField, metadata: &FieldUiMetadata) -> String {
    let mut parts = Vec::new();
    parts.push(metadata.help.clone());
    if !field.validation.is_empty() {
        parts.push(format!("validation: {}", field.validation));
    }
    if let (Some(min), Some(max)) = (field.min, field.max) {
        parts.push(format!("range: {min}..{max}"));
    }
    if field.rebuild_required {
        parts.push("takes effect after runtime rebuild or rematerialization".to_string());
    }
    parts.push(format!("apply: {}", field.apply_mode.label()));
    parts.join("; ")
}

fn collect_sidecars(config_dir: &Path) -> Vec<ConfigUiSidecar> {
    let mut sidecars = CURRENT_MANAGED_CONFIG_FILE_NAMES
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
        .collect::<Vec<_>>();
    let cursor_path = crate::user_config_paths::shared_cursor_config(config_dir);
    let cursor_present = fs::symlink_metadata(&cursor_path).is_ok();
    sidecars.push(ConfigUiSidecar {
        name: "yazelix_cursors/settings.jsonc".to_string(),
        owner: classify_path_owner(&cursor_path, cursor_present),
        read_only: path_is_read_only(&cursor_path),
        path: cursor_path,
        present: cursor_present,
    });
    sidecars
}

fn classify_path_owner(path: &Path, present: bool) -> ConfigUiPathOwner {
    if !present {
        return ConfigUiPathOwner::Default;
    }
    if path_owned_by_home_manager(path) {
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

fn effective_json_path<'a>(
    active: &'a JsonValue,
    default: &'a JsonValue,
    path: &str,
) -> Option<&'a JsonValue> {
    get_json_path(active, path).or_else(|| get_json_path(default, path))
}

fn effective_string_config(
    active: &JsonValue,
    default: &JsonValue,
    path: &str,
    fallback: &str,
) -> String {
    effective_json_path(active, default, path)
        .and_then(JsonValue::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn effective_string_list_config(
    active: &JsonValue,
    default: &JsonValue,
    path: &str,
    fallback: &[&str],
) -> Vec<String> {
    let values = effective_json_path(active, default, path)
        .and_then(JsonValue::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(JsonValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if values.is_empty() {
        fallback.iter().map(|value| (*value).to_string()).collect()
    } else {
        values
    }
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

fn render_json_edit_value(value: &JsonValue) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| render_json_value(value))
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

fn apply_status_style(status: &ConfigUiApplyStatus) -> Style {
    if status.pending {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    }
}

fn sidecar_status_style(present: bool) -> Style {
    if present {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    }
}

fn native_status_style(status: &NativeConfigStatusEntry) -> Style {
    match status_code_for_entry(status)
        .map(|code| code.doctor_severity())
        .unwrap_or("info")
    {
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

fn fixed_label(value: &str, width: usize) -> String {
    let label = format!("{value:<width$}");
    if label.ends_with(' ') {
        label
    } else {
        format!("{label} ")
    }
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
    use tempfile::tempdir;

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
            edit_value: current_value.to_string(),
            default_value: "no default".to_string(),
            state: ConfigUiValueState::Explicit,
            description: String::new(),
            allowed_values: allowed_values
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            validation: String::new(),
            rebuild_required: false,
            apply_mode: RuntimeApplyMode::TabSessionRestart,
            apply_status: apply_status_for_mode(RuntimeApplyMode::TabSessionRestart),
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
            runtime
                .join("config_metadata")
                .join("config_ui_metadata.toml"),
            include_str!("../../../config_metadata/config_ui_metadata.toml"),
        )
        .expect("config ui metadata");
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

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn lines_text(lines: &[Line<'_>]) -> String {
        lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
    }

    fn select_field_path(app: &mut ConfigUiApp, path: &str) {
        let field = app
            .model
            .fields
            .iter()
            .find(|field| field.path == path)
            .expect("field");
        app.selected_tab = app
            .model
            .tabs
            .iter()
            .position(|tab| tab == &field.tab)
            .expect("tab");
        app.selected_row = app
            .visible_rows()
            .iter()
            .position(|row| {
                matches!(
                    row,
                    UiRowRef::Field(index) if app.model.fields[*index].path == path
                )
            })
            .expect("row");
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

    // Regression: summarized list displays must not become the edit buffer, because placeholders like `[7 items]` are not JSON.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn list_fields_edit_from_full_json_not_display_summary() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let field = model
            .fields
            .iter()
            .find(|field| field.path == "zellij.widget_tray")
            .expect("widget tray");

        assert_eq!(field.current_value, "[7 items]");
        assert_eq!(field.apply_mode, RuntimeApplyMode::GeneratedRuntimeRefresh);
        let input = edit_input_for_field(field);
        assert!(input.starts_with("[\"editor\",\"shell\",\"term\""));
        assert_eq!(
            parse_edit_input(field, &input).expect("string list"),
            json!([
                "editor",
                "shell",
                "term",
                "cursor",
                "codex_usage",
                "cpu",
                "ram"
            ])
        );
    }

    // Defends: the keybinding tab renders Yazelix action registry labels, scoped ids, defaults, remaps, and disabled actions instead of an opaque JSON object.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn zellij_keybinding_details_use_action_registry_metadata() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        fs::write(
            config.path().join("settings.jsonc"),
            r#"{
  "zellij": {
    "keybindings": {
      "popup": ["Alt x"],
      "menu": [],
      "unknown_action": ["Alt z"]
    }
  }
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

        select_field_path(&mut app, "zellij.keybindings");
        let details = lines_text(&app.render_details(app.visible_rows()[app.selected_row]));

        assert!(details.contains("Toggle the managed popup program"));
        assert!(details.contains("zellij.popup"));
        assert!(details.contains("Alt x (remapped)"));
        assert!(details.contains("Alt t"));
        assert!(details.contains("Open the Yazelix command palette popup"));
        assert!(details.contains("disabled (disabled)"));
        assert!(details.contains("empty list disables this action"));
        assert!(details.contains("unsupported"));
        assert!(details.contains("unknown_action"));
    }

    // Defends: machine-readable apply modes from main_config_contract.toml reach the config UI model for the first live slice and restart-scoped fields.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn model_exposes_runtime_apply_modes_from_contract() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");

        let popup_width = model
            .fields
            .iter()
            .find(|field| field.path == "zellij.popup_width_percent")
            .expect("popup width");
        assert_eq!(
            popup_width.apply_mode,
            RuntimeApplyMode::LiveWithPaneRefresh
        );
        assert_eq!(popup_width.apply_status.summary, "pane refresh");
        assert!(popup_width.apply_status.pending);
        assert!(popup_width.apply_status.detail.contains("pane or plugin"));

        let editor_command = model
            .fields
            .iter()
            .find(|field| field.path == "editor.command")
            .expect("editor command");
        assert_eq!(
            editor_command.apply_mode,
            RuntimeApplyMode::TabSessionRestart
        );
        assert_eq!(editor_command.apply_status.summary, "tab restart");

        let terminal_config_mode = model
            .fields
            .iter()
            .find(|field| field.path == "terminal.config_mode")
            .expect("terminal config mode");
        assert_eq!(
            terminal_config_mode.apply_mode,
            RuntimeApplyMode::ShellTerminalRestart
        );
        assert_eq!(terminal_config_mode.apply_status.summary, "shell restart");

        let widget_tray = model
            .fields
            .iter()
            .find(|field| field.path == "zellij.widget_tray")
            .expect("widget tray");
        assert_eq!(widget_tray.apply_status.summary, "gen refresh");
        assert!(
            widget_tray
                .apply_status
                .detail
                .contains("managed runtime config")
        );
    }

    // Defends: Home Manager-owned settings are presented as activation-scoped even when the field's intrinsic apply mode is narrower.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[cfg(unix)]
    #[test]
    fn home_manager_owned_settings_use_activation_apply_mode() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let hm_dir = config.path().join("profile-home-manager-files");
        fs::create_dir_all(&hm_dir).expect("home manager dir");
        let hm_settings = hm_dir.join("settings.jsonc");
        fs::write(&hm_settings, "{}\n").expect("home manager settings");
        std::os::unix::fs::symlink(&hm_settings, config.path().join("settings.jsonc"))
            .expect("settings symlink");

        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let popup_width = model
            .fields
            .iter()
            .find(|field| field.path == "zellij.popup_width_percent")
            .expect("popup width");

        assert_eq!(model.config_owner, ConfigUiPathOwner::HomeManager);
        assert_eq!(
            popup_width.apply_mode,
            RuntimeApplyMode::PackageHomeManagerActivation
        );
        assert_eq!(popup_width.apply_status.summary, "HM activate");
        assert!(
            popup_width
                .apply_status
                .detail
                .contains("home-manager switch")
        );
    }

    // Defends: the config UI consumes the shared native-config status labels instead of maintaining separate sidecar wording.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn model_includes_native_config_status_entries() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());

        let model = build_config_ui_model(&request).expect("model");
        let settings = model
            .native_config_statuses
            .iter()
            .find(|status| status.surface == "settings.main")
            .expect("settings status");

        assert_eq!(settings.status, "canonical_settings");
        assert_eq!(settings.label, "Canonical Yazelix settings");
        assert!(
            model
                .native_config_statuses
                .iter()
                .any(|status| status.surface == "zellij.generated"
                    && status.status == "generated_runtime")
        );
    }

    // Defends: enum-backed string lists use an enable/disable picker instead of forcing users to edit JSON arrays.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn enum_string_list_picker_toggles_subvalues_with_space() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
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

        select_field_path(&mut app, "terminal.terminals");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let edit = app.edit.clone().expect("edit");
        assert_eq!(edit.mode, ConfigUiEditMode::MultiChoice);
        let details = lines_text(&app.render_details(UiRowRef::Field(edit.field_index)));
        assert!(details.contains("> [x] ghostty"));
        assert!(details.contains("  [ ] alacritty"));

        app.handle_edit_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));

        let field = app.model.fields[edit.field_index].clone();
        let input = app.edit.as_ref().expect("edit").input.clone();
        assert_eq!(
            parse_string_list_values(&field, &input).expect("values"),
            vec!["ghostty", "wezterm", "alacritty"]
        );

        app.handle_edit_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.edit.is_none());
        let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
        assert_eq!(
            get_json_path(&value, "terminal.terminals"),
            Some(&json!(["ghostty", "wezterm", "alacritty"]))
        );
    }

    // Defends: keyboard-oriented quick edits produce deterministic toggles/cycles from the value shown in the UI.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn quick_edit_helpers_toggle_bool_and_cycle_enum() {
        let bool_field = test_field("core.debug_mode", "bool", "true", &[]);
        assert_eq!(field_bool_value(&bool_field), Some(true));
        assert_eq!(edit_mode_for_field(&bool_field), ConfigUiEditMode::Choice);

        let enum_field = test_field(
            "zellij.tab_label_mode",
            "string",
            "\"compact\"",
            &["short", "compact", "full"],
        );
        assert_eq!(edit_input_for_field(&enum_field), "compact");
        assert_eq!(edit_mode_for_field(&enum_field), ConfigUiEditMode::Choice);
        assert_eq!(next_allowed_value(&enum_field), "full");
    }

    // Defends: bool edits stay direct controls while enum edit mode behaves like a single-select picker.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn choice_edit_keys_toggle_bool_and_move_enum_picker() {
        let mut app = ConfigUiApp {
            request: ConfigUiRequest {
                runtime_dir: PathBuf::from("/runtime"),
                config_dir: PathBuf::from("/home/lucca/.config/yazelix"),
                config_override: None,
            },
            model: ConfigUiModel {
                active_config_path: PathBuf::from("/home/lucca/.config/yazelix/settings.jsonc"),
                cursor_config_path: PathBuf::from(
                    "/home/lucca/.config/yazelix_cursors/settings.jsonc",
                ),
                default_cursor_config_path: PathBuf::from("/runtime/yazelix_cursors_default.toml"),
                active_config_exists: true,
                config_owner: ConfigUiPathOwner::User,
                config_read_only: false,
                tabs: vec!["general".to_string()],
                fields: vec![
                    test_field("core.debug_mode", "bool", "true", &[]),
                    test_field(
                        "zellij.tab_label_mode",
                        "string",
                        "\"compact\"",
                        &["short", "compact", "full"],
                    ),
                ],
                sidecars: Vec::new(),
                native_config_statuses: Vec::new(),
                diagnostics: Vec::new(),
            },
            selected_tab: 0,
            selected_row: 0,
            search: String::new(),
            search_active: false,
            edit: Some(ConfigUiEditState {
                field_index: 0,
                input: "true".to_string(),
                mode: ConfigUiEditMode::Choice,
                choice_index: 0,
            }),
            notice: None,
        };

        app.handle_edit_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.edit.as_ref().expect("bool edit").input, "false");

        app.edit = Some(ConfigUiEditState {
            field_index: 1,
            input: "compact".to_string(),
            mode: ConfigUiEditMode::Choice,
            choice_index: 1,
        });
        app.handle_edit_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(app.edit.as_ref().expect("enum edit").choice_index, 2);
        assert_eq!(app.edit.as_ref().expect("enum edit").input, "compact");
        app.handle_edit_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        assert_eq!(app.edit.as_ref().expect("enum edit").input, "full");
    }

    // Defends: enum rows open a single-select picker that can be driven with hjkl and saved through the JSONC patcher.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn scalar_enum_enter_opens_single_select_picker() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
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

        select_field_path(&mut app, "terminal.config_mode");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let edit = app.edit.clone().expect("edit");
        assert_eq!(edit.mode, ConfigUiEditMode::Choice);
        let details = lines_text(&app.render_details(UiRowRef::Field(edit.field_index)));
        assert!(details.contains("> (x) yazelix"));
        assert!(details.contains("  ( ) user"));

        app.handle_edit_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        let details = lines_text(&app.render_details(UiRowRef::Field(edit.field_index)));
        assert!(details.contains("> ( ) user"));
        app.handle_edit_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        let details = lines_text(&app.render_details(UiRowRef::Field(edit.field_index)));
        assert!(details.contains("> (x) user"));

        app.handle_edit_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.edit.is_none());
        let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
        assert_eq!(
            get_json_path(&value, "terminal.config_mode"),
            Some(&json!("user"))
        );
    }

    // Defends: Enter on bool rows performs the direct control action instead of opening an edit session.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn enter_directly_applies_bool_field_without_edit_mode() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        fs::write(
            &settings_path,
            r#"{
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

        select_field_path(&mut app, "editor.hide_sidebar_on_file_open");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.edit.is_none());
        let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
        assert_eq!(
            get_json_path(&value, "editor.hide_sidebar_on_file_open"),
            Some(&json!(true))
        );
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

        let outcome = app
            .write_field_value("editor.hide_sidebar_on_file_open", &json!(true))
            .expect("write");

        assert_eq!(outcome.mutation, SettingsJsoncPatchMutation::Replaced);
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

    // Regression: a save-time refresh failure remains visible as pending apply work instead of hiding the fact that the setting was already persisted.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn write_notice_keeps_saved_setting_visible_when_apply_fails() {
        let outcome = ConfigUiWriteOutcome {
            mutation: SettingsJsoncPatchMutation::Replaced,
            apply_status: None,
            apply_error: Some(
                "Apply pending: Saved yazi.theme, but generated config refresh failed.".to_string(),
            ),
        };

        let notice = write_notice_text("Saved", "yazi.theme", &outcome);

        assert!(notice.contains("Saved yazi.theme."));
        assert!(notice.contains("Apply pending"));
        assert!(notice.contains("generated config refresh failed"));
    }
}
