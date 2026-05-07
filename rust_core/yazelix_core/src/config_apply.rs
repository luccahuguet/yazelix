// Test lane: default
//! Save-time apply handling for semantic Yazelix config edits.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::runtime_apply_mode::RuntimeApplyMode;
use crate::runtime_materialization::{
    RuntimeMaterializationPlanRequest, materialize_runtime_state,
};
use crate::zellij_commands::run_pane_orchestrator_runtime_config_reload;
use serde::Serialize;
use serde_json::{Value as JsonValue, json};
use std::path::{Path, PathBuf};

pub const PANE_ORCHESTRATOR_RUNTIME_RELOAD_SCHEMA_VERSION: u64 = 1;
const ZELLIJ_GENERATION_METADATA_NAME: &str = ".yazelix_generation.json";
const LIVE_PANE_REFRESH_SETTINGS: &[&str] = &[
    "zellij.popup_width_percent",
    "zellij.popup_height_percent",
    "zellij.screen_saver_enabled",
    "zellij.screen_saver_idle_seconds",
    "zellij.screen_saver_style",
];

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
pub struct PaneOrchestratorRuntimeRefreshStatus {
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone)]
pub struct PaneOrchestratorRuntimeRefreshRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub zellij_config_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaneOrchestratorRuntimeConfig {
    pub popup_width_percent: usize,
    pub popup_height_percent: usize,
    pub screen_saver_enabled: bool,
    pub screen_saver_idle_seconds: u64,
    pub screen_saver_style: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaneOrchestratorRuntimeReloadPayload {
    pub schema_version: u64,
    pub generation: String,
    pub runtime_config: PaneOrchestratorRuntimeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEditApplyStatus {
    pub setting_path: String,
    pub apply_mode: RuntimeApplyMode,
    pub generated_refresh: Option<GeneratedConfigRefreshStatus>,
    pub pane_orchestrator_refresh: Option<PaneOrchestratorRuntimeRefreshStatus>,
}

#[derive(Debug, Clone)]
pub struct ConfigEditApplyRequest {
    pub setting_path: String,
    pub contract_path: PathBuf,
    pub runtime_materialization: Option<RuntimeMaterializationPlanRequest>,
    pub pane_orchestrator_refresh: Option<PaneOrchestratorRuntimeRefreshRequest>,
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
    let pane_orchestrator_refresh = if apply_mode == RuntimeApplyMode::LiveWithPaneRefresh {
        Some(refresh_pane_orchestrator_runtime_config(request)?)
    } else {
        None
    };

    Ok(ConfigEditApplyStatus {
        setting_path: request.setting_path.clone(),
        apply_mode,
        generated_refresh,
        pane_orchestrator_refresh,
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

fn refresh_pane_orchestrator_runtime_config(
    request: &ConfigEditApplyRequest,
) -> Result<PaneOrchestratorRuntimeRefreshStatus, CoreError> {
    if !LIVE_PANE_REFRESH_SETTINGS
        .iter()
        .any(|setting| setting == &request.setting_path.as_str())
    {
        return Err(CoreError::classified(
            ErrorClass::Internal,
            "unsupported_live_pane_refresh_setting",
            format!(
                "Config contract marks {} as live-with-pane-refresh, but Yazelix does not know how to reload it.",
                request.setting_path
            ),
            "Fix config_metadata/main_config_contract.toml or add the setting to the pane-orchestrator reload payload.",
            json!({ "setting": request.setting_path }),
        ));
    }

    let reload_request = request.pane_orchestrator_refresh.as_ref().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "missing_pane_orchestrator_refresh_request",
            format!(
                "Cannot refresh pane-orchestrator runtime config for {} without a reload request.",
                request.setting_path
            ),
            "Report this as a Yazelix internal error.",
            json!({ "setting": request.setting_path }),
        )
    })?;
    let payload = build_pane_orchestrator_runtime_reload_payload(reload_request)?;
    let payload_text = serde_json::to_string(&payload).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_pane_orchestrator_runtime_config_reload",
            format!("Could not serialize pane-orchestrator runtime config reload: {source}"),
            "Report this as a Yazelix internal error.",
            json!({ "setting": request.setting_path }),
        )
    })?;
    let response =
        run_pane_orchestrator_runtime_config_reload(&payload_text).map_err(|source| {
            pane_refresh_error(
                &request.setting_path,
                "pane_orchestrator_runtime_config_pipe_failed",
                "Yazelix could not send the pane-orchestrator runtime-config reload.",
                "Run this from inside an active Yazelix/Zellij session with the pane orchestrator loaded, then retry or restart the tab.",
                json!({
                    "source_code": source.code(),
                    "source_class": source.class().as_str(),
                    "source_message": source.message(),
                    "source_remediation": source.remediation(),
                    "source_details": source.details(),
                }),
            )
        })?;
    pane_orchestrator_runtime_reload_status(&request.setting_path, response.trim())
}

