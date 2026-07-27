use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use ratconfig::toml_adapter::{get_toml_path, parse_toml_value};
use ratconfig::{
    ConfigUiApplyStatus, ConfigUiCapability, ConfigUiChoice, ConfigUiDiagnostic,
    ConfigUiDiagnosticScope, ConfigUiFieldId, ConfigUiFieldSnapshot, ConfigUiFieldSpec,
    ConfigUiListColumn, ConfigUiListTable, ConfigUiModel, ConfigUiOverride, ConfigUiResolvedValue,
    ConfigUiSource, ConfigUiTextEncoding, ConfigUiTheme, ConfigUiThemeMapping,
    ConfigUiThemeSwitcher, ConfigUiTomlDocumentSpec, build_toml_document_fields,
};
use serde_json::Value as JsonValue;
use yazelix_cursors::{
    CursorConfigFieldKind, CursorConfigFieldSpec as CursorOwnerFieldSpec, CursorRegistry,
    cursor_config_field_specs,
};

use crate::{
    catalog::*,
    common::*,
    file_actions::build_file_actions,
    native_config::{cursor_defaults, validate_mars_field},
    paths::ConfigPaths,
    root_config::{
        bar_widgets, default_config, default_config_path_value, read_optional_toml_file_value,
        validate_root_config,
    },
    yazi_config::build_yazi_fields,
    zellij_sidecar::{
        active_zellij_runtime_config_path, packaged_zellij_defaults, packaged_zellij_theme_choices,
        parse_zellij_sidecar, read_zellij_sidecar,
    },
};

