use crate::active_config_surface::primary_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateRequest, compute_config_state,
    record_config_state,
};
use crate::control_plane::config_dir_from_env;
use crate::ghostty_cursor_registry::{CursorRegistry, USER_CURSOR_CONFIG_FILENAME};
use crate::yazi_materialization::{
    YaziMaterializationData, YaziMaterializationRequest, generate_yazi_materialization,
};
use crate::zellij_materialization::{
    ZellijMaterializationData, ZellijMaterializationRequest, generate_zellij_materialization,
};
use crate::zellij_render_plan::managed_sidebar_layout_name;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub migration: Option<RuntimeCursorConfigMigrationData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materialization: Option<RuntimeMaterializationRunData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMaterializationRepairEvaluateRequest {
    pub plan: RuntimeMaterializationPlanRequest,
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RepairSuccessKind {
    RepairedMissingArtifacts,
    Repaired,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeCursorConfigMigrationData {
    pub action: String,
    pub moved_fields: Vec<String>,
    pub config_path: String,
    pub cursor_config_path: String,
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
        &config_state.config,
        &request.zellij_layout_dir,
        request.layout_override.as_deref(),
    )?;
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

pub fn evaluate_runtime_materialization_repair(
    request: &RuntimeMaterializationRepairEvaluateRequest,
) -> Result<RuntimeMaterializationRepairEvaluateData, CoreError> {
    let plan = plan_runtime_materialization(&request.plan)?;
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
    let migration = migrate_moved_ghostty_cursor_fields(&request.plan)?;
    let plan = plan_runtime_materialization(&request.plan)?;
    let repair = build_repair_directive(&plan, request.force);

    match &repair {
        RuntimeRepairDirective::Noop { .. } => Ok(RuntimeMaterializationRepairRunData {
            status: "noop".to_string(),
            plan,
            repair,
            migration,
            materialization: None,
        }),
        RuntimeRepairDirective::Regenerate { result_status, .. } => {
            let materialization = materialize_runtime_state_from_plan(&request.plan, plan.clone())?;
            Ok(RuntimeMaterializationRepairRunData {
                status: result_status.clone(),
                plan,
                repair,
                migration,
                materialization: Some(materialization),
            })
        }
    }
}

const MOVED_GHOSTTY_CURSOR_SETTINGS: &[MovedGhosttyCursorSetting] = &[
    MovedGhosttyCursorSetting {
        old_key: "ghostty_trail_color",
        sidecar_key: "trail",
        value_kind: MovedGhosttyCursorValueKind::String,
    },
    MovedGhosttyCursorSetting {
        old_key: "ghostty_trail_effect",
        sidecar_key: "trail_effect",
        value_kind: MovedGhosttyCursorValueKind::String,
    },
    MovedGhosttyCursorSetting {
        old_key: "ghostty_trail_duration",
        sidecar_key: "duration",
        value_kind: MovedGhosttyCursorValueKind::Number,
    },
    MovedGhosttyCursorSetting {
        old_key: "ghostty_mode_effect",
        sidecar_key: "mode_effect",
        value_kind: MovedGhosttyCursorValueKind::String,
    },
    MovedGhosttyCursorSetting {
        old_key: "ghostty_trail_glow",
        sidecar_key: "glow",
        value_kind: MovedGhosttyCursorValueKind::String,
    },
];

#[derive(Debug, Clone, Copy)]
struct MovedGhosttyCursorSetting {
    old_key: &'static str,
    sidecar_key: &'static str,
    value_kind: MovedGhosttyCursorValueKind,
}

#[derive(Debug, Clone, Copy)]
enum MovedGhosttyCursorValueKind {
    String,
    Number,
}

#[derive(Debug, Clone)]
struct MovedGhosttyCursorValue {
    old_path: String,
    sidecar_key: &'static str,
    rendered_value: String,
}

fn migrate_moved_ghostty_cursor_fields(
    request: &RuntimeMaterializationPlanRequest,
) -> Result<Option<RuntimeCursorConfigMigrationData>, CoreError> {
    let raw_config = fs::read_to_string(&request.config_path).map_err(|source| {
        CoreError::io(
            "read_moved_cursor_migration_config",
            "Could not read Yazelix config for cursor migration",
            "Ensure ~/.config/yazelix/user_configs/yazelix.toml exists and is readable.",
            request.config_path.to_string_lossy(),
            source,
        )
    })?;
    let parsed_config = toml::from_str::<toml::Table>(&raw_config).map_err(|source| {
        CoreError::toml(
            "invalid_toml",
            "Could not parse Yazelix config for cursor migration",
            "Fix the TOML syntax in user_configs/yazelix.toml and retry.",
            request.config_path.to_string_lossy(),
            source,
        )
    })?;
    let moved_values = moved_ghostty_cursor_values(&parsed_config, &request.config_path)?;
    if moved_values.is_empty() {
        return Ok(None);
    }

    let cursor_config_path = request
        .config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(USER_CURSOR_CONFIG_FILENAME);
    let raw_cursor_config =
        read_cursor_sidecar_or_default(&cursor_config_path, &request.runtime_dir)?;
    let updated_cursor_config = apply_cursor_sidecar_settings(&raw_cursor_config, &moved_values)?;
    CursorRegistry::parse_str(&cursor_config_path, &updated_cursor_config)?;

    let updated_config = remove_moved_ghostty_cursor_fields(&raw_config, &parsed_config)?;
    write_text_atomic(&cursor_config_path, &updated_cursor_config)?;
    write_text_atomic(&request.config_path, &updated_config)?;

    Ok(Some(RuntimeCursorConfigMigrationData {
        action: "moved_ghostty_cursor_fields".to_string(),
        moved_fields: moved_values
            .iter()
            .map(|value| value.old_path.clone())
            .collect(),
        config_path: request.config_path.to_string_lossy().to_string(),
        cursor_config_path: cursor_config_path.to_string_lossy().to_string(),
    }))
}

fn moved_ghostty_cursor_values(
    config: &toml::Table,
    config_path: &Path,
) -> Result<Vec<MovedGhosttyCursorValue>, CoreError> {
    let Some(terminal) = config.get("terminal").and_then(TomlValue::as_table) else {
        return Ok(Vec::new());
    };

    let mut moved_values = Vec::new();
    for setting in MOVED_GHOSTTY_CURSOR_SETTINGS {
        let Some(value) = terminal.get(setting.old_key) else {
            continue;
        };
        moved_values.push(MovedGhosttyCursorValue {
            old_path: format!("terminal.{}", setting.old_key),
            sidecar_key: setting.sidecar_key,
            rendered_value: render_moved_cursor_value(config_path, setting, value)?,
        });
    }
    Ok(moved_values)
}

fn render_moved_cursor_value(
    config_path: &Path,
    setting: &MovedGhosttyCursorSetting,
    value: &TomlValue,
) -> Result<String, CoreError> {
    match setting.value_kind {
        MovedGhosttyCursorValueKind::String => value
            .as_str()
            .map(|value| format!("{value:?}"))
            .ok_or_else(|| invalid_moved_cursor_field_type(config_path, setting, "string")),
        MovedGhosttyCursorValueKind::Number => toml_numeric_value(value)
            .map(render_cursor_duration_value)
            .ok_or_else(|| invalid_moved_cursor_field_type(config_path, setting, "number")),
    }
}

fn toml_numeric_value(value: &TomlValue) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
}

fn render_cursor_duration_value(value: f64) -> String {
    let mut rendered = format!("{value:.3}");
    while rendered.contains('.') && rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.push('0');
    }
    rendered
}

