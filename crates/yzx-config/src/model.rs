use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use ratconfig::toml_adapter::{get_toml_path, parse_toml_value};
use ratconfig::{
    ConfigUiApplyStatus, ConfigUiCapability, ConfigUiChoice, ConfigUiDiagnosticScope,
    ConfigUiFieldId, ConfigUiFieldSpec, ConfigUiListColumn, ConfigUiListTable, ConfigUiModel,
    ConfigUiOverride, ConfigUiSource, ConfigUiTextEncoding, ConfigUiTheme, ConfigUiThemeMapping,
    ConfigUiThemeSwitcher, ConfigUiTomlDocumentSpec, build_toml_document_fields,
};
use serde_json::Value as JsonValue;
use yazelix_cursors::{
    CursorRegistry, SUPPORTED_GLOW_LEVELS, SUPPORTED_MODE_EFFECTS, SUPPORTED_TRAIL_EFFECTS,
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
        packaged_zellij_defaults, packaged_zellij_theme_choices, parse_zellij_sidecar,
        read_zellij_sidecar,
    },
};

pub(crate) fn build_model(paths: &ConfigPaths) -> Result<ConfigUiModel> {
    let config_active = read_optional_toml_file_value(&paths.root, "config.toml")?;
    validate_root_config(&config_active)?;
    let config_default = default_config()?;
    let mars_active = read_optional_toml_file_value(&paths.mars, "invalid mars/config.toml")?;
    let mars_default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Mars config", error))?;
    let starship_active = read_optional_toml_file_value(&paths.starship, "invalid starship.toml")?;
    let starship_default = parse_toml_value(DEFAULT_STARSHIP_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Starship config", error))?;
    let cursors_active = yazelix_cursors::load_cursor_config(&paths.cursors)?;
    let cursors_default = cursor_defaults(&cursors_active)?;
    let (zellij_active, zellij_invalid, diagnostics) =
        parse_zellij_sidecar(&read_zellij_sidecar(&paths.zellij)?);
    let zellij_default = packaged_zellij_defaults();
    let yazi = build_yazi_fields(paths)?;

    let mut fields: Vec<_> = CONFIG_FIELDS
        .iter()
        .map(|spec| build_root_config_field(&config_active, &config_default, spec))
        .collect::<Result<_>>()?;
    fields.push(build_bar_widgets_field(&config_active, &config_default)?);
    fields.extend(build_custom_popup_fields(&paths.root)?);
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
            zellij_apply_status(spec.path),
            false,
        );
        if let Some(input) = zellij_invalid.get(spec.path) {
            field.snapshot.intent = ConfigUiOverride::Invalid {
                input: input.clone(),
            };
            field.snapshot.effective = None;
        }
        if spec.path == "theme" {
            let mut choices = packaged_zellij_theme_choices();
            if let Some(custom) = current.and_then(JsonValue::as_str)
                && !choices.iter().any(|theme| theme == custom)
            {
                choices.push(custom.to_string());
            }
            field.capability = string_choice_capability(choices);
        }
        fields.push(field);
    }
    let source = |id, label, path| build_config_source(paths, id, label, path);
    let yazi_dir = paths.yazi_config.parent().expect("Yazi config directory");
    fields.extend(yazi);
    let sources = vec![
        source(SOURCE_CONFIG, "config.toml", &paths.root),
        source(SOURCE_MARS, "mars/config.toml", &paths.mars),
        source(SOURCE_CURSORS, "cursors.toml", &paths.cursors),
        source(SOURCE_ZELLIJ, "zellij/config.kdl", &paths.zellij),
        source(SOURCE_STARSHIP, "starship.toml", &paths.starship),
        source(SOURCE_HELIX, "helix", &paths.helix_dir),
        source(SOURCE_YAZI_CONFIG, "yazi/yazi.toml", &paths.yazi_config),
        source(SOURCE_YAZI_THEME, "yazi/theme.toml", &paths.yazi_theme),
        ConfigUiSource {
            id: SOURCE_YAZI.to_string(),
            label: "yazi files".to_string(),
            path: yazi_dir.to_path_buf(),
            exists: yazi_dir.exists(),
            owner_label: Some("User".to_string()),
            read_only: false,
        },
        ConfigUiSource {
            id: SOURCE_ADVANCED.to_string(),
            label: "advanced files".to_string(),
            path: paths.root.parent().unwrap_or(Path::new(".")).to_path_buf(),
            exists: true,
            owner_label: Some("User".to_string()),
            read_only: false,
        },
        ConfigUiSource {
            id: SOURCE_KEYS.to_string(),
            label: "key bindings".to_string(),
            path: PathBuf::from("packaged-key-bindings"),
            exists: true,
            owner_label: Some("Packaged".to_string()),
            read_only: true,
        },
    ];
    apply_source_policy(&mut fields, &sources);
    let recommended_fields = Some(
        fields
            .iter()
            .filter(|field| {
                field.source_id != SOURCE_CONFIG
                    || ROOT_CONFIG_RECOMMENDED_PATHS.contains(&field.path.as_str())
                    || diagnostics.iter().any(|diagnostic| {
                        matches!(
                            &diagnostic.scope,
                            ConfigUiDiagnosticScope::Field(identity) if identity == &field.id()
                        )
                    })
            })
            .map(|field| field.id())
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
        file_actions: build_file_actions(paths),
        sidecars: Vec::new(),
        native_config_statuses: Vec::new(),
        diagnostics,
        theme_switcher: Some(ConfigUiThemeSwitcher {
            field: ConfigUiFieldId::new(SOURCE_MARS, MARS_APPEARANCE_PRESET_PATH),
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
fn build_key_binding_field(
    [group, chord, action, owner, source]: &[&str; 5],
) -> ratconfig::ConfigUiField {
    let value = JsonValue::String(format!("{owner} / {source}"));
    let mut spec = ConfigUiFieldSpec::new(
        SOURCE_KEYS,
        *chord,
        TAB_KEYS,
        format!("Group: {group}. Owner: {owner}. Source: {source}. Editable: no."),
        ConfigUiCapability::ReadOnly {
            reason: KEY_READ_ONLY_REASON.to_string(),
            file_action_id: None,
        },
        KEY_READ_ONLY_REASON,
        apply_status("read-only", "read-only", KEY_READ_ONLY_REASON),
    );
    spec.display_label = format!("{group}: {chord} - {action}");
    spec.list_cells = [*group, *chord, *action, *owner]
        .into_iter()
        .map(str::to_string)
        .collect();
    spec.build("string", Some(&value), None)
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
    Ok(build_config_field(
        SOURCE_CONFIG,
        root_config_tab(spec.field.path),
        &spec.field,
        current,
        Some(&default),
        apply_status(spec.apply_summary, "runtime", spec.apply_detail),
        false,
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
    fields.iter_mut().for_each(|field| field.list_cells.clear());
    Ok(fields)
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
    let mut field_spec = ConfigUiFieldSpec::new(
        source_id,
        spec.path,
        tab,
        spec.description,
        field_capability(spec),
        spec.validation,
        apply_status,
    );
    field_spec.can_unset = true;
    let mut field = field_spec.build(spec.kind, current, default);
    if invalid {
        field.snapshot.intent = ConfigUiOverride::Invalid {
            input: current.map_or_else(String::new, ratconfig::render_json_edit_value),
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
    let enabled = JsonValue::Array(
        active
            .enabled_cursors
            .iter()
            .cloned()
            .map(JsonValue::String)
            .collect(),
    );
    let enabled_baseline = JsonValue::Array(
        defaults
            .enabled_cursors
            .iter()
            .cloned()
            .map(JsonValue::String)
            .collect(),
    );
    let mut enabled_spec = ConfigUiFieldSpec::new(
        SOURCE_CURSORS,
        CURSOR_ENABLED_PATH,
        TAB_CURSORS,
        CURSOR_FIELDS[0].description,
        ConfigUiCapability::MultiChoice {
            choices: active
                .definitions
                .keys()
                .cloned()
                .map(JsonValue::String)
                .map(ConfigUiChoice::new)
                .collect(),
            ordered: true,
        },
        CURSOR_FIELDS[0].validation,
        cursor_apply_status(CURSOR_ENABLED_PATH),
    );
    enabled_spec.can_unset = false;
    let mut fields =
        vec![enabled_spec.build("string_list", Some(&enabled), Some(&enabled_baseline))];
    for spec in &CURSOR_FIELDS[1..] {
        let mut field = build_config_field(
            SOURCE_CURSORS,
            TAB_CURSORS,
            spec,
            get_toml_path(&active_json, spec.path),
            get_toml_path(&default_json, spec.path),
            cursor_apply_status(spec.path),
            false,
        );
        let choices = cursor_allowed_values(active, spec.path);
        if !choices.is_empty() {
            field.capability = string_choice_capability(choices);
        }
        field.can_unset = false;
        fields.push(field);
    }
    Ok(fields)
}
fn cursor_allowed_values(registry: &CursorRegistry, path: &str) -> Vec<String> {
    match path {
        CURSOR_TRAIL_PATH => registry
            .enabled_cursors
            .iter()
            .filter(|name| !matches!(name.as_str(), "random" | "none"))
            .map(String::as_str)
            .chain(["random", "none"])
            .map(str::to_string)
            .collect(),
        "settings.trail_effect" | "settings.mode_effect" => (if path == "settings.trail_effect" {
            SUPPORTED_TRAIL_EFFECTS
        } else {
            SUPPORTED_MODE_EFFECTS
        })
        .iter()
        .copied()
        .chain(["random", "none"])
        .map(str::to_string)
        .collect(),
        "settings.glow" => string_values(SUPPORTED_GLOW_LEVELS),
        _ => Vec::new(),
    }
}
fn cursor_apply_status(path: &str) -> ConfigUiApplyStatus {
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
        } else {
            "Mars reads the saved cursor pool and selection on its next launch."
        },
    )
}
fn build_bar_widgets_field(
    active: &JsonValue,
    defaults: &JsonValue,
) -> Result<ratconfig::ConfigUiField> {
    let current = get_toml_path(active, BAR_WIDGETS_PATH)
        .map(bar_widgets)
        .transpose()?;
    let default = bar_widgets(&default_config_path_value(defaults, BAR_WIDGETS_PATH)?)?;
    let current =
        current.map(|values| JsonValue::Array(values.into_iter().map(JsonValue::String).collect()));
    let default = JsonValue::Array(default.into_iter().map(JsonValue::String).collect());
    let mut spec = ConfigUiFieldSpec::new(
        SOURCE_CONFIG,
        BAR_WIDGETS_PATH,
        TAB_CONFIG,
        "Top bar widgets, left to right.",
        ConfigUiCapability::MultiChoice {
            choices: string_values(BAR_WIDGET_VALUES)
                .into_iter()
                .map(JsonValue::String)
                .map(ConfigUiChoice::new)
                .collect(),
            ordered: true,
        },
        "known widget ids",
        apply_status(
            "next launch",
            "bar",
            "Saved widget order applies to newly launched Yazelix sessions.",
        ),
    );
    spec.can_unset = true;
    Ok(spec.build("string_list", current.as_ref(), Some(&default)))
}

fn field_capability(spec: &FieldSpec) -> ConfigUiCapability {
    if spec.kind == "boolean" {
        return ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(JsonValue::Bool(false)),
            on: ConfigUiChoice::new(JsonValue::Bool(true)),
        };
    }
    if !spec.allowed_values.is_empty() {
        return string_choice_capability(string_values(spec.allowed_values));
    }
    ConfigUiCapability::FreeText {
        encoding: if spec.kind == "string" {
            ConfigUiTextEncoding::String
        } else {
            ConfigUiTextEncoding::Json
        },
    }
}

fn string_choice_capability(values: Vec<String>) -> ConfigUiCapability {
    ConfigUiCapability::Choice {
        choices: values
            .into_iter()
            .map(JsonValue::String)
            .map(ConfigUiChoice::new)
            .collect(),
    }
}

fn apply_source_policy(fields: &mut [ratconfig::ConfigUiField], sources: &[ConfigUiSource]) {
    for field in fields {
        let source = sources
            .iter()
            .find(|source| source.id == field.source_id)
            .expect("every field source is declared");
        if let Some(effective) = &mut field.snapshot.effective
            && effective.origin.is_none()
        {
            effective.origin = Some(match field.snapshot.intent {
                ConfigUiOverride::Absent => "Yazelix packaged baseline".to_string(),
                ConfigUiOverride::Explicit(_) | ConfigUiOverride::Invalid { .. } => source
                    .owner_label
                    .clone()
                    .unwrap_or_else(|| source.label.clone()),
            });
        }
        if let Some(baseline) = &mut field.snapshot.baseline
            && baseline.origin.is_none()
        {
            baseline.origin = Some("Yazelix packaged baseline".to_string());
        }
        if source.owner_label.as_deref() == Some("Home Manager") {
            field.snapshot.external_manager = Some("Home Manager".to_string());
            field.capability = ConfigUiCapability::ReadOnly {
                reason: "Managed by Home Manager.".to_string(),
                file_action_id: source_file_action(&field.source_id).map(str::to_string),
            };
            field.can_unset = false;
        }
    }
}

fn source_file_action(source_id: &str) -> Option<&'static str> {
    match source_id {
        SOURCE_CURSORS => Some(ACTION_CURSORS_CONFIG),
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
        MARS_APPEARANCE_PRESET_PATH => (
            "live",
            "mars/ui",
            "Saved appearance changes apply live to Mars and this config UI.",
        ),
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

fn zellij_apply_status(path: &str) -> ConfigUiApplyStatus {
    if path == "theme" {
        return apply_status(
            "live",
            "zellij",
            "Inside a managed session, saved themes update the watched active config; outside a session they apply on the next launch.",
        );
    }
    apply_status(
        "session",
        "zellij",
        "Inside a session, saves and resets update the active config; many scalars apply live, some need a new session.",
    )
}