pub(crate) fn build_model(paths: &ConfigPaths) -> Result<ConfigUiModel> {
    let (config_active, root_document_valid, mut diagnostics) = load_root_config(&paths.root)?;
    let config_default = default_config()?;
    let mars_active = read_optional_toml_file_value(&paths.mars, "invalid mars/config.toml")?;
    let mars_default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Mars config", error))?;
    let starship_active = read_optional_toml_file_value(&paths.starship, "invalid starship.toml")?;
    let starship_default = parse_toml_value(DEFAULT_STARSHIP_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Starship config", error))?;
    let cursors_active = yazelix_cursors::load_cursor_config(&paths.cursors)?;
    let cursors_default = cursor_defaults()?;
    let (zellij_active, zellij_invalid, zellij_diagnostics) =
        parse_zellij_sidecar(&read_zellij_sidecar(&paths.zellij)?);
    diagnostics.extend(zellij_diagnostics);
    let zellij_default = packaged_zellij_defaults();
    let zellij_runtime_active = active_zellij_runtime_config_path().is_some();
    let mut zellij_themes = packaged_zellij_theme_choices();
    for custom in ["theme_dark", "theme_light"]
        .into_iter()
        .filter_map(|path| zellij_active.get(path).and_then(JsonValue::as_str))
    {
        if !zellij_themes.iter().any(|theme| theme == custom) {
            zellij_themes.push(custom.to_string());
        }
    }
    let yazi = build_yazi_fields(paths)?;
    let file_actions = build_file_actions(paths);

    let mut fields: Vec<_> = CONFIG_FIELDS
        .iter()
        .map(|spec| build_root_config_field(&config_active, &config_default, spec))
        .collect::<Result<_>>()?;
    fields.push(build_bar_widgets_field(&config_active, &config_default)?);
    if root_document_valid {
        fields.extend(build_custom_popup_fields(&paths.root)?);
    }
    fields.extend(KEY_BINDINGS.iter().map(build_key_binding_field));
    fields.extend(build_cursor_fields(&cursors_active, &cursors_default)?);
    for spec in MARS_FIELDS {
        let current = get_toml_path(&mars_active, spec.path);
        fields.push(build_config_field(
            SOURCE_MARS,
            TAB_MARS,
            spec,
            current,
            get_toml_path(&mars_default, spec.path),
            mars_apply_status(spec.path),
            current.is_some_and(|value| validate_mars_field(spec, value).is_err()),
        ));
    }
    for spec in STARSHIP_FIELDS {
        let current = get_toml_path(&starship_active, spec.path);
        fields.push(build_config_field(
            SOURCE_STARSHIP,
            TAB_STARSHIP,
            spec,
            current,
            get_toml_path(&starship_default, spec.path),
            apply_status(
                "new prompts",
                "starship",
                "Saved values apply to newly rendered managed Nu prompts.",
            ),
            current.is_some_and(|value| spec.json_choice(value).is_err()),
        ));
    }
    for spec in ZELLIJ_FIELDS {
        let current = zellij_active.get(spec.path);
        let default = zellij_default.get(spec.path).expect("packaged default");
        let mut field = build_config_field(
            SOURCE_ZELLIJ,
            TAB_ZELLIJ,
            spec,
            current,
            Some(default),
            zellij_apply_status(spec.path, zellij_runtime_active),
            false,
        );
        if matches!(spec.path, "theme_dark" | "theme_light") {
            field.capability = choice_capability(zellij_themes.clone());
            field.display_label = if spec.path == "theme_dark" {
                "Dark theme"
            } else {
                "Light theme"
            }
            .to_string();
        }
        if let Some(input) = zellij_invalid.get(spec.path) {
            field.snapshot.intent = ConfigUiOverride::Invalid {
                input: input.clone(),
            };
            field.snapshot.effective = None;
            field.can_unset = true;
        }
        fields.push(field);
    }
    let yazi_dir = paths.yazi_config.parent().expect("Yazi config directory");
    let advanced_dir = paths.nu_config.parent().expect("Nushell config directory");
    let sources = vec![
        build_config_source(paths, SOURCE_CONFIG, "config.toml", &paths.root),
        build_config_source(paths, SOURCE_MARS, "mars/config.toml", &paths.mars),
        build_config_source(paths, SOURCE_CURSORS, "cursors.toml", &paths.cursors),
        build_config_source(paths, SOURCE_ZELLIJ, "zellij/config.kdl", &paths.zellij),
        build_config_source(paths, SOURCE_STARSHIP, "starship.toml", &paths.starship),
        build_config_source(paths, SOURCE_HELIX, "helix", &paths.helix_dir),
        build_config_source(
            paths,
            SOURCE_YAZI_CONFIG,
            "yazi/yazi.toml",
            &paths.yazi_config,
        ),
        build_config_source(
            paths,
            SOURCE_YAZI_THEME,
            "yazi/theme.toml",
            &paths.yazi_theme,
        ),
        build_config_source(paths, SOURCE_YAZI, "yazi", yazi_dir),
        build_config_source(paths, SOURCE_ADVANCED, "advanced files", advanced_dir),
        ConfigUiSource {
            id: SOURCE_KEYS.to_string(),
            label: "key bindings".to_string(),
            path: PathBuf::from("packaged-key-bindings"),
            exists: true,
            owner_label: Some("Yazelix".to_string()),
            read_only: true,
        },
    ];
    let mut fields = fields.into_iter().chain(yazi).collect::<Vec<_>>();
    apply_source_policy(&mut fields, &sources);
    if !root_document_valid {
        for field in fields
            .iter_mut()
            .filter(|field| field.source_id == SOURCE_CONFIG)
        {
            field.capability = ConfigUiCapability::ReadOnly {
                reason: "Repair config.toml before editing individual fields.".to_string(),
                file_action_id: Some(ACTION_ROOT_CONFIG.to_string()),
            };
            field.can_unset = false;
        }
    }
    let recommended_fields = Some(
        fields
            .iter()
            .filter(|field| match field.source_id.as_str() {
                SOURCE_CONFIG => ROOT_CONFIG_RECOMMENDED_PATHS.contains(&field.path.as_str()),
                SOURCE_CURSORS => CURSOR_RECOMMENDED_PATHS.contains(&field.path.as_str()),
                SOURCE_ZELLIJ => ZELLIJ_RECOMMENDED_PATHS.contains(&field.path.as_str()),
                _ => true,
            })
            .map(|field| ConfigUiFieldId::new(&field.source_id, &field.path))
            .collect(),
    );

    Ok(ConfigUiModel {
        sources,
        tabs: vec![
            TAB_CONFIG.to_string(),
            TAB_POPUPS.to_string(),
            TAB_MARS.to_string(),
            TAB_CURSORS.to_string(),
            TAB_ZELLIJ.to_string(),
            TAB_STARSHIP.to_string(),
            TAB_HELIX.to_string(),
            TAB_YAZI.to_string(),
            TAB_KEYS.to_string(),
            TAB_ADVANCED.to_string(),
        ],
        operational_tab: Some(TAB_ADVANCED.to_string()),
        tab_list_tables: BTreeMap::from([(
            TAB_KEYS.to_string(),
            ConfigUiListTable {
                columns: KEY_COLUMNS
                    .iter()
                    .map(|(title, width)| ConfigUiListColumn {
                        title: (*title).to_string(),
                        width: *width,
                    })
                    .collect(),
            },
        )]),
        fields,
        recommended_fields,
        file_actions,
        sidecars: Vec::new(),
        native_config_statuses: Vec::new(),
        diagnostics,
        theme_switcher: Some(ConfigUiThemeSwitcher {
            field: ConfigUiFieldId::new(SOURCE_CONFIG, APPEARANCE_MODE_PATH),
            mappings: vec![
                ConfigUiThemeMapping {
                    value: JsonValue::String("dark".to_string()),
                    theme: ConfigUiTheme::Dark,
                },
                ConfigUiThemeMapping {
                    value: JsonValue::String("light".to_string()),
                    theme: ConfigUiTheme::Light,
                },
            ],
        }),
    })
}

fn load_root_config(path: &Path) -> Result<(JsonValue, bool, Vec<ConfigUiDiagnostic>)> {
    if !path_entry_exists(path)? {
        return Ok((JsonValue::Object(Default::default()), true, Vec::new()));
    }
    let raw = fs::read_to_string(path)?;
    let active = match parse_toml_value(&raw) {
        Ok(active) => active,
        Err(source) => {
            return Ok((
                JsonValue::Object(Default::default()),
                false,
                vec![root_source_diagnostic(format!("invalid TOML: {source:?}"))],
            ));
        }
    };

    let mut diagnostics = root_field_diagnostics(&active);
    let source_error = validate_root_config(&active)
        .err()
        .map(|source| source.to_string())
        .filter(|message| {
            !diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .detail_lines
                    .iter()
                    .any(|detail| detail == message)
            })
        });
    let document_valid = source_error.is_none();
    if let Some(message) = source_error {
        diagnostics.push(root_source_diagnostic(message));
    }
    Ok((active, document_valid, diagnostics))
}