pub fn build_pane_orchestrator_runtime_reload_payload(
    request: &PaneOrchestratorRuntimeRefreshRequest,
) -> Result<PaneOrchestratorRuntimeReloadPayload, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: false,
    })?;
    let generation = read_zellij_generation_fingerprint(&request.zellij_config_dir)?;
    Ok(PaneOrchestratorRuntimeReloadPayload {
        schema_version: PANE_ORCHESTRATOR_RUNTIME_RELOAD_SCHEMA_VERSION,
        generation,
        runtime_config: PaneOrchestratorRuntimeConfig {
            popup_width_percent: normalized_usize(
                &normalized.normalized_config,
                "popup_width_percent",
                "zellij.popup_width_percent",
            )?,
            popup_height_percent: normalized_usize(
                &normalized.normalized_config,
                "popup_height_percent",
                "zellij.popup_height_percent",
            )?,
            screen_saver_enabled: normalized_bool(
                &normalized.normalized_config,
                "screen_saver_enabled",
                "zellij.screen_saver_enabled",
            )?,
            screen_saver_idle_seconds: normalized_u64(
                &normalized.normalized_config,
                "screen_saver_idle_seconds",
                "zellij.screen_saver_idle_seconds",
            )?,
            screen_saver_style: normalized_string(
                &normalized.normalized_config,
                "screen_saver_style",
                "zellij.screen_saver_style",
            )?,
        },
    })
}

fn read_zellij_generation_fingerprint(zellij_config_dir: &Path) -> Result<String, CoreError> {
    let metadata_path = zellij_config_dir.join(ZELLIJ_GENERATION_METADATA_NAME);
    let raw = std::fs::read_to_string(&metadata_path).map_err(|source| {
        CoreError::io(
            "read_zellij_generation_metadata_for_live_reload",
            "Could not read generated Zellij metadata for pane-orchestrator live reload",
            "Run `yzx doctor --fix` or restart Yazelix so generated Zellij state is current.",
            metadata_path.display().to_string(),
            source,
        )
    })?;
    let parsed = serde_json::from_str::<JsonValue>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_zellij_generation_metadata_for_live_reload",
            format!("Could not parse generated Zellij metadata: {source}"),
            "Run `yzx doctor --fix` or restart Yazelix so generated Zellij state is current.",
            json!({ "path": metadata_path.display().to_string() }),
        )
    })?;
    let fingerprint = parsed
        .get("fingerprint")
        .and_then(JsonValue::as_str)
        .map(str::trim)
        .filter(|fingerprint| !fingerprint.is_empty())
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Runtime,
                "missing_zellij_generation_fingerprint_for_live_reload",
                "Generated Zellij metadata is missing a generation fingerprint for pane-orchestrator live reload.",
                "Run `yzx doctor --fix` or restart Yazelix so generated Zellij state is current.",
                json!({ "path": metadata_path.display().to_string() }),
            )
        })?;
    Ok(fingerprint.to_string())
}

