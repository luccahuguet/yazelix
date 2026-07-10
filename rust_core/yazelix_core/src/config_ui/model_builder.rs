use super::*;
use std::collections::BTreeMap;

const POPUP_COMMANDS_FIELD_PATH: &str = "zellij.popup_commands";
const YAZELIX_SCHEMA_EXTENSION_KEY: &str = "x-yazelix";
const BUILTIN_POPUP_COMMANDS: &[(&str, &str)] = &[
    ("bottom_popup", "Bottom popup command"),
    ("top_popup", "Top popup command"),
    ("menu", "Menu popup command"),
];

pub fn build_config_ui_model(request: &ConfigUiRequest) -> Result<ConfigUiModel, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let schema = read_json_file(
        &paths.settings_schema_path,
        "read_settings_schema",
        "Could not read the Yazelix settings schema",
    )?;
    let schema_tab_order = schema_tabs(&schema, YAZELIX_SCHEMA_EXTENSION_KEY, DEFAULT_TABS);
    let ui_metadata =
        load_config_ui_metadata(&config_ui_metadata_path(&paths.settings_schema_path))?;
    ensure_ui_metadata_tabs_match_schema(&ui_metadata.tabs, &schema_tab_order)?;
    let cursor_component_enabled = runtime_component_enabled(&request.runtime_dir, "cursors")?;
    let tabs = if cursor_component_enabled {
        ui_metadata.tabs.clone()
    } else {
        ui_metadata
            .tabs
            .iter()
            .filter(|tab| tab.as_str() != "cursors")
            .cloned()
            .collect()
    };
    let active_config_path = active_config_path(&paths, request.config_override.as_deref());
    let active_config_exists = path_present(&active_config_path);
    let config_owner = classify_path_owner(&active_config_path, active_config_exists);
    let active_main_value = if active_config_exists {
        read_active_config_value(&active_config_path)?
    } else {
        JsonValue::Object(JsonMap::new())
    };
    ensure_root_object(&active_config_path, &active_main_value)?;
    let active_value = compose_config_ui_value(
        active_main_value,
        if cursor_component_enabled {
            read_cursor_config_value(&paths.user_cursor_config)?
        } else {
            JsonValue::Object(JsonMap::new())
        },
    )?;

    let default_raw = render_default_settings_jsonc(&paths.default_config_path)?;
    let default_main_value = parse_jsonc_value(&paths.default_config_path, &default_raw)?;
    let default_value = compose_config_ui_value(
        default_main_value,
        if cursor_component_enabled {
            read_default_cursor_config_value(&paths.default_cursor_config_path)?
        } else {
            JsonValue::Object(JsonMap::new())
        },
    )?;
    ensure_root_object(&paths.default_config_path, &default_value)?;

    let mars_config_path = user_config_paths::mars_config(&request.config_dir);
    let packaged_mars_config_path = user_config_paths::packaged_mars_config(&request.runtime_dir);
    let mars_config_exists = path_present(&mars_config_path);
    let mars_config_owner = classify_path_owner(&mars_config_path, mars_config_exists);
    let packaged_mars_config = read_native_config_text(
        &packaged_mars_config_path,
        "read_packaged_mars_config",
        "Could not read the packaged Mars config",
    )?;
    let active_mars_config = if mars_config_exists {
        read_native_config_text(
            &mars_config_path,
            "read_mars_config",
            "Could not read ~/.config/yazelix/mars/config.toml",
        )?
    } else {
        String::new()
    };
    let mars_rows = build_toml_document_fields(ConfigUiTomlDocumentSpec {
        source_id: MARS_SOURCE_ID,
        tab: MARS_TAB,
        section_label: "Mars native config",
        current_toml: &active_mars_config,
        default_toml: (!mars_config_exists).then_some(packaged_mars_config.as_str()),
        validation: "valid Mars TOML value",
        rebuild_required: false,
        apply_status: ConfigUiApplyStatus {
            summary: "next Mars window".to_string(),
            label: "next Mars window".to_string(),
            detail: "Mars reads this complete native config when a new window opens.".to_string(),
            pending: true,
        },
    })
    .map_err(|message| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_mars_config",
            message,
            "Fix ~/.config/yazelix/mars/config.toml or remove it to use the packaged complete config.",
            json!({ "path": mars_config_path.display().to_string() }),
        )
    })?;

    let contract_fields = load_contract_fields(&paths.contract_path)?;
    let diagnostics = if active_config_exists {
        collect_config_diagnostics(&active_config_path, &paths)?
    } else {
        Vec::new()
    };
    let blocking_paths = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.blocking)
        .map(|diagnostic| diagnostic.path.clone())
        .collect::<BTreeSet<_>>();

    let mut fields = Vec::new();
    for field in contract_fields.values() {
        let metadata = field_ui_metadata(&ui_metadata, &field.path)?;
        let current = get_json_path(&active_value, &field.path);
        let default = get_json_path(&default_value, &field.path)
            .cloned()
            .or_else(|| field.default_value.clone());
        let apply_mode = apply_mode_for_config_owner(config_owner, field)?;
        fields.push(build_field_row(
            SETTINGS_SOURCE_ID,
            &field.path,
            &metadata.tab,
            &field.kind,
            current,
            default.as_ref(),
            field_description(field, metadata),
            field.allowed_values.clone(),
            field.validation.clone(),
            field.rebuild_required,
            apply_mode,
            blocking_paths.contains(&field.path),
            edit_behavior_for_field_path(&field.path),
        ));
    }
    append_keybinding_action_fields(
        &mut fields,
        &contract_fields,
        config_owner,
        &active_value,
        &default_value,
        &blocking_paths,
    )?;
    append_builtin_popup_command_fields(
        &mut fields,
        &contract_fields,
        config_owner,
        &active_value,
        &default_value,
        &blocking_paths,
    )?;
    append_custom_popup_fields(
        &mut fields,
        &contract_fields,
        config_owner,
        &active_value,
        &default_value,
        &blocking_paths,
    )?;
    fields.extend(mars_rows.fields);

    if cursor_component_enabled {
        let cursor_definition_names = cursor_definition_names(&active_value, &default_value);
        let cursor_enabled_names = cursor_enabled_names(&active_value, &default_value)
            .into_iter()
            .filter(|name| cursor_definition_names.contains(name))
            .collect::<Vec<_>>();
        for mut schema_field in collect_cursor_schema_fields(&schema) {
            if fields.iter().any(|field| field.path == schema_field.path) {
                continue;
            }
            enrich_cursor_schema_field(
                &mut schema_field,
                &cursor_definition_names,
                &cursor_enabled_names,
            );
            let metadata = field_ui_metadata(&ui_metadata, &schema_field.path)?;
            let current = get_json_path(&active_value, &schema_field.path);
            let default = get_json_path(&default_value, &schema_field.path);
            fields.push(build_field_row(
                CURSORS_SOURCE_ID,
                &schema_field.path,
                &metadata.tab,
                &schema_field.kind,
                current,
                default,
                metadata.help.clone(),
                schema_field.allowed_values,
                String::new(),
                false,
                RuntimeApplyMode::ShellTerminalRestart,
                blocking_paths.contains(&schema_field.path),
                edit_behavior_for_field_path(&schema_field.path),
            ));
        }
    }

    fields.sort_by(|left, right| {
        tab_index(&tabs, &left.tab)
            .cmp(&tab_index(&tabs, &right.tab))
            .then_with(|| left.path.cmp(&right.path))
    });

    let home_dir = home_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let native_config_statuses = map_native_statuses(&classify_native_config_statuses(
        &NativeConfigStatusRequest {
            xdg_config_home: xdg_config_home_from_env(&home_dir),
            config_dir: request.config_dir.clone(),
            runtime_dir: request.runtime_dir.clone(),
            state_dir,
            active_terminal: crate::terminal_variant::active_terminal_from_runtime_dir(
                &request.runtime_dir,
            )?,
            settings_home_manager_read_only: config_owner == ConfigUiPathOwner::HomeManager,
        },
    ));

    Ok(ConfigUiModel {
        active_config_path: active_config_path.clone(),
        cursor_config_path: paths.user_cursor_config.clone(),
        default_cursor_config_path: paths.default_cursor_config_path.clone(),
        active_config_exists,
        config_owner,
        config_read_only: path_is_read_only(&active_config_path),
        sources: collect_config_sources(
            &tabs,
            &active_config_path,
            active_config_exists,
            config_owner,
            &paths.user_cursor_config,
            cursor_component_enabled,
            &mars_config_path,
        ),
        tabs,
        tab_list_tables: BTreeMap::from([(MARS_TAB.to_string(), mars_rows.list_table)]),
        fields,
        file_actions: vec![ConfigUiFileAction {
            source_id: MARS_SOURCE_ID.to_string(),
            action_id: MARS_CONFIG_ACTION_ID.to_string(),
            tab: MARS_TAB.to_string(),
            label: "mars/config.toml".to_string(),
            description: "Create or edit the complete native Mars config.".to_string(),
            path: mars_config_path.clone(),
            exists: mars_config_exists,
            read_only: mars_config_owner == ConfigUiPathOwner::HomeManager
                || path_is_read_only(&mars_config_path),
            create_if_missing: true,
            disabled_reason: None,
        }],
        sidecars: collect_sidecars(&request.config_dir),
        native_config_statuses,
        diagnostics,
        theme_switcher: None,
    })
}