fn root_field_diagnostics(active: &JsonValue) -> Vec<ConfigUiDiagnostic> {
    let mut diagnostics = CONFIG_FIELDS
        .iter()
        .map(|spec| &spec.field)
        .filter_map(|spec| {
            let value = get_toml_path(active, spec.path)?;
            crate::root_config::validate_config_value(spec.path, value)
                .err()
                .map(|source| ConfigUiDiagnostic {
                    path: spec.path.to_string(),
                    status: "invalid".to_string(),
                    headline: format!("invalid config value for `{}`", spec.path),
                    blocking: true,
                    scope: ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(
                        SOURCE_CONFIG,
                        spec.path,
                    )),
                    detail_lines: vec![source.to_string()],
                })
        })
        .collect::<Vec<_>>();
    if let Some(value) = get_toml_path(active, BAR_WIDGETS_PATH)
        && let Err(source) = bar_widgets(value)
    {
        diagnostics.push(ConfigUiDiagnostic {
            path: BAR_WIDGETS_PATH.to_string(),
            status: "invalid".to_string(),
            headline: format!("invalid config value for `{BAR_WIDGETS_PATH}`"),
            blocking: true,
            scope: ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(
                SOURCE_CONFIG,
                BAR_WIDGETS_PATH,
            )),
            detail_lines: vec![source.to_string()],
        });
    }
    diagnostics
}

