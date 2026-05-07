// Test lane: default
//! Save-time apply handling for semantic Yazelix config edits.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_apply_mode::RuntimeApplyMode;
use crate::runtime_materialization::{
    RuntimeMaterializationPlanRequest, materialize_runtime_state,
};
use serde::Serialize;
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedRuntimeTool {
    Yazi,
    Zellij,
    Helix,
}

impl GeneratedRuntimeTool {
    pub fn label(self) -> &'static str {
        match self {
            GeneratedRuntimeTool::Yazi => "Yazi",
            GeneratedRuntimeTool::Zellij => "Zellij",
            GeneratedRuntimeTool::Helix => "Helix",
        }
    }

    pub fn restart_guidance(self) -> &'static str {
        match self {
            GeneratedRuntimeTool::Yazi => "restart or reopen the affected Yazi/sidebar pane",
            GeneratedRuntimeTool::Zellij => "restart this Yazelix tab/session",
            GeneratedRuntimeTool::Helix => "restart or reopen the affected Helix editor pane",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedConfigRefreshStatus {
    pub tools: Vec<GeneratedRuntimeTool>,
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEditApplyStatus {
    pub setting_path: String,
    pub apply_mode: RuntimeApplyMode,
    pub generated_refresh: Option<GeneratedConfigRefreshStatus>,
}

#[derive(Debug, Clone)]
pub struct ConfigEditApplyRequest {
    pub setting_path: String,
    pub contract_path: PathBuf,
    pub runtime_materialization: Option<RuntimeMaterializationPlanRequest>,
}

pub fn apply_status_after_config_edit(
    request: &ConfigEditApplyRequest,
) -> Result<ConfigEditApplyStatus, CoreError> {
    let apply_mode = apply_mode_for_setting(&request.contract_path, &request.setting_path)?
        .unwrap_or(RuntimeApplyMode::NeverLive);

    let generated_refresh = if apply_mode == RuntimeApplyMode::GeneratedRuntimeRefresh {
        Some(refresh_generated_runtime_config(request)?)
    } else {
        None
    };

    Ok(ConfigEditApplyStatus {
        setting_path: request.setting_path.clone(),
        apply_mode,
        generated_refresh,
    })
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
        return Ok(None);
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

pub fn generated_runtime_tools_for_setting(setting_path: &str) -> Vec<GeneratedRuntimeTool> {
    if setting_path.starts_with("yazi.") {
        vec![GeneratedRuntimeTool::Yazi]
    } else if setting_path.starts_with("zellij.") {
        vec![GeneratedRuntimeTool::Zellij]
    } else if setting_path.starts_with("helix.") {
        vec![GeneratedRuntimeTool::Helix]
    } else {
        vec![GeneratedRuntimeTool::Yazi, GeneratedRuntimeTool::Zellij]
    }
}

pub fn runtime_materialization_request(
    runtime_dir: &Path,
    config_dir: &Path,
    config_override: Option<&str>,
    state_dir: &Path,
) -> Result<RuntimeMaterializationPlanRequest, CoreError> {
    let paths = resolve_active_config_paths(runtime_dir, config_dir, config_override)?;
    let zellij_config_dir = state_dir.join("configs").join("zellij");
    Ok(RuntimeMaterializationPlanRequest {
        config_path: paths.config_file,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        runtime_dir: runtime_dir.to_path_buf(),
        state_path: state_dir.join("state").join("rebuild_hash"),
        yazi_config_dir: state_dir.join("configs").join("yazi"),
        zellij_config_dir: zellij_config_dir.clone(),
        zellij_layout_dir: zellij_config_dir.join("layouts"),
        layout_override: None,
    })
}

fn refresh_generated_runtime_config(
    request: &ConfigEditApplyRequest,
) -> Result<GeneratedConfigRefreshStatus, CoreError> {
    let tools = generated_runtime_tools_for_setting(&request.setting_path);
    let runtime_materialization = request.runtime_materialization.as_ref().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "missing_generated_config_refresh_request",
            format!(
                "Cannot refresh generated runtime config for {} without a materialization request.",
                request.setting_path
            ),
            "Report this as a Yazelix internal error.",
            json!({ "setting": request.setting_path }),
        )
    })?;
    materialize_runtime_state(runtime_materialization).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "generated_config_refresh_failed",
            format!(
                "Saved {}, but Yazelix could not refresh generated runtime config.",
                request.setting_path
            ),
            "Run `yzx doctor --fix` or restart Yazelix after fixing the underlying materialization error.",
            json!({
                "setting": request.setting_path,
                "tools": tools,
                "source_code": source.code(),
                "source_class": source.class().as_str(),
                "source_message": source.message(),
                "source_remediation": source.remediation(),
                "source_details": source.details(),
            }),
        )
    })?;
    Ok(refresh_status_for_tools(&tools))
}