pub(super) fn apply_contract_path_for_setting_path(setting_path: &str) -> &str {
    keybinding_parent_path_for_field_path(setting_path)
        .or_else(|| popup_commands_parent_path_for_field_path(setting_path))
        .or_else(|| custom_popups_parent_path_for_field_path(setting_path))
        .unwrap_or(setting_path)
}

fn active_config_path(paths: &PrimaryConfigPaths, config_override: Option<&str>) -> PathBuf {
    match config_override.map(str::trim).filter(|raw| !raw.is_empty()) {
        Some(raw) => PathBuf::from(raw),
        None => paths.user_config.clone(),
    }
}

fn read_active_config_value(path: &Path) -> Result<JsonValue, CoreError> {
    if is_settings_config_path(path) {
        return read_settings_jsonc_value(path);
    }

    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_active_config",
            "Could not read the active Yazelix config",
            "Fix permissions or choose a readable config path, then retry.",
            path.display().to_string(),
            source,
        )
    })?;
    let table = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_toml",
            format!(
                "Could not parse the active Yazelix config at {}: {source}.",
                path.display()
            ),
            "Fix the TOML syntax in the reported file and retry.",
            json!({ "path": path.display().to_string() }),
        )
    })?;
    toml_value_to_json(&toml::Value::Table(table)).map_err(|message| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_toml_value",
            format!("Could not convert active Yazelix config TOML to JSON: {message}."),
            "Fix the TOML value in the reported file and retry.",
            json!({ "path": path.display().to_string() }),
        )
    })
}