fn root_source_diagnostic(message: String) -> ConfigUiDiagnostic {
    ConfigUiDiagnostic {
        path: "config.toml".to_string(),
        status: "blocked".to_string(),
        headline: "config.toml needs native-file repair".to_string(),
        blocking: true,
        scope: ConfigUiDiagnosticScope::Source {
            source_id: SOURCE_CONFIG.to_string(),
        },
        detail_lines: vec![message],
    }
}
fn build_key_binding_field(
    [group, chord, action, owner, source]: &[&str; 5],
) -> ratconfig::ConfigUiField {
    ratconfig::ConfigUiField {
        source_id: SOURCE_KEYS.to_string(),
        path: chord.to_string(),
        tab: TAB_KEYS.to_string(),
        display_label: format!("{group}: {chord} - {action}"),
        section_label: String::new(),
        list_cells: [*group, *chord, *action, *owner]
            .into_iter()
            .map(str::to_string)
            .collect(),
        type_label: Some("string".to_string()),
        snapshot: ConfigUiFieldSnapshot {
            intent: ConfigUiOverride::Explicit(JsonValue::String(format!("{owner} / {source}"))),
            effective: Some(ConfigUiResolvedValue {
                value: JsonValue::String(format!("{owner} / {source}")),
                origin: Some("Yazelix packaged keymap".to_string()),
            }),
            baseline: None,
            external_manager: None,
        },
        description: format!("Group: {group}. Owner: {owner}. Source: {source}. Editable: no."),
        validation: KEY_READ_ONLY_REASON.to_string(),
        rebuild_required: false,
        apply_status: apply_status("read-only", "read-only", KEY_READ_ONLY_REASON),
        capability: ConfigUiCapability::ReadOnly {
            reason: KEY_READ_ONLY_REASON.to_string(),
            file_action_id: None,
        },
        can_unset: false,
    }
}
fn build_config_source(paths: &ConfigPaths, id: &str, label: &str, path: &Path) -> ConfigUiSource {
    let home_manager_owned = paths.is_home_manager_owned(path);
    ConfigUiSource {
        id: id.to_string(),
        label: label.to_string(),
        path: path.to_path_buf(),
        exists: path.exists(),
        owner_label: Some(if home_manager_owned {
            "Home Manager".to_string()
        } else {
            "User".to_string()
        }),
        read_only: home_manager_owned || path_read_only(path),
    }
}
fn build_root_config_field(
    active: &JsonValue,
    defaults: &JsonValue,
    spec: &ConfigFieldSpec,
) -> Result<ratconfig::ConfigUiField> {
    let default = default_config_path_value(defaults, spec.field.path)?;
    let current = get_toml_path(active, spec.field.path);
    let invalid = current.is_some_and(|value| {
        crate::root_config::validate_config_value(spec.field.path, value).is_err()
    });
    Ok(build_config_field(
        SOURCE_CONFIG,
        root_config_tab(spec.field.path),
        &spec.field,
        current,
        Some(&default),
        apply_status(spec.apply_summary, "runtime", spec.apply_detail),
        invalid,
    ))
}
fn build_custom_popup_fields(path: &Path) -> Result<Vec<ratconfig::ConfigUiField>> {
    let raw = if path_entry_exists(path)? {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut fields = build_toml_document_fields(ConfigUiTomlDocumentSpec {
        source_id: SOURCE_CONFIG,
        tab: TAB_POPUPS,
        section_label: "custom popups",
        current_toml: &raw,
        baseline_toml: None,
        validation: "",
        rebuild_required: false,
        apply_status: apply_status(
            "next launch",
            "runtime",
            "Saved custom popup settings apply to newly launched Yazelix sessions.",
        ),
    })
    .map_err(|source| error(source.to_string()))?
    .fields;
    fields.retain(|field| field.path.starts_with("popups."));
    remove_toml_parent_fields(&mut fields);
    for field in &mut fields {
        field.list_cells.clear();
        if let ConfigUiOverride::Explicit(value) = &field.snapshot.intent {
            field.snapshot.effective = Some(ConfigUiResolvedValue {
                value: value.clone(),
                origin: Some("User config.toml".to_string()),
            });
        }
        if let Some((capability, can_unset)) = custom_popup_controls(&field.path) {
            field.capability = capability;
            field.can_unset = can_unset;
        }
    }
    Ok(fields)
}
fn custom_popup_controls(path: &str) -> Option<(ConfigUiCapability, bool)> {
    let mut segments = path.split('.');
    if segments.next() != Some("popups") || segments.next().is_none() {
        return None;
    }
    let field = segments.next()?;
    if segments.next().is_some() {
        return None;
    }
    let capability = match field {
        "command" | "title" | "keybinding" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        },
        "args" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::Json,
        },
        "keep_alive" => ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(JsonValue::Bool(false)),
            on: ConfigUiChoice::new(JsonValue::Bool(true)),
        },
        _ => return None,
    };
    Some((capability, !matches!(field, "command" | "keybinding")))
}
fn root_config_tab(path: &str) -> &'static str {
    if matches!(
        path,
        AGENT_COMMAND_PATH
            | AGENT_ARGS_PATH
            | POPUP_SIDE_MARGIN_PATH
            | POPUP_VERTICAL_MARGIN_PATH
            | KEYBINDINGS_CONFIG_PATH
            | KEYBINDINGS_AGENT_PATH
            | KEYBINDINGS_GIT_PATH
            | KEYBINDINGS_MENU_PATH
            | KEYBINDINGS_SCREEN_PATH
    ) {
        TAB_POPUPS
    } else {
        TAB_CONFIG
    }
}
fn build_config_field(
    source_id: &'static str,
    tab: &'static str,
    spec: &FieldSpec,
    current: Option<&JsonValue>,
    default: Option<&JsonValue>,
    apply_status: ConfigUiApplyStatus,
    invalid: bool,
) -> ratconfig::ConfigUiField {
    let mut field = ConfigUiFieldSpec {
        can_unset: current.is_some(),
        ..ConfigUiFieldSpec::new(
            source_id,
            spec.path,
            tab,
            spec.description,
            field_capability(spec, string_values(spec.allowed_values)),
            spec.validation,
            apply_status,
        )
    }
    .build(spec.kind, current, default);
    set_snapshot_origins(&mut field, source_id);
    if invalid {
        field.snapshot.intent = ConfigUiOverride::Invalid {
            input: current.map_or_else(String::new, ToString::to_string),
        };
        field.snapshot.effective = None;
    }
    field
}
fn build_cursor_fields(
    active: &CursorRegistry,
    defaults: &CursorRegistry,
) -> Result<Vec<ratconfig::ConfigUiField>> {
    let active_json = serde_json::to_value(active)?;
    let default_json = serde_json::to_value(defaults)?;
    cursor_config_field_specs()
        .iter()
        .map(|spec| {
            let current = cursor_config_value(active, &active_json, spec)?;
            let default = cursor_config_value(defaults, &default_json, spec)?;
            let (type_label, capability) = cursor_field_capability(active, spec.kind);
            let mut field = ConfigUiFieldSpec::new(
                SOURCE_CURSORS,
                spec.path,
                TAB_CURSORS,
                spec.description,
                capability,
                spec.validation,
                cursor_apply_status(spec.path),
            )
            .build(type_label, Some(&current), Some(&default));
            set_snapshot_origins(&mut field, SOURCE_CURSORS);
            Ok(field)
        })
        .collect()
}

