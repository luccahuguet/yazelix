// Test lane: default
//! Yazelix adapter for ratconfig deterministic `settings.jsonc` contracts.

use crate::bridge::{CoreError, ErrorClass};
use crate::settings_surface::read_settings_jsonc_value;
use ratconfig::contract::{
    ConfigContract, ContractChange, ContractError, join_jsonc_contract_text_from_version,
    read_jsonc_contract_state_text,
};
use ratconfig::migration::{MigrationError, MigrationOp};
use serde_json::{Value as JsonValue, json};
use std::path::Path;

pub const SETTINGS_CONTRACT_ID: &str = "yazelix.settings";
pub const SETTINGS_CONTRACT_STATE_PATH: &str = "ratconfig.contract";
const SETTINGS_CONTRACT_BASELINE_VERSION: u64 = 1;
pub const SETTINGS_CONTRACT_CURRENT_VERSION: u64 = 12;
const CHANGE_RENAME_EDITOR_SIDEBAR_TO_WORKSPACE_LEFT_SIDEBAR: &str =
    "rename-editor-sidebar-to-workspace-left-sidebar";
const CHANGE_REPLACE_NATIVE_MOVEMENT_DEFAULTS: &str = "replace-native-movement-defaults";
const CHANGE_ADD_CURRENT_DEFAULT_SETTINGS: &str = "add-current-default-settings";
const CHANGE_REPAIR_NATIVE_MOVEMENT_KEY_SPELLING: &str = "repair-native-movement-key-spelling";
const CHANGE_ENABLE_KITTY_KEYBOARD_PROTOCOL_DEFAULT: &str =
    "enable-kitty-keyboard-protocol-default";
const CHANGE_REPLACE_DEFAULT_BTM_POPUP_WITH_ZENITH: &str = "replace-default-btm-popup-with-zenith";
const CHANGE_MOVE_DEFAULT_ZENITH_POPUP_TO_INFORMATION_KEY: &str =
    "move-default-zenith-popup-to-information-key";
const CHANGE_ROUTE_DEFAULT_RIGHT_SIDEBAR_THROUGH_YZX_AGENT: &str =
    "route-default-right-sidebar-through-yzx-agent";
const CHANGE_REMOVE_RETIRED_CURSOR_WIDGET_TRAY_VALUE: &str =
    "remove-retired-cursor-widget-tray-value";
const CHANGE_REMOVE_CPU_RAM_FROM_DEFAULT_WIDGET_TRAY: &str =
    "remove-cpu-ram-from-default-widget-tray";
