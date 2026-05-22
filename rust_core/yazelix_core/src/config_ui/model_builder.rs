use super::*;

pub fn build_config_ui_model(request: &ConfigUiRequest) -> Result<ConfigUiModel, CoreError> {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let schema = read_json_file(
        &paths.settings_schema_path,
        "read_settings_schema",
        "Could not read the Yazelix settings schema",
    )?;
    let schema_tab_order = schema_tabs(&schema);
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
        let apply_mode = if config_owner == ConfigUiPathOwner::HomeManager {
            RuntimeApplyMode::PackageHomeManagerActivation
        } else {
            field.apply_mode
        };
        fields.push(build_field_row(
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
    );

    if cursor_component_enabled {
        let cursor_choice_values = cursor_choice_values(&active_value, &default_value);
        for mut schema_field in collect_cursor_schema_fields(&schema) {
            if fields.iter().any(|field| field.path == schema_field.path) {
                continue;
            }
            enrich_cursor_schema_field(&mut schema_field, &cursor_choice_values);
            let metadata = field_ui_metadata(&ui_metadata, &schema_field.path)?;
            let current = get_json_path(&active_value, &schema_field.path);
            let default = get_json_path(&default_value, &schema_field.path);
            fields.push(build_field_row(
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
            home_dir,
            config_dir: request.config_dir.clone(),
            state_dir,
            platform: current_platform_name(),
            terminal_config_mode: effective_string_config(
                &active_value,
                &default_value,
                "terminal.config_mode",
                "yazelix",
            ),
            selected_terminals: effective_string_list_config(
                &active_value,
                &default_value,
                "terminal.terminals",
                &["ghostty", "wezterm"],
            ),
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
        tabs,
        fields,
        sidecars: collect_sidecars(&request.config_dir),
        native_config_statuses,
        diagnostics,
    })
}

pub(super) fn apply_contract_path_for_setting_path(setting_path: &str) -> &str {
    keybinding_parent_path_for_field_path(setting_path).unwrap_or(setting_path)
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
        CoreError::toml(
            "invalid_toml",
            "Could not parse the active Yazelix config",
            "Fix the TOML syntax in the reported file and retry.",
            path.display().to_string(),
            source,
        )
    })?;
    toml_value_to_json(&TomlValue::Table(table))
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
            "Fix permissions for ~/.config/yazelix_ghostty_cursors/settings.jsonc, then retry.",
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
            "Reinstall Yazelix so the runtime includes yazelix_ghostty_cursors_default.toml.",
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

fn load_contract_fields(path: &Path) -> Result<BTreeMap<String, ContractField>, CoreError> {
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
        CoreError::toml(
            "invalid_config_contract",
            "Could not parse the Yazelix config contract",
            "Reinstall Yazelix so the runtime includes a valid config contract.",
            path.display().to_string(),
            source,
        )
    })?;
    let fields_table = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "missing_contract_fields",
                "The Yazelix config contract is missing its fields table.",
                "Reinstall Yazelix so the runtime includes the current config contract.",
                json!({ "path": path.display().to_string() }),
            )
        })?;

    let mut fields = BTreeMap::new();
    for (field_path, value) in fields_table {
        let table = value.as_table().ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_contract_field",
                format!("Config contract field {field_path} must be a TOML table."),
                "Reinstall Yazelix so the runtime includes a valid config contract.",
                json!({ "field": field_path }),
            )
        })?;
        let kind = table
            .get("kind")
            .and_then(TomlValue::as_str)
            .unwrap_or("unknown")
            .to_string();
        let validation = table
            .get("validation")
            .and_then(TomlValue::as_str)
            .unwrap_or("")
            .to_string();
        let allowed_values = string_array(table.get("allowed_values"));
        let min = table.get("min").and_then(toml_number_as_f64);
        let max = table.get("max").and_then(toml_number_as_f64);
        let rebuild_required = table
            .get("rebuild_required")
            .and_then(TomlValue::as_bool)
            .unwrap_or(false);
        let apply_mode = table
            .get("apply_mode")
            .and_then(TomlValue::as_str)
            .ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Config,
                    "missing_apply_mode",
                    format!("Config contract field {field_path} is missing apply_mode."),
                    "Reinstall Yazelix so the runtime includes the current config contract.",
                    json!({ "field": field_path }),
                )
            })?
            .parse::<RuntimeApplyMode>()
            .map_err(|message| {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_apply_mode",
                    format!("Config contract field {field_path} has {message}."),
                    "Reinstall Yazelix so the runtime includes a valid config contract.",
                    json!({ "field": field_path }),
                )
            })?;
        let default_value = table.get("default").map(toml_value_to_json).transpose()?;
        fields.insert(
            field_path.clone(),
            ContractField {
                path: field_path.clone(),
                kind,
                default_value,
                validation,
                allowed_values,
                min,
                max,
                rebuild_required,
                apply_mode,
            },
        );
    }

    Ok(fields)
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
        CoreError::toml(
            "invalid_config_ui_metadata",
            "Could not parse the Yazelix config UI metadata",
            "Reinstall Yazelix so the runtime includes a valid config UI metadata file.",
            path.display().to_string(),
            source,
        )
    })?;

    let tabs = metadata
        .get("tab_order")
        .and_then(TomlValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(TomlValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_config_ui_tabs",
            "The Yazelix config UI metadata is missing tab_order.",
            "Reinstall Yazelix so the runtime includes current config UI metadata.",
            json!({ "path": path.display().to_string() }),
        ));
    }

    let fields_table = metadata
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "missing_config_ui_fields",
                "The Yazelix config UI metadata is missing its fields table.",
                "Reinstall Yazelix so the runtime includes current config UI metadata.",
                json!({ "path": path.display().to_string() }),
            )
        })?;

    let mut fields = BTreeMap::new();
    for (field_path, value) in fields_table {
        let table = value.as_table().ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_config_ui_field",
                format!("Config UI metadata field {field_path} must be a TOML table."),
                "Reinstall Yazelix so the runtime includes valid config UI metadata.",
                json!({ "field": field_path }),
            )
        })?;
        fields.insert(
            field_path.clone(),
            FieldUiMetadata {
                tab: required_toml_string(table, field_path, "tab")?,
                help: required_toml_string(table, field_path, "help")?,
            },
        );
    }

    Ok(ConfigUiMetadata { tabs, fields })
}