fn cursor_config_value(
    registry: &CursorRegistry,
    registry_json: &JsonValue,
    spec: &CursorOwnerFieldSpec,
) -> Result<JsonValue> {
    if matches!(spec.kind, CursorConfigFieldKind::DefinitionTables) {
        return Ok(serde_json::to_value(
            registry.definitions.values().collect::<Vec<_>>(),
        )?);
    }
    get_toml_path(registry_json, spec.path)
        .cloned()
        .ok_or_else(|| {
            error(format!(
                "cursor owner schema path is missing: {}",
                spec.path
            ))
        })
}

fn cursor_field_capability(
    registry: &CursorRegistry,
    kind: CursorConfigFieldKind,
) -> (&'static str, ConfigUiCapability) {
    match kind {
        CursorConfigFieldKind::FixedInteger(_) => (
            "integer",
            ConfigUiCapability::ReadOnly {
                reason: "The cursor schema version is child-owned metadata.".to_string(),
                file_action_id: Some(ACTION_CURSORS_CONFIG.to_string()),
            },
        ),
        CursorConfigFieldKind::DefinitionList => (
            "string_list",
            multi_choice_capability(registry.definitions.keys().cloned(), true),
        ),
        CursorConfigFieldKind::DefinitionChoice { extra_values } => (
            "string",
            choice_capability(
                registry
                    .enabled_cursors
                    .iter()
                    .map(String::as_str)
                    .chain(extra_values.iter().copied())
                    .map(str::to_string),
            ),
        ),
        CursorConfigFieldKind::StringChoice {
            values,
            extra_values,
        } => (
            "string",
            choice_capability(
                values
                    .iter()
                    .chain(extra_values)
                    .map(|value| (*value).to_string()),
            ),
        ),
        CursorConfigFieldKind::NumberRange { .. } => (
            "float",
            ConfigUiCapability::FreeText {
                encoding: ConfigUiTextEncoding::Json,
            },
        ),
        CursorConfigFieldKind::DefinitionTables => (
            "table array",
            ConfigUiCapability::ReadOnly {
                reason: "Edit cursor definition tables in the complete cursors.toml.".to_string(),
                file_action_id: Some(ACTION_CURSORS_CONFIG.to_string()),
            },
        ),
    }
}
fn cursor_apply_status(path: &str) -> ConfigUiApplyStatus {
    if path == "schema_version" {
        return apply_status(
            "—",
            "not applicable",
            "The child-owned format version is metadata with no independent runtime effect.",
        );
    }
    if matches!(
        path,
        "settings.mode_effect" | "settings.glow" | "settings.duration"
    ) {
        return apply_status(
            "stored",
            "cursors",
            "Saved for compatible consumers; Mars does not use this setting yet.",
        );
    }
    apply_status(
        "next launch",
        "cursors",
        if path == "settings.trail_effect" {
            "Mars currently reads only none versus enabled; compatible consumers may use the named effect."
        } else if path == "cursor" {
            "Mars reads the selected cursor definition on its next launch."
        } else {
            "Mars reads the saved cursor pool and selection on its next launch."
        },
    )
}
fn build_bar_widgets_field(
    active: &JsonValue,
    defaults: &JsonValue,
) -> Result<ratconfig::ConfigUiField> {
    let current = get_toml_path(active, BAR_WIDGETS_PATH);
    let default = default_config_path_value(defaults, BAR_WIDGETS_PATH)?;
    let invalid = current.is_some_and(|value| bar_widgets(value).is_err());
    let mut field = ConfigUiFieldSpec {
        can_unset: current.is_some(),
        ..ConfigUiFieldSpec::new(
            SOURCE_CONFIG,
            BAR_WIDGETS_PATH,
            TAB_CONFIG,
            "Top bar widgets, left to right.",
            multi_choice_capability(string_values(BAR_WIDGET_VALUES), true),
            "known widget ids",
            apply_status(
                "next launch",
                "bar",
                "Saved widget order applies to newly launched Yazelix sessions.",
            ),
        )
    }
    .build("string_list", current, Some(&default));
    set_snapshot_origins(&mut field, SOURCE_CONFIG);
    if invalid {
        field.snapshot.intent = ConfigUiOverride::Invalid {
            input: current.map_or_else(String::new, ToString::to_string),
        };
        field.snapshot.effective = None;
    }
    Ok(field)
}