const CHANGE_ADD_SESSION_WIDGET_TRAY_VALUE: &str = "add-session-widget-tray-value";
pub const SETTINGS_CONTRACT_APPLIED_CHANGE_IDS: &[&str] = &[
    CHANGE_RENAME_EDITOR_SIDEBAR_TO_WORKSPACE_LEFT_SIDEBAR,
    CHANGE_REPLACE_NATIVE_MOVEMENT_DEFAULTS,
    CHANGE_ADD_CURRENT_DEFAULT_SETTINGS,
    CHANGE_REPAIR_NATIVE_MOVEMENT_KEY_SPELLING,
    CHANGE_ENABLE_KITTY_KEYBOARD_PROTOCOL_DEFAULT,
    CHANGE_REPLACE_DEFAULT_BTM_POPUP_WITH_ZENITH,
    CHANGE_MOVE_DEFAULT_ZENITH_POPUP_TO_INFORMATION_KEY,
    CHANGE_ROUTE_DEFAULT_RIGHT_SIDEBAR_THROUGH_YZX_AGENT,
    CHANGE_REMOVE_RETIRED_CURSOR_WIDGET_TRAY_VALUE,
    CHANGE_REMOVE_CPU_RAM_FROM_DEFAULT_WIDGET_TRAY,
    CHANGE_ADD_SESSION_WIDGET_TRAY_VALUE,
];
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
    let previous_state = read_jsonc_contract_state_text(raw, SETTINGS_CONTRACT_STATE_PATH)
        .map_err(|error| contract_error_to_core_error(source_path, error))?;
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
        state_changed: match previous_state {
            Some(previous) => previous != outcome.state,
            None => outcome.state_mutation != ratconfig::patch::PatchMutation::Unchanged,
        },
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
                CHANGE_RENAME_EDITOR_SIDEBAR_TO_WORKSPACE_LEFT_SIDEBAR,
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
                CHANGE_REPLACE_NATIVE_MOVEMENT_DEFAULTS,
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
            ContractChange::automatic(CHANGE_ADD_CURRENT_DEFAULT_SETTINGS, 3, 4, add_default_ops),
            ContractChange::automatic(
                CHANGE_REPAIR_NATIVE_MOVEMENT_KEY_SPELLING,
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
            ContractChange::automatic(
                CHANGE_ENABLE_KITTY_KEYBOARD_PROTOCOL_DEFAULT,
                5,
                6,
                vec![MigrationOp::Transform {
                    path: "zellij.support_kitty_keyboard_protocol".to_string(),
                    transform: enable_kitty_keyboard_protocol_default,
                }],
            ),
            ContractChange::automatic(
                CHANGE_REPLACE_DEFAULT_BTM_POPUP_WITH_ZENITH,
                6,
                7,
                vec![MigrationOp::Transform {
                    path: "zellij.custom_popups".to_string(),
                    transform: replace_default_btm_popup_with_zenith,
                }],
            ),
            ContractChange::automatic(
                CHANGE_MOVE_DEFAULT_ZENITH_POPUP_TO_INFORMATION_KEY,
                7,
                8,
                vec![MigrationOp::Transform {
                    path: "zellij.custom_popups".to_string(),
                    transform: move_default_zenith_popup_to_information_key,
                }],
            ),
            ContractChange::automatic(
                CHANGE_ROUTE_DEFAULT_RIGHT_SIDEBAR_THROUGH_YZX_AGENT,
                8,
                9,
                vec![MigrationOp::Transform {
                    path: "workspace.right_sidebar".to_string(),
                    transform: route_default_right_sidebar_through_yzx_agent,
                }],
            ),
            ContractChange::automatic(
                CHANGE_REMOVE_RETIRED_CURSOR_WIDGET_TRAY_VALUE,
                9,
                10,
                vec![MigrationOp::Transform {
                    path: "zellij.widget_tray".to_string(),
                    transform: remove_retired_cursor_widget_tray_value,
                }],
            ),
            ContractChange::automatic(
                CHANGE_REMOVE_CPU_RAM_FROM_DEFAULT_WIDGET_TRAY,
                10,
                11,
                vec![MigrationOp::Transform {
                    path: "zellij.widget_tray".to_string(),
                    transform: remove_cpu_ram_from_default_widget_tray,
                }],
            ),
            ContractChange::automatic(
                CHANGE_ADD_SESSION_WIDGET_TRAY_VALUE,
                11,
                12,
                vec![MigrationOp::Transform {
                    path: "zellij.widget_tray".to_string(),
                    transform: add_session_to_widget_tray,
                }],
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

fn enable_kitty_keyboard_protocol_default(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    let enabled = value
        .as_bool()
        .ok_or_else(|| "expected a boolean setting".to_string())?;
    if enabled {
        Ok(None)
    } else {
        Ok(Some(json!(true)))
    }
}

fn replace_default_btm_popup_with_zenith(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    let popups = value
        .as_array()
        .ok_or_else(|| "expected a custom popup array".to_string())?;
    let mut changed = false;
    let has_zenith = popups.iter().any(is_zenith_popup);
    let mut next = Vec::with_capacity(popups.len());

    for popup in popups {
        if is_default_btm_popup(popup) {
            changed = true;
            if !has_zenith {
                next.push(json!({
                    "id": "zenith",
                    "command": ["zenith"],
                    "keybindings": ["Alt Shift I"],
                    "keep_alive": true,
                }));
            }
        } else {
            next.push(popup.clone());
        }
    }

    if changed {
        Ok(Some(JsonValue::Array(next)))
    } else {
        Ok(None)
    }
}

fn is_zenith_popup(value: &JsonValue) -> bool {
    value
        .as_object()
        .and_then(|object| object.get("id"))
        .and_then(JsonValue::as_str)
        == Some("zenith")
}

fn move_default_zenith_popup_to_information_key(
    value: &JsonValue,
) -> Result<Option<JsonValue>, String> {
    let popups = value
        .as_array()
        .ok_or_else(|| "expected a custom popup array".to_string())?;
    let mut changed = false;
    let mut next = Vec::with_capacity(popups.len());

    for popup in popups {
        if is_default_zenith_popup_on_bottom_key(popup) {
            let mut migrated = popup.clone();
            migrated["keybindings"] = json!(["Alt Shift I"]);
            next.push(migrated);
            changed = true;
        } else {
            next.push(popup.clone());
        }
    }

    if changed {
        Ok(Some(JsonValue::Array(next)))
    } else {
        Ok(None)
    }
}

fn is_default_zenith_popup_on_bottom_key(value: &JsonValue) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("id").and_then(JsonValue::as_str) == Some("zenith")
        && string_array_equals(object.get("command"), &["zenith"])
        && string_array_equals(object.get("keybindings"), &["Alt Shift B"])
        && object
            .get("keep_alive")
            .map(|value| value.as_bool() == Some(true) || value.is_null())
            .unwrap_or(true)
}

fn route_default_right_sidebar_through_yzx_agent(
    value: &JsonValue,
) -> Result<Option<JsonValue>, String> {
    let Some(object) = value.as_object() else {
        return Err("expected a right sidebar object".to_string());
    };
    if object.get("command").and_then(JsonValue::as_str) != Some("codex")
        || !string_array_is_empty_or_absent(object.get("args"))
    {
        return Ok(None);
    }

    let mut next = object.clone();
    next.insert("command".to_string(), json!("yzx"));
    next.insert("args".to_string(), json!(["agent"]));
    Ok(Some(JsonValue::Object(next)))
}

fn remove_retired_cursor_widget_tray_value(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| "expected a widget_tray array".to_string())?;
    let mut changed = false;
    let mut next = Vec::with_capacity(items.len());

    for item in items {
        if item.as_str() == Some("cursor") {
            changed = true;
        } else {
            next.push(item.clone());
        }
    }

    if changed {
        Ok(Some(JsonValue::Array(next)))
    } else {
        Ok(None)
    }
}