fn pane_orchestrator_runtime_reload_status(
    setting_path: &str,
    response: &str,
) -> Result<PaneOrchestratorRuntimeRefreshStatus, CoreError> {
    match response {
        "ok" => Ok(PaneOrchestratorRuntimeRefreshStatus {
            message: "Refreshed pane-orchestrator runtime config.".to_string(),
            remediation:
                "Popup geometry and screen saver changes are active in this Yazelix session."
                    .to_string(),
        }),
        "not_ready" => Err(pane_refresh_error(
            setting_path,
            "pane_orchestrator_runtime_config_not_ready",
            "The pane orchestrator is not ready to reload runtime config.",
            "Wait for the Yazelix tab to finish loading, then save again or restart the tab.",
            json!({ "response": response }),
        )),
        "permissions_denied" => Err(pane_refresh_error(
            setting_path,
            "pane_orchestrator_runtime_config_permission_denied",
            "The pane orchestrator is missing required Zellij permissions for runtime reload.",
            "Run `yzx doctor --fix`, then restart Yazelix.",
            json!({ "response": response }),
        )),
        "invalid_payload" => Err(pane_refresh_error(
            setting_path,
            "pane_orchestrator_runtime_config_invalid_payload",
            "The pane orchestrator rejected Yazelix's runtime-config reload payload.",
            "Report this as a Yazelix internal error.",
            json!({ "response": response }),
        )),
        "version_mismatch" => Err(pane_refresh_error(
            setting_path,
            "pane_orchestrator_runtime_config_version_mismatch",
            "The active pane orchestrator does not support this runtime-config reload version.",
            "Restart Yazelix after rebuilding or reinstalling the current pane orchestrator plugin.",
            json!({
                "response": response,
                "schema_version": PANE_ORCHESTRATOR_RUNTIME_RELOAD_SCHEMA_VERSION,
            }),
        )),
        "stale_generation" => Err(pane_refresh_error(
            setting_path,
            "pane_orchestrator_runtime_config_stale_generation",
            "The active pane orchestrator was loaded from a different generated Zellij config generation.",
            "Restart this Yazelix tab or session so the plugin and generated config are aligned.",
            json!({ "response": response }),
        )),
        "" => Err(pane_refresh_error(
            setting_path,
            "pane_orchestrator_runtime_config_missing_response",
            "The pane orchestrator did not respond to the runtime-config reload.",
            "Restart this Yazelix tab or session so the current pane orchestrator plugin is loaded.",
            json!({ "response": response }),
        )),
        other => Err(pane_refresh_error(
            setting_path,
            "pane_orchestrator_runtime_config_unknown_response",
            format!(
                "The pane orchestrator returned an unknown runtime-config reload response: {other}"
            ),
            "Restart this Yazelix tab or session, then save again.",
            json!({ "response": other }),
        )),
    }
}

fn pane_refresh_error(
    setting_path: &str,
    code: &str,
    message: impl Into<String>,
    remediation: impl Into<String>,
    mut details: JsonValue,
) -> CoreError {
    if let Some(object) = details.as_object_mut() {
        object.insert(
            "setting".to_string(),
            JsonValue::String(setting_path.to_string()),
        );
    }
    CoreError::classified(
        ErrorClass::Runtime,
        code,
        format!("Saved {setting_path}, but {}.", message.into()),
        remediation,
        details,
    )
}

fn normalized_usize(
    config: &serde_json::Map<String, JsonValue>,
    key: &str,
    setting_path: &str,
) -> Result<usize, CoreError> {
    let value = normalized_u64(config, key, setting_path)?;
    usize::try_from(value).map_err(|_| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_pane_orchestrator_runtime_config_number",
            format!("{setting_path} is too large for the running pane orchestrator."),
            "Use a smaller value, then retry.",
            json!({ "field": setting_path, "value": value }),
        )
    })
}

