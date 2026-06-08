use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const CUSTOM_POPUPS_FIELD_PATH: &str = "zellij.custom_popups";
const ADD_CUSTOM_POPUP_FIELD_PATH: &str = "zellij.custom_popups.$add";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ConfigUiCustomPopup {
    id: String,
    command: Vec<String>,
    keybindings: Vec<String>,
    keep_alive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CustomPopupPath {
    Add,
    Overview { id: String },
    Id { id: String },
    Command { id: String },
    Keybindings { id: String },
    KeepAlive { id: String },
}

pub(super) fn append_custom_popup_fields(
    fields: &mut Vec<ConfigUiField>,
    contract_fields: &BTreeMap<String, ConfigUiContractField>,
    config_owner: ConfigUiPathOwner,
    active_value: &JsonValue,
    default_value: &JsonValue,
    blocking_paths: &BTreeSet<String>,
) -> Result<(), CoreError> {
    let Some(parent_field) = contract_fields.get(CUSTOM_POPUPS_FIELD_PATH) else {
        return Ok(());
    };
    let apply_mode = if config_owner == ConfigUiPathOwner::HomeManager {
        RuntimeApplyMode::PackageHomeManagerActivation
    } else {
        apply_mode_for_contract_field(parent_field)?
    };
    fields.push(build_field_row(
        ADD_CUSTOM_POPUP_FIELD_PATH,
        "workspace",
        "string",
        None,
        None,
        "Add a custom popup by id; the initial command argv defaults to the id.".to_string(),
        Vec::new(),
        parent_field.validation.clone(),
        parent_field.rebuild_required,
        apply_mode,
        blocking_paths.contains(CUSTOM_POPUPS_FIELD_PATH),
        ConfigUiEditBehavior::Default,
    ));

    let current_value = get_json_path(active_value, CUSTOM_POPUPS_FIELD_PATH);
    let default_value = get_json_path(default_value, CUSTOM_POPUPS_FIELD_PATH);
    let current_by_id = custom_popup_map(current_value);
    let default_by_id = custom_popup_map(default_value);
    let effective_popups = custom_popup_list(current_value.or(default_value));

    for popup in effective_popups {
        let current = current_by_id.get(&popup.id);
        let default = default_by_id.get(&popup.id);
        push_custom_popup_rows(
            fields,
            parent_field,
            apply_mode,
            blocking_paths.contains(CUSTOM_POPUPS_FIELD_PATH),
            current,
            default,
            &popup,
        );
    }

    Ok(())
}

fn push_custom_popup_rows(
    fields: &mut Vec<ConfigUiField>,
    parent_field: &ConfigUiContractField,
    apply_mode: RuntimeApplyMode,
    has_blocking_diagnostic: bool,
    current: Option<&ConfigUiCustomPopup>,
    default: Option<&ConfigUiCustomPopup>,
    popup: &ConfigUiCustomPopup,
) {
    let overview_path = custom_popup_overview_path(&popup.id);
    let current_overview = current.map(custom_popup_value);
    let default_overview = default.map(custom_popup_value);
    fields.push(build_field_row(
        &overview_path,
        "workspace",
        "custom_popup",
        current_overview.as_ref(),
        default_overview.as_ref(),
        format!("Custom popup {}", popup.id),
        Vec::new(),
        parent_field.validation.clone(),
        parent_field.rebuild_required,
        apply_mode,
        has_blocking_diagnostic,
        ConfigUiEditBehavior::StructuredOnly {
            notice: "Select a custom popup child row to edit it, or press u on the popup row to remove it."
                .to_string(),
        },
    ));

    let current_id = current.map(|popup| JsonValue::String(popup.id.clone()));
    let default_id = default.map(|popup| JsonValue::String(popup.id.clone()));
    push_custom_popup_field_row(
        fields,
        parent_field,
        apply_mode,
        has_blocking_diagnostic,
        &format!("{overview_path}.id"),
        "string",
        "Rename this custom popup id".to_string(),
        current_id.as_ref(),
        default_id.as_ref(),
        ConfigUiEditBehavior::Default,
    );

    let current_command = current.map(|popup| string_list_value(&popup.command));
    let default_command = default.map(|popup| string_list_value(&popup.command));
    push_custom_popup_field_row(
        fields,
        parent_field,
        apply_mode,
        has_blocking_diagnostic,
        &format!("{overview_path}.command"),
        "string_list",
        "Set this custom popup command argv".to_string(),
        current_command.as_ref(),
        default_command.as_ref(),
        ConfigUiEditBehavior::FriendlyStringList,
    );

    let current_keybindings = current.map(|popup| string_list_value(&popup.keybindings));
    let default_keybindings = default.map(|popup| string_list_value(&popup.keybindings));
    push_custom_popup_field_row(
        fields,
        parent_field,
        apply_mode,
        has_blocking_diagnostic,
        &format!("{overview_path}.keybindings"),
        "string_list",
        "Set this custom popup Zellij keybindings list".to_string(),
        current_keybindings.as_ref(),
        default_keybindings.as_ref(),
        ConfigUiEditBehavior::FriendlyStringList,
    );

    let current_keep_alive = current.map(|popup| JsonValue::Bool(popup.keep_alive));
    let default_keep_alive = default.map(|popup| JsonValue::Bool(popup.keep_alive));
    push_custom_popup_field_row(
        fields,
        parent_field,
        apply_mode,
        has_blocking_diagnostic,
        &format!("{overview_path}.keep_alive"),
        "bool",
        "Hide this popup on focused toggle instead of closing its process".to_string(),
        current_keep_alive.as_ref(),
        default_keep_alive.as_ref(),
        ConfigUiEditBehavior::Default,
    );
}

fn push_custom_popup_field_row(
    fields: &mut Vec<ConfigUiField>,
    parent_field: &ConfigUiContractField,
    apply_mode: RuntimeApplyMode,
    has_blocking_diagnostic: bool,
    path: &str,
    kind: &str,
    description: String,
    current: Option<&JsonValue>,
    default: Option<&JsonValue>,
    edit_behavior: ConfigUiEditBehavior,
) {
    fields.push(build_field_row(
        path,
        "workspace",
        kind,
        current,
        default,
        description,
        Vec::new(),
        parent_field.validation.clone(),
        parent_field.rebuild_required,
        apply_mode,
        has_blocking_diagnostic,
        edit_behavior,
    ));
}

pub(super) fn custom_popup_detail_lines(field: &ConfigUiField) -> Option<Vec<Line<'static>>> {
    if field.path == CUSTOM_POPUPS_FIELD_PATH {
        return Some(custom_popup_parent_detail_lines(field));
    }
    let path = custom_popup_path(&field.path)?;
    match path {
        CustomPopupPath::Add => Some(custom_popup_add_detail_lines(field)),
        CustomPopupPath::Overview { .. } => Some(custom_popup_overview_detail_lines(field)),
        CustomPopupPath::Id { .. }
        | CustomPopupPath::Command { .. }
        | CustomPopupPath::Keybindings { .. }
        | CustomPopupPath::KeepAlive { .. } => Some(default_field_detail_lines(field)),
    }
}