fn remove_cpu_ram_from_default_widget_tray(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    let _ = value
        .as_array()
        .ok_or_else(|| "expected a widget_tray array".to_string())?;
    if string_array_equals(
        Some(value),
        &["editor", "shell", "term", "codex_usage", "cpu", "ram"],
    ) {
        Ok(Some(json!(["editor", "shell", "term", "codex_usage"])))
    } else {
        Ok(None)
    }
}

fn add_session_to_widget_tray(value: &JsonValue) -> Result<Option<JsonValue>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| "expected a widget_tray array".to_string())?;
    if items.iter().any(|item| item.as_str() == Some("session")) {
        return Ok(None);
    }

    let mut next = Vec::with_capacity(items.len() + 1);
    next.push(json!("session"));
    next.extend(items.iter().cloned());
    Ok(Some(JsonValue::Array(next)))
}

fn string_array_is_empty_or_absent(value: Option<&JsonValue>) -> bool {
    match value {
        None => true,
        Some(JsonValue::Array(values)) => values.is_empty(),
        Some(JsonValue::Null) => true,
        Some(_) => false,
    }
}

fn is_default_btm_popup(value: &JsonValue) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("id").and_then(JsonValue::as_str) == Some("btm")
        && string_array_equals(object.get("command"), &["btm"])
        && string_array_equals(object.get("keybindings"), &["Alt Shift B"])
        && object
            .get("keep_alive")
            .map(|value| value.as_bool() == Some(true) || value.is_null())
            .unwrap_or(true)
}