fn invalid_moved_cursor_field_type(
    config_path: &Path,
    setting: &MovedGhosttyCursorSetting,
    expected: &str,
) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        "invalid_moved_cursor_config_field",
        format!(
            "Moved cursor config field terminal.{} has the wrong type",
            setting.old_key
        ),
        "Update the moved terminal.ghostty_* cursor field manually, then retry repair.",
        json!({
            "path": config_path.to_string_lossy(),
            "field": format!("terminal.{}", setting.old_key),
            "expected": expected,
        }),
    )
}

fn read_cursor_sidecar_or_default(
    cursor_config_path: &Path,
    runtime_dir: &Path,
) -> Result<String, CoreError> {
    if cursor_config_path.exists() {
        return fs::read_to_string(cursor_config_path).map_err(|source| {
            CoreError::io(
                "read_cursor_config_for_migration",
                "Could not read Yazelix cursor config for migration",
                "Ensure user_configs/yazelix_cursors.toml is readable, then retry.",
                cursor_config_path.to_string_lossy(),
                source,
            )
        });
    }

    let default_path = CursorRegistry::default_config_path(runtime_dir);
    fs::read_to_string(&default_path).map_err(|source| {
        CoreError::io(
            "read_default_cursor_config_for_migration",
            "Could not read the default Yazelix cursor config for migration",
            "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml, then retry.",
            default_path.to_string_lossy(),
            source,
        )
    })
}

