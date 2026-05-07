use crate::native_config_status::NativeConfigStatusEntry;
use crate::runtime_apply_mode::RuntimeApplyMode;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigUiRequest {
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_override: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiModel {
    pub active_config_path: PathBuf,
    pub cursor_config_path: PathBuf,
    pub default_cursor_config_path: PathBuf,
    pub active_config_exists: bool,
    pub config_owner: ConfigUiPathOwner,
    pub config_read_only: bool,
    pub tabs: Vec<String>,
    pub fields: Vec<ConfigUiField>,
    pub sidecars: Vec<ConfigUiSidecar>,
    pub native_config_statuses: Vec<NativeConfigStatusEntry>,
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
    pub(crate) edit_value: String,
    pub default_value: String,
    pub state: ConfigUiValueState,
    pub description: String,
    pub allowed_values: Vec<String>,
    pub validation: String,
    pub rebuild_required: bool,
    pub apply_mode: RuntimeApplyMode,
    pub apply_status: ConfigUiApplyStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiApplyStatus {
    pub summary: String,
    pub label: String,
    pub detail: String,
    pub pending: bool,
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

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the extracted config UI model can represent a non-Yazelix field namespace before a future ratconfig crate split.
    #[test]
    fn model_accepts_non_yazelix_field_namespace() {
        let model = ConfigUiModel {
            active_config_path: PathBuf::from("/tmp/ratconfig_demo/settings.jsonc"),
            cursor_config_path: PathBuf::from("/tmp/ratconfig_demo/cursors.jsonc"),
            default_cursor_config_path: PathBuf::from("/tmp/ratconfig_demo/default_cursors.jsonc"),
            active_config_exists: true,
            config_owner: ConfigUiPathOwner::User,
            config_read_only: false,
            tabs: vec!["network".to_string()],
            fields: vec![ConfigUiField {
                path: "network.timeout_seconds".to_string(),
                tab: "network".to_string(),
                kind: "int".to_string(),
                current_value: "30".to_string(),
                edit_value: "30".to_string(),
                default_value: "15".to_string(),
                state: ConfigUiValueState::Explicit,
                description: "Request timeout".to_string(),
                allowed_values: Vec::new(),
                validation: "1..300".to_string(),
                rebuild_required: false,
                apply_mode: RuntimeApplyMode::Live,
                apply_status: ConfigUiApplyStatus {
                    summary: "live".to_string(),
                    label: "Applies now".to_string(),
                    detail: "Active after save".to_string(),
                    pending: false,
                },
            }],
            sidecars: Vec::new(),
            native_config_statuses: Vec::new(),
            diagnostics: Vec::new(),
        };

        assert_eq!(model.tabs, vec!["network"]);
        assert_eq!(model.fields[0].path, "network.timeout_seconds");
        assert_eq!(model.fields[0].apply_mode, RuntimeApplyMode::Live);
    }
}
