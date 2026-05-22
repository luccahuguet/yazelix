//! Terminal UI for inspecting and editing the canonical Yazelix config surface.

mod app;
mod apply_adapter;
mod details;
mod keybindings;
mod model_builder;

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
pub use app::run_config_ui;
#[cfg(test)]
use app::write_notice_text;
use apply_adapter::apply_after_field_write;
use keybindings::*;
#[cfg(test)]
use model_builder::apply_status_for_setting;
pub use model_builder::build_config_ui_model;
use model_builder::{
    apply_contract_path_for_setting_path, build_field_row, classify_path_owner,
    default_main_setting_value_for_ui, default_main_settings_text_for_ui, path_is_read_only,
    path_present, read_settings_for_edit, validate_patched_settings_for_ui, write_settings_edit,
};

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

#[cfg(test)]
mod tests;
