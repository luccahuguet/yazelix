use super::model_builder::active_config_path;
use super::{ConfigUiRequest, apply_contract_path_for_setting_path};
use crate::active_config_surface::primary_config_paths;
use crate::bridge::CoreError;
use crate::config_apply::{
    ConfigEditApplyRequest, ConfigEditApplyStatus, PaneOrchestratorRuntimeRefreshRequest,
    apply_mode_for_setting, apply_status_after_config_edit, runtime_materialization_request,
};
use crate::control_plane::state_dir_from_env;
use crate::runtime_apply_mode::RuntimeApplyMode;

pub(super) fn apply_after_field_write(
    request: &ConfigUiRequest,
    setting_path: &str,
) -> Result<ConfigEditApplyStatus, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let contract_setting_path = apply_contract_path_for_setting_path(setting_path);
    let apply_mode = apply_mode_for_setting(&paths.contract_path, contract_setting_path)?;
    let runtime_materialization = if apply_mode == Some(RuntimeApplyMode::GeneratedRuntimeRefresh) {
        let state_dir = state_dir_from_env()?;
        Some(runtime_materialization_request(
            &request.runtime_dir,
            &request.config_dir,
            request.config_override.as_deref(),
            &state_dir,
        )?)
    } else {
        None
    };
    let pane_orchestrator_refresh = if apply_mode == Some(RuntimeApplyMode::LiveWithPaneRefresh) {
        let state_dir = state_dir_from_env()?;
        Some(PaneOrchestratorRuntimeRefreshRequest {
            config_path: active_config_path(&paths, request.config_override.as_deref()),
            default_config_path: paths.default_config_path.clone(),
            contract_path: paths.contract_path.clone(),
            zellij_config_dir: state_dir.join("configs").join("zellij"),
        })
    } else {
        None
    };
    apply_status_after_config_edit(&ConfigEditApplyRequest {
        setting_path: contract_setting_path.to_string(),
        contract_path: paths.contract_path,
        runtime_materialization,
        pane_orchestrator_refresh,
    })
}