fn compose_config_ui_value(
    mut main_value: JsonValue,
    cursor_value: JsonValue,
) -> Result<JsonValue, CoreError> {
    let Some(object) = main_value.as_object_mut() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "settings_jsonc_not_object",
            "Yazelix settings must contain a JSON object.",
            "Replace the settings file with a valid object, then retry.",
            json!({}),
        ));
    };
    object.insert("cursors".to_string(), cursor_value);
    Ok(main_value)
}

fn read_cursor_config_value(path: &Path) -> Result<JsonValue, CoreError> {
    if !path.exists() {
        return Ok(JsonValue::Object(JsonMap::new()));
    }
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_cursor_config",
            "Could not read the Yazelix cursor settings",
            "Fix permissions for ~/.config/yazelix_cursors/settings.jsonc, then retry.",
            path.display().to_string(),
            source,
        )
    })?;
    parse_jsonc_value(path, &raw)
}

fn read_default_cursor_config_value(path: &Path) -> Result<JsonValue, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_default_cursor_config",
            "Could not read the default Yazelix cursor settings",
            "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml.",
            path.display().to_string(),
            source,
        )
    })?;
    let registry = CursorRegistry::parse_str(path, &raw)?;
    let rendered = render_cursor_settings_jsonc(&registry);
    parse_jsonc_value(path, &rendered)
}

fn ensure_root_object(path: &Path, value: &JsonValue) -> Result<(), CoreError> {
    if value.is_object() {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Config,
        "settings_jsonc_not_object",
        "Yazelix settings must contain a JSON object.",
        "Replace the settings file with a valid object, then retry.",
        json!({ "path": path.display().to_string() }),
    ))
}

