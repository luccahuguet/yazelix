//! Yazelix adapter for ratconfig deterministic `settings.jsonc` contracts.

use crate::bridge::{CoreError, ErrorClass};
use crate::settings_surface::read_settings_jsonc_value;
use serde_json::{Value as JsonValue, json};
use std::path::Path;
use yazelix_ratconfig::contract::{
    ConfigContract, ContractChange, ContractError, join_jsonc_contract_text_from_version,
};
use yazelix_ratconfig::migration::{MigrationError, MigrationOp};

pub const SETTINGS_CONTRACT_ID: &str = "yazelix.settings";
pub const SETTINGS_CONTRACT_STATE_PATH: &str = "ratconfig.contract";
const SETTINGS_CONTRACT_BASELINE_VERSION: u64 = 1;
const SETTINGS_CONTRACT_CURRENT_VERSION: u64 = 5;
const OPTIONAL_ADDITIVE_DEFAULT_PATHS: &[&str] = &["zellij.custom_popups"];

const LEGACY_SIDEBAR_SETTING_RENAMES: &[(&str, &str)] = &[
    ("editor.sidebar_command", "workspace.left_sidebar.command"),
    ("editor.sidebar_args", "workspace.left_sidebar.args"),
    (
        "editor.sidebar_width_percent",
        "workspace.left_sidebar.width_percent",
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsContractReconcileOutcome {
    pub text: String,
    pub applied_change_ids: Vec<String>,
    pub state_changed: bool,
}

impl SettingsContractReconcileOutcome {
    pub fn changed(&self) -> bool {
        self.state_changed || !self.applied_change_ids.is_empty()
    }
}

pub fn reconcile_settings_contract_text(
    source_path: &Path,
    raw: &str,
    default_main_config: &Path,
) -> Result<SettingsContractReconcileOutcome, CoreError> {
    let defaults = read_settings_jsonc_value(default_main_config)?;
    let contract = settings_contract_for_defaults(&defaults);
    let outcome = join_jsonc_contract_text_from_version(
        raw,
        &contract,
        SETTINGS_CONTRACT_STATE_PATH,
        SETTINGS_CONTRACT_BASELINE_VERSION,
    )
    .map_err(|error| contract_error_to_core_error(source_path, error))?;

    Ok(SettingsContractReconcileOutcome {
        text: outcome.text,
        applied_change_ids: outcome
            .applied_changes
            .iter()
            .map(|change| change.id.clone())
            .collect(),
        state_changed: outcome.state_mutation != yazelix_ratconfig::patch::PatchMutation::Unchanged,
    })
}

fn settings_contract_for_defaults(defaults: &JsonValue) -> ConfigContract {
    let mut add_default_ops = Vec::new();
    collect_add_default_operations(defaults, &mut Vec::new(), &mut add_default_ops);

    ConfigContract {
        id: SETTINGS_CONTRACT_ID.to_string(),
        baseline_version: SETTINGS_CONTRACT_BASELINE_VERSION,
        current_version: SETTINGS_CONTRACT_CURRENT_VERSION,
        changes: vec![
            ContractChange::automatic(
                "rename-editor-sidebar-to-workspace-left-sidebar",
                1,
                2,
                LEGACY_SIDEBAR_SETTING_RENAMES
                    .iter()
                    .map(|(from, to)| MigrationOp::Rename {
                        from: (*from).to_string(),
                        to: (*to).to_string(),
                    })
                    .collect(),
            ),
            ContractChange::automatic(
                "replace-native-movement-defaults",
                2,
                3,
                vec![
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_tab_left".to_string(),
                        transform: replace_move_tab_left_default,
                    },
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_tab_right".to_string(),
                        transform: replace_move_tab_right_default,
                    },
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_pane_down".to_string(),
                        transform: replace_move_pane_down_default,
                    },
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_pane_up".to_string(),
                        transform: replace_move_pane_up_default,
                    },
                ],
            ),
            ContractChange::automatic("add-current-default-settings", 3, 4, add_default_ops),
            ContractChange::automatic(
                "repair-native-movement-key-spelling",
                4,
                5,
                vec![
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_tab_left".to_string(),
                        transform: lowercase_move_tab_left_default,
                    },
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_tab_right".to_string(),
                        transform: lowercase_move_tab_right_default,
                    },
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_pane_down".to_string(),
                        transform: lowercase_move_pane_down_default,
                    },
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_pane_up".to_string(),
                        transform: lowercase_move_pane_up_default,
                    },
                    MigrationOp::Transform {
                        path: "zellij.native_keybindings.move_mode_unbind".to_string(),
                        transform: clear_move_mode_unbind_default,
                    },
                ],
            ),
        ],
    }
}

fn collect_add_default_operations(
    value: &JsonValue,
    path: &mut Vec<String>,
    operations: &mut Vec<MigrationOp>,
) {
    let Some(object) = value.as_object() else {
        let setting_path = path.join(".");
        if !setting_path.is_empty()
            && !OPTIONAL_ADDITIVE_DEFAULT_PATHS.contains(&setting_path.as_str())
        {
            operations.push(MigrationOp::AddDefault {
                path: setting_path,
                value: value.clone(),
            });
        }
        return;
    };

    for (key, child) in object {
        path.push(key.clone());
        collect_add_default_operations(child, path, operations);
        path.pop();
    }
}

fn replace_move_tab_left_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Shift H", "Ctrl Alt H")
}

fn replace_move_tab_right_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Shift L", "Ctrl Alt L")
}

