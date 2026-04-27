// Test lane: default
//! Startup handoff context capture for diagnosing repair-driven launch failures.

use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const HANDOFF_SCHEMA_VERSION: i32 = 1;
const HANDOFF_DIR: &str = "startup_handoff";
const LATEST_FILE: &str = "latest.json";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StartupHandoffCaptureRequest {
    pub state_dir: PathBuf,
    pub working_dir: String,
    pub session_default_cwd: String,
    pub launch_process_cwd: String,
    pub zellij_config_dir: String,
    pub layout_path: String,
    pub default_shell: String,
    pub materialization_status: String,
    pub materialization_reason: String,
    pub materialization_should_regenerate: bool,
    pub materialization_should_sync_static_assets: bool,
    #[serde(default)]
    pub missing_artifacts: Vec<StartupHandoffArtifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct StartupHandoffArtifact {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StartupHandoffCaptureData {
    pub recorded: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct StartupHandoffContext<'a> {
    schema_version: i32,
    captured_at_unix_seconds: u64,
    trigger: &'a str,
    working_dir: &'a str,
    session_default_cwd: &'a str,
    launch_process_cwd: &'a str,
    zellij_config_dir: &'a str,
    layout_path: &'a str,
    default_shell: &'a str,
    materialization: StartupHandoffMaterialization<'a>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct StartupHandoffMaterialization<'a> {
    status: &'a str,
    reason: &'a str,
    should_regenerate: bool,
    should_sync_static_assets: bool,
    missing_artifacts: &'a [StartupHandoffArtifact],
}

pub fn capture_startup_handoff_context(
    request: &StartupHandoffCaptureRequest,
) -> Result<StartupHandoffCaptureData, CoreError> {
    if !should_record_handoff_context(request) {
        return Ok(StartupHandoffCaptureData {
            recorded: false,
            reason: "materialization_noop".to_string(),
            context_path: None,
            latest_path: None,
        });
    }

    let timestamp = unix_seconds()?;
    let capture_dir = startup_handoff_capture_dir(&request.state_dir);
    fs::create_dir_all(&capture_dir).map_err(|source| {
        CoreError::io(
            "startup_handoff_dir",
            format!(
                "Could not create startup handoff context directory {}.",
                capture_dir.display()
            ),
            "Check permissions for the Yazelix state directory and retry the launch.",
            capture_dir.to_string_lossy(),
            source,
        )
    })?;

    let context = build_startup_handoff_context(request, timestamp);
    let serialized = serde_json::to_string_pretty(&context).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "startup_handoff_serialize",
            format!("Could not serialize startup handoff context: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    let context_path = capture_dir.join(format!("handoff_{timestamp}.json"));
    let latest_path = capture_dir.join(LATEST_FILE);
    write_text_atomic(&context_path, &serialized)?;
    write_text_atomic(&latest_path, &serialized)?;

    Ok(StartupHandoffCaptureData {
        recorded: true,
        reason: "materialization_repaired".to_string(),
        context_path: Some(context_path.to_string_lossy().to_string()),
        latest_path: Some(latest_path.to_string_lossy().to_string()),
    })
}

fn should_record_handoff_context(request: &StartupHandoffCaptureRequest) -> bool {
    request.materialization_should_regenerate
        || request.materialization_status.trim() != "noop"
        || !request.missing_artifacts.is_empty()
}

fn build_startup_handoff_context(
    request: &StartupHandoffCaptureRequest,
    captured_at_unix_seconds: u64,
) -> StartupHandoffContext<'_> {
    StartupHandoffContext {
        schema_version: HANDOFF_SCHEMA_VERSION,
        captured_at_unix_seconds,
        trigger: "generated_state_materialization",
        working_dir: request.working_dir.as_str(),
        session_default_cwd: request.session_default_cwd.as_str(),
        launch_process_cwd: request.launch_process_cwd.as_str(),
        zellij_config_dir: request.zellij_config_dir.as_str(),
        layout_path: request.layout_path.as_str(),
        default_shell: request.default_shell.as_str(),
        materialization: StartupHandoffMaterialization {
            status: request.materialization_status.as_str(),
            reason: request.materialization_reason.as_str(),
            should_regenerate: request.materialization_should_regenerate,
            should_sync_static_assets: request.materialization_should_sync_static_assets,
            missing_artifacts: request.missing_artifacts.as_slice(),
        },
    }
}

fn startup_handoff_capture_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("logs").join(HANDOFF_DIR)
}

fn unix_seconds() -> Result<u64, CoreError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|source| {
            CoreError::classified(
                ErrorClass::Runtime,
                "startup_handoff_time",
                format!("System clock error while recording startup handoff context: {source}"),
                "Check the system clock and retry the launch.",
                json!({}),
            )
        })
}

fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).map_err(|source| {
        CoreError::io(
            "startup_handoff_write",
            format!("Could not write startup handoff context {}", tmp.display()),
            "Check permissions for the Yazelix state directory and retry the launch.",
            tmp.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&tmp, path).map_err(|source| {
        CoreError::io(
            "startup_handoff_replace",
            format!(
                "Could not publish startup handoff context {}",
                path.display()
            ),
            "Check permissions for the Yazelix state directory and retry the launch.",
            path.to_string_lossy(),
            source,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(status: &str) -> StartupHandoffCaptureRequest {
        StartupHandoffCaptureRequest {
            state_dir: PathBuf::from("/tmp/yazelix-state"),
            working_dir: "/repo".into(),
            session_default_cwd: "/repo".into(),
            launch_process_cwd: "/repo".into(),
            zellij_config_dir: "/state/configs/zellij".into(),
            layout_path: "/state/layouts/yzx_side.kdl".into(),
            default_shell: "/state/shells/nu".into(),
            materialization_status: status.into(),
            materialization_reason: "generated runtime artifacts missing: generated Zellij layout"
                .into(),
            materialization_should_regenerate: status != "noop",
            materialization_should_sync_static_assets: false,
            missing_artifacts: vec![StartupHandoffArtifact {
                label: "generated Zellij layout".into(),
                path: "/state/layouts/yzx_side.kdl".into(),
            }],
        }
    }

    // Defends: startup handoff capture is gated to repair/materialization launches instead of logging every startup.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn handoff_capture_records_only_when_materialization_changed_state() {
        let mut noop = request("noop");
        noop.materialization_should_regenerate = false;
        noop.missing_artifacts.clear();

        assert!(!should_record_handoff_context(&noop));
        assert!(should_record_handoff_context(&request(
            "repair_missing_artifacts"
        )));
        assert!(should_record_handoff_context(&request("refresh_required")));
    }

    // Defends: the durable startup handoff context carries the facts needed to diagnose repair-then-Zellij crashes.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn handoff_context_contains_repair_and_zellij_handoff_facts() {
        let req = request("repair_missing_artifacts");
        let context = build_startup_handoff_context(&req, 1234);
        let value = serde_json::to_value(&context).unwrap();

        assert_eq!(value["schema_version"], HANDOFF_SCHEMA_VERSION);
        assert_eq!(value["captured_at_unix_seconds"], 1234);
        assert_eq!(value["trigger"], "generated_state_materialization");
        assert_eq!(value["working_dir"], "/repo");
        assert_eq!(value["layout_path"], "/state/layouts/yzx_side.kdl");
        assert_eq!(value["default_shell"], "/state/shells/nu");
        assert_eq!(
            value["materialization"]["status"],
            "repair_missing_artifacts"
        );
        assert_eq!(
            value["materialization"]["missing_artifacts"][0]["label"],
            "generated Zellij layout"
        );
    }
}
