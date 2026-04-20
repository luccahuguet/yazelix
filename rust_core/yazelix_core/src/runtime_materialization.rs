use crate::bridge::{CoreError, ErrorClass};
use crate::config_state::{
    compute_config_state, record_config_state, ComputeConfigStateRequest, ConfigStateData,
    RecordConfigStateRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RuntimeMaterializationPlanRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub state_path: PathBuf,
    pub yazi_config_dir: PathBuf,
    pub zellij_config_dir: PathBuf,
    pub zellij_layout_dir: PathBuf,
    pub layout_override: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeMaterializationApplyRequest {
    pub config_file: String,
    pub managed_config_path: PathBuf,
    pub state_path: PathBuf,
    pub config_hash: String,
    pub runtime_hash: String,
    pub expected_artifacts: Vec<RuntimeArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeArtifact {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeMaterializationPlanData {
    #[serde(flatten)]
    pub config_state: ConfigStateData,
    pub yazi_config_dir: String,
    pub zellij_config_dir: String,
    pub zellij_layout_path: String,
    pub expected_artifacts: Vec<RuntimeArtifact>,
    pub missing_artifacts: Vec<RuntimeArtifact>,
    pub status: String,
    pub reason: String,
    pub should_regenerate: bool,
    pub should_sync_static_assets: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeMaterializationApplyData {
    pub recorded: bool,
    pub checked_artifacts: usize,
}

pub fn plan_runtime_materialization(
    request: &RuntimeMaterializationPlanRequest,
) -> Result<RuntimeMaterializationPlanData, CoreError> {
    let config_state = compute_config_state(&ComputeConfigStateRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        runtime_dir: request.runtime_dir.clone(),
        state_path: request.state_path.clone(),
    })?;
    let zellij_layout_path = resolve_zellij_layout_path(
        &config_state.config,
        &request.zellij_layout_dir,
        request.layout_override.as_deref(),
    );
    let expected_artifacts = vec![
        RuntimeArtifact {
            label: "generated Yazi config".to_string(),
            path: request
                .yazi_config_dir
                .join("yazi.toml")
                .to_string_lossy()
                .to_string(),
        },
        RuntimeArtifact {
            label: "generated Yazi keymap".to_string(),
            path: request
                .yazi_config_dir
                .join("keymap.toml")
                .to_string_lossy()
                .to_string(),
        },
        RuntimeArtifact {
            label: "generated Yazi init.lua".to_string(),
            path: request
                .yazi_config_dir
                .join("init.lua")
                .to_string_lossy()
                .to_string(),
        },
        RuntimeArtifact {
            label: "generated Zellij config".to_string(),
            path: request
                .zellij_config_dir
                .join("config.kdl")
                .to_string_lossy()
                .to_string(),
        },
        RuntimeArtifact {
            label: "generated Zellij layout".to_string(),
            path: zellij_layout_path.clone(),
        },
    ];
    let missing_artifacts = expected_artifacts
        .iter()
        .filter(|artifact| is_missing_file(Path::new(&artifact.path)))
        .cloned()
        .collect::<Vec<_>>();

    let (status, reason) = if config_state.needs_refresh {
        ("refresh_required", config_state.refresh_reason.clone())
    } else if !missing_artifacts.is_empty() {
        (
            "repair_missing_artifacts",
            format!(
                "generated runtime artifacts missing: {}",
                missing_artifacts
                    .iter()
                    .map(|artifact| artifact.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
    } else {
        (
            "noop",
            "generated runtime state is already up to date".to_string(),
        )
    };

    Ok(RuntimeMaterializationPlanData {
        config_state,
        yazi_config_dir: request.yazi_config_dir.to_string_lossy().to_string(),
        zellij_config_dir: request.zellij_config_dir.to_string_lossy().to_string(),
        zellij_layout_path,
        expected_artifacts,
        missing_artifacts: missing_artifacts.clone(),
        status: status.to_string(),
        reason,
        should_regenerate: status != "noop",
        should_sync_static_assets: status == "refresh_required",
    })
}

pub fn apply_runtime_materialization(
    request: &RuntimeMaterializationApplyRequest,
) -> Result<RuntimeMaterializationApplyData, CoreError> {
    let missing_artifacts = request
        .expected_artifacts
        .iter()
        .filter(|artifact| is_missing_file(Path::new(&artifact.path)))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_artifacts.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_generated_artifacts",
            "Yazelix generated runtime artifacts are missing after materialization",
            "Regenerate the managed runtime state and retry.",
            json!({ "missing_artifacts": missing_artifacts }),
        ));
    }

    let record = record_config_state(&RecordConfigStateRequest {
        config_file: request.config_file.clone(),
        managed_config_path: request.managed_config_path.clone(),
        state_path: request.state_path.clone(),
        config_hash: request.config_hash.clone(),
        runtime_hash: request.runtime_hash.clone(),
    })?;

    Ok(RuntimeMaterializationApplyData {
        recorded: record.recorded,
        checked_artifacts: request.expected_artifacts.len(),
    })
}

fn resolve_zellij_layout_path(
    config: &JsonMap<String, JsonValue>,
    zellij_layout_dir: &Path,
    layout_override: Option<&str>,
) -> String {
    let override_value = layout_override
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let layout = if let Some(layout) = override_value {
        layout.to_string()
    } else if json_bool(config.get("enable_sidebar"), true) {
        "yzx_side".to_string()
    } else {
        "yzx_no_side".to_string()
    };

    if layout.contains('/') || layout.ends_with(".kdl") {
        layout
    } else {
        zellij_layout_dir
            .join(format!("{layout}.kdl"))
            .to_string_lossy()
            .to_string()
    }
}

fn json_bool(value: Option<&JsonValue>, default: bool) -> bool {
    match value {
        Some(JsonValue::Bool(value)) => *value,
        Some(JsonValue::String(value)) => match value.as_str() {
            "true" => true,
            "false" => false,
            _ => default,
        },
        _ => default,
    }
}

fn is_missing_file(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => !metadata.is_file(),
        Err(_) => true,
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_state::RecordConfigStateRequest;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::tempdir;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    fn plan_request_for(
        config_path: PathBuf,
        runtime_dir: PathBuf,
        state_path: PathBuf,
        yazi_dir: PathBuf,
        zellij_dir: PathBuf,
        zellij_layout_dir: PathBuf,
    ) -> RuntimeMaterializationPlanRequest {
        let repo = repo_root();
        RuntimeMaterializationPlanRequest {
            config_path,
            default_config_path: repo.join("yazelix_default.toml"),
            contract_path: repo.join("config_metadata/main_config_contract.toml"),
            runtime_dir,
            state_path,
            yazi_config_dir: yazi_dir,
            zellij_config_dir: zellij_dir,
            zellij_layout_dir,
            layout_override: None,
        }
    }

    // Defends: runtime materialization stays on the repair-missing-artifacts path when hashes are current but files are absent.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn plan_marks_missing_artifacts_without_forcing_refresh_when_state_is_current() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let config_path = runtime_dir.join("yazelix_default.toml");
        let state_path = dir.path().join("state/rebuild_hash");
        let yazi_dir = dir.path().join("configs/yazi");
        let zellij_dir = dir.path().join("configs/zellij");
        let zellij_layout_dir = zellij_dir.join("layouts");

        fs::create_dir_all(&zellij_layout_dir).unwrap();
        let baseline = compute_config_state(&ComputeConfigStateRequest {
            config_path: config_path.clone(),
            default_config_path: runtime_dir.join("yazelix_default.toml"),
            contract_path: runtime_dir.join("config_metadata/main_config_contract.toml"),
            runtime_dir: runtime_dir.clone(),
            state_path: state_path.clone(),
        })
        .unwrap();
        record_config_state(&RecordConfigStateRequest {
            config_file: config_path.to_string_lossy().to_string(),
            managed_config_path: config_path.clone(),
            state_path: state_path.clone(),
            config_hash: baseline.config_hash.clone(),
            runtime_hash: baseline.runtime_hash.clone(),
        })
        .unwrap();

        let plan = plan_runtime_materialization(&plan_request_for(
            config_path,
            runtime_dir,
            state_path,
            yazi_dir,
            zellij_dir,
            zellij_layout_dir,
        ))
        .unwrap();

        assert!(!plan.config_state.needs_refresh);
        assert_eq!(plan.status, "repair_missing_artifacts");
        assert_eq!(plan.should_regenerate, true);
        assert_eq!(plan.should_sync_static_assets, false);
        assert_eq!(plan.missing_artifacts.len(), 5);
    }

    // Defends: runtime materialization apply refuses to record success when expected generated artifacts are still missing.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn apply_rejects_missing_expected_artifacts() {
        let dir = tempdir().expect("tempdir");
        let state_path = dir.path().join("state/rebuild_hash");
        let error = apply_runtime_materialization(&RuntimeMaterializationApplyRequest {
            config_file: dir
                .path()
                .join("yazelix.toml")
                .to_string_lossy()
                .to_string(),
            managed_config_path: dir.path().join("yazelix.toml"),
            state_path,
            config_hash: "cfg".to_string(),
            runtime_hash: "runtime".to_string(),
            expected_artifacts: vec![RuntimeArtifact {
                label: "generated Yazi config".to_string(),
                path: dir
                    .path()
                    .join("configs/yazi/yazi.toml")
                    .to_string_lossy()
                    .to_string(),
            }],
        })
        .unwrap_err();

        assert_eq!(error.class().as_str(), "runtime");
        assert_eq!(error.code(), "missing_generated_artifacts");
    }
}
