//! Terminal UI for inspecting and editing the canonical Yazelix config surface.

mod app;
mod apply_adapter;
mod details;
mod model_builder;

use crate::active_config_surface::{PrimaryConfigPaths, primary_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{ConfigDiagnostic, ConfigDiagnosticReport, NormalizeConfigRequest};
use crate::control_plane::{home_dir_from_env, state_dir_from_env};
use crate::native_config_status::{
    NativeConfigStatusEntry, NativeConfigStatusRequest, classify_native_config_statuses,
    path_owned_by_home_manager, status_code_for_entry, xdg_config_home_from_env,
};
use crate::runtime_apply_mode::RuntimeApplyMode;
use crate::runtime_component_enabled;
use crate::settings_surface::{
    HOME_MANAGER_CURSORS_REMEDIATION, HOME_MANAGER_MARS_REMEDIATION,
    HOME_MANAGER_SETTINGS_REMEDIATION,
};
use crate::settings_surface::{SETTINGS_SCHEMA_FILENAME, render_default_config};
use crate::settings_surface::{
    is_settings_config_path, parse_config_value, read_config_value,
    sparse_config_is_semantically_empty,
};
use crate::user_config_paths::{self, CURRENT_MANAGED_CONFIG_FILE_NAMES, SETTINGS_CONFIG};
use ratatui::text::Line;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, IsTerminal};
#[cfg(test)]
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use yazelix_cursors::CursorRegistry;

pub use app::run_config_ui;
use apply_adapter::apply_after_field_write;
use details::render_details;
pub use model_builder::build_config_ui_model;
use model_builder::{
    classify_path_owner, path_is_read_only, path_present, read_settings_for_edit,
    validate_patched_settings_for_ui, write_settings_edit,
};
use ratconfig::patch::{PatchMutation, PatchOutcome};
pub use ratconfig::{
    ConfigUiApp, ConfigUiApplyStatus, ConfigUiContractField, ConfigUiDiagnostic,
    ConfigUiEditBehavior, ConfigUiEditMode, ConfigUiField, ConfigUiFieldMetadata,
    ConfigUiFieldSpec, ConfigUiFileAction, ConfigUiIntent, ConfigUiKey, ConfigUiMetadata,
    ConfigUiModel, ConfigUiNativeStatus, ConfigUiPathOwner, ConfigUiSchemaField, ConfigUiSidecar,
    ConfigUiSource, ConfigUiTomlDocumentSpec, ConfigUiValueState, DEFAULT_CONFIG_SOURCE_ID,
    UiRowRef,
};
use ratconfig::{
    CrosstermRunnerError, build_toml_document_fields, collect_config_ui_schema_fields,
    config_contract_fields_from_toml, config_ui_metadata_from_toml, default_field_detail_lines,
    diagnostic_detail_lines, file_action_detail_lines, get_json_path, is_scalar_enum_field,
    multi_choice_detail_lines, native_status_detail_lines,
    run_config_ui_with_details as run_ratconfig_config_ui_with_details, schema_tabs,
    sidecar_detail_lines, single_choice_detail_lines, single_choice_field_detail_lines, tab_index,
    toml_value_to_json,
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
const SETTINGS_SOURCE_ID: &str = DEFAULT_CONFIG_SOURCE_ID;
const CURSORS_SOURCE_ID: &str = "cursors";
const CURSORS_FIELD_PREFIX: &str = "cursors.";
const CURSORS_CONFIG_ACTION_ID: &str = "cursors.config";
const MARS_SOURCE_ID: &str = "mars";
const MARS_TAB: &str = "terminal";
const MARS_CONFIG_ACTION_ID: &str = "mars.config";

#[derive(Debug, Clone)]
pub struct ConfigUiRequest {
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_override: Option<String>,
}

#[cfg(test)]
pub(crate) struct YazelixConfigUiApp {
    pub(crate) request: ConfigUiRequest,
    pub(crate) ui: ConfigUiApp,
}

#[derive(Debug)]
struct ConfigUiWriteOutcome {
    mutation: PatchMutation,
    apply_notice: Option<String>,
}

struct ConfigUiEditTarget {
    path: PathBuf,
    path_in_file: String,
    kind: ConfigUiEditTargetKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigUiEditTargetKind {
    Main,
    Cursors,
    Mars,
}

#[cfg(test)]
impl Deref for YazelixConfigUiApp {
    type Target = ConfigUiApp;

    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

#[cfg(test)]
impl DerefMut for YazelixConfigUiApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui
    }
}

#[cfg(test)]
mod tests;
