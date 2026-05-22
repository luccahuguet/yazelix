//! Terminal UI for inspecting and editing the canonical Yazelix config surface.

mod apply_adapter;

use crate::action_registry::{
    YAZI_ACTIONS, YazelixActionMetadata, ZELLIJ_ACTIONS, ZELLIJ_NATIVE_KEYBINDINGS,
};
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
use crate::runtime_component_enabled;
use crate::settings_jsonc_patch::{
    SettingsJsoncPatchMutation, SettingsJsoncPatchOutcome, set_settings_jsonc_value_text,
    unset_settings_jsonc_value_text,
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
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;
use yazelix_ghostty_cursors::{CursorRegistry, render_cursor_settings_jsonc};

pub use crate::yazelix_ratconfig::{
    ConfigUiApplyStatus, ConfigUiDiagnostic, ConfigUiField, ConfigUiModel, ConfigUiNativeStatus,
    ConfigUiPathOwner, ConfigUiSidecar, ConfigUiValueState,
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
const ZELLIJ_KEYBINDINGS_FIELD_PATH: &str = "zellij.keybindings";
const ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH: &str = "zellij.native_keybindings";
const YAZI_KEYBINDINGS_FIELD_PATH: &str = "yazi.keybindings";

#[derive(Debug, Clone)]
pub struct ConfigUiRequest {
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_override: Option<String>,
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

#[derive(Debug, Clone, Default)]
struct CursorChoiceValues {
    definition_names: Vec<String>,
    enabled_names: Vec<String>,
}

pub(crate) struct YazelixConfigUiApp {
    pub(crate) request: ConfigUiRequest,
    pub(crate) ui: ConfigUiApp,
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

impl Deref for YazelixConfigUiApp {
    type Target = ConfigUiApp;

    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

impl DerefMut for YazelixConfigUiApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui
    }
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
    let cursor_component_enabled = runtime_component_enabled(&request.runtime_dir, "cursors")?;
    let tabs = if cursor_component_enabled {
        ui_metadata.tabs.clone()
    } else {
        ui_metadata
            .tabs
            .iter()
            .filter(|tab| tab.as_str() != "cursors")
            .cloned()
            .collect()
    };
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
        if cursor_component_enabled {
            read_cursor_config_value(&paths.user_cursor_config)?
        } else {
            JsonValue::Object(JsonMap::new())
        },
    )?;

    let default_raw = render_default_settings_jsonc(&paths.default_config_path)?;
    let default_main_value = parse_jsonc_value(&paths.default_config_path, &default_raw)?;
    let default_value = compose_config_ui_value(
        default_main_value,
        if cursor_component_enabled {
            read_default_cursor_config_value(&paths.default_cursor_config_path)?
        } else {
            JsonValue::Object(JsonMap::new())
        },
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
            edit_behavior_for_field_path(&field.path),
        ));
    }
    append_keybinding_action_fields(
        &mut fields,
        &contract_fields,
        config_owner,
        &active_value,
        &default_value,
        &blocking_paths,
    );

    if cursor_component_enabled {
        let cursor_choice_values = cursor_choice_values(&active_value, &default_value);
        for mut schema_field in collect_cursor_schema_fields(&schema) {
            if fields.iter().any(|field| field.path == schema_field.path) {
                continue;
            }
            enrich_cursor_schema_field(&mut schema_field, &cursor_choice_values);
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
                edit_behavior_for_field_path(&schema_field.path),
            ));
        }
    }

    fields.sort_by(|left, right| {
        tab_index(&tabs, &left.tab)
            .cmp(&tab_index(&tabs, &right.tab))
            .then_with(|| left.path.cmp(&right.path))
    });

    let home_dir = home_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let native_config_statuses = map_native_statuses(&classify_native_config_statuses(
        &NativeConfigStatusRequest {
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
        },
    ));

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
    let mut app = YazelixConfigUiApp::new(request, model);

    loop {
        app.clamp_selection();
        terminal
            .draw(|frame| draw_config_ui(frame, &mut app.ui))
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

impl YazelixConfigUiApp {
    pub(crate) fn new(request: ConfigUiRequest, model: ConfigUiModel) -> Self {
        Self {
            request,
            ui: ConfigUiApp::new(model),
        }
    }

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
                ConfigUiEditMode::Choice | ConfigUiEditMode::MultiChoice => {
                    self.handle_choice_edit_key(key, mode);
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

    fn handle_choice_edit_key(&mut self, key: KeyEvent, mode: ConfigUiEditMode) {
        let scalar_enum = self
            .edit
            .as_ref()
            .and_then(|edit| self.model.fields.get(edit.field_index))
            .is_some_and(is_scalar_enum_field);
        let multi_choice = mode == ConfigUiEditMode::MultiChoice;
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
                if scalar_enum || multi_choice =>
            {
                self.notice = None;
                self.move_choice_edit(-1);
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l')
                if scalar_enum || multi_choice =>
            {
                self.notice = None;
                self.move_choice_edit(1);
            }
            KeyCode::Char(' ') if multi_choice => {
                self.notice = None;
                self.toggle_multi_choice_edit();
            }
            KeyCode::Char(' ') if scalar_enum => {
                self.notice = None;
                self.select_single_choice_edit();
            }
            KeyCode::Up | KeyCode::Right | KeyCode::Down | KeyCode::Left | KeyCode::Char(' ')
                if !multi_choice =>
            {
                self.notice = None;
                self.cycle_choice_edit();
            }
            _ => {}
        }
    }

    fn cycle_choice_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let next = if edit.input.trim() == "true" {
            "false".to_string()
        } else {
            "true".to_string()
        };
        if let Some(edit) = &mut self.edit {
            edit.input = next;
        }
    }

    fn move_choice_edit(&mut self, delta: isize) {
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
        let value = value.clone();
        if let Some(edit) = &mut self.edit {
            edit.input = value;
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
        if let Some(message) = structured_only_edit_notice(field).map(str::to_string) {
            self.notice_info(message);
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
        if is_bool_field(field) {
            if let Err(error) = self.ensure_editable_config(&field.path) {
                self.notice_error(error.message());
                return;
            }
            let value = JsonValue::Bool(!field_bool_value(field).unwrap_or(false));
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
        self.finish_field_write(setting_path, &target, outcome)
    }

    fn unset_field_value(&mut self, setting_path: &str) -> Result<ConfigUiWriteOutcome, CoreError> {
        self.ensure_editable_config(setting_path)?;
        let target = self.edit_target(setting_path);
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome = match target.kind {
            ConfigUiEditTargetKind::Main => {
                let value = default_main_setting_value_for_ui(&self.request, &target.path_in_file)?;
                set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, &value)?
            }
            ConfigUiEditTargetKind::Cursors => {
                unset_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file)?
            }
        };
        self.finish_field_write(setting_path, &target, outcome)
    }

    fn finish_field_write(
        &mut self,
        setting_path: &str,
        target: &ConfigUiEditTarget,
        outcome: SettingsJsoncPatchOutcome,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
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
            return read_settings_for_edit(&target.path);
        }
        match target.kind {
            ConfigUiEditTargetKind::Main => default_main_settings_text_for_ui(&self.request),
            ConfigUiEditTargetKind::Cursors => {
                let raw =
                    fs::read_to_string(&self.model.default_cursor_config_path).map_err(|source| {
                        CoreError::io(
                            "read_default_cursor_config_for_ui_edit",
                            "Could not read the default Yazelix cursor settings",
                            "Reinstall Yazelix so the runtime includes yazelix_ghostty_cursors_default.toml.",
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
}

impl ConfigUiApp {
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
                if is_scalar_enum_field(field) {
                    return single_choice_field_detail_lines(field);
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
}

fn field_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    if is_keybinding_map_field_path(&field.path) {
        keybinding_map_detail_lines(field)
    } else if let Some(action) = keybinding_action_metadata_for_field_path(&field.path) {
        keybinding_action_detail_lines(field, action)
    } else {
        default_field_detail_lines(field)
    }
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
struct SemanticKeybindingObject {
    entries: BTreeMap<String, Vec<String>>,
    malformed_entries: Vec<String>,
    malformed_object: Option<String>,
}

fn keybinding_map_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let object = semantic_keybinding_object_for_field(field);
    let parent_path = field.path.as_str();
    let actions = keybinding_actions_for_parent_path(parent_path);
    let supported_actions = actions
        .iter()
        .map(|action| action.local_id)
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
        detail_line("takes effect", &field.apply_status.label),
        detail_line("after save", &field.apply_status.detail),
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
        keybinding_surface_title(parent_path),
        metadata_key_style().add_modifier(Modifier::BOLD),
    )));

    for action in actions {
        let default_keys = action.default_keys;
        let explicit_keys = object.entries.get(action.local_id);
        let current_label = explicit_keys
            .map(|keys| keybinding_keys_label(keys.as_slice()))
            .unwrap_or_else(|| keybinding_keys_label(default_keys));
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
        lines.push(detail_line("default", &keybinding_keys_label(default_keys)));
        for (label, value) in keybinding_action_metadata_lines(parent_path, action.local_id) {
            lines.push(detail_line(label, &value));
        }
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

fn keybinding_action_detail_lines(
    field: &ConfigUiField,
    action: &'static YazelixActionMetadata,
) -> Vec<Line<'static>> {
    let parent_path =
        keybinding_parent_path_for_field_path(&field.path).unwrap_or(field.path.as_str());
    let mut lines = default_field_detail_lines(field);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        action.label.to_string(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(detail_line("action", action.id));
    lines.push(detail_line(
        "keys",
        &keybinding_keys_label_from_field(field),
    ));
    for (label, value) in keybinding_action_metadata_lines(parent_path, action.local_id) {
        lines.push(detail_line(label, &value));
    }
    lines.push(detail_line("backend", action.backend.as_str()));
    lines.push(detail_line("command", action.generated_command));
    if action.disable_policy.empty_binding_list_allowed() {
        lines.push(detail_line("disable", "empty list disables this action"));
    } else {
        lines.push(detail_line("disable", "binding required"));
    }
    lines
}

fn semantic_keybinding_object_for_field(field: &ConfigUiField) -> SemanticKeybindingObject {
    if !matches!(
        field.state,
        ConfigUiValueState::Explicit | ConfigUiValueState::Invalid
    ) {
        return SemanticKeybindingObject {
            entries: BTreeMap::new(),
            malformed_entries: Vec::new(),
            malformed_object: None,
        };
    }

    let value = match serde_json::from_str::<JsonValue>(&field.edit_value) {
        Ok(value) => value,
        Err(source) => {
            return SemanticKeybindingObject {
                entries: BTreeMap::new(),
                malformed_entries: Vec::new(),
                malformed_object: Some(format!("not valid JSON: {source}")),
            };
        }
    };
    let Some(object) = value.as_object() else {
        return SemanticKeybindingObject {
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

    SemanticKeybindingObject {
        entries,
        malformed_entries,
        malformed_object: None,
    }
}

fn keybinding_keys_label(keys: &[impl AsRef<str>]) -> String {
    if keys.is_empty() {
        "disabled".to_string()
    } else {
        keys.iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn keybinding_keys_label_from_field(field: &ConfigUiField) -> String {
    serde_json::from_str::<Vec<String>>(&field.edit_value)
        .map(|keys| keybinding_keys_label(keys.as_slice()))
        .unwrap_or_else(|_| field.current_value.clone())
}

fn append_keybinding_action_fields(
    fields: &mut Vec<ConfigUiField>,
    contract_fields: &BTreeMap<String, ContractField>,
    config_owner: ConfigUiPathOwner,
    active_value: &JsonValue,
    default_value: &JsonValue,
    blocking_paths: &BTreeSet<String>,
) {
    append_keybinding_surface_action_fields(
        fields,
        contract_fields,
        config_owner,
        active_value,
        default_value,
        blocking_paths,
        ZELLIJ_KEYBINDINGS_FIELD_PATH,
    );
    append_keybinding_surface_action_fields(
        fields,
        contract_fields,
        config_owner,
        active_value,
        default_value,
        blocking_paths,
        YAZI_KEYBINDINGS_FIELD_PATH,
    );
}

fn append_keybinding_surface_action_fields(
    fields: &mut Vec<ConfigUiField>,
    contract_fields: &BTreeMap<String, ContractField>,
    config_owner: ConfigUiPathOwner,
    active_value: &JsonValue,
    default_value: &JsonValue,
    blocking_paths: &BTreeSet<String>,
    parent_path: &'static str,
) {
    let Some(parent_field) = contract_fields.get(parent_path) else {
        return;
    };
    let apply_mode = if config_owner == ConfigUiPathOwner::HomeManager {
        RuntimeApplyMode::PackageHomeManagerActivation
    } else {
        parent_field.apply_mode
    };
    for action in keybinding_actions_for_parent_path(parent_path) {
        let path = format!("{parent_path}.{}", action.local_id);
        let default = get_json_path(default_value, &path)
            .cloned()
            .unwrap_or_else(|| keybinding_default_value(action));
        fields.push(build_field_row(
            &path,
            "keybindings",
            "string_list",
            get_json_path(active_value, &path),
            Some(&default),
            action.label.to_string(),
            Vec::new(),
            parent_field.validation.clone(),
            parent_field.rebuild_required,
            apply_mode,
            blocking_paths.contains(&path) || blocking_paths.contains(parent_path),
            ConfigUiEditBehavior::FriendlyStringList,
        ));
    }
}

fn keybinding_default_value(action: &YazelixActionMetadata) -> JsonValue {
    JsonValue::Array(
        action
            .default_keys
            .iter()
            .map(|key| JsonValue::String((*key).to_string()))
            .collect(),
    )
}

fn keybinding_actions_for_parent_path(parent_path: &str) -> Vec<&'static YazelixActionMetadata> {
    match parent_path {
        ZELLIJ_KEYBINDINGS_FIELD_PATH => ZELLIJ_ACTIONS.iter().map(|spec| &spec.action).collect(),
        ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH => ZELLIJ_NATIVE_KEYBINDINGS
            .iter()
            .map(|spec| &spec.action)
            .collect(),
        YAZI_KEYBINDINGS_FIELD_PATH => YAZI_ACTIONS.iter().map(|spec| &spec.action).collect(),
        _ => Vec::new(),
    }
}

fn keybinding_action_metadata_lines(
    parent_path: &str,
    local_id: &str,
) -> Vec<(&'static str, String)> {
    if parent_path == ZELLIJ_KEYBINDINGS_FIELD_PATH
        && let Some(spec) = ZELLIJ_ACTIONS
            .iter()
            .find(|spec| spec.action.local_id == local_id)
    {
        return vec![("mode", spec.mode.to_string())];
    }
    if parent_path == YAZI_KEYBINDINGS_FIELD_PATH
        && let Some(spec) = YAZI_ACTIONS
            .iter()
            .find(|spec| spec.action.local_id == local_id)
    {
        return vec![
            ("section", spec.section.to_string()),
            ("keymap", spec.keymap_list.to_string()),
        ];
    }
    if parent_path == ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH
        && let Some(spec) = ZELLIJ_NATIVE_KEYBINDINGS
            .iter()
            .find(|spec| spec.action.local_id == local_id)
    {
        return vec![(
            "mode",
            spec.blocks
                .iter()
                .map(|block| block.mode)
                .collect::<Vec<_>>()
                .join(", "),
        )];
    }
    Vec::new()
}

fn keybinding_surface_title(parent_path: &str) -> &'static str {
    match parent_path {
        ZELLIJ_KEYBINDINGS_FIELD_PATH => "Yazelix Zellij actions",
        ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH => "Yazelix native Zellij policy",
        YAZI_KEYBINDINGS_FIELD_PATH => "Yazelix Yazi actions",
        _ => "Yazelix actions",
    }
}

pub(crate) fn is_keybinding_map_field_path(path: &str) -> bool {
    matches!(
        path,
        ZELLIJ_KEYBINDINGS_FIELD_PATH
            | ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH
            | YAZI_KEYBINDINGS_FIELD_PATH
    )
}

pub(crate) fn keybinding_parent_path_for_field_path(path: &str) -> Option<&'static str> {
    for parent_path in [
        ZELLIJ_KEYBINDINGS_FIELD_PATH,
        ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH,
        YAZI_KEYBINDINGS_FIELD_PATH,
    ] {
        let Some(action) = path
            .strip_prefix(parent_path)
            .and_then(|rest| rest.strip_prefix('.'))
        else {
            continue;
        };
        if keybinding_actions_for_parent_path(parent_path)
            .iter()
            .any(|metadata| metadata.local_id == action)
        {
            return Some(parent_path);
        }
    }
    None
}

pub(crate) fn keybinding_action_metadata_for_field_path(
    path: &str,
) -> Option<&'static YazelixActionMetadata> {
    let parent_path = keybinding_parent_path_for_field_path(path)?;
    let action_id = path.strip_prefix(parent_path)?.strip_prefix('.')?;
    keybinding_actions_for_parent_path(parent_path)
        .into_iter()
        .find(|metadata| metadata.local_id == action_id)
}

fn apply_contract_path_for_setting_path(setting_path: &str) -> &str {
    keybinding_parent_path_for_field_path(setting_path).unwrap_or(setting_path)
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
            "Fix permissions for ~/.config/yazelix_ghostty_cursors/settings.jsonc, then retry.",
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
            "Reinstall Yazelix so the runtime includes yazelix_ghostty_cursors_default.toml.",
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

fn map_native_statuses(statuses: &[NativeConfigStatusEntry]) -> Vec<ConfigUiNativeStatus> {
    statuses
        .iter()
        .map(|status| ConfigUiNativeStatus {
            surface: status.surface.clone(),
            tool: status.tool.clone(),
            description: status.description.clone(),
            status: status.status.clone(),
            label: status.label.clone(),
            severity: status_code_for_entry(status)
                .map(|code| code.doctor_severity())
                .unwrap_or("info")
                .to_string(),
            active_path: status.active_path.clone(),
            managed_path: status.managed_path.clone(),
            native_paths: status.native_paths.clone(),
            generated_path: status.generated_path.clone(),
            allowed_action: status.allowed_action.clone(),
            read_only_reason: status.read_only_reason.clone(),
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

    if kind == "array"
        && let Some(items) = schema.get("items")
        && items.get("type").and_then(JsonValue::as_str) == Some("string")
    {
        out.push(schema_field(schema, path, "string_list".to_string()));
        return;
    }

    out.push(schema_field(schema, path, kind));
}

fn schema_field(schema: &JsonValue, path: &str, kind: String) -> SchemaField {
    let allowed_values = if kind == "string_list" {
        schema
            .get("items")
            .map(schema_enum_values)
            .unwrap_or_default()
    } else {
        schema_enum_values(schema)
    };
    SchemaField {
        path: path.to_string(),
        kind,
        allowed_values,
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

fn enrich_cursor_schema_field(field: &mut SchemaField, values: &CursorChoiceValues) {
    match field.path.as_str() {
        "cursors.enabled_cursors" => {
            field.allowed_values = values.definition_names.clone();
        }
        "cursors.settings.trail" => {
            field.allowed_values = vec!["none".to_string(), "random".to_string()];
            field.allowed_values.extend(values.enabled_names.clone());
        }
        _ => {}
    }
}

fn cursor_choice_values(active: &JsonValue, default: &JsonValue) -> CursorChoiceValues {
    let definition_names = cursor_definition_names(active, default);
    let enabled_names = cursor_enabled_names(active, default)
        .into_iter()
        .filter(|name| definition_names.iter().any(|definition| definition == name))
        .collect();
    CursorChoiceValues {
        definition_names,
        enabled_names,
    }
}

fn cursor_definition_names(active: &JsonValue, default: &JsonValue) -> Vec<String> {
    let definitions = get_json_path(active, "cursors.cursor")
        .and_then(JsonValue::as_array)
        .filter(|values| !values.is_empty())
        .or_else(|| {
            get_json_path(default, "cursors.cursor")
                .and_then(JsonValue::as_array)
                .filter(|values| !values.is_empty())
        });
    let Some(definitions) = definitions else {
        return Vec::new();
    };
    definitions
        .iter()
        .filter_map(|definition| definition.get("name").and_then(JsonValue::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn cursor_enabled_names(active: &JsonValue, default: &JsonValue) -> Vec<String> {
    let enabled = get_json_path(active, "cursors.enabled_cursors")
        .and_then(JsonValue::as_array)
        .filter(|values| !values.is_empty())
        .or_else(|| {
            get_json_path(default, "cursors.enabled_cursors")
                .and_then(JsonValue::as_array)
                .filter(|values| !values.is_empty())
        });
    let Some(enabled) = enabled else {
        return Vec::new();
    };
    enabled
        .iter()
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
    edit_behavior: ConfigUiEditBehavior,
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
        apply_status: apply_status_for_setting(path, apply_mode),
        edit_behavior,
    }
}

fn edit_behavior_for_field_path(path: &str) -> ConfigUiEditBehavior {
    if is_keybinding_map_field_path(path) {
        return ConfigUiEditBehavior::StructuredOnly {
            notice: "Select an action row below to edit one binding list.".to_string(),
        };
    }
    if path == "cursors.cursor" {
        return ConfigUiEditBehavior::StructuredOnly {
            notice:
                "Cursor registry definitions are edited in the source file; run `yzx edit cursors`."
                    .to_string(),
        };
    }
    ConfigUiEditBehavior::Default
}

fn apply_status_for_setting(path: &str, apply_mode: RuntimeApplyMode) -> ConfigUiApplyStatus {
    let (summary, detail, pending) = match apply_mode {
        RuntimeApplyMode::Live => ("now", "Saved changes are active immediately.", false),
        RuntimeApplyMode::LiveWithPaneRefresh => (
            "now",
            "Yazelix reloads this in the active pane owner when you save.",
            false,
        ),
        RuntimeApplyMode::GeneratedRuntimeRefresh => generated_runtime_effect_status(path),
        RuntimeApplyMode::TabSessionRestart => (
            "after Yazelix restart",
            "Saved changes are read from the launch snapshot when Yazelix starts.",
            true,
        ),
        RuntimeApplyMode::ShellTerminalRestart => (
            "after Yazelix restart",
            "Saved changes affect the shell or terminal environment that Yazelix starts with.",
            true,
        ),
        RuntimeApplyMode::PackageHomeManagerActivation => (
            "after Home Manager switch",
            "Edit the Home Manager source and run home-manager switch before Yazelix can use this value.",
            true,
        ),
        RuntimeApplyMode::NeverLive => (
            "not applicable",
            "This setting is an ownership boundary and is not live-applicable.",
            true,
        ),
    };
    ConfigUiApplyStatus {
        summary: summary.to_string(),
        label: summary.to_string(),
        detail: detail.to_string(),
        pending,
    }
}

fn generated_runtime_effect_status(path: &str) -> (&'static str, &'static str, bool) {
    if path.starts_with("yazi.") || path.starts_with("helix.") {
        (
            "after pane reopen",
            "Yazelix regenerates managed config; reopen the affected pane to use it.",
            true,
        )
    } else {
        (
            "after Yazelix restart",
            "Yazelix regenerates managed config; restart Yazelix to use it.",
            true,
        )
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
        name: "yazelix_ghostty_cursors/settings.jsonc".to_string(),
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

fn read_settings_for_edit(path: &Path) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_settings_jsonc_for_edit",
            "Could not read Yazelix settings.jsonc for editing",
            "Fix permissions or restore the settings file, then retry.",
            path.display().to_string(),
            source,
        )
    })
}

fn default_main_settings_text_for_ui(request: &ConfigUiRequest) -> Result<String, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    render_default_settings_jsonc(&paths.default_config_path)
}

fn default_main_setting_value_for_ui(
    request: &ConfigUiRequest,
    path: &str,
) -> Result<JsonValue, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let defaults = read_settings_jsonc_value(&paths.default_config_path)?;
    get_json_path(&defaults, path).cloned().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Usage,
            "unsupported_settings_path",
            format!("Cannot reset {path} because it is not part of the canonical main settings defaults."),
            "Use a supported settings.jsonc path from the Yazelix config contract.",
            json!({ "path": path }),
        )
    })
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
            apply_status: apply_status_for_setting(path, RuntimeApplyMode::TabSessionRestart),
            edit_behavior: ConfigUiEditBehavior::Default,
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
            runtime.join("settings_default.jsonc"),
            include_str!("../../../settings_default.jsonc"),
        )
        .expect("main defaults");
        fs::write(
            runtime.join(DEFAULT_CURSOR_CONFIG_FILENAME),
            include_str!("../../../yazelix_ghostty_cursors_default.toml"),
        )
        .expect("cursor defaults");
        fs::write(
            runtime.join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": true, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .expect("runtime component manifest");
    }

    fn test_request(runtime: &Path, config: &Path) -> ConfigUiRequest {
        ConfigUiRequest {
            runtime_dir: runtime.to_path_buf(),
            config_dir: config.to_path_buf(),
            config_override: None,
        }
    }

    fn write_main_settings(
        runtime: &Path,
        config: &Path,
        mutate: impl FnOnce(&mut JsonValue),
    ) -> PathBuf {
        write_main_settings_with_prefix(runtime, config, "", mutate)
    }

    fn write_main_settings_with_prefix(
        runtime: &Path,
        config: &Path,
        prefix: &str,
        mutate: impl FnOnce(&mut JsonValue),
    ) -> PathBuf {
        let mut value = read_settings_jsonc_value(&runtime.join("settings_default.jsonc"))
            .expect("default settings");
        mutate(&mut value);
        let path = config.join("settings.jsonc");
        fs::create_dir_all(config).expect("config dir");
        fs::write(
            &path,
            format!(
                "{}{}\n",
                prefix,
                serde_json::to_string_pretty(&value).expect("settings json")
            ),
        )
        .expect("settings");
        path
    }

    fn model_field<'a>(model: &'a ConfigUiModel, path: &str) -> &'a ConfigUiField {
        model
            .fields
            .iter()
            .find(|field| field.path == path)
            .expect("field")
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
    #[test]
    fn list_fields_edit_from_full_json_not_display_summary() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let field = model_field(&model, "zellij.widget_tray");

        assert_eq!(field.current_value, "[7 items]");
        assert_eq!(field.apply_status.summary, "after Yazelix restart");
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

    // Defends: config UI does not expose cursor editor fields when the packaged runtime disables the cursor component.
    #[test]
    fn disabled_cursor_component_removes_cursor_editor_fields() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        fs::write(
            runtime.path().join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": false, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .expect("runtime component manifest");
        fs::remove_file(runtime.path().join(DEFAULT_CURSOR_CONFIG_FILENAME))
            .expect("remove cursor defaults");

        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");

        assert!(!model.tabs.contains(&"cursors".to_string()));
        assert!(
            model
                .fields
                .iter()
                .all(|field| !field.path.starts_with("cursors."))
        );
    }

    // Defends: the keybinding tab renders Yazelix action registry labels, scoped ids, defaults, remaps, and disabled actions instead of an opaque JSON object.
    #[test]
    fn zellij_keybinding_details_use_action_registry_metadata() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        write_main_settings(runtime.path(), config.path(), |settings| {
            settings["zellij"]["keybindings"]["popup"] = json!(["Alt x"]);
            settings["zellij"]["keybindings"]["menu"] = json!([]);
            settings["zellij"]["keybindings"]["unknown_action"] = json!(["Alt z"]);
        });
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

        select_field_path(&mut app, "zellij.keybindings");
        let details = lines_text(&app.render_details(app.visible_rows()[app.selected_row]));

        assert!(details.contains("Toggle the managed popup program"));
        assert!(details.contains("zellij.popup"));
        assert!(details.contains("Alt x (remapped)"));
        assert!(details.contains("Alt Shift J"));
        assert!(details.contains("Alt Shift K"));
        assert!(details.contains("Open the Yazelix command palette popup"));
        assert!(details.contains("disabled (disabled)"));
        assert!(details.contains("empty list disables this action"));
        assert!(details.contains("unsupported"));
        assert!(details.contains("unknown_action"));
    }

    // Defends: native Zellij policy keybindings use the same structured action-row editor as semantic Yazelix bindings.
    #[test]
    fn zellij_native_keybinding_details_use_policy_registry_metadata() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        write_main_settings(runtime.path(), config.path(), |settings| {
            settings["zellij"]["native_keybindings"]["scroll_mode"] = json!(["Ctrl Alt x"]);
            settings["zellij"]["native_keybindings"]["scroll_mode_unbind"] = json!([]);
            settings["zellij"]["native_keybindings"]["unknown_policy"] = json!(["Alt z"]);
        });
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

        select_field_path(&mut app, "zellij.native_keybindings");
        let details = lines_text(&app.render_details(app.visible_rows()[app.selected_row]));

        assert!(details.contains("Yazelix native Zellij policy"));
        assert!(details.contains("Toggle scroll mode"));
        assert!(details.contains("Ctrl Alt x (remapped)"));
        assert!(details.contains("Ctrl Alt s"));
        assert!(details.contains("Unbind default scroll-mode key"));
        assert!(details.contains("disabled (disabled)"));
        assert!(details.contains("unsupported"));
        assert!(details.contains("unknown_policy"));
    }

    // Regression: keybinding map parents are structured overviews; pressing Enter must not open the whole map as one raw JSON editing line.
    #[test]
    fn keybinding_map_parent_does_not_open_raw_object_editor() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

        select_field_path(&mut app, "zellij.keybindings");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.edit.is_none());
        assert_eq!(
            app.notice.as_ref().expect("notice").text,
            "Select an action row below to edit one binding list."
        );
    }

    // Defends: complex array/object fields without a dedicated structured editor do not fall back to an unreadable one-line JSON editor.
    #[test]
    fn complex_registry_field_does_not_open_raw_array_editor() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

        select_field_path(&mut app, "cursors.cursor");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.edit.is_none());
        assert_eq!(
            app.notice.as_ref().expect("notice").text,
            "Cursor registry definitions are edited in the source file; run `yzx edit cursors`."
        );
    }

    // Defends: cursor preset selection is a picker-backed string list, not a rejected generic JSON array.
    #[test]
    fn cursor_enabled_cursors_opens_multi_choice_picker_and_writes_cursor_config() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let cursor_path = crate::user_config_paths::shared_cursor_config(config.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let field = model_field(&model, "cursors.enabled_cursors");

        assert_eq!(field.kind, "string_list");
        assert!(field.allowed_values.contains(&"blaze".to_string()));
        assert!(field.allowed_values.contains(&"snow".to_string()));

        let mut app = YazelixConfigUiApp::new(request, model);
        select_field_path(&mut app, "cursors.enabled_cursors");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let edit = app.edit.clone().expect("edit");
        assert_eq!(edit.mode, ConfigUiEditMode::MultiChoice);
        let details = lines_text(&app.render_details(UiRowRef::Field(edit.field_index)));
        assert!(details.contains("> [x] blaze"));

        app.handle_edit_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        app.handle_edit_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.edit.is_none());
        let value = read_settings_jsonc_value(&cursor_path).expect("cursor settings jsonc");
        let enabled = get_json_path(&value, "enabled_cursors")
            .and_then(JsonValue::as_array)
            .expect("enabled cursors");
        assert!(!enabled.iter().any(|value| value.as_str() == Some("blaze")));
        assert!(enabled.iter().any(|value| value.as_str() == Some("snow")));
    }

    // Defends: dynamic cursor trail selection is a single-select picker over none, random, and enabled cursor names.
    #[test]
    fn cursor_trail_uses_dynamic_single_choice_picker() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let field = model_field(&model, "cursors.settings.trail");

        assert_eq!(field.kind, "string");
        assert_eq!(field.allowed_values[0], "none");
        assert_eq!(field.allowed_values[1], "random");
        assert!(field.allowed_values.contains(&"blaze".to_string()));

        let mut app = YazelixConfigUiApp::new(request, model);
        select_field_path(&mut app, "cursors.settings.trail");
        let details = lines_text(&app.render_details(app.visible_rows()[app.selected_row]));
        assert!(details.contains("  ( ) none"));
        assert!(details.contains("  (x) random"));
        assert!(!details.contains("> (x) random"));

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let edit = app.edit.clone().expect("edit");
        assert_eq!(edit.mode, ConfigUiEditMode::Choice);
        let details = lines_text(&app.render_details(UiRowRef::Field(edit.field_index)));
        assert!(details.contains("  ( ) none"));
        assert!(details.contains("> (x) random"));
    }

    // Defends: keybinding actions are editable as one semantic action row with friendly key-list input instead of forcing a full object edit.
    #[test]
    fn keybinding_action_row_writes_single_binding_list() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        write_main_settings(runtime.path(), config.path(), |settings| {
            settings["zellij"]["keybindings"]["popup"] = json!(["Alt x"]);
        });
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

        select_field_path(&mut app, "zellij.keybindings.popup");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let edit = app.edit.as_mut().expect("edit");
        assert_eq!(edit.mode, ConfigUiEditMode::Text);
        assert_eq!(edit.input, "Alt x");
        edit.input = "Alt Shift X".to_string();
        app.handle_edit_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.edit.is_none());
        let value = read_settings_jsonc_value(&settings_path).expect("settings jsonc");
        assert_eq!(
            get_json_path(&value, "zellij.keybindings.popup"),
            Some(&json!(["Alt Shift X"]))
        );
    }

    // Defends: the same structured keybinding map treatment covers Yazi actions instead of leaving a second raw object editor in the keybindings tab.
    #[test]
    fn yazi_keybinding_details_use_action_registry_metadata() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        write_main_settings(runtime.path(), config.path(), |settings| {
            settings["yazi"]["keybindings"]["open_zoxide_in_editor"] = json!([]);
        });
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

        select_field_path(&mut app, "yazi.keybindings");
        let details = lines_text(&app.render_details(app.visible_rows()[app.selected_row]));

        assert!(details.contains("Yazelix Yazi actions"));
        assert!(details.contains("Retarget the managed editor through the Yazi zoxide picker"));
        assert!(details.contains("yazi.open_zoxide_in_editor"));
        assert!(details.contains("disabled (disabled)"));
        assert!(details.contains("section"));
        assert!(details.contains("keymap"));
    }

    // Defends: machine-readable apply modes from main_config_contract.toml reach clear user-facing takes-effect labels.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn model_exposes_apply_statuses_from_contract() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");

        let screen_saver = model_field(&model, "zellij.screen_saver_enabled");
        assert_eq!(screen_saver.apply_status.summary, "now");
        assert!(!screen_saver.apply_status.pending);
        assert!(
            screen_saver
                .apply_status
                .detail
                .contains("active pane owner")
        );

        let editor_command = model_field(&model, "editor.command");
        assert_eq!(editor_command.apply_status.summary, "after Yazelix restart");

        let terminal_config_mode = model_field(&model, "terminal.config_mode");
        assert_eq!(
            terminal_config_mode.apply_status.summary,
            "after Yazelix restart"
        );

        let widget_tray = model_field(&model, "zellij.widget_tray");
        assert_eq!(widget_tray.apply_status.summary, "after Yazelix restart");
        assert!(
            widget_tray
                .apply_status
                .detail
                .contains("regenerates managed config")
        );

        let popup_width = model_field(&model, "zellij.popup_width_percent");
        assert_eq!(popup_width.apply_status.summary, "after Yazelix restart");

        let yazi_theme = model_field(&model, "yazi.theme");
        assert_eq!(yazi_theme.apply_status.summary, "after pane reopen");
    }

    // Defends: Home Manager-owned settings are presented as activation-scoped even when the field's intrinsic apply mode is narrower.
    #[cfg(unix)]
    #[test]
    fn home_manager_owned_settings_use_activation_apply_mode() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let hm_dir = config.path().join("profile-home-manager-files");
        fs::create_dir_all(&hm_dir).expect("home manager dir");
        let hm_settings = hm_dir.join("settings.jsonc");
        fs::write(
            &hm_settings,
            render_default_settings_jsonc(&runtime.path().join("settings_default.jsonc")).unwrap(),
        )
        .expect("home manager settings");
        std::os::unix::fs::symlink(&hm_settings, config.path().join("settings.jsonc"))
            .expect("settings symlink");

        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let popup_width = model_field(&model, "zellij.popup_width_percent");

        assert_eq!(model.config_owner, ConfigUiPathOwner::HomeManager);
        assert_eq!(
            popup_width.apply_status.summary,
            "after Home Manager switch"
        );
        assert!(
            popup_width
                .apply_status
                .detail
                .contains("home-manager switch")
        );
    }

    // Defends: the config UI consumes the shared native-config status labels instead of maintaining separate sidecar wording.
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
    #[test]
    fn enum_string_list_picker_toggles_subvalues_with_space() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

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

    // Defends: bools keep direct choice edits while scalar enums use the single-select picker mode.
    #[test]
    fn edit_helpers_use_choice_modes_for_bool_and_enum() {
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
    }

    // Defends: bool edits stay direct controls while enum edit mode behaves like a single-select picker.
    #[test]
    fn choice_edit_keys_toggle_bool_and_move_enum_picker() {
        let request = ConfigUiRequest {
            runtime_dir: PathBuf::from("/runtime"),
            config_dir: PathBuf::from("/home/lucca/.config/yazelix"),
            config_override: None,
        };
        let model = ConfigUiModel {
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
        };
        let mut app = YazelixConfigUiApp::new(request, model);
        app.edit = Some(ConfigUiEditState {
            field_index: 0,
            input: "true".to_string(),
            mode: ConfigUiEditMode::Choice,
            choice_index: 0,
        });

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
    #[test]
    fn scalar_enum_enter_opens_single_select_picker() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

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

    // Defends: Space remains a direct toggle for bools, but scalar selects open the picker instead of cycling blindly.
    #[test]
    fn scalar_enum_space_opens_picker_without_writing() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

        select_field_path(&mut app, "terminal.config_mode");
        app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));

        let edit = app.edit.clone().expect("edit");
        assert_eq!(edit.mode, ConfigUiEditMode::Choice);
        let details = lines_text(&app.render_details(UiRowRef::Field(edit.field_index)));
        assert!(details.contains("> (x) yazelix"));
        assert!(!settings_path.exists());
    }

    // Defends: Enter on bool rows performs the direct control action instead of opening an edit session.
    #[test]
    fn enter_directly_applies_bool_field_without_edit_mode() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        write_main_settings(runtime.path(), config.path(), |settings| {
            settings["editor"]["hide_sidebar_on_file_open"] = json!(false);
        });
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

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
    #[test]
    fn write_field_value_patches_settings_jsonc_and_reloads_model() {
        let runtime = tempdir().expect("runtime");
        let config = tempdir().expect("config");
        write_runtime_layout(runtime.path());
        let settings_path = config.path().join("settings.jsonc");
        write_main_settings_with_prefix(
            runtime.path(),
            config.path(),
            "// keep this comment\n",
            |settings| {
                settings["editor"]["hide_sidebar_on_file_open"] = json!(false);
            },
        );
        let request = test_request(runtime.path(), config.path());
        let model = build_config_ui_model(&request).expect("model");
        let mut app = YazelixConfigUiApp::new(request, model);

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
        let field = model_field(&app.model, "editor.hide_sidebar_on_file_open");
        assert_eq!(field.state, ConfigUiValueState::Explicit);
        assert_eq!(field.current_value, "true");
    }

    // Regression: a save-time refresh failure remains visible as pending apply work instead of hiding the fact that the setting was already persisted.
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