fn read_json_file(path: &Path, code: &'static str, message: &str) -> Result<JsonValue, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            code,
            message,
            "Reinstall Yazelix so the runtime metadata exists and is readable.",
            path.display().to_string(),
            source,
        )
    })?;
    serde_json::from_str(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_settings_schema_json",
            format!(
                "Could not parse {SETTINGS_SCHEMA_FILENAME} at {}: {source}.",
                path.display()
            ),
            "Reinstall Yazelix so the runtime includes a valid settings schema.",
            json!({ "path": path.display().to_string() }),
        )
    })
}

fn load_contract_fields(path: &Path) -> Result<BTreeMap<String, ConfigUiContractField>, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_contract",
            "Could not read the Yazelix config contract",
            "Reinstall Yazelix so the runtime includes config_metadata/main_config_contract.toml.",
            path.display().to_string(),
            source,
        )
    })?;
    let contract = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_config_contract",
            format!(
                "Could not parse the Yazelix config contract at {}: {source}.",
                path.display()
            ),
            "Reinstall Yazelix so the runtime includes a valid config contract.",
            json!({ "path": path.display().to_string() }),
        )
    })?;
    config_contract_fields_from_toml(&contract).map_err(|message| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_config_contract",
            format!(
                "Invalid Yazelix config contract at {}: {message}.",
                path.display()
            ),
            "Reinstall Yazelix so the runtime includes the current config contract.",
            json!({ "path": path.display().to_string() }),
        )
    })
}

fn apply_mode_for_contract_field(
    field: &ConfigUiContractField,
) -> Result<RuntimeApplyMode, CoreError> {
    field
        .apply_mode
        .parse::<RuntimeApplyMode>()
        .map_err(|message| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_apply_mode",
                format!("Config contract field {} has {message}.", field.path),
                "Reinstall Yazelix so the runtime includes a valid config contract.",
                json!({ "field": field.path }),
            )
        })
}

pub(super) fn apply_mode_for_config_owner(
    config_owner: ConfigUiPathOwner,
    field: &ConfigUiContractField,
) -> Result<RuntimeApplyMode, CoreError> {
    if config_owner == ConfigUiPathOwner::HomeManager {
        Ok(RuntimeApplyMode::PackageHomeManagerActivation)
    } else {
        apply_mode_for_contract_field(field)
    }
}

fn config_ui_metadata_path(settings_schema_path: &Path) -> PathBuf {
    settings_schema_path.with_file_name(CONFIG_UI_METADATA_FILENAME)
}

fn load_config_ui_metadata(path: &Path) -> Result<ConfigUiMetadata, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_config_ui_metadata",
            "Could not read the Yazelix config UI metadata",
            "Reinstall Yazelix so the runtime includes config_metadata/config_ui_metadata.toml.",
            path.display().to_string(),
            source,
        )
    })?;
    let metadata = toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_config_ui_metadata",
            format!(
                "Could not parse the Yazelix config UI metadata at {}: {source}.",
                path.display()
            ),
            "Reinstall Yazelix so the runtime includes a valid config UI metadata file.",
            json!({ "path": path.display().to_string() }),
        )
    })?;
    config_ui_metadata_from_toml(&metadata).map_err(|message| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_config_ui_metadata",
            format!(
                "Invalid Yazelix config UI metadata at {}: {message}.",
                path.display()
            ),
            "Reinstall Yazelix so the runtime includes current config UI metadata.",
            json!({ "path": path.display().to_string() }),
        )
    })
}

fn ensure_ui_metadata_tabs_match_schema(
    metadata_tabs: &[String],
    schema_tabs: &[String],
) -> Result<(), CoreError> {
    if metadata_tabs == schema_tabs {
        return Ok(());
    }

    Err(CoreError::classified(
        ErrorClass::Config,
        "config_ui_tab_order_mismatch",
        "The Yazelix config UI metadata tab order does not match the settings schema.",
        "Reinstall Yazelix so config_metadata/config_ui_metadata.toml and yazelix_settings.schema.json come from the same version.",
        json!({
            "metadata_tabs": metadata_tabs,
            "schema_tabs": schema_tabs,
        }),
    ))
}

