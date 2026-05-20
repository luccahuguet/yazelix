//! Typed `yzx status` summary construction (machine-readable and human-rendered).

use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_components::read_runtime_component_manifest;
use crate::runtime_materialization::{
    RuntimeMaterializationPlanRequest, plan_runtime_materialization,
};
use crate::session_config_snapshot::{
    load_session_config_snapshot_from_path, session_config_snapshot_path_from_env,
};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct StatusReportData {
    pub title: String,
    pub summary: JsonMap<String, JsonValue>,
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn default_terminals_value() -> JsonValue {
    json!(["ghostty"])
}

fn logs_dir_from_state_path(state_path: &Path) -> Result<String, CoreError> {
    let state_dir = state_path.parent().and_then(Path::parent).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_runtime_state_path_for_status",
            "Could not derive Yazelix state directory from runtime materialization state path.",
            "Report this as a Yazelix internal error.",
            json!({ "state_path": path_to_string(state_path) }),
        )
    })?;
    Ok(path_to_string(&state_dir.join("logs")))
}

fn runtime_components_summary(runtime_dir: &Path) -> JsonValue {
    match read_runtime_component_manifest(runtime_dir) {
        Ok(manifest) => {
            let disabled = manifest
                .iter()
                .filter(|(_, component)| !component.enabled)
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>();
            json!({
                "status": "ok",
                "disabled": disabled,
            })
        }
        Err(error) => json!({
            "status": "error",
            "error_code": error.code(),
            "message": error.message(),
            "remediation": error.remediation(),
        }),
    }
}

pub fn session_config_snapshot_summary() -> JsonValue {
    let Some(path) = session_config_snapshot_path_from_env() else {
        return json!({
            "status": "not_set",
            "path": JsonValue::Null,
        });
    };

    match load_session_config_snapshot_from_path(&path) {
        Ok(snapshot) => json!({
            "status": "ok",
            "path": path_to_string(&path),
            "snapshot_id": snapshot.snapshot_id,
            "created_at_unix_seconds": snapshot.created_at_unix_seconds,
            "source_config_file": snapshot.source_config.path,
            "source_config_hash": snapshot.source_config.hash,
            "runtime_dir": snapshot.runtime.dir,
            "runtime_hash": snapshot.runtime.hash,
            "runtime_version": snapshot.runtime.version,
        }),
        Err(error) => json!({
            "status": "error",
            "path": path_to_string(&path),
            "error_code": error.code(),
            "message": error.message(),
            "remediation": error.remediation(),
        }),
    }
}

/// Build the structured status report consumed by `yzx status` / `yzx status --json`.
pub fn compute_status_report(
    request: &RuntimeMaterializationPlanRequest,
    yazelix_version: &str,
    yazelix_description: &str,
) -> Result<StatusReportData, CoreError> {
    let plan = plan_runtime_materialization(request)?;
    let cfg = &plan.config_state.config;

    let default_shell = cfg
        .get("default_shell")
        .cloned()
        .unwrap_or_else(|| JsonValue::String(String::new()));

    let terminals = match cfg.get("terminals") {
        Some(JsonValue::Array(items)) if !items.is_empty() => JsonValue::Array(items.clone()),
        _ => default_terminals_value(),
    };

    let helix_runtime = match cfg.get("helix_runtime_path") {
        None => JsonValue::Null,
        Some(JsonValue::Null) => JsonValue::Null,
        Some(v) => v.clone(),
    };

    let runtime_dir_str = path_to_string(&request.runtime_dir);
    let logs_dir = logs_dir_from_state_path(&request.state_path)?;

    let mut summary = JsonMap::new();
    summary.insert("version".to_string(), json!(yazelix_version));
    summary.insert("description".to_string(), json!(yazelix_description));
    summary.insert(
        "config_file".to_string(),
        json!(plan.config_state.config_file),
    );
    summary.insert("runtime_dir".to_string(), json!(runtime_dir_str));
    summary.insert("logs_dir".to_string(), json!(logs_dir));
    summary.insert(
        "generated_state_repair_needed".to_string(),
        json!(plan.should_regenerate),
    );
    summary.insert(
        "generated_state_materialization_status".to_string(),
        json!(plan.status),
    );
    summary.insert(
        "generated_state_materialization_reason".to_string(),
        json!(plan.reason),
    );
    summary.insert("default_shell".to_string(), default_shell);
    summary.insert("terminals".to_string(), terminals);
    summary.insert("helix_runtime".to_string(), helix_runtime);
    summary.insert(
        "session_config_snapshot".to_string(),
        session_config_snapshot_summary(),
    );
    summary.insert(
        "runtime_components".to_string(),
        runtime_components_summary(&request.runtime_dir),
    );

    Ok(StatusReportData {
        title: "Yazelix status".to_string(),
        summary,
    })
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Defends: `yzx status --json` exposes intentionally disabled runtime components for diagnostics and config UIs.
    #[test]
    fn runtime_components_summary_lists_disabled_components() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": false, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .unwrap();

        let summary = runtime_components_summary(tmp.path());

        assert_eq!(summary["status"], "ok");
        assert_eq!(summary["disabled"], json!(["cursors"]));
    }

    // Defends: `yzx status` points users at the state-owned logs directory used by startup.
    #[test]
    fn logs_dir_comes_from_yazelix_state_path() {
        let tmp = TempDir::new().unwrap();
        let state_path = tmp.path().join("state").join("rebuild_hash");

        let logs_dir = logs_dir_from_state_path(&state_path).unwrap();

        assert_eq!(logs_dir, path_to_string(&tmp.path().join("logs")));
    }
}
