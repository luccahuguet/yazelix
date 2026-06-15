use crate::active_config_surface::primary_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateRequest, compute_config_state,
    record_config_state,
};
use crate::control_plane::config_dir_from_env;
use crate::yazi_materialization::{
    YaziMaterializationData, YaziMaterializationRequest, generate_yazi_materialization,
    generated_yazi_static_assets_missing,
};
use crate::zellij_materialization::{
    ZellijMaterializationData, ZellijMaterializationRequest, generate_zellij_materialization,
    generated_zellij_config_has_yazelix_markers, zellij_permissions_cache_path,
};
use crate::zellij_render_plan::MANAGED_SIDEBAR_LAYOUT_NAME;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMaterializationPlanRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub state_path: PathBuf,
    pub yazi_config_dir: PathBuf,
    pub zellij_config_dir: PathBuf,
    pub zellij_layout_dir: PathBuf,
    #[serde(default)]
    pub zellij_permissions_cache_path: Option<PathBuf>,
    pub layout_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeMaterializationRunData {
    pub plan: RuntimeMaterializationPlanData,
    pub yazi: YaziMaterializationData,
    pub zellij: ZellijMaterializationData,
    pub apply: RuntimeMaterializationApplyData,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeMaterializationRepairRunData {
    pub status: String,
    pub plan: RuntimeMaterializationPlanData,
    pub repair: RuntimeRepairDirective,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materialization: Option<RuntimeMaterializationRunData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMaterializationRepairEvaluateRequest {
    pub plan: RuntimeMaterializationPlanRequest,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RuntimeRepairDirective {
    Noop {
        lines: Vec<String>,
    },
    Regenerate {
        reason: String,
        progress_message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        missing_artifacts_detail_line: Option<String>,
        success_lines: Vec<String>,
        /// `repaired_missing_artifacts` or `repaired` for machine-readable callers.
        result_status: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeMaterializationRepairEvaluateData {
    pub plan: RuntimeMaterializationPlanData,
    pub repair: RuntimeRepairDirective,
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
        &request.zellij_layout_dir,
        request.layout_override.as_deref(),
    )?;
    let zellij_permissions_path = match &request.zellij_permissions_cache_path {
        Some(path) => path.clone(),
        None => zellij_permissions_cache_path()?,
    };
    let expected_artifacts = [
        (
            "generated Yazi config",
            request.yazi_config_dir.join("yazi.toml"),
        ),
        (
            "generated Yazi keymap",
            request.yazi_config_dir.join("keymap.toml"),
        ),
        (
            "generated Yazi init.lua",
            request.yazi_config_dir.join("init.lua"),
        ),
        (
            "generated Zellij config",
            request.zellij_config_dir.join("config.kdl"),
        ),
        (
            "generated Zellij layout",
            PathBuf::from(&zellij_layout_path),
        ),
        ("Zellij plugin permissions cache", zellij_permissions_path),
    ]
    .into_iter()
    .map(runtime_artifact)
    .collect::<Vec<_>>();
    let mut missing_artifacts = Vec::new();
    for artifact in &expected_artifacts {
        if runtime_artifact_needs_repair(artifact)? {
            missing_artifacts.push(artifact.clone());
        }
    }
    if !config_state.needs_refresh
        && missing_artifacts.is_empty()
        && generated_yazi_static_assets_missing(&request.runtime_dir, &request.yazi_config_dir)?
    {
        missing_artifacts.push(runtime_artifact((
            "generated Yazi static assets",
            request.yazi_config_dir.join("plugins"),
        )));
    }

    let (status, reason) = if config_state.needs_refresh {
        ("refresh_required", config_state.refresh_reason.clone())
    } else if !missing_artifacts.is_empty() {
        (
            "repair_missing_artifacts",
            format!(
                "generated runtime artifacts missing or invalid: {}",
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

pub fn evaluate_runtime_materialization_repair(
    request: &RuntimeMaterializationRepairEvaluateRequest,
) -> Result<RuntimeMaterializationRepairEvaluateData, CoreError> {
    let plan = plan_runtime_materialization_for_repair(request)?;
    let repair = build_repair_directive(&plan, request.force);
    Ok(RuntimeMaterializationRepairEvaluateData { plan, repair })
}

pub fn materialize_runtime_state(
    request: &RuntimeMaterializationPlanRequest,
) -> Result<RuntimeMaterializationRunData, CoreError> {
    let plan = plan_runtime_materialization(request)?;
    materialize_runtime_state_from_plan(request, plan)
}

pub fn repair_runtime_materialization(
    request: &RuntimeMaterializationRepairEvaluateRequest,
) -> Result<RuntimeMaterializationRepairRunData, CoreError> {
    let plan = plan_runtime_materialization_for_repair(request)?;
    let repair = build_repair_directive(&plan, request.force);

    match &repair {
        RuntimeRepairDirective::Noop { .. } => Ok(RuntimeMaterializationRepairRunData {
            status: "noop".to_string(),
            plan,
            repair,
            materialization: None,
        }),
        RuntimeRepairDirective::Regenerate { result_status, .. } => {
            let materialization = materialize_runtime_state_from_plan(&request.plan, plan.clone())?;
            Ok(RuntimeMaterializationRepairRunData {
                status: result_status.clone(),
                plan,
                repair,
                materialization: Some(materialization),
            })
        }
    }
}

fn plan_runtime_materialization_for_repair(
    request: &RuntimeMaterializationRepairEvaluateRequest,
) -> Result<RuntimeMaterializationPlanData, CoreError> {
    let mut plan = plan_runtime_materialization(&request.plan)?;
    if request.force {
        plan.should_sync_static_assets = true;
    }
    Ok(plan)
}

fn materialize_runtime_state_from_plan(
    request: &RuntimeMaterializationPlanRequest,
    plan: RuntimeMaterializationPlanData,
) -> Result<RuntimeMaterializationRunData, CoreError> {
    let yazi = generate_yazi_materialization(&YaziMaterializationRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        runtime_dir: request.runtime_dir.clone(),
        yazi_config_dir: request.yazi_config_dir.clone(),
        sync_static_assets: plan.should_sync_static_assets,
    })?;

    let zellij = generate_zellij_materialization(&ZellijMaterializationRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        runtime_dir: request.runtime_dir.clone(),
        zellij_config_dir: request.zellij_config_dir.clone(),
        seed_plugin_permissions: true,
    })?;

    let config_dir = config_dir_from_env()?;
    let managed_config_path = primary_config_paths(&request.runtime_dir, &config_dir).user_config;
    let apply = apply_runtime_materialization(
        &plan.config_state,
        managed_config_path,
        request.state_path.clone(),
        &plan.expected_artifacts,
    )?;

    Ok(RuntimeMaterializationRunData {
        plan,
        yazi,
        zellij,
        apply,
    })
}

fn build_repair_directive(
    plan: &RuntimeMaterializationPlanData,
    force: bool,
) -> RuntimeRepairDirective {
    if !force && plan.status == "noop" {
        return RuntimeRepairDirective::Noop {
            lines: vec![
                "✅ Yazelix generated state is already up to date.".to_string(),
                "   Nothing to repair.".to_string(),
            ],
        };
    }

    let reason = if force {
        "manual repair requested".to_string()
    } else {
        plan.reason.clone()
    };
    let progress_message = format!("♻️  Repairing generated runtime state ({reason})...");

    let missing_artifacts_detail_line =
        if plan.status == "repair_missing_artifacts" && !plan.missing_artifacts.is_empty() {
            Some(format!(
                "   Repairing missing artifacts: {}",
                plan.missing_artifacts
                    .iter()
                    .map(|artifact| artifact.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else {
            None
        };

    let (result_status, success_lines) = if !force && plan.status == "repair_missing_artifacts" {
        (
            "repaired_missing_artifacts".to_string(),
            vec!["✅ Repaired the missing generated runtime artifacts.".to_string()],
        )
    } else {
        (
            "repaired".to_string(),
            vec![
                "✅ Generated runtime state repaired.".to_string(),
                "   Generated Yazi/Zellij state now matches the active runtime config.".to_string(),
            ],
        )
    };

    RuntimeRepairDirective::Regenerate {
        reason,
        progress_message,
        missing_artifacts_detail_line,
        success_lines,
        result_status,
    }
}

fn apply_runtime_materialization(
    config_state: &ConfigStateData,
    managed_config_path: PathBuf,
    state_path: PathBuf,
    expected_artifacts: &[RuntimeArtifact],
) -> Result<RuntimeMaterializationApplyData, CoreError> {
    let mut missing_artifacts = Vec::new();
    for artifact in expected_artifacts {
        if runtime_artifact_needs_repair(artifact)? {
            missing_artifacts.push(artifact.clone());
        }
    }
    if !missing_artifacts.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_generated_artifacts",
            "Yazelix generated runtime artifacts are missing or invalid after materialization",
            "Regenerate the managed runtime state and retry.",
            json!({ "missing_artifacts": missing_artifacts }),
        ));
    }

    let record = record_config_state(&RecordConfigStateRequest {
        config_file: config_state.config_file.clone(),
        managed_config_path,
        state_path,
        config_hash: config_state.config_hash.clone(),
        runtime_hash: config_state.runtime_hash.clone(),
    })?;

    Ok(RuntimeMaterializationApplyData {
        recorded: record.recorded,
        checked_artifacts: expected_artifacts.len(),
    })
}

fn resolve_zellij_layout_path(
    zellij_layout_dir: &Path,
    layout_override: Option<&str>,
) -> Result<String, CoreError> {
    let override_value = layout_override
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let layout = if let Some(layout) = override_value {
        layout.to_string()
    } else {
        MANAGED_SIDEBAR_LAYOUT_NAME.to_string()
    };

    let path = if layout.contains('/') || layout.ends_with(".kdl") {
        layout
    } else {
        zellij_layout_dir
            .join(format!("{layout}.kdl"))
            .to_string_lossy()
            .to_string()
    };
    Ok(path)
}

fn is_missing_file(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => !metadata.is_file(),
        Err(_) => true,
    }
}

fn runtime_artifact((label, path): (&str, PathBuf)) -> RuntimeArtifact {
    RuntimeArtifact {
        label: label.to_string(),
        path: path.to_string_lossy().to_string(),
    }
}

fn runtime_artifact_needs_repair(artifact: &RuntimeArtifact) -> Result<bool, CoreError> {
    let path = Path::new(&artifact.path);
    if is_missing_file(path) {
        return Ok(true);
    }
    if artifact.label == "generated Zellij config" {
        return Ok(!generated_zellij_config_has_yazelix_markers(path)?);
    }
    Ok(false)
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
            default_config_path: repo.join("settings_default.jsonc"),
            contract_path: repo.join("config_metadata/main_config_contract.toml"),
            runtime_dir,
            state_path: state_path.clone(),
            yazi_config_dir: yazi_dir,
            zellij_config_dir: zellij_dir,
            zellij_layout_dir,
            zellij_permissions_cache_path: Some(state_path.with_file_name("permissions.kdl")),
            layout_override: None,
        }
    }

    struct RecordedPlanFixture {
        request: RuntimeMaterializationPlanRequest,
        runtime_dir: PathBuf,
        yazi_dir: PathBuf,
        zellij_dir: PathBuf,
    }

    fn recorded_plan_fixture(root: &Path) -> RecordedPlanFixture {
        let runtime_dir = repo_root();
        let config_path = runtime_dir.join("settings_default.jsonc");
        let state_path = root.join("state/rebuild_hash");
        let yazi_dir = root.join("configs/yazi");
        let zellij_dir = root.join("configs/zellij");
        let zellij_layout_dir = zellij_dir.join("layouts");

        fs::create_dir_all(&zellij_layout_dir).unwrap();
        let baseline = compute_config_state(&ComputeConfigStateRequest {
            config_path: config_path.clone(),
            default_config_path: runtime_dir.join("settings_default.jsonc"),
            contract_path: runtime_dir.join("config_metadata/main_config_contract.toml"),
            runtime_dir: runtime_dir.clone(),
            state_path: state_path.clone(),
        })
        .unwrap();
        record_config_state(&RecordConfigStateRequest {
            config_file: config_path.to_string_lossy().to_string(),
            managed_config_path: config_path.clone(),
            state_path: state_path.clone(),
            config_hash: baseline.config_hash,
            runtime_hash: baseline.runtime_hash,
        })
        .unwrap();

        RecordedPlanFixture {
            request: plan_request_for(
                config_path,
                runtime_dir.clone(),
                state_path,
                yazi_dir.clone(),
                zellij_dir.clone(),
                zellij_layout_dir,
            ),
            runtime_dir,
            yazi_dir,
            zellij_dir,
        }
    }

    fn plan_recorded_fixture(fixture: &RecordedPlanFixture) -> RuntimeMaterializationPlanData {
        plan_runtime_materialization(&fixture.request).unwrap()
    }

    // Defends: runtime materialization stays on the repair-missing-artifacts path when hashes are current but files are absent.
    #[test]
    fn plan_marks_missing_artifacts_without_forcing_refresh_when_state_is_current() {
        let dir = tempdir().expect("tempdir");
        let fixture = recorded_plan_fixture(dir.path());
        let plan = plan_recorded_fixture(&fixture);

        assert!(!plan.config_state.needs_refresh);
        assert_eq!(plan.status, "repair_missing_artifacts");
        assert_eq!(plan.should_regenerate, true);
        assert_eq!(plan.should_sync_static_assets, false);
        assert_eq!(plan.missing_artifacts.len(), 6);
    }

    // Defends: runtime materialization apply refuses to record success when expected generated artifacts are still missing.
    #[test]
    fn apply_rejects_missing_expected_artifacts() {
        let dir = tempdir().expect("tempdir");
        let state_path = dir.path().join("state/rebuild_hash");
        let config_state = ConfigStateData {
            config: serde_json::Map::new(),
            config_file: dir
                .path()
                .join("yazelix.toml")
                .to_string_lossy()
                .to_string(),
            needs_refresh: false,
            refresh_reason: "current".to_string(),
            config_changed: false,
            inputs_changed: false,
            inputs_require_refresh: false,
            config_hash: "cfg".to_string(),
            runtime_hash: "runtime".to_string(),
            combined_hash: "combined".to_string(),
        };
        let expected_artifacts = vec![RuntimeArtifact {
            label: "generated Yazi config".to_string(),
            path: dir
                .path()
                .join("configs/yazi/yazi.toml")
                .to_string_lossy()
                .to_string(),
        }];
        let error = apply_runtime_materialization(
            &config_state,
            dir.path().join("yazelix.toml"),
            state_path,
            &expected_artifacts,
        )
        .unwrap_err();

        assert_eq!(error.class().as_str(), "runtime");
        assert_eq!(error.code(), "missing_generated_artifacts");
    }

    fn touch_plan_artifacts(plan: &RuntimeMaterializationPlanData) {
        for artifact in &plan.expected_artifacts {
            let path = Path::new(&artifact.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let content = if artifact.label == "generated Zellij config" {
                "GENERATED ZELLIJ CONFIG (YAZELIX)\nyazelix_pane_orchestrator\nyzpp\n"
            } else {
                ""
            };
            fs::write(path, content).unwrap();
        }
    }

    fn mirror_yazi_static_assets(runtime_dir: &Path, yazi_dir: &Path) {
        mirror_tree(
            &runtime_dir.join("configs/yazi/plugins"),
            &yazi_dir.join("plugins"),
            runtime_dir,
        );
        mirror_tree(
            &runtime_dir.join("configs/yazi/flavors"),
            &yazi_dir.join("flavors"),
            runtime_dir,
        );
        let starship_source = runtime_dir.join("configs/yazi/yazelix_starship.toml");
        if starship_source.exists() {
            fs::create_dir_all(yazi_dir).unwrap();
            let bytes = fs::read(starship_source).unwrap();
            let rendered = render_test_yazi_asset_content(&bytes, runtime_dir);
            fs::write(yazi_dir.join("yazelix_starship.toml"), rendered).unwrap();
        }
    }

    fn mirror_tree(source: &Path, target: &Path, runtime_dir: &Path) {
        if !source.exists() {
            return;
        }
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                mirror_tree(&source_path, &target_path, runtime_dir);
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                let bytes = fs::read(source_path).unwrap();
                let rendered = render_test_yazi_asset_content(&bytes, runtime_dir);
                fs::write(target_path, rendered).unwrap();
            }
        }
    }

    fn render_test_yazi_asset_content(bytes: &[u8], runtime_dir: &Path) -> Vec<u8> {
        match std::str::from_utf8(bytes) {
            Ok(text) => text
                .replace(
                    "__YAZELIX_RUNTIME_DIR__",
                    runtime_dir.to_string_lossy().as_ref(),
                )
                .into_bytes(),
            Err(_) => bytes.to_vec(),
        }
    }

    // Regression: startup can skip warm materialization only when the plan still catches missing or stale generated Yazi asset trees.
    #[test]
    fn plan_marks_missing_or_stale_yazi_static_assets_without_forcing_refresh() {
        let dir = tempdir().expect("tempdir");
        let fixture = recorded_plan_fixture(dir.path());
        let initial_plan = plan_recorded_fixture(&fixture);
        touch_plan_artifacts(&initial_plan);
        mirror_yazi_static_assets(&fixture.runtime_dir, &fixture.yazi_dir);
        fs::remove_file(fixture.yazi_dir.join("plugins/sidebar-state.yazi/main.lua")).unwrap();

        let plan = plan_recorded_fixture(&fixture);

        assert!(!plan.config_state.needs_refresh);
        assert_eq!(plan.status, "repair_missing_artifacts");
        assert_eq!(plan.should_regenerate, true);
        assert_eq!(plan.should_sync_static_assets, false);
        assert_eq!(plan.missing_artifacts.len(), 1);
        assert_eq!(
            plan.missing_artifacts[0].label,
            "generated Yazi static assets"
        );

        mirror_yazi_static_assets(&fixture.runtime_dir, &fixture.yazi_dir);
        fs::write(
            fixture.yazi_dir.join("plugins/sidebar-state.yazi/main.lua"),
            "return 'stale generated plugin'\n",
        )
        .unwrap();

        let stale_plan = plan_recorded_fixture(&fixture);

        assert!(!stale_plan.config_state.needs_refresh);
        assert_eq!(stale_plan.status, "repair_missing_artifacts");
        assert_eq!(stale_plan.should_regenerate, true);
        assert_eq!(stale_plan.should_sync_static_assets, false);
        assert_eq!(stale_plan.missing_artifacts.len(), 1);
        assert_eq!(
            stale_plan.missing_artifacts[0].label,
            "generated Yazi static assets"
        );
    }

    // Regression: deleting only Zellij's plugin permission cache must still make startup reach the materializer that re-seeds Yazelix plugin permissions.
    #[test]
    fn plan_marks_missing_zellij_plugin_permissions_without_forcing_refresh() {
        let dir = tempdir().expect("tempdir");
        let fixture = recorded_plan_fixture(dir.path());
        let initial_plan = plan_recorded_fixture(&fixture);
        touch_plan_artifacts(&initial_plan);
        mirror_yazi_static_assets(&fixture.runtime_dir, &fixture.yazi_dir);
        let permissions_artifact = initial_plan
            .expected_artifacts
            .iter()
            .find(|artifact| artifact.label == "Zellij plugin permissions cache")
            .expect("permissions artifact");
        fs::remove_file(&permissions_artifact.path).unwrap();

        let plan = plan_recorded_fixture(&fixture);

        assert!(!plan.config_state.needs_refresh);
        assert_eq!(plan.status, "repair_missing_artifacts");
        assert_eq!(plan.should_regenerate, true);
        assert_eq!(plan.should_sync_static_assets, false);
        assert_eq!(plan.missing_artifacts.len(), 1);
        assert_eq!(
            plan.missing_artifacts[0].label,
            "Zellij plugin permissions cache"
        );
    }

    // Regression: a plain native Zellij config in the generated path must not satisfy the generated-state contract.
    #[test]
    fn plan_marks_plain_native_zellij_config_for_repair_without_forcing_refresh() {
        let dir = tempdir().expect("tempdir");
        let fixture = recorded_plan_fixture(dir.path());
        let initial_plan = plan_recorded_fixture(&fixture);
        touch_plan_artifacts(&initial_plan);
        mirror_yazi_static_assets(&fixture.runtime_dir, &fixture.yazi_dir);
        fs::write(
            fixture.zellij_dir.join("config.kdl"),
            "keybinds clear-defaults=true {\n    normal {}\n}\n",
        )
        .unwrap();

        let plan = plan_recorded_fixture(&fixture);

        assert!(!plan.config_state.needs_refresh);
        assert_eq!(plan.status, "repair_missing_artifacts");
        assert_eq!(plan.should_regenerate, true);
        assert_eq!(plan.should_sync_static_assets, false);
        assert_eq!(plan.missing_artifacts.len(), 1);
        assert_eq!(plan.missing_artifacts[0].label, "generated Zellij config");
    }

    // Defends: repair evaluation returns a noop directive when the plan is noop and force is false.
    #[test]
    fn repair_evaluate_is_noop_when_plan_is_noop_and_not_forced() {
        let dir = tempdir().expect("tempdir");
        let fixture = recorded_plan_fixture(dir.path());
        let plan = plan_recorded_fixture(&fixture);
        touch_plan_artifacts(&plan);
        mirror_yazi_static_assets(&fixture.runtime_dir, &fixture.yazi_dir);

        let evaluated =
            evaluate_runtime_materialization_repair(&RuntimeMaterializationRepairEvaluateRequest {
                plan: fixture.request,
                force: false,
            })
            .unwrap();

        assert_eq!(evaluated.plan.status, "noop");
        match evaluated.repair {
            RuntimeRepairDirective::Noop { lines } => {
                assert_eq!(lines.len(), 2);
            }
            other => panic!("expected noop directive, got {other:?}"),
        }
    }

    // Defends: repair evaluation forces regeneration when the user passes --force even if the plan is noop.
    #[test]
    fn repair_evaluate_regenerates_when_forced_even_if_plan_is_noop() {
        let dir = tempdir().expect("tempdir");
        let fixture = recorded_plan_fixture(dir.path());
        let plan_before = plan_recorded_fixture(&fixture);
        touch_plan_artifacts(&plan_before);
        mirror_yazi_static_assets(&fixture.runtime_dir, &fixture.yazi_dir);
        let plan_after = plan_recorded_fixture(&fixture);
        assert_eq!(plan_after.status, "noop");

        let evaluated =
            evaluate_runtime_materialization_repair(&RuntimeMaterializationRepairEvaluateRequest {
                plan: fixture.request,
                force: true,
            })
            .unwrap();

        assert!(evaluated.plan.should_sync_static_assets);
        match evaluated.repair {
            RuntimeRepairDirective::Regenerate {
                reason,
                missing_artifacts_detail_line,
                success_lines,
                result_status,
                ..
            } => {
                assert_eq!(reason, "manual repair requested");
                assert!(missing_artifacts_detail_line.is_none());
                assert_eq!(success_lines.len(), 2);
                assert_eq!(result_status, "repaired");
            }
            other => panic!("expected regenerate directive, got {other:?}"),
        }
    }
}