fn custom_popup_parent_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let mut lines = default_field_detail_lines(field);
    let popups = custom_popup_list_from_field(field);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Custom popup definitions",
        metadata_key_style().add_modifier(Modifier::BOLD),
    )));
    if popups.is_empty() {
        lines.push(detail_line("popups", "none"));
    }
    for popup in popups {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            popup.id.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(detail_line("command", &popup.command.join(" ")));
        lines.push(detail_line(
            "keybindings",
            &keybinding_keys_label(popup.keybindings.as_slice()),
        ));
        lines.push(detail_line("keep alive", bool_label(popup.keep_alive)));
    }
    lines
}

fn custom_popup_add_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let mut lines = default_field_detail_lines(field);
    lines.push(Line::from(""));
    lines.push(detail_line("add", "Enter a stable id such as gitui"));
    lines.push(detail_line(
        "initial command",
        "new popup command argv starts as [id]",
    ));
    lines.push(detail_line("initial keys", "disabled"));
    lines.push(detail_line("initial keep alive", "false"));
    lines
}

fn custom_popup_overview_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let mut lines = default_field_detail_lines(field);
    if let Some(popup) = custom_popup_from_field_value(field) {
        lines.push(Line::from(""));
        lines.push(detail_line("id", &popup.id));
        lines.push(detail_line("command", &popup.command.join(" ")));
        lines.push(detail_line(
            "keybindings",
            &keybinding_keys_label(popup.keybindings.as_slice()),
        ));
        lines.push(detail_line("keep alive", bool_label(popup.keep_alive)));
        lines.push(detail_line("remove", "press u on this row"));
    }
    lines
}

pub(super) fn custom_popups_parent_path_for_field_path(path: &str) -> Option<&'static str> {
    custom_popup_path(path).map(|_| CUSTOM_POPUPS_FIELD_PATH)
}