fn normalized_u64(
    config: &serde_json::Map<String, JsonValue>,
    key: &str,
    setting_path: &str,
) -> Result<u64, CoreError> {
    config
        .get(key)
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| invalid_normalized_runtime_field(setting_path, "integer"))
}

fn normalized_bool(
    config: &serde_json::Map<String, JsonValue>,
    key: &str,
    setting_path: &str,
) -> Result<bool, CoreError> {
    config
        .get(key)
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| invalid_normalized_runtime_field(setting_path, "boolean"))
}

fn normalized_string(
    config: &serde_json::Map<String, JsonValue>,
    key: &str,
    setting_path: &str,
) -> Result<String, CoreError> {
    config
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::to_string)
        .ok_or_else(|| invalid_normalized_runtime_field(setting_path, "string"))
}

fn invalid_normalized_runtime_field(setting_path: &str, expected: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        "invalid_pane_orchestrator_runtime_config_field",
        format!("Normalized config field {setting_path} is not a {expected}."),
        "Fix settings.jsonc, then retry.",
        json!({ "field": setting_path, "expected": expected }),
    )
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
            pane_orchestrator_refresh: None,
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

    // Defends: live pane-refresh settings serialize one bounded versioned reload payload from normalized settings and generated Zellij metadata.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn builds_pane_orchestrator_runtime_reload_payload_from_saved_config() {
        let repo = repo_root();
        let temp = tempdir().expect("tempdir");
        let config_path = temp.path().join("settings.jsonc");
        let default_config_path = temp.path().join("yazelix_default.toml");
        let contract_path = temp.path().join("main_config_contract.toml");
        let zellij_config_dir = temp.path().join("configs/zellij");
        std::fs::create_dir_all(&zellij_config_dir).unwrap();
        std::fs::copy(repo.join("yazelix_default.toml"), &default_config_path).unwrap();
        std::fs::copy(
            repo.join("config_metadata/main_config_contract.toml"),
            &contract_path,
        )
        .unwrap();
        std::fs::write(
            &config_path,
            r#"{
  "zellij": {
    "popup_width_percent": 82,
    "popup_height_percent": 76,
    "screen_saver_enabled": true,
    "screen_saver_idle_seconds": 120,
    "screen_saver_style": "mandelbrot"
  }
}
"#,
        )
        .unwrap();
        std::fs::write(
            zellij_config_dir.join(".yazelix_generation.json"),
            r#"{"fingerprint":"gen-a"}"#,
        )
        .unwrap();

        let payload = build_pane_orchestrator_runtime_reload_payload(
            &PaneOrchestratorRuntimeRefreshRequest {
                config_path,
                default_config_path,
                contract_path,
                zellij_config_dir,
            },
        )
        .unwrap();

        assert_eq!(payload.schema_version, 1);
        assert_eq!(payload.generation, "gen-a");
        assert_eq!(
            payload.runtime_config,
            PaneOrchestratorRuntimeConfig {
                popup_width_percent: 82,
                popup_height_percent: 76,
                screen_saver_enabled: true,
                screen_saver_idle_seconds: 120,
                screen_saver_style: "mandelbrot".to_string()
            }
        );
    }

    // Regression: stale pane-orchestrator generations stay visibly field-scoped pending work after the setting is saved.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn stale_pane_orchestrator_generation_error_is_field_scoped() {
        let error = pane_orchestrator_runtime_reload_status(
            "zellij.popup_width_percent",
            "stale_generation",
        )
        .unwrap_err();

        assert_eq!(
            error.code(),
            "pane_orchestrator_runtime_config_stale_generation"
        );
        assert!(error.message().contains("Saved zellij.popup_width_percent"));
        assert!(error.remediation().contains("Restart this Yazelix tab"));
        assert_eq!(error.details()["setting"], "zellij.popup_width_percent");
    }
}
