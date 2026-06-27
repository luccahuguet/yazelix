use super::*;
use std::collections::BTreeMap;

pub(super) const ZELLIJ_KEYBINDINGS_FIELD_PATH: &str = "zellij.keybindings";
pub(super) const ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH: &str = "zellij.native_keybindings";
pub(super) const YAZI_KEYBINDINGS_FIELD_PATH: &str = "yazi.keybindings";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticKeybindingObject {
    entries: BTreeMap<String, Vec<String>>,
    malformed_entries: Vec<String>,
    malformed_object: Option<String>,
}

pub(super) fn keybinding_map_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    let object = semantic_keybinding_object_for_field(field);
    let parent_path = field.path.as_str();
    let actions = keybinding_actions_for_parent_path(parent_path);
    let supported_actions = actions
        .iter()
        .map(|action| action.local_id)
        .collect::<BTreeSet<_>>();
    let unsupported_entries = object
        .entries
        .keys()
        .filter(|action| !supported_actions.contains(action.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let mut lines = vec![
        Line::from(Span::styled(
            field.path.clone(),
            config_key_style().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        detail_line("state", state_label(field.state)),
        detail_line("current", &field.current_value),
        detail_line("default", &field.default_value),
        detail_line("type", &field.kind),
        detail_line("takes effect", &field.apply_status.label),
        detail_line("after save", &field.apply_status.detail),
    ];
    if !field.validation.is_empty() {
        lines.push(detail_line("validation", &field.validation));
    }
    if field.rebuild_required {
        lines.push(detail_line("rebuild", "required"));
    }
    if let Some(message) = object.malformed_object {
        lines.push(detail_line("invalid", &message));
    }
    if !object.malformed_entries.is_empty() {
        lines.push(detail_line("invalid", &object.malformed_entries.join("; ")));
    }
    if !unsupported_entries.is_empty() {
        lines.push(detail_line("unsupported", &unsupported_entries.join(", ")));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        keybinding_surface_title(parent_path),
        metadata_key_style().add_modifier(Modifier::BOLD),
    )));

    for action in actions {
        let default_keys = action.default_keys;
        let explicit_keys = object.entries.get(action.local_id);
        let current_label = explicit_keys
            .map(|keys| keybinding_keys_label(keys.as_slice()))
            .unwrap_or_else(|| keybinding_keys_label(default_keys));
        let source_label = if let Some(keys) = explicit_keys {
            if keys.is_empty() {
                "disabled"
            } else {
                "remapped"
            }
        } else {
            "default"
        };

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            action.label.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(detail_line("action", action.id));
        lines.push(detail_line(
            "current",
            &format!("{current_label} ({source_label})"),
        ));
        lines.push(detail_line("default", &keybinding_keys_label(default_keys)));
        for (label, value) in keybinding_action_metadata_lines(parent_path, action.local_id) {
            lines.push(detail_line(label, &value));
        }
        lines.push(detail_line("backend", action.backend.as_str()));
        if action.disable_policy.empty_binding_list_allowed() {
            lines.push(detail_line("disable", "empty list disables this action"));
        } else {
            lines.push(detail_line("disable", "binding required"));
        }
    }

    if !field.description.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(field.description.clone()));
    }

    lines
}

pub(super) fn keybinding_action_detail_lines(
    field: &ConfigUiField,
    action: &'static YazelixActionMetadata,
) -> Vec<Line<'static>> {
    let parent_path =
        keybinding_parent_path_for_field_path(&field.path).unwrap_or(field.path.as_str());
    let mut lines = default_field_detail_lines(field);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        action.label.to_string(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(detail_line("action", action.id));
    lines.push(detail_line(
        "keys",
        &keybinding_keys_label_from_field(field),
    ));
    for (label, value) in keybinding_action_metadata_lines(parent_path, action.local_id) {
        lines.push(detail_line(label, &value));
    }
    lines.push(detail_line("backend", action.backend.as_str()));
    lines.push(detail_line("command", action.generated_command));
    if action.disable_policy.empty_binding_list_allowed() {
        lines.push(detail_line("disable", "empty list disables this action"));
    } else {
        lines.push(detail_line("disable", "binding required"));
    }
    lines
}