fn string_array_equals(value: Option<&JsonValue>, expected: &[&str]) -> bool {
    let Some(values) = value.and_then(JsonValue::as_array) else {
        return false;
    };
    values.len() == expected.len()
        && values
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.as_str() == Some(*expected))
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
        "Update the reported stale settings manually, then retry. Use `yzx reset config` only as a blunt fallback. Yazelix only applies deterministic contract rewrites when every affected path is unambiguous.",
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratconfig::contract::plan_contract_migration;
    use ratconfig::jsonc::{get_json_path, parse_jsonc_value};

    // Defends: every ratconfig version bump has a real linear migration or manual blocker.
    #[test]
    fn settings_contract_versions_are_linear_and_nonempty() {
        let defaults_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../settings_default.jsonc");
        let defaults = read_settings_jsonc_value(&defaults_path).unwrap();
        let contract = settings_contract_for_defaults(&defaults);
        let plan = plan_contract_migration(&contract, SETTINGS_CONTRACT_BASELINE_VERSION).unwrap();

        assert_eq!(plan.to_version, SETTINGS_CONTRACT_CURRENT_VERSION);
        assert_eq!(
            plan.changes.len() as u64,
            SETTINGS_CONTRACT_CURRENT_VERSION - SETTINGS_CONTRACT_BASELINE_VERSION
        );
        let change_ids = contract
            .changes
            .iter()
            .map(|change| change.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(change_ids, SETTINGS_CONTRACT_APPLIED_CHANGE_IDS);
        assert!(
            contract
                .changes
                .iter()
                .all(|change| !change.operations.is_empty() || !change.manual_steps.is_empty())
        );
    }

    // Regression: retiring the cursor status widget must not strand joined configs with a rejected enum value.
    #[test]
    fn removes_retired_cursor_status_widget_from_joined_config() {
        let contract = settings_contract_for_defaults(&json!({}));
        let raw = r#"{
  "zellij": {
    "widget_tray": ["editor", "cursor", "ram"]
  }
}
"#;

        let migrated =
            join_jsonc_contract_text_from_version(raw, &contract, SETTINGS_CONTRACT_STATE_PATH, 9)
                .unwrap();
        let value = parse_jsonc_value(&migrated.text).unwrap();

        assert_eq!(
            get_json_path(&value, "zellij.widget_tray"),
            Some(&json!(["session", "editor", "ram"]))
        );
        assert_eq!(
            get_json_path(&value, "ratconfig.contract.version"),
            Some(&json!(SETTINGS_CONTRACT_CURRENT_VERSION))
        );
        assert!(
            migrated
                .applied_changes
                .iter()
                .any(|change| change.id == "remove-retired-cursor-widget-tray-value")
        );
    }

    // Regression: only old default-shaped status trays lose CPU/RAM; explicit custom opt-ins are preserved.
    #[test]
    fn removes_cpu_ram_only_from_default_widget_tray() {
        assert_eq!(
            remove_cpu_ram_from_default_widget_tray(&json!([
                "editor",
                "shell",
                "term",
                "codex_usage",
                "cpu",
                "ram"
            ]))
            .unwrap(),
            Some(json!(["editor", "shell", "term", "codex_usage"]))
        );
        assert_eq!(
            remove_cpu_ram_from_default_widget_tray(&json!(["editor", "workspace", "cpu"]))
                .unwrap(),
            None
        );
    }

    // Regression: the session name moved from hardcoded bar text into the widget tray, so existing configs must keep showing it after migration.
    #[test]
    fn adds_session_to_existing_widget_tray() {
        assert_eq!(
            add_session_to_widget_tray(&json!(["editor", "workspace", "cpu"])).unwrap(),
            Some(json!(["session", "editor", "workspace", "cpu"]))
        );
        assert_eq!(
            add_session_to_widget_tray(&json!(["session", "editor"])).unwrap(),
            None
        );
    }

    // Regression: joined user settings from the previous contract version preserve the old visible session label by adding the new session widget token.
    #[test]
    fn migrates_existing_widget_tray_to_include_session_widget() {
        let contract = settings_contract_for_defaults(&json!({}));
        let raw = r#"{
  "zellij": {
    "widget_tray": ["editor", "shell", "term", "codex_usage"]
  }
}
"#;

        let migrated =
            join_jsonc_contract_text_from_version(raw, &contract, SETTINGS_CONTRACT_STATE_PATH, 11)
                .unwrap();
        let value = parse_jsonc_value(&migrated.text).unwrap();

        assert_eq!(
            get_json_path(&value, "zellij.widget_tray"),
            Some(&json!([
                "session",
                "editor",
                "shell",
                "term",
                "codex_usage"
            ]))
        );
        assert!(
            migrated
                .applied_changes
                .iter()
                .any(|change| change.id == "add-session-widget-tray-value")
        );
    }

    // Regression: the old default process popup must migrate to Zenith when users carry an explicit default-shaped btm entry.
    #[test]
    fn rewrites_default_btm_popup_to_zenith() {
        let migrated = replace_default_btm_popup_with_zenith(&json!([
            {
                "id": "btm",
                "command": ["btm"],
                "keybindings": ["Alt Shift B"],
                "keep_alive": true
            }
        ]))
        .unwrap()
        .unwrap();

        assert_eq!(
            migrated,
            json!([
                {
                    "id": "zenith",
                    "command": ["zenith"],
                    "keybindings": ["Alt Shift I"],
                    "keep_alive": true
                }
            ])
        );
    }

    // Regression: the short-lived default Zenith binding follows the information mnemonic without touching user-owned popup shapes.
    #[test]
    fn rewrites_default_zenith_popup_to_information_key() {
        let migrated = move_default_zenith_popup_to_information_key(&json!([
            {
                "id": "zenith",
                "command": ["zenith"],
                "keybindings": ["Alt Shift B"],
                "keep_alive": true
            },
            {
                "id": "zenith",
                "command": ["zenith", "--layout", "process"],
                "keybindings": ["Alt Shift B"],
                "keep_alive": true
            }
        ]))
        .unwrap()
        .unwrap();

        assert_eq!(
            migrated,
            json!([
                {
                    "id": "zenith",
                    "command": ["zenith"],
                    "keybindings": ["Alt Shift I"],
                    "keep_alive": true
                },
                {
                    "id": "zenith",
                    "command": ["zenith", "--layout", "process"],
                    "keybindings": ["Alt Shift B"],
                    "keep_alive": true
                }
            ])
        );
    }

    // Regression: the old raw Codex sidebar default routes through yzx agent so missing Codex can render an actionable pane.
    #[test]
    fn routes_default_right_sidebar_through_yzx_agent() {
        let migrated = route_default_right_sidebar_through_yzx_agent(&json!({
            "command": "codex",
            "args": [],
            "width_percent": 40
        }))
        .unwrap()
        .unwrap();

        assert_eq!(
            migrated,
            json!({
                "command": "yzx",
                "args": ["agent"],
                "width_percent": 40
            })
        );
    }

    // Defends: user-owned Codex sidebar arguments are not rewritten as if they were the old default.
    #[test]
    fn preserves_custom_codex_right_sidebar_args() {
        let migrated = route_default_right_sidebar_through_yzx_agent(&json!({
            "command": "codex",
            "args": ["--model", "gpt-5.5"],
            "width_percent": 40
        }))
        .unwrap();

        assert_eq!(migrated, None);
    }

    // Defends: user-owned btm customizations are not rewritten merely because they share the old default id.
    #[test]
    fn preserves_custom_btm_popup() {
        let migrated = replace_default_btm_popup_with_zenith(&json!([
            {
                "id": "btm",
                "command": ["btm", "--basic"],
                "keybindings": ["Alt Shift B"],
                "keep_alive": true
            }
        ]))
        .unwrap();

        assert_eq!(migrated, None);
    }

    // Regression: Home Manager-generated compact JSON can already carry the current contract state without needing a write-only formatting repair.
    #[test]
    fn current_contract_state_is_idempotent_even_when_compact() {
        let defaults_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../settings_default.jsonc");
        let applied_change_ids = serde_json::to_string(&SETTINGS_CONTRACT_APPLIED_CHANGE_IDS)
            .expect("serialize applied change ids");
        let raw = format!(
            r#"{{
  "ratconfig": {{"contract":{{"applied_change_ids":{applied_change_ids},"contract_id":"{SETTINGS_CONTRACT_ID}","schema_version":1,"version":{SETTINGS_CONTRACT_CURRENT_VERSION}}}}}
}}
"#
        );

        let outcome =
            reconcile_settings_contract_text(Path::new("settings.jsonc"), &raw, &defaults_path)
                .expect("reconcile settings contract");

        assert!(!outcome.changed());
        assert!(outcome.applied_change_ids.is_empty());
    }
}