fn field_ui_metadata<'a>(
    metadata: &'a ConfigUiMetadata,
    path: &str,
) -> Result<&'a ConfigUiFieldMetadata, CoreError> {
    metadata.fields.get(path).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Config,
            "missing_config_ui_field_metadata",
            format!("The Yazelix config UI metadata is missing field {path}."),
            "Reinstall Yazelix so the config UI metadata covers the current settings surface.",
            json!({ "field": path }),
        )
    })
}

fn collect_config_diagnostics(
    config_path: &Path,
    paths: &PrimaryConfigPaths,
) -> Result<Vec<ConfigUiDiagnostic>, CoreError> {
    let request = NormalizeConfigRequest {
        config_path: config_path.to_path_buf(),
        default_config_path: paths.default_config_path.clone(),
        contract_path: paths.contract_path.clone(),
        include_missing: true,
    };

    match crate::config_normalize::normalize_config(&request) {
        Ok(data) => Ok(map_diagnostics(
            data.diagnostic_report.doctor_diagnostics.as_slice(),
        )),
        Err(error) if error.code() == "unsupported_config" => {
            let report = serde_json::from_value::<ConfigDiagnosticReport>(error.details())
                .map_err(|source| {
                    CoreError::classified(
                        ErrorClass::Internal,
                        "invalid_config_ui_diagnostic_report",
                        format!("Could not decode config diagnostics for the config UI: {source}"),
                        "Rebuild or reinstall Yazelix so the Rust config helpers agree.",
                        json!({ "config_path": config_path.display().to_string() }),
                    )
                })?;
            Ok(map_diagnostics(report.doctor_diagnostics.as_slice()))
        }
        Err(error) => Err(error),
    }
}

fn map_diagnostics(diagnostics: &[ConfigDiagnostic]) -> Vec<ConfigUiDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| ConfigUiDiagnostic {
            path: diagnostic.path.clone(),
            status: diagnostic.status.clone(),
            headline: diagnostic.headline.clone(),
            blocking: diagnostic.blocking,
            detail_lines: diagnostic.detail_lines.clone(),
        })
        .collect()
}

fn map_native_statuses(statuses: &[NativeConfigStatusEntry]) -> Vec<ConfigUiNativeStatus> {
    statuses
        .iter()
        .map(|status| ConfigUiNativeStatus {
            surface: status.surface.clone(),
            tool: status.tool.clone(),
            description: status.description.clone(),
            status: status.status.clone(),
            label: status.label.clone(),
            severity: status_code_for_entry(status)
                .map(|code| code.doctor_severity())
                .unwrap_or("info")
                .to_string(),
            active_path: status.active_path.clone(),
            managed_path: status.managed_path.clone(),
            native_paths: status.native_paths.clone(),
            generated_path: status.generated_path.clone(),
            allowed_action: status.allowed_action.clone(),
            read_only_reason: status.read_only_reason.clone(),
        })
        .collect()
}

fn collect_cursor_schema_fields(schema: &JsonValue) -> Vec<ConfigUiSchemaField> {
    let Some(cursors) = schema
        .get("properties")
        .and_then(|properties| properties.get("cursors"))
    else {
        return Vec::new();
    };
    collect_config_ui_schema_fields(cursors, "cursors")
}

fn enrich_cursor_schema_field(
    field: &mut ConfigUiSchemaField,
    definition_names: &[String],
    enabled_names: &[String],
) {
    match field.path.as_str() {
        "cursors.enabled_cursors" => {
            field.allowed_values = definition_names.to_vec();
        }
        "cursors.settings.trail" => {
            field.allowed_values = vec!["none".to_string(), "random".to_string()];
            field.allowed_values.extend_from_slice(enabled_names);
        }
        _ => {}
    }
}

