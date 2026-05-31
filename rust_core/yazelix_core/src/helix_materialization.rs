mod helix_config;
mod import_notice;
mod steel;

#[cfg(test)]
mod tests;

use crate::atomic_fs::write_text_atomic;
use crate::bridge::{CoreError, ErrorClass};
use helix_config::prepare_managed_helix_config;
use import_notice::build_import_notice;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
    fs,
    path::{Path, PathBuf},
};
use steel::{load_steel_plugin_selection, materialize_steel_config};

pub(crate) const MANAGED_REVEAL_COMMAND: &str = ":sh yzx reveal \"%{buffer_name}\"";
pub(crate) const MANAGED_COMMAND_MODE_KEY: &str = ":";
pub(crate) const MANAGED_COMMAND_MODE_COMMAND: &str = "command_mode";
pub(crate) const REVEAL_KEY: &str = "A-r";
pub(crate) const STEEL_CONFIG_MODULE: &str = "helix.scm";
pub(crate) const STEEL_INIT_MODULE: &str = "init.scm";

#[derive(Debug, Clone)]
pub struct HelixMaterializationRequest {
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub show_splash: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelixImportNotice {
    pub marker_path: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SteelCommandMetadata {
    pub name: String,
    pub owner: String,
    pub description: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelixMaterializationData {
    pub generated_path: String,
    pub generated_steel_config_dir: String,
    pub generated_steel_module_path: String,
    pub generated_steel_init_path: String,
    pub template_path: String,
    pub user_config_merged: bool,
    pub reveal_binding_enforced: bool,
    pub enabled_steel_plugins: Vec<String>,
    pub steel_plugin_files: Vec<String>,
    pub steel_commands: Vec<SteelCommandMetadata>,
    pub import_notice: Option<HelixImportNotice>,
}

pub(crate) fn build_managed_helix_contract_json(
    runtime_dir: &Path,
    config_dir: &Path,
) -> Result<JsonValue, CoreError> {
    let prepared = prepare_managed_helix_config(runtime_dir, config_dir)?;
    serde_json::to_value(prepared.config).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_helix_contract_json",
            format!("Could not serialize the expected Helix config contract as JSON: {source}"),
            "This is an internal error. File a bug report if it persists.",
            serde_json::json!({
                "runtime_dir": runtime_dir.to_string_lossy(),
                "config_dir": config_dir.to_string_lossy(),
            }),
        )
    })
}

pub fn generate_helix_materialization(
    request: &HelixMaterializationRequest,
) -> Result<HelixMaterializationData, CoreError> {
    crate::managed_user_config_stubs::ensure_helix_surface_stub(&request.config_dir)?;
    let prepared = prepare_managed_helix_config(&request.runtime_dir, &request.config_dir)?;
    let plugin_selection = load_steel_plugin_selection(&request.runtime_dir, &request.config_dir)?;

    let generated_dir = request.state_dir.join("configs").join("helix");
    fs::create_dir_all(&generated_dir).map_err(|source| {
        CoreError::io(
            "create_helix_output_dir",
            "Could not create the managed Helix output directory",
            "Check permissions for the Yazelix state directory and retry.",
            generated_dir.to_string_lossy(),
            source,
        )
    })?;

    let generated_path = generated_dir.join("config.toml");
    let output = toml::to_string_pretty(&prepared.config).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_helix_config",
            format!("Could not serialize the merged Helix config as TOML: {source}"),
            "This is an internal error. File a bug report if it persists.",
            serde_json::json!({ "path": generated_path.to_string_lossy() }),
        )
    })?;

    write_text_atomic(&generated_path, &output)?;

    let import_notice = build_import_notice(request, &prepared.user_config_path)?;
    let steel = materialize_steel_config(
        &request.runtime_dir,
        &request.config_dir,
        &generated_dir,
        &plugin_selection,
        request.show_splash,
    )?;

    Ok(HelixMaterializationData {
        generated_path: generated_path.to_string_lossy().into_owned(),
        generated_steel_config_dir: steel.config_dir.to_string_lossy().into_owned(),
        generated_steel_module_path: steel.helix_module_path.to_string_lossy().into_owned(),
        generated_steel_init_path: steel.init_path.to_string_lossy().into_owned(),
        template_path: prepared.template_path.to_string_lossy().into_owned(),
        user_config_merged: prepared.user_config_merged,
        reveal_binding_enforced: true,
        enabled_steel_plugins: steel.enabled_plugins,
        steel_plugin_files: steel.copied_plugin_files,
        steel_commands: steel.commands,
        import_notice,
    })
}