fn required_toml_string(
    table: &toml::Table,
    field_path: &str,
    key: &str,
) -> Result<String, CoreError> {
    table
        .get(key)
        .and_then(TomlValue::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_config_ui_field_metadata",
                format!("Config UI metadata field {field_path} is missing {key}."),
                "Reinstall Yazelix so the runtime includes complete config UI metadata.",
                json!({ "field": field_path, "key": key }),
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
) -> Result<&'a FieldUiMetadata, CoreError> {
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

fn schema_tabs(schema: &JsonValue) -> Vec<String> {
    let mut tabs = schema
        .get("x-yazelix")
        .and_then(|value| value.get("tabs"))
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        tabs = DEFAULT_TABS.iter().map(|tab| (*tab).to_string()).collect();
    }
    if !tabs.iter().any(|tab| tab == "advanced") {
        tabs.push("advanced".to_string());
    }
    tabs
}

fn collect_cursor_schema_fields(schema: &JsonValue) -> Vec<SchemaField> {
    let mut fields = Vec::new();
    let Some(cursors) = schema
        .get("properties")
        .and_then(|properties| properties.get("cursors"))
    else {
        return fields;
    };
    collect_schema_fields(cursors, "cursors", &mut fields);
    fields
}

fn collect_schema_fields(schema: &JsonValue, path: &str, out: &mut Vec<SchemaField>) {
    let kind = schema_type(schema);
    if kind == "object" {
        let Some(properties) = schema.get("properties").and_then(JsonValue::as_object) else {
            out.push(schema_field(schema, path, kind));
            return;
        };
        for (name, property) in properties {
            collect_schema_fields(property, &format!("{path}.{name}"), out);
        }
        return;
    }

    if kind == "array"
        && let Some(items) = schema.get("items")
        && items.get("type").and_then(JsonValue::as_str) == Some("object")
    {
        out.push(schema_field(schema, path, kind));
        return;
    }

    if kind == "array"
        && let Some(items) = schema.get("items")
        && items.get("type").and_then(JsonValue::as_str) == Some("string")
    {
        out.push(schema_field(schema, path, "string_list".to_string()));
        return;
    }

    out.push(schema_field(schema, path, kind));
}

fn schema_field(schema: &JsonValue, path: &str, kind: String) -> SchemaField {
    let allowed_values = if kind == "string_list" {
        schema
            .get("items")
            .map(schema_enum_values)
            .unwrap_or_default()
    } else {
        schema_enum_values(schema)
    };
    SchemaField {
        path: path.to_string(),
        kind,
        allowed_values,
    }
}