fn apply_cursor_sidecar_settings(
    raw: &str,
    moved_values: &[MovedGhosttyCursorValue],
) -> Result<String, CoreError> {
    let mut output = String::new();
    let mut in_settings = false;
    let mut seen = std::collections::BTreeSet::new();

    for line_with_newline in raw.split_inclusive('\n') {
        let (line, newline) = split_line_ending(line_with_newline);
        let trimmed = line.trim();
        if is_toml_section_header(trimmed) {
            if in_settings {
                insert_missing_cursor_settings(&mut output, moved_values, &seen);
            }
            in_settings = trimmed == "[settings]";
        }

        if in_settings {
            if let Some(key) = toml_assignment_key(line) {
                if let Some(moved) = moved_values
                    .iter()
                    .find(|value| value.sidecar_key == key.as_str())
                {
                    let indent = line
                        .chars()
                        .take_while(|character| character.is_whitespace())
                        .collect::<String>();
                    output.push_str(&format!(
                        "{indent}{} = {}{newline}",
                        moved.sidecar_key, moved.rendered_value
                    ));
                    seen.insert(moved.sidecar_key);
                    continue;
                }
            }
        }

        output.push_str(line);
        output.push_str(newline);
    }

    if in_settings {
        insert_missing_cursor_settings(&mut output, moved_values, &seen);
    }

    Ok(output)
}

fn insert_missing_cursor_settings(
    output: &mut String,
    moved_values: &[MovedGhosttyCursorValue],
    seen: &std::collections::BTreeSet<&'static str>,
) {
    for moved in moved_values {
        if !seen.contains(moved.sidecar_key) {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&format!(
                "{} = {}\n",
                moved.sidecar_key, moved.rendered_value
            ));
        }
    }
}