fn field_capability(spec: &FieldSpec, values: Vec<String>) -> ConfigUiCapability {
    match spec.kind {
        "boolean" => ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(JsonValue::Bool(false)),
            on: ConfigUiChoice::new(JsonValue::Bool(true)),
        },
        "string" if !values.is_empty() => choice_capability(values),
        "string" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        },
        "string_list" if !values.is_empty() => multi_choice_capability(values, true),
        "string_list" | "integer" | "float" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::Json,
        },
        _ => ConfigUiCapability::ReadOnly {
            reason: format!("Unsupported owner type {}.", spec.kind),
            file_action_id: None,
        },
    }
}

fn choice_capability(values: impl IntoIterator<Item = String>) -> ConfigUiCapability {
    ConfigUiCapability::Choice {
        choices: values
            .into_iter()
            .map(|value| ConfigUiChoice::new(JsonValue::String(value)))
            .collect(),
    }
}

fn multi_choice_capability(
    values: impl IntoIterator<Item = String>,
    ordered: bool,
) -> ConfigUiCapability {
    ConfigUiCapability::MultiChoice {
        choices: values
            .into_iter()
            .map(|value| ConfigUiChoice::new(JsonValue::String(value)))
            .collect(),
        ordered,
    }
}

fn field_origins(source_id: &str) -> (&'static str, &'static str) {
    match source_id {
        SOURCE_CONFIG => ("User config.toml", "Yazelix packaged default"),
        SOURCE_MARS => ("User mars/config.toml", "Yazelix packaged default"),
        SOURCE_CURSORS => ("User cursors.toml", "Packaged cursor defaults"),
        SOURCE_ZELLIJ => ("User zellij/config.kdl", "Packaged Zellij defaults"),
        SOURCE_STARSHIP => ("User starship.toml", "Yazelix packaged default"),
        SOURCE_YAZI_CONFIG => ("User yazi/yazi.toml", "Packaged Yazi config"),
        SOURCE_YAZI_THEME => ("User yazi/theme.toml", "Yazi default theme"),
        _ => ("User configuration", "Yazelix packaged default"),
    }
}

fn set_snapshot_origins(field: &mut ratconfig::ConfigUiField, source_id: &str) {
    let (effective_origin, baseline_origin) = field_origins(source_id);
    if let Some(baseline) = &mut field.snapshot.baseline {
        baseline.origin = Some(baseline_origin.to_string());
    }
    let origin = if matches!(field.snapshot.intent, ConfigUiOverride::Absent)
        && field.snapshot.baseline.is_some()
    {
        baseline_origin
    } else {
        effective_origin
    };
    if let Some(effective) = &mut field.snapshot.effective {
        effective.origin = Some(origin.to_string());
    }
}