fn cursor_definition_names(active: &JsonValue, default: &JsonValue) -> Vec<String> {
    let definitions = get_json_path(active, "cursors.cursor")
        .and_then(JsonValue::as_array)
        .filter(|values| !values.is_empty())
        .or_else(|| {
            get_json_path(default, "cursors.cursor")
                .and_then(JsonValue::as_array)
                .filter(|values| !values.is_empty())
        });
    let Some(definitions) = definitions else {
        return Vec::new();
    };
    definitions
        .iter()
        .filter_map(|definition| definition.get("name").and_then(JsonValue::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn cursor_enabled_names(active: &JsonValue, default: &JsonValue) -> Vec<String> {
    let enabled = get_json_path(active, "cursors.enabled_cursors")
        .and_then(JsonValue::as_array)
        .filter(|values| !values.is_empty())
        .or_else(|| {
            get_json_path(default, "cursors.enabled_cursors")
                .and_then(JsonValue::as_array)
                .filter(|values| !values.is_empty())
        });
    let Some(enabled) = enabled else {
        return Vec::new();
    };
    enabled
        .iter()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn build_field_row(
    source_id: &str,
    path: &str,
    tab: &str,
    kind: &str,
    current: Option<&JsonValue>,
    default: Option<&JsonValue>,
    description: String,
    allowed_values: Vec<String>,
    validation: String,
    rebuild_required: bool,
    apply_mode: RuntimeApplyMode,
    has_blocking_diagnostic: bool,
    edit_behavior: ConfigUiEditBehavior,
) -> ConfigUiField {
    build_config_ui_field(ConfigUiFieldRowSpec {
        source_id,
        path,
        display_label: String::new(),
        section_label: String::new(),
        list_cells: Vec::new(),
        tab,
        kind,
        current,
        default,
        description,
        allowed_values,
        validation,
        rebuild_required,
        apply_status: apply_status_for_setting(path, apply_mode),
        has_blocking_diagnostic,
        edit_behavior,
    })
}

fn edit_behavior_for_field_path(path: &str) -> ConfigUiEditBehavior {
    if path == POPUP_COMMANDS_FIELD_PATH {
        return ConfigUiEditBehavior::StructuredOnly {
            notice: "Select a built-in popup row below to edit one command argv list.".to_string(),
        };
    }
    if is_keybinding_map_field_path(path) {
        return ConfigUiEditBehavior::StructuredOnly {
            notice: "Select an action row below to edit one binding list.".to_string(),
        };
    }
    if path == "cursors.cursor" {
        return ConfigUiEditBehavior::StructuredOnly {
            notice:
                "Cursor registry definitions are edited in the source file; run `yzx edit cursors`."
                    .to_string(),
        };
    }
    if path == CUSTOM_POPUPS_FIELD_PATH {
        return ConfigUiEditBehavior::StructuredOnly {
            notice: "Select a custom popup row below to edit one popup definition.".to_string(),
        };
    }
    ConfigUiEditBehavior::Default
}

fn append_builtin_popup_command_fields(
    fields: &mut Vec<ConfigUiField>,
    contract_fields: &BTreeMap<String, ConfigUiContractField>,
    config_owner: ConfigUiPathOwner,
    active_value: &JsonValue,
    default_value: &JsonValue,
    blocking_paths: &BTreeSet<String>,
) -> Result<(), CoreError> {
    let Some(parent_field) = contract_fields.get(POPUP_COMMANDS_FIELD_PATH) else {
        return Ok(());
    };
    let apply_mode = apply_mode_for_config_owner(config_owner, parent_field)?;
    for (id, label) in BUILTIN_POPUP_COMMANDS {
        let path = format!("{POPUP_COMMANDS_FIELD_PATH}.{id}");
        fields.push(build_field_row(
            SETTINGS_SOURCE_ID,
            &path,
            "workspace",
            "string_list",
            get_json_path(active_value, &path),
            get_json_path(default_value, &path),
            (*label).to_string(),
            Vec::new(),
            parent_field.validation.clone(),
            parent_field.rebuild_required,
            apply_mode,
            blocking_paths.contains(&path) || blocking_paths.contains(POPUP_COMMANDS_FIELD_PATH),
            ConfigUiEditBehavior::FriendlyStringList,
        ));
    }
    Ok(())
}

fn popup_commands_parent_path_for_field_path(path: &str) -> Option<&'static str> {
    let id = path
        .strip_prefix(POPUP_COMMANDS_FIELD_PATH)
        .and_then(|rest| rest.strip_prefix('.'))?;
    BUILTIN_POPUP_COMMANDS
        .iter()
        .any(|(known, _)| *known == id)
        .then_some(POPUP_COMMANDS_FIELD_PATH)
}

pub(super) fn apply_status_for_setting(
    path: &str,
    apply_mode: RuntimeApplyMode,
) -> ConfigUiApplyStatus {
    let (summary, detail, pending) = match apply_mode {
        RuntimeApplyMode::Live => ("now", "Saved changes are active immediately.", false),
        RuntimeApplyMode::LiveWithPaneRefresh => (
            "now",
            "Yazelix reloads this in the active pane owner when you save.",
            false,
        ),
        RuntimeApplyMode::GeneratedRuntimeRefresh => generated_runtime_effect_status(path),
        RuntimeApplyMode::TabSessionRestart => (
            "after Yazelix restart",
            "Saved changes are read from the launch snapshot when Yazelix starts.",
            true,
        ),
        RuntimeApplyMode::ShellTerminalRestart => (
            "after Yazelix restart",
            "Saved changes affect the shell or terminal environment that Yazelix starts with.",
            true,
        ),
        RuntimeApplyMode::PackageHomeManagerActivation => (
            "after Home Manager switch",
            "Edit the Home Manager source and run home-manager switch before Yazelix can use this value.",
            true,
        ),
        RuntimeApplyMode::NeverLive => (
            "not applicable",
            "This setting is an ownership boundary and is not live-applicable.",
            true,
        ),
    };
    ConfigUiApplyStatus {
        summary: summary.to_string(),
        label: summary.to_string(),
        detail: detail.to_string(),
        pending,
    }
}

fn generated_runtime_effect_status(path: &str) -> (&'static str, &'static str, bool) {
    if path.starts_with("yazi.") || path.starts_with("helix.") {
        (
            "after pane reopen",
            "Yazelix regenerates managed config; reopen the affected pane to use it.",
            true,
        )
    } else {
        (
            "after Yazelix restart",
            "Yazelix regenerates managed config; restart Yazelix to use it.",
            true,
        )
    }
}

fn field_description(field: &ConfigUiContractField, metadata: &ConfigUiFieldMetadata) -> String {
    let mut parts = Vec::new();
    parts.push(metadata.help.clone());
    if !field.validation.is_empty() {
        parts.push(format!("validation: {}", field.validation));
    }
    if let (Some(min), Some(max)) = (field.min, field.max) {
        parts.push(format!("range: {min}..{max}"));
    }
    if field.rebuild_required {
        parts.push("takes effect after runtime rebuild or rematerialization".to_string());
    }
    parts.join("; ")
}

fn collect_config_sources(
    tabs: &[String],
    active_config_path: &Path,
    active_config_exists: bool,
    config_owner: ConfigUiPathOwner,
    cursor_config_path: &Path,
    cursor_component_enabled: bool,
    mars_config_path: &Path,
) -> Vec<ConfigUiSource> {
    let mars_present = path_present(mars_config_path);
    let mars_owner = classify_path_owner(mars_config_path, mars_present);
    tabs.iter()
        .filter(|tab| tab.as_str() != "advanced")
        .map(|tab| {
            if tab == MARS_TAB {
                config_source(
                    MARS_SOURCE_ID,
                    tab,
                    "mars/config.toml",
                    mars_config_path,
                    mars_present,
                    mars_owner,
                )
            } else if tab == "cursors" && cursor_component_enabled {
                let cursor_present = path_present(cursor_config_path);
                config_source(
                    CURSORS_SOURCE_ID,
                    tab,
                    "yazelix_cursors/settings.jsonc",
                    cursor_config_path,
                    cursor_present,
                    classify_path_owner(cursor_config_path, cursor_present),
                )
            } else {
                config_source(
                    SETTINGS_SOURCE_ID,
                    tab,
                    "settings.jsonc",
                    active_config_path,
                    active_config_exists,
                    config_owner,
                )
            }
        })
        .collect()
}

fn config_source(
    id: &str,
    tab: &str,
    label: &str,
    path: &Path,
    exists: bool,
    owner: ConfigUiPathOwner,
) -> ConfigUiSource {
    ConfigUiSource {
        id: id.to_string(),
        tab: tab.to_string(),
        label: label.to_string(),
        path: path.to_path_buf(),
        exists,
        owner,
        read_only: path_is_read_only(path),
    }
}

fn collect_sidecars(config_dir: &Path) -> Vec<ConfigUiSidecar> {
    CURRENT_MANAGED_CONFIG_FILE_NAMES
        .iter()
        .filter(|name| **name != SETTINGS_CONFIG && **name != user_config_paths::MARS_CONFIG)
        .map(|name| {
            let path = config_dir.join(name);
            let present = fs::symlink_metadata(&path).is_ok();
            ConfigUiSidecar {
                name: (*name).to_string(),
                owner: classify_path_owner(&path, present),
                read_only: path_is_read_only(&path),
                path,
                present,
            }
        })
        .collect()
}

fn read_native_config_text(
    path: &Path,
    code: &'static str,
    message: &'static str,
) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            code,
            message,
            "Fix the config path or reinstall Yazelix, then retry.",
            path.display().to_string(),
            source,
        )
    })
}