fn remove_moved_ghostty_cursor_fields(
    raw_config: &str,
    parsed_config: &toml::Table,
) -> Result<String, CoreError> {
    let mut output = String::new();
    let mut current_section = String::new();

    for line_with_newline in raw_config.split_inclusive('\n') {
        let (line, newline) = split_line_ending(line_with_newline);
        let trimmed = line.trim();
        if is_toml_section_header(trimmed) {
            current_section = trimmed
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim()
                .to_string();
        }

        let remove_line = toml_assignment_key(line).is_some_and(|key| {
            if current_section == "terminal" {
                moved_ghostty_cursor_old_key(&key)
            } else {
                MOVED_GHOSTTY_CURSOR_SETTINGS
                    .iter()
                    .any(|setting| key == format!("terminal.{}", setting.old_key))
            }
        });
        if remove_line {
            continue;
        }

        output.push_str(line);
        output.push_str(newline);
    }

    if !config_has_moved_ghostty_cursor_fields(&parse_toml_table_lossy(&output)) {
        return Ok(output);
    }

    let mut normalized = parsed_config.clone();
    if let Some(terminal) = normalized
        .get_mut("terminal")
        .and_then(TomlValue::as_table_mut)
    {
        for setting in MOVED_GHOSTTY_CURSOR_SETTINGS {
            terminal.remove(setting.old_key);
        }
    }
    toml::to_string_pretty(&normalized).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_migrated_config",
            format!("Could not serialize migrated Yazelix config: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })
}

fn parse_toml_table_lossy(raw: &str) -> Option<toml::Table> {
    toml::from_str::<toml::Table>(raw).ok()
}

fn config_has_moved_ghostty_cursor_fields(config: &Option<toml::Table>) -> bool {
    config
        .as_ref()
        .and_then(|config| config.get("terminal"))
        .and_then(TomlValue::as_table)
        .is_some_and(|terminal| {
            MOVED_GHOSTTY_CURSOR_SETTINGS
                .iter()
                .any(|setting| terminal.contains_key(setting.old_key))
        })
}

fn moved_ghostty_cursor_old_key(key: &str) -> bool {
    MOVED_GHOSTTY_CURSOR_SETTINGS
        .iter()
        .any(|setting| setting.old_key == key)
}

fn split_line_ending(line_with_newline: &str) -> (&str, &str) {
    if let Some(line) = line_with_newline.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = line_with_newline.strip_suffix('\n') {
        (line, "\n")
    } else {
        (line_with_newline, "")
    }
}

fn is_toml_section_header(trimmed: &str) -> bool {
    trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.starts_with("[[")
}

fn toml_assignment_key(line: &str) -> Option<String> {
    let before_comment = line.split_once('#').map_or(line, |(before, _)| before);
    let (key, _) = before_comment.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some(key.trim_matches('"').to_string())
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
    let apply = apply_runtime_materialization(&RuntimeMaterializationApplyRequest {
        config_file: plan.config_state.config_file.clone(),
        managed_config_path,
        state_path: request.state_path.clone(),
        config_hash: plan.config_state.config_hash.clone(),
        runtime_hash: plan.config_state.runtime_hash.clone(),
        expected_artifacts: plan.expected_artifacts.clone(),
    })?;

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

    let success_kind = if !force && plan.status == "repair_missing_artifacts" {
        RepairSuccessKind::RepairedMissingArtifacts
    } else {
        RepairSuccessKind::Repaired
    };
    let (result_status, success_lines) = match success_kind {
        RepairSuccessKind::RepairedMissingArtifacts => (
            "repaired_missing_artifacts".to_string(),
            vec!["✅ Repaired the missing generated runtime artifacts.".to_string()],
        ),
        RepairSuccessKind::Repaired => (
            "repaired".to_string(),
            vec![
                "✅ Generated runtime state repaired.".to_string(),
                "   Generated Yazi/Zellij state now matches the active runtime config.".to_string(),
            ],
        ),
    };

    RuntimeRepairDirective::Regenerate {
        reason,
        progress_message,
        missing_artifacts_detail_line,
        success_lines,
        result_status,
    }
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
) -> Result<String, CoreError> {
    let override_value = layout_override
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let layout = if let Some(layout) = override_value {
        layout.to_string()
    } else {
        managed_sidebar_layout_name(json_bool(config.get("enable_sidebar"), true)).to_string()
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

fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Io,
            "atomic_write_no_parent",
            "Cannot write a file without a parent directory",
            "Report this as a Yazelix internal error.",
            json!({ "path": path.to_string_lossy() }),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "atomic_write_mkdir",
            "Could not create parent directory for Yazelix config migration",
            "Check directory permissions, then retry.",
            parent.to_string_lossy(),
            source,
        )
    })?;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "migrated".into());
    let temp_path = parent.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        unique
    ));
    fs::write(&temp_path, content).map_err(|source| {
        CoreError::io(
            "atomic_write_file",
            "Could not write temporary Yazelix config migration file",
            "Check directory permissions, then retry.",
            temp_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temp_path, path).map_err(|source| {
        CoreError::io(
            "atomic_write_rename",
            "Could not replace Yazelix config during migration",
            "Check directory permissions, then retry.",
            path.to_string_lossy(),
            source,
        )
    })
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

    fn touch_plan_artifacts(plan: &RuntimeMaterializationPlanData) {
        for artifact in &plan.expected_artifacts {
            let path = Path::new(&artifact.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, "").unwrap();
        }
    }

    // Defends: repair evaluation returns a noop directive when the plan is noop and force is false.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn repair_evaluate_is_noop_when_plan_is_noop_and_not_forced() {
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
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
            yazi_dir.clone(),
            zellij_dir.clone(),
            zellij_layout_dir.clone(),
        ))
        .unwrap();
        touch_plan_artifacts(&plan);

        let evaluated =
            evaluate_runtime_materialization_repair(&RuntimeMaterializationRepairEvaluateRequest {
                plan: plan_request_for(
                    config_path,
                    runtime_dir,
                    state_path,
                    yazi_dir,
                    zellij_dir,
                    zellij_layout_dir,
                ),
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
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn repair_evaluate_regenerates_when_forced_even_if_plan_is_noop() {
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

        let plan_before = plan_runtime_materialization(&plan_request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
            yazi_dir.clone(),
            zellij_dir.clone(),
            zellij_layout_dir.clone(),
        ))
        .unwrap();
        touch_plan_artifacts(&plan_before);
        let plan_after = plan_runtime_materialization(&plan_request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
            yazi_dir.clone(),
            zellij_dir.clone(),
            zellij_layout_dir.clone(),
        ))
        .unwrap();
        assert_eq!(plan_after.status, "noop");

        let evaluated =
            evaluate_runtime_materialization_repair(&RuntimeMaterializationRepairEvaluateRequest {
                plan: plan_request_for(
                    config_path,
                    runtime_dir,
                    state_path,
                    yazi_dir,
                    zellij_dir,
                    zellij_layout_dir,
                ),
                force: true,
            })
            .unwrap();

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