fn apply_source_policy(fields: &mut [ratconfig::ConfigUiField], sources: &[ConfigUiSource]) {
    for field in fields {
        let source = sources
            .iter()
            .find(|source| source.id == field.source_id)
            .expect("every field source is declared");
        if field.source_id == SOURCE_CURSORS {
            field.can_unset = false;
        }
        let home_manager_owned = source.owner_label.as_deref() == Some("Home Manager");
        if home_manager_owned {
            if matches!(
                field.snapshot.intent,
                ConfigUiOverride::Explicit(_) | ConfigUiOverride::Invalid { .. }
            ) && let Some(effective) = &mut field.snapshot.effective
            {
                effective.origin = Some("Home Manager".to_string());
            }
            field.snapshot.external_manager = Some("Home Manager".to_string());
        }
        if source.read_only {
            if home_manager_owned
                || !matches!(field.capability, ConfigUiCapability::ReadOnly { .. })
            {
                field.capability = ConfigUiCapability::ReadOnly {
                    reason: if home_manager_owned {
                        "Managed by Home Manager."
                    } else {
                        "Source is read-only."
                    }
                    .to_string(),
                    file_action_id: source_file_action(&field.source_id).map(str::to_string),
                };
            }
            field.can_unset = false;
        }
        if let ConfigUiCapability::ReadOnly { file_action_id, .. } = &mut field.capability
            && file_action_id.is_none()
        {
            *file_action_id = source_file_action(&field.source_id).map(str::to_string);
        }
    }
}

fn source_file_action(source_id: &str) -> Option<&'static str> {
    match source_id {
        SOURCE_CONFIG => Some(ACTION_ROOT_CONFIG),
        SOURCE_CURSORS => Some(ACTION_CURSORS_CONFIG),
        SOURCE_ZELLIJ => Some(ACTION_ZELLIJ_CONFIG),
        SOURCE_YAZI_CONFIG => Some(ACTION_YAZI_CONFIG),
        SOURCE_YAZI_THEME => Some(ACTION_YAZI_THEME),
        _ => None,
    }
}
fn apply_status(summary: &str, label: &str, detail: &str) -> ConfigUiApplyStatus {
    ConfigUiApplyStatus {
        summary: summary.to_string(),
        label: label.to_string(),
        detail: detail.to_string(),
        pending: false,
    }
}

fn mars_apply_status(path: &str) -> ConfigUiApplyStatus {
    let (summary, label, detail) = match path {
        "window.width" | "window.height" => (
            "new windows",
            "mars",
            "Saved dimensions apply to newly created Mars windows; existing windows keep their size.",
        ),
        "window.opacity" | "fonts.size" | "line-height" | "enable-scroll-bar" => {
            ("live", "mars", "Saved values update open Mars windows.")
        }
        "bell.audio" | "bell.visual" => (
            "live",
            "mars",
            "Saved bell settings apply to the next bell in open Mars windows.",
        ),
        _ => unreachable!("Mars field {path} has no apply timing"),
    };

    apply_status(summary, label, detail)
}

pub(crate) fn zellij_apply_status(path: &str, runtime_active: bool) -> ConfigUiApplyStatus {
    match (path, runtime_active) {
        ("theme_dark" | "theme_light", true) => apply_status(
            "now/next mode",
            "zellij",
            "The active mode applies live inside a managed session; the other applies on the next mode change or launch.",
        ),
        (
            "pane_frames" | "copy_on_select" | "copy_clipboard" | "ui.pane_frames.rounded_corners",
            true,
        ) => apply_status(
            "now",
            "zellij",
            "Saved values update the active managed Zellij session.",
        ),
        (
            "theme_dark"
            | "theme_light"
            | "pane_frames"
            | "mouse_mode"
            | "scroll_buffer_size"
            | "copy_on_select"
            | "copy_clipboard"
            | "styled_underlines"
            | "show_startup_tips"
            | "ui.pane_frames.rounded_corners",
            _,
        ) => apply_status(
            "next session",
            "zellij",
            "Saved values apply to the next managed Zellij session.",
        ),
        _ => unreachable!("Zellij field {path} has no apply timing"),
    }
}