pub(super) fn classify_path_owner(path: &Path, present: bool) -> ConfigUiPathOwner {
    if !present {
        return ConfigUiPathOwner::Default;
    }
    if path_owned_by_home_manager(path) {
        return ConfigUiPathOwner::HomeManager;
    }
    ConfigUiPathOwner::User
}

pub(super) fn path_is_read_only(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().readonly())
        .unwrap_or(false)
}

pub(super) fn path_present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

pub(super) fn read_settings_for_edit(path: &Path) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_settings_jsonc_for_edit",
            "Could not read Yazelix settings.jsonc for editing",
            "Fix permissions or restore the settings file, then retry.",
            path.display().to_string(),
            source,
        )
    })
}

pub(super) fn default_main_settings_text_for_ui(
    request: &ConfigUiRequest,
) -> Result<String, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    render_default_settings_jsonc(&paths.default_config_path)
}

pub(super) fn default_main_setting_value_for_ui(
    request: &ConfigUiRequest,
    path: &str,
) -> Result<JsonValue, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let defaults = read_settings_jsonc_value(&paths.default_config_path)?;
    get_json_path(&defaults, path).cloned().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Usage,
            "unsupported_settings_path",
            format!("Cannot reset {path} because it is not part of the canonical main settings defaults."),
            "Use a supported settings.jsonc path from the Yazelix config contract.",
            json!({ "path": path }),
        )
    })
}

