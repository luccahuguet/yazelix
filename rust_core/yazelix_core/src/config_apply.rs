//! Apply timing for semantic Yazelix config edits.

use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_apply_mode::RuntimeApplyMode;
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEditApplyStatus {
    pub apply_mode: RuntimeApplyMode,
}

#[derive(Debug, Clone)]
pub struct ConfigEditApplyRequest {
    pub setting_path: String,
    pub contract_path: PathBuf,
}

pub fn apply_status_after_config_edit(
    request: &ConfigEditApplyRequest,
) -> Result<ConfigEditApplyStatus, CoreError> {
    let apply_mode = apply_mode_for_setting(&request.contract_path, &request.setting_path)?
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Internal,
                "missing_config_apply_mode",
                format!(
                    "Saved {}, but the config contract has no activation timing for it.",
                    request.setting_path
                ),
                "Report this as a Yazelix config-contract error.",
                json!({ "setting": request.setting_path }),
            )
        })?;
    Ok(ConfigEditApplyStatus { apply_mode })
}

pub fn apply_mode_for_setting(
    contract_path: &Path,
    setting_path: &str,
) -> Result<Option<RuntimeApplyMode>, CoreError> {
    let raw = std::fs::read_to_string(contract_path).map_err(|source| {
        CoreError::io(
            "read_config_contract_for_apply",
            "Could not read the Yazelix config contract for apply-mode handling",
            "Reinstall Yazelix so config_metadata/main_config_contract.toml is available.",
            contract_path.to_string_lossy(),
            source,
        )
    })?;
    let root = toml::from_str::<toml::Value>(&raw).map_err(|source| {
        CoreError::toml(
            "parse_config_contract_for_apply",
            "Could not parse the Yazelix config contract for apply-mode handling",
            "Fix config_metadata/main_config_contract.toml, then retry.",
            contract_path.to_string_lossy(),
            source,
        )
    })?;
    let Some(field) = root
        .get("fields")
        .and_then(toml::Value::as_table)
        .and_then(|fields| fields.get(setting_path))
        .and_then(toml::Value::as_table)
    else {
        return Ok(match setting_path {
            path if path == "popups" || path.starts_with("popups.") => {
                Some(RuntimeApplyMode::TabSessionRestart)
            }
            path if path.starts_with("cursors.") => Some(RuntimeApplyMode::ShellTerminalRestart),
            _ => None,
        });
    };
    let Some(raw_mode) = field.get("apply_mode").and_then(toml::Value::as_str) else {
        return Ok(None);
    };
    raw_mode
        .parse::<RuntimeApplyMode>()
        .map(Some)
        .map_err(|message| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_config_apply_mode",
                format!("Config contract field {setting_path} has invalid apply_mode."),
                "Fix config_metadata/main_config_contract.toml, then retry.",
                json!({
                    "path": contract_path.to_string_lossy(),
                    "field": setting_path,
                    "apply_mode": raw_mode,
                    "error": message,
                }),
            )
        })
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: dynamic popup and cursor writes report the same next-launch timing as their runtime consumers.
    #[test]
    fn dynamic_config_namespaces_have_explicit_apply_modes() {
        let temp = tempfile::tempdir().unwrap();
        let contract = temp.path().join("contract.toml");
        std::fs::write(&contract, "[fields]\n").unwrap();

        assert_eq!(
            apply_mode_for_setting(&contract, "popups.logs.command").unwrap(),
            Some(RuntimeApplyMode::TabSessionRestart)
        );
        assert_eq!(
            apply_mode_for_setting(&contract, "cursors.settings.trail").unwrap(),
            Some(RuntimeApplyMode::ShellTerminalRestart)
        );
        assert_eq!(apply_mode_for_setting(&contract, "unknown").unwrap(), None);
        let error = apply_status_after_config_edit(&ConfigEditApplyRequest {
            setting_path: "unknown".to_string(),
            contract_path: contract,
        })
        .unwrap_err();
        assert_eq!(error.code(), "missing_config_apply_mode");
    }
}