pub(super) fn custom_popup_path(path: &str) -> Option<CustomPopupPath> {
    if path == ADD_CUSTOM_POPUP_FIELD_PATH {
        return Some(CustomPopupPath::Add);
    }
    let rest = path
        .strip_prefix(CUSTOM_POPUPS_FIELD_PATH)?
        .strip_prefix('.')?;
    let mut parts = rest.split('.');
    let id = parts.next()?.to_string();
    if id.is_empty() || id == "$add" {
        return None;
    }
    match (parts.next(), parts.next()) {
        (None, None) => Some(CustomPopupPath::Overview { id }),
        (Some("id"), None) => Some(CustomPopupPath::Id { id }),
        (Some("command"), None) => Some(CustomPopupPath::Command { id }),
        (Some("keybindings"), None) => Some(CustomPopupPath::Keybindings { id }),
        (Some("keep_alive"), None) => Some(CustomPopupPath::KeepAlive { id }),
        _ => None,
    }
}

pub(super) fn custom_popup_list_value_after_write(
    root: &JsonValue,
    default_value: &JsonValue,
    setting_path: &str,
    value: &JsonValue,
) -> Result<Option<JsonValue>, CoreError> {
    let Some(path) = custom_popup_path(setting_path) else {
        return Ok(None);
    };
    let mut popups = custom_popup_effective_list(root, default_value);
    match path {
        CustomPopupPath::Add => {
            let id = string_value(value, setting_path)?.trim().to_string();
            popups.push(ConfigUiCustomPopup {
                command: vec![id.clone()],
                id,
                keybindings: Vec::new(),
                keep_alive: false,
            });
        }
        CustomPopupPath::Id { id } => {
            let next_id = string_value(value, setting_path)?.trim().to_string();
            let popup = find_custom_popup_mut(&mut popups, &id, setting_path)?;
            popup.id = next_id;
        }
        CustomPopupPath::Command { id } => {
            let command = string_list_from_value(value, setting_path)?;
            let popup = find_custom_popup_mut(&mut popups, &id, setting_path)?;
            popup.command = command;
        }
        CustomPopupPath::Keybindings { id } => {
            let keybindings = string_list_from_value(value, setting_path)?;
            let popup = find_custom_popup_mut(&mut popups, &id, setting_path)?;
            popup.keybindings = keybindings;
        }
        CustomPopupPath::KeepAlive { id } => {
            let keep_alive = value.as_bool().ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Usage,
                    "invalid_custom_popup_keep_alive_edit",
                    format!("{setting_path} must be true or false."),
                    "Use the boolean toggle for custom popup keep_alive.",
                    json!({ "path": setting_path, "actual": value }),
                )
            })?;
            let popup = find_custom_popup_mut(&mut popups, &id, setting_path)?;
            popup.keep_alive = keep_alive;
        }
        CustomPopupPath::Overview { .. } => {
            return Err(CoreError::classified(
                ErrorClass::Usage,
                "unsupported_custom_popup_overview_edit",
                "Custom popup overview rows are not directly editable.",
                "Select id, command, keybindings, or keep_alive under the popup.",
                json!({ "path": setting_path }),
            ));
        }
    }
    Ok(Some(custom_popup_list_value(&popups)))
}

pub(super) fn custom_popup_list_value_after_unset(
    root: &JsonValue,
    default_value: &JsonValue,
    setting_path: &str,
) -> Result<Option<JsonValue>, CoreError> {
    let Some(path) = custom_popup_path(setting_path) else {
        return Ok(None);
    };
    let CustomPopupPath::Overview { id } = path else {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "unsupported_custom_popup_field_unset",
            "Only custom popup overview rows can be removed.",
            "Press u on the popup id row to remove the whole custom popup.",
            json!({ "path": setting_path }),
        ));
    };
    let mut popups = custom_popup_effective_list(root, default_value);
    let original_len = popups.len();
    popups.retain(|popup| popup.id != id);
    if popups.len() == original_len {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "missing_custom_popup",
            format!("No custom popup with id {id:?} exists."),
            "Select an existing custom popup row.",
            json!({ "path": setting_path, "id": id }),
        ));
    }
    Ok(Some(custom_popup_list_value(&popups)))
}

fn custom_popup_effective_list(
    root: &JsonValue,
    default_value: &JsonValue,
) -> Vec<ConfigUiCustomPopup> {
    let current = get_json_path(root, CUSTOM_POPUPS_FIELD_PATH);
    custom_popup_list(current.or(Some(default_value)))
}

