use super::ConfigUiRequest;
use crate::active_config_surface::primary_config_paths;
use crate::bridge::CoreError;
use crate::config_apply::{
    ConfigEditApplyRequest, ConfigEditApplyStatus, apply_status_after_config_edit,
};

pub(super) fn apply_after_field_write(
    request: &ConfigUiRequest,
    setting_path: &str,
) -> Result<ConfigEditApplyStatus, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    apply_status_after_config_edit(&ConfigEditApplyRequest {
        setting_path: setting_path.to_string(),
        contract_path: paths.contract_path,
    })
}
