//! Typed `yzx status` summary construction (machine-readable and human-rendered).

use crate::bridge::CoreError;
use crate::runtime_materialization::{plan_runtime_materialization, RuntimeMaterializationPlanRequest};
use serde::Serialize;
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct StatusReportData {
    pub title: String,
    pub summary: JsonMap<String, JsonValue>,
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn persistent_sessions_from_config(cfg: &JsonMap<String, JsonValue>) -> bool {
    match cfg.get("persistent_sessions") {
        Some(JsonValue::Bool(b)) => *b,
        Some(JsonValue::String(s)) => s == "true",
        Some(JsonValue::Number(n)) => n.as_i64() == Some(1),
        _ => false,
    }
}

fn session_name_for_summary(cfg: &JsonMap<String, JsonValue>, persistent: bool) -> JsonValue {
    if !persistent {
        return JsonValue::Null;
    }
    match cfg.get("session_name") {
        None => JsonValue::Null,
        Some(JsonValue::Null) => JsonValue::Null,
        Some(v) => v.clone(),
    }
}

fn default_terminals_value() -> JsonValue {
    json!(["ghostty"])
}

/// Build the structured status report consumed by `yzx status` / `yzx status --json`.
pub fn compute_status_report(
    request: &RuntimeMaterializationPlanRequest,
    yazelix_version: &str,
    yazelix_description: &str,
) -> Result<StatusReportData, CoreError> {
    let plan = plan_runtime_materialization(request)?;
    let cfg = &plan.config_state.config;
    let persistent = persistent_sessions_from_config(cfg);

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
    let logs_dir = path_to_string(&request.runtime_dir.join("logs"));

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
    summary.insert("persistent_sessions".to_string(), json!(persistent));
    summary.insert(
        "session_name".to_string(),
        session_name_for_summary(cfg, persistent),
    );

    Ok(StatusReportData {
        title: "Yazelix status".to_string(),
        summary,
    })
}

#[cfg(test)]
mod tests {
    // Test lane: default
    // Defends: status summary derives persistent_sessions from normalized config shapes.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    use super::*;

    #[test]
    fn persistent_sessions_accepts_bool_and_string() {
        let mut a = JsonMap::new();
        a.insert("persistent_sessions".into(), json!(true));
        assert!(persistent_sessions_from_config(&a));

        let mut b = JsonMap::new();
        b.insert("persistent_sessions".into(), json!("true"));
        assert!(persistent_sessions_from_config(&b));

        let mut c = JsonMap::new();
        c.insert("persistent_sessions".into(), json!("false"));
        assert!(!persistent_sessions_from_config(&c));

        let d = JsonMap::new();
        assert!(!persistent_sessions_from_config(&d));
    }
}