pub fn refresh_status_for_tools(tools: &[GeneratedRuntimeTool]) -> GeneratedConfigRefreshStatus {
    let labels = tools
        .iter()
        .copied()
        .map(GeneratedRuntimeTool::label)
        .collect::<Vec<_>>();
    let guidance = tools
        .iter()
        .copied()
        .map(GeneratedRuntimeTool::restart_guidance)
        .collect::<Vec<_>>();

    GeneratedConfigRefreshStatus {
        tools: tools.to_vec(),
        message: format!("Refreshed generated {} config.", join_human_list(&labels)),
        remediation: format!(
            "To use it in running tools, {}.",
            join_human_list(&guidance)
        ),
    }
}

fn join_human_list(values: &[&str]) -> String {
    match values {
        [] => "runtime".to_string(),
        [one] => (*one).to_string(),
        [left, right] => format!("{left} and {right}"),
        many => {
            let (last, rest) = many.split_last().expect("non-empty");
            format!("{}, and {last}", rest.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    // Defends: save/apply handling reads generated-runtime metadata from the canonical main config contract instead of hardcoding current UI rows.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn reads_apply_mode_from_main_config_contract() {
        let contract = repo_root().join("config_metadata/main_config_contract.toml");

        assert_eq!(
            apply_mode_for_setting(&contract, "yazi.theme").unwrap(),
            Some(RuntimeApplyMode::GeneratedRuntimeRefresh)
        );
        assert_eq!(
            apply_mode_for_setting(&contract, "zellij.widget_tray").unwrap(),
            Some(RuntimeApplyMode::GeneratedRuntimeRefresh)
        );
        assert_eq!(
            apply_mode_for_setting(&contract, "editor.command").unwrap(),
            Some(RuntimeApplyMode::TabSessionRestart)
        );
        assert_eq!(
            apply_mode_for_setting(&contract, "missing.setting").unwrap(),
            None
        );
    }

    // Defends: generated config refresh reports the concrete affected tool boundary instead of a vague "restart later" message.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn maps_generated_settings_to_tool_specific_refresh_guidance() {
        assert_eq!(
            generated_runtime_tools_for_setting("yazi.theme"),
            vec![GeneratedRuntimeTool::Yazi]
        );
        assert_eq!(
            generated_runtime_tools_for_setting("zellij.custom_text"),
            vec![GeneratedRuntimeTool::Zellij]
        );
        assert_eq!(
            generated_runtime_tools_for_setting("helix.generated_config"),
            vec![GeneratedRuntimeTool::Helix]
        );

        let yazi_status = refresh_status_for_tools(&[GeneratedRuntimeTool::Yazi]);
        assert!(yazi_status.message.contains("Yazi"));
        assert!(yazi_status.remediation.contains("Yazi/sidebar"));

        let mixed_status = refresh_status_for_tools(&[
            GeneratedRuntimeTool::Yazi,
            GeneratedRuntimeTool::Zellij,
            GeneratedRuntimeTool::Helix,
        ]);
        assert!(mixed_status.message.contains("Yazi, Zellij, and Helix"));
        assert!(mixed_status.remediation.contains("Helix editor pane"));
    }

    // Regression: generated-refresh failures must report the saved field and underlying materializer error instead of pretending the saved value is active.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn generated_refresh_error_is_field_scoped_and_actionable() {
        let temp = tempdir().expect("tempdir");
        let contract = temp.path().join("contract.toml");
        std::fs::write(
            &contract,
            r#"[fields."yazi.theme"]
apply_mode = "generated_runtime_refresh"
"#,
        )
        .expect("contract");
        let missing = temp.path().join("missing");
        let request = ConfigEditApplyRequest {
            setting_path: "yazi.theme".to_string(),
            contract_path: contract,
            runtime_materialization: Some(RuntimeMaterializationPlanRequest {
                config_path: missing.join("settings.jsonc"),
                default_config_path: missing.join("default.toml"),
                contract_path: missing.join("contract.toml"),
                runtime_dir: missing.join("runtime"),
                state_path: temp.path().join("state/rebuild_hash"),
                yazi_config_dir: temp.path().join("configs/yazi"),
                zellij_config_dir: temp.path().join("configs/zellij"),
                zellij_layout_dir: temp.path().join("configs/zellij/layouts"),
                layout_override: None,
            }),
        };

        let error = apply_status_after_config_edit(&request).unwrap_err();

        assert_eq!(error.code(), "generated_config_refresh_failed");
        assert!(error.message().contains("Saved yazi.theme"));
        assert!(error.remediation().contains("yzx doctor --fix"));
        assert_eq!(error.details()["setting"], "yazi.theme");
        assert!(
            error.details()["source_code"]
                .as_str()
                .is_some_and(|code| !code.is_empty())
        );
    }
}