fn schema_type(schema: &JsonValue) -> String {
    schema
        .get("type")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn schema_enum_values(schema: &JsonValue) -> Vec<String> {
    schema
        .get("enum")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn enrich_cursor_schema_field(field: &mut SchemaField, values: &CursorChoiceValues) {
    match field.path.as_str() {
        "cursors.enabled_cursors" => {
            field.allowed_values = values.definition_names.clone();
        }
        "cursors.settings.trail" => {
            field.allowed_values = vec!["none".to_string(), "random".to_string()];
            field.allowed_values.extend(values.enabled_names.clone());
        }
        _ => {}
    }
}

fn cursor_choice_values(active: &JsonValue, default: &JsonValue) -> CursorChoiceValues {
    let definition_names = cursor_definition_names(active, default);
    let enabled_names = cursor_enabled_names(active, default)
        .into_iter()
        .filter(|name| definition_names.iter().any(|definition| definition == name))
        .collect();
    CursorChoiceValues {
        definition_names,
        enabled_names,
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
    let state = if has_blocking_diagnostic {
        ConfigUiValueState::Invalid
    } else if current.is_some() {
        ConfigUiValueState::Explicit
    } else if default.is_some() {
        ConfigUiValueState::Defaulted
    } else {
        ConfigUiValueState::Unset
    };
    ConfigUiField {
        path: path.to_string(),
        tab: tab.to_string(),
        kind: kind.to_string(),
        current_value: current
            .or(default)
            .map(render_json_value)
            .unwrap_or_else(|| "not set".to_string()),
        edit_value: current
            .or(default)
            .map(render_json_edit_value)
            .unwrap_or_default(),
        default_value: default
            .map(render_json_value)
            .unwrap_or_else(|| "no default".to_string()),
        state,
        description,
        allowed_values,
        validation,
        rebuild_required,
        apply_status: apply_status_for_setting(path, apply_mode),
        edit_behavior,
    }
}

fn edit_behavior_for_field_path(path: &str) -> ConfigUiEditBehavior {
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
    ConfigUiEditBehavior::Default
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

fn field_description(field: &ContractField, metadata: &FieldUiMetadata) -> String {
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

fn collect_sidecars(config_dir: &Path) -> Vec<ConfigUiSidecar> {
    let mut sidecars = CURRENT_MANAGED_CONFIG_FILE_NAMES
        .iter()
        .filter(|name| **name != SETTINGS_CONFIG)
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
        .collect::<Vec<_>>();
    let cursor_path = crate::user_config_paths::shared_cursor_config(config_dir);
    let cursor_present = fs::symlink_metadata(&cursor_path).is_ok();
    sidecars.push(ConfigUiSidecar {
        name: "yazelix_ghostty_cursors/settings.jsonc".to_string(),
        owner: classify_path_owner(&cursor_path, cursor_present),
        read_only: path_is_read_only(&cursor_path),
        path: cursor_path,
        present: cursor_present,
    });
    sidecars
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
        crate::config_normalize::normalize_config(&NormalizeConfigRequest {
            config_path: temp_config,
            default_config_path: paths.default_config_path.clone(),
            contract_path: paths.contract_path.clone(),
            include_missing: true,
        })?;
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

fn toml_value_to_json(value: &TomlValue) -> Result<JsonValue, CoreError> {
    match value {
        TomlValue::String(value) => Ok(JsonValue::String(value.clone())),
        TomlValue::Integer(value) => Ok(JsonValue::Number((*value).into())),
        TomlValue::Float(value) => serde_json::Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| {
                CoreError::classified(
                    ErrorClass::Config,
                    "non_finite_toml_float",
                    "Could not convert a TOML float to JSON.",
                    "Use a finite number in the settings input.",
                    json!({ "value": value.to_string() }),
                )
            }),
        TomlValue::Boolean(value) => Ok(JsonValue::Bool(*value)),
        TomlValue::Datetime(value) => Ok(JsonValue::String(value.to_string())),
        TomlValue::Array(values) => values
            .iter()
            .map(toml_value_to_json)
            .collect::<Result<Vec<_>, _>>()
            .map(JsonValue::Array),
        TomlValue::Table(table) => {
            let mut object = JsonMap::new();
            for (key, value) in table {
                object.insert(key.clone(), toml_value_to_json(value)?);
            }
            Ok(JsonValue::Object(object))
        }
    }
}

fn string_array(value: Option<&TomlValue>) -> Vec<String> {
    value
        .and_then(TomlValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(TomlValue::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn toml_number_as_f64(value: &TomlValue) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}