fn replace_move_pane_down_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Shift J", "Ctrl Alt J")
}

fn replace_move_pane_up_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Shift K", "Ctrl Alt K")
}

fn lowercase_move_tab_left_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Alt H", "Ctrl Alt h")
}

fn lowercase_move_tab_right_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Alt L", "Ctrl Alt l")
}

fn lowercase_move_pane_down_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Alt J", "Ctrl Alt j")
}

fn lowercase_move_pane_up_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding(value, "Ctrl Alt K", "Ctrl Alt k")
}

fn clear_move_mode_unbind_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding_with_value(value, "Ctrl h", json!([]))
}

fn replace_default_keybinding(
    value: &JsonValue,
    old_default: &str,
    current_default: &str,
) -> Result<Option<JsonValue>, String> {
    replace_default_keybinding_with_value(value, old_default, json!([current_default]))
}

fn replace_default_keybinding_with_value(
    value: &JsonValue,
    old_default: &str,
    current_value: JsonValue,
) -> Result<Option<JsonValue>, String> {
    let values = value
        .as_array()
        .ok_or_else(|| "expected a keybinding array".to_string())?;
    if values.len() == 1 && values[0].as_str() == Some(old_default) {
        Ok(Some(current_value))
    } else {
        Ok(None)
    }
}

fn contract_error_to_core_error(source_path: &Path, error: ContractError) -> CoreError {
    let code = contract_error_code(&error);
    CoreError::classified(
        ErrorClass::Config,
        code,
        format!(
            "Yazelix could not reconcile settings.jsonc with the {SETTINGS_CONTRACT_ID} contract."
        ),
        "Update the reported stale settings manually, then retry. Yazelix only applies deterministic contract rewrites when every affected path is unambiguous.",
        json!({
            "path": source_path.display().to_string(),
            "state_path": SETTINGS_CONTRACT_STATE_PATH,
            "detail": contract_error_detail(&error),
        }),
    )
}

fn contract_error_code(error: &ContractError) -> &'static str {
    match error {
        ContractError::JsoncMigration {
            error: MigrationError::DestinationExists { .. },
            ..
        } => "settings_contract_destination_exists",
        ContractError::JsoncMigration {
            error: MigrationError::OverlappingPaths { .. },
            ..
        } => "settings_contract_overlapping_paths",
        ContractError::ManualRequired { .. } => "settings_contract_manual_action_required",
        ContractError::InvalidState { .. } => "settings_contract_invalid_state",
        ContractError::ContractMismatch { .. } => "settings_contract_mismatch",
        ContractError::UnsupportedStateVersion { .. } => "settings_contract_unsupported_version",
        ContractError::MissingMigration { .. } => "settings_contract_missing_migration",
        _ => "settings_contract_reconciliation_failed",
    }
}

fn contract_error_detail(error: &ContractError) -> JsonValue {
    match error {
        ContractError::InvalidContract { detail } => json!({
            "type": "invalid_contract",
            "detail": detail,
        }),
        ContractError::InvalidState { state_path, detail } => json!({
            "type": "invalid_state",
            "state_path": state_path,
            "detail": detail,
        }),
        ContractError::NotJoined { state_path } => json!({
            "type": "not_joined",
            "state_path": state_path,
        }),
        ContractError::ContractMismatch { expected, found } => json!({
            "type": "contract_mismatch",
            "expected": expected,
            "found": found,
        }),
        ContractError::UnsupportedStateVersion {
            version,
            baseline_version,
            current_version,
        } => json!({
            "type": "unsupported_state_version",
            "version": version,
            "baseline_version": baseline_version,
            "current_version": current_version,
        }),
        ContractError::MissingMigration {
            from_version,
            target_version,
        } => json!({
            "type": "missing_migration",
            "from_version": from_version,
            "target_version": target_version,
        }),
        ContractError::ManualRequired { plan } => json!({
            "type": "manual_required",
            "from_version": plan.from_version,
            "to_version": plan.to_version,
            "manual_steps": plan.manual_steps.iter().map(|step| {
                json!({
                    "id": step.id,
                    "path": step.path,
                    "reason": step.reason,
                    "remediation": step.remediation,
                })
            }).collect::<Vec<_>>(),
        }),
        ContractError::JsoncMigration { change_id, error } => json!({
            "type": "jsonc_migration",
            "change_id": change_id,
            "migration_error": migration_error_detail(error),
        }),
        ContractError::JsoncPatch(error) => json!({
            "type": "jsonc_patch",
            "error": format!("{error:?}"),
        }),
        ContractError::TomlMigration { change_id, error } => json!({
            "type": "toml_migration",
            "change_id": change_id,
            "error": format!("{error:?}"),
        }),
        ContractError::TomlPatch(error) => json!({
            "type": "toml_patch",
            "error": format!("{error:?}"),
        }),
    }
}

fn migration_error_detail(error: &MigrationError) -> JsonValue {
    match error {
        MigrationError::Patch(error) => json!({
            "type": "patch",
            "error": format!("{error:?}"),
        }),
        MigrationError::DestinationExists { from, to } => json!({
            "type": "destination_exists",
            "from": from,
            "to": to,
        }),
        MigrationError::OverlappingPaths { from, to } => json!({
            "type": "overlapping_paths",
            "from": from,
            "to": to,
        }),
        MigrationError::TransformFailed { path, message } => json!({
            "type": "transform_failed",
            "path": path,
            "message": message,
        }),
    }
}