pub(super) fn semantic_keybinding_object_for_field(
    field: &ConfigUiField,
) -> SemanticKeybindingObject {
    if !matches!(
        field.state,
        ConfigUiValueState::Explicit | ConfigUiValueState::Invalid
    ) {
        return SemanticKeybindingObject {
            entries: BTreeMap::new(),
            malformed_entries: Vec::new(),
            malformed_object: None,
        };
    }

    let value = match serde_json::from_str::<JsonValue>(&field.edit_value) {
        Ok(value) => value,
        Err(source) => {
            return SemanticKeybindingObject {
                entries: BTreeMap::new(),
                malformed_entries: Vec::new(),
                malformed_object: Some(format!("not valid JSON: {source}")),
            };
        }
    };
    let Some(object) = value.as_object() else {
        return SemanticKeybindingObject {
            entries: BTreeMap::new(),
            malformed_entries: Vec::new(),
            malformed_object: Some("must be a JSON object".to_string()),
        };
    };

    let mut entries = BTreeMap::new();
    let mut malformed_entries = Vec::new();
    for (action, raw_keys) in object {
        let Some(values) = raw_keys.as_array() else {
            malformed_entries.push(format!("{action}: not a list"));
            continue;
        };
        let mut keys = Vec::with_capacity(values.len());
        let mut invalid = false;
        for value in values {
            let Some(key) = value.as_str() else {
                invalid = true;
                break;
            };
            keys.push(key.to_string());
        }
        if invalid {
            malformed_entries.push(format!("{action}: contains a non-string key"));
        } else {
            entries.insert(action.clone(), keys);
        }
    }

    SemanticKeybindingObject {
        entries,
        malformed_entries,
        malformed_object: None,
    }
}