pub(super) fn write_settings_edit(path: &Path, raw: &str) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "create_settings_jsonc_parent",
                "Could not create the Yazelix config directory",
                "Fix permissions for the config directory, then retry.",
                parent.display().to_string(),
                source,
            )
        })?;
    }
    fs::write(path, raw).map_err(|source| {
        CoreError::io(
            "write_settings_jsonc_edit",
            "Could not write Yazelix settings.jsonc",
            "Fix permissions for the settings file, then retry.",
            path.display().to_string(),
            source,
        )
    })
}

pub(super) fn validate_patched_settings_for_ui(
    request: &ConfigUiRequest,
    raw: &str,
) -> Result<(), CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let temp_dir = std::env::temp_dir().join(format!(
        "yazelix_config_ui_settings_check_{}_{}",
        std::process::id(),
        monotonic_suffix()
    ));
    fs::create_dir_all(&temp_dir).map_err(|source| {
        CoreError::io(
            "create_settings_validation_temp_dir",
            "Could not create a temporary directory to validate settings.jsonc",
            "Check the system temporary directory permissions, then retry.",
            temp_dir.display().to_string(),
            source,
        )
    })?;
    let temp_config = temp_dir.join(SETTINGS_CONFIG);
    let result = (|| {
        fs::write(&temp_config, raw).map_err(|source| {
            CoreError::io(
                "write_settings_validation_temp_config",
                "Could not write a temporary settings.jsonc validation file",
                "Check the system temporary directory permissions, then retry.",
                temp_config.display().to_string(),
                source,
            )
        })?;
        let normalized = crate::config_normalize::normalize_config(&NormalizeConfigRequest {
            config_path: temp_config,
            default_config_path: paths.default_config_path.clone(),
            contract_path: paths.contract_path.clone(),
            include_missing: true,
        })?;
        crate::zellij_materialization::validate_zellij_custom_popup_config(
            &normalized.normalized_config,
        )?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn monotonic_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}