fn custom_popup_list_from_field(field: &ConfigUiField) -> Vec<ConfigUiCustomPopup> {
    if matches!(
        field.state,
        ConfigUiValueState::Explicit | ConfigUiValueState::Defaulted | ConfigUiValueState::Invalid
    ) && let Ok(value) = serde_json::from_str::<JsonValue>(&field.edit_value)
    {
        return custom_popup_list(Some(&value));
    }
    Vec::new()
}

fn custom_popup_from_field_value(field: &ConfigUiField) -> Option<ConfigUiCustomPopup> {
    serde_json::from_str::<JsonValue>(&field.edit_value)
        .ok()
        .as_ref()
        .and_then(|value| custom_popup_from_value(value))
}

fn custom_popup_map(value: Option<&JsonValue>) -> BTreeMap<String, ConfigUiCustomPopup> {
    custom_popup_list(value)
        .into_iter()
        .map(|popup| (popup.id.clone(), popup))
        .collect()
}

fn custom_popup_list(value: Option<&JsonValue>) -> Vec<ConfigUiCustomPopup> {
    let Some(values) = value.and_then(JsonValue::as_array) else {
        return Vec::new();
    };
    let mut seen = BTreeSet::new();
    values
        .iter()
        .filter_map(custom_popup_from_value)
        .filter(|popup| seen.insert(popup.id.clone()))
        .collect()
}

fn custom_popup_from_value(value: &JsonValue) -> Option<ConfigUiCustomPopup> {
    let object = value.as_object()?;
    let id = object.get("id")?.as_str()?.to_string();
    let command = object
        .get("command")?
        .as_array()?
        .iter()
        .map(JsonValue::as_str)
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let keybindings = object
        .get("keybindings")
        .and_then(JsonValue::as_array)
        .map(|values| {
            values
                .iter()
                .map(JsonValue::as_str)
                .collect::<Option<Vec<_>>>()
                .map(|values| values.into_iter().map(ToOwned::to_owned).collect())
        })
        .unwrap_or_else(|| Some(Vec::new()))?;
    let keep_alive = object
        .get("keep_alive")
        .and_then(JsonValue::as_bool)
        .unwrap_or_else(|| default_keep_alive(&id, &command));
    Some(ConfigUiCustomPopup {
        id,
        command,
        keybindings,
        keep_alive,
    })
}

fn find_custom_popup_mut<'a>(
    popups: &'a mut [ConfigUiCustomPopup],
    id: &str,
    setting_path: &str,
) -> Result<&'a mut ConfigUiCustomPopup, CoreError> {
    popups
        .iter_mut()
        .find(|popup| popup.id == id)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Usage,
                "missing_custom_popup",
                format!("No custom popup with id {id:?} exists."),
                "Select an existing custom popup row.",
                json!({ "path": setting_path, "id": id }),
            )
        })
}

fn custom_popup_list_value(popups: &[ConfigUiCustomPopup]) -> JsonValue {
    JsonValue::Array(popups.iter().map(custom_popup_value).collect())
}

fn custom_popup_value(popup: &ConfigUiCustomPopup) -> JsonValue {
    json!({
        "id": popup.id,
        "command": popup.command,
        "keybindings": popup.keybindings,
        "keep_alive": popup.keep_alive,
    })
}

fn string_list_value(values: &[String]) -> JsonValue {
    JsonValue::Array(values.iter().cloned().map(JsonValue::String).collect())
}

fn custom_popup_overview_path(id: &str) -> String {
    format!("{CUSTOM_POPUPS_FIELD_PATH}.{id}")
}

fn string_value<'a>(value: &'a JsonValue, setting_path: &str) -> Result<&'a str, CoreError> {
    value.as_str().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_custom_popup_string_edit",
            format!("{setting_path} must be a string."),
            "Use the text editor for this custom popup field.",
            json!({ "path": setting_path, "actual": value }),
        )
    })
}

fn string_list_from_value(value: &JsonValue, setting_path: &str) -> Result<Vec<String>, CoreError> {
    let values = value.as_array().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_custom_popup_string_list_edit",
            format!("{setting_path} must be a string list."),
            "Use the list editor for this custom popup field.",
            json!({ "path": setting_path, "actual": value }),
        )
    })?;
    values
        .iter()
        .map(|value| {
            value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Usage,
                    "invalid_custom_popup_string_list_edit",
                    format!("{setting_path} must contain only strings."),
                    "Use the list editor for this custom popup field.",
                    json!({ "path": setting_path, "actual": value }),
                )
            })
        })
        .collect()
}

fn default_keep_alive(id: &str, command: &[String]) -> bool {
    id == "zenith" && command.len() == 1 && command[0] == "zenith"
}

fn bool_label(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