pub(super) fn keybinding_keys_label(keys: &[impl AsRef<str>]) -> String {
    if keys.is_empty() {
        "disabled".to_string()
    } else {
        keys.iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub(super) fn keybinding_keys_label_from_field(field: &ConfigUiField) -> String {
    serde_json::from_str::<Vec<String>>(&field.edit_value)
        .map(|keys| keybinding_keys_label(keys.as_slice()))
        .unwrap_or_else(|_| field.current_value.clone())
}

pub(super) fn append_keybinding_action_fields(
    fields: &mut Vec<ConfigUiField>,
    contract_fields: &BTreeMap<String, ConfigUiContractField>,
    config_owner: ConfigUiPathOwner,
    active_value: &JsonValue,
    default_value: &JsonValue,
    blocking_paths: &BTreeSet<String>,
) -> Result<(), CoreError> {
    append_keybinding_surface_action_fields(
        fields,
        contract_fields,
        config_owner,
        active_value,
        default_value,
        blocking_paths,
        ZELLIJ_KEYBINDINGS_FIELD_PATH,
    )?;
    append_keybinding_surface_action_fields(
        fields,
        contract_fields,
        config_owner,
        active_value,
        default_value,
        blocking_paths,
        YAZI_KEYBINDINGS_FIELD_PATH,
    )?;
    Ok(())
}

pub(super) fn append_keybinding_surface_action_fields(
    fields: &mut Vec<ConfigUiField>,
    contract_fields: &BTreeMap<String, ConfigUiContractField>,
    config_owner: ConfigUiPathOwner,
    active_value: &JsonValue,
    default_value: &JsonValue,
    blocking_paths: &BTreeSet<String>,
    parent_path: &'static str,
) -> Result<(), CoreError> {
    let Some(parent_field) = contract_fields.get(parent_path) else {
        return Ok(());
    };
    let apply_mode = apply_mode_for_config_owner(config_owner, parent_field)?;
    for action in keybinding_actions_for_parent_path(parent_path) {
        let path = format!("{parent_path}.{}", action.local_id);
        let default = get_json_path(default_value, &path)
            .cloned()
            .unwrap_or_else(|| keybinding_default_value(action));
        fields.push(build_field_row(
            SETTINGS_SOURCE_ID,
            &path,
            "keybindings",
            "string_list",
            get_json_path(active_value, &path),
            Some(&default),
            action.label.to_string(),
            Vec::new(),
            parent_field.validation.clone(),
            parent_field.rebuild_required,
            apply_mode,
            blocking_paths.contains(&path) || blocking_paths.contains(parent_path),
            ConfigUiEditBehavior::FriendlyStringList,
        ));
    }
    Ok(())
}

pub(super) fn keybinding_default_value(action: &YazelixActionMetadata) -> JsonValue {
    JsonValue::Array(
        action
            .default_keys
            .iter()
            .map(|key| JsonValue::String((*key).to_string()))
            .collect(),
    )
}

pub(super) fn keybinding_actions_for_parent_path(
    parent_path: &str,
) -> Vec<&'static YazelixActionMetadata> {
    match parent_path {
        ZELLIJ_KEYBINDINGS_FIELD_PATH => ZELLIJ_ACTIONS.iter().map(|spec| &spec.action).collect(),
        ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH => ZELLIJ_NATIVE_KEYBINDINGS
            .iter()
            .map(|spec| &spec.action)
            .collect(),
        YAZI_KEYBINDINGS_FIELD_PATH => YAZI_ACTIONS.iter().map(|spec| &spec.action).collect(),
        _ => Vec::new(),
    }
}

pub(super) fn keybinding_action_metadata_lines(
    parent_path: &str,
    local_id: &str,
) -> Vec<(&'static str, String)> {
    if parent_path == ZELLIJ_KEYBINDINGS_FIELD_PATH
        && let Some(spec) = ZELLIJ_ACTIONS
            .iter()
            .find(|spec| spec.action.local_id == local_id)
    {
        return vec![("mode", spec.mode.to_string())];
    }
    if parent_path == YAZI_KEYBINDINGS_FIELD_PATH
        && let Some(spec) = YAZI_ACTIONS
            .iter()
            .find(|spec| spec.action.local_id == local_id)
    {
        return vec![
            ("section", spec.section.to_string()),
            ("keymap", spec.keymap_list.to_string()),
        ];
    }
    if parent_path == ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH
        && let Some(spec) = ZELLIJ_NATIVE_KEYBINDINGS
            .iter()
            .find(|spec| spec.action.local_id == local_id)
    {
        return vec![(
            "mode",
            spec.blocks
                .iter()
                .map(|block| block.mode)
                .collect::<Vec<_>>()
                .join(", "),
        )];
    }
    Vec::new()
}

pub(super) fn keybinding_surface_title(parent_path: &str) -> &'static str {
    match parent_path {
        ZELLIJ_KEYBINDINGS_FIELD_PATH => "Yazelix Zellij actions",
        ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH => "Yazelix native Zellij policy",
        YAZI_KEYBINDINGS_FIELD_PATH => "Yazelix Yazi actions",
        _ => "Yazelix actions",
    }
}

pub(super) fn is_keybinding_map_field_path(path: &str) -> bool {
    matches!(
        path,
        ZELLIJ_KEYBINDINGS_FIELD_PATH
            | ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH
            | YAZI_KEYBINDINGS_FIELD_PATH
    )
}

pub(super) fn keybinding_parent_path_for_field_path(path: &str) -> Option<&'static str> {
    for parent_path in [
        ZELLIJ_KEYBINDINGS_FIELD_PATH,
        ZELLIJ_NATIVE_KEYBINDINGS_FIELD_PATH,
        YAZI_KEYBINDINGS_FIELD_PATH,
    ] {
        let Some(action) = path
            .strip_prefix(parent_path)
            .and_then(|rest| rest.strip_prefix('.'))
        else {
            continue;
        };
        if keybinding_actions_for_parent_path(parent_path)
            .iter()
            .any(|metadata| metadata.local_id == action)
        {
            return Some(parent_path);
        }
    }
    None
}

pub(super) fn keybinding_action_metadata_for_field_path(
    path: &str,
) -> Option<&'static YazelixActionMetadata> {
    let parent_path = keybinding_parent_path_for_field_path(path)?;
    let action_id = path.strip_prefix(parent_path)?.strip_prefix('.')?;
    keybinding_actions_for_parent_path(parent_path)
        .into_iter()
        .find(|metadata| metadata.local_id == action_id)
}
