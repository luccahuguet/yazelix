use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use ratconfig::toml_adapter::{get_toml_path, parse_toml_value};
use ratconfig::{
    ConfigUiApplyStatus, ConfigUiEditBehavior, ConfigUiFieldSpec, ConfigUiListColumn,
    ConfigUiListTable, ConfigUiModel, ConfigUiPathOwner, ConfigUiSource, ConfigUiTheme,
    ConfigUiThemeMapping, ConfigUiThemeSwitcher,
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
    zellij_sidecar::{packaged_zellij_defaults, parse_zellij_sidecar, read_zellij_sidecar},
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
    let (zellij_active, diagnostics) = parse_zellij_sidecar(&read_zellij_sidecar(&paths.zellij)?);
    let zellij_default = packaged_zellij_defaults();
    let zellij_blocking = diagnostics.iter().any(|diagnostic| diagnostic.blocking);
    let yazi = build_yazi_fields(paths)?;

    let mut fields: Vec<_> = CONFIG_FIELDS
        .iter()
        .map(|spec| build_root_config_field(&config_active, &config_default, spec))
        .collect::<Result<_>>()?;
    fields.push(build_bar_widgets_field(&config_active, &config_default)?);
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
        fields.push(build_config_field(
            SOURCE_ZELLIJ,
            TAB_ZELLIJ,
            spec,
            current,
            Some(default),
            apply_status(
                "session",
                "zellij",
                "Inside a session, saves and resets update the active config; many scalars apply live, some need a new session.",
            ),
            zellij_blocking,
        ));
    }
    let source = |id, tab, label, path| build_config_source(paths, id, tab, label, path);
    let yazi_dir = paths.yazi_config.parent().expect("Yazi config directory");

    Ok(ConfigUiModel {
        sources: vec![
            source(SOURCE_CONFIG, TAB_CONFIG, "config.toml", &paths.root),
            source(SOURCE_CONFIG, TAB_POPUPS, "config.toml", &paths.root),
            source(SOURCE_MARS, TAB_MARS, "mars/config.toml", &paths.mars),
            source(SOURCE_CURSORS, TAB_CURSORS, "cursors.toml", &paths.cursors),
            source(
                SOURCE_ZELLIJ,
                TAB_ZELLIJ,
                "zellij/config.kdl",
                &paths.zellij,
            ),
            source(
                SOURCE_STARSHIP,
                TAB_STARSHIP,
                "starship.toml",
                &paths.starship,
            ),
            source(SOURCE_HELIX, TAB_HELIX, "helix", &paths.helix_dir),
            source(SOURCE_YAZI, TAB_YAZI, "yazi", yazi_dir),
            ConfigUiSource {
                id: SOURCE_KEYS.to_string(),
                tab: TAB_KEYS.to_string(),
                label: "key bindings".to_string(),
                path: PathBuf::from("packaged-key-bindings"),
                exists: true,
                owner: ConfigUiPathOwner::Default,
                read_only: true,
            },
        ],
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
        fields: fields.into_iter().chain(yazi).collect(),
        file_actions: build_file_actions(paths),
        sidecars: Vec::new(),
        native_config_statuses: Vec::new(),
        diagnostics,
        theme_switcher: Some(ConfigUiThemeSwitcher {
            source_id: SOURCE_MARS.to_string(),
            field_path: MARS_APPEARANCE_PRESET_PATH.to_string(),
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
        kind: "string".to_string(),
        current_value: format!("{owner} / {source}"),
        edit_value: String::new(),
        default_value: ratconfig::NO_CONFIG_DEFAULT_VALUE_LABEL.to_string(),
        state: ratconfig::ConfigUiValueState::Explicit,
        description: format!("Group: {group}. Owner: {owner}. Source: {source}. Editable: no."),
        allowed_values: Vec::new(),
        validation: KEY_READ_ONLY_REASON.to_string(),
        rebuild_required: false,
        apply_status: apply_status("read-only", "read-only", KEY_READ_ONLY_REASON),
        edit_behavior: ConfigUiEditBehavior::StructuredOnly {
            notice: KEY_READ_ONLY_REASON.to_string(),
        },
    }
}
fn build_config_source(
    paths: &ConfigPaths,
    id: &str,
    tab: &str,
    label: &str,
    path: &Path,
) -> ConfigUiSource {
    let home_manager_owned = paths.is_home_manager_owned(path);
    ConfigUiSource {
        id: id.to_string(),
        tab: tab.to_string(),
        label: label.to_string(),
        path: path.to_path_buf(),
        exists: path.exists(),
        owner: if home_manager_owned {
            ConfigUiPathOwner::HomeManager
        } else {
            ConfigUiPathOwner::User
        },
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
    has_blocking_diagnostic: bool,
) -> ratconfig::ConfigUiField {
    ConfigUiFieldSpec {
        has_blocking_diagnostic,
        ..ConfigUiFieldSpec::new(
            source_id,
            spec.path,
            tab,
            spec.description,
            string_values(spec.allowed_values),
            spec.validation,
            apply_status,
        )
    }
    .build(spec.kind, current, default)
}
fn build_cursor_fields(
    active: &CursorRegistry,
    defaults: &CursorRegistry,
) -> Result<Vec<ratconfig::ConfigUiField>> {
    let active_json = serde_json::to_value(active)?;
    let default_json = serde_json::to_value(defaults)?;
    let mut fields = vec![
        ConfigUiFieldSpec {
            edit_behavior: ConfigUiEditBehavior::OrderedStringList,
            ..ConfigUiFieldSpec::new(
                SOURCE_CURSORS,
                CURSOR_ENABLED_PATH,
                TAB_CURSORS,
                CURSOR_FIELDS[0].description,
                active.definitions.keys().cloned().collect(),
                CURSOR_FIELDS[0].validation,
                cursor_apply_status(CURSOR_ENABLED_PATH),
            )
        }
        .build_string_list(
            Some(active.enabled_cursors.clone()),
            Some(defaults.enabled_cursors.clone()),
        )
        .map_err(error)?,
    ];
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
        field.allowed_values = cursor_allowed_values(active, spec.path);
        fields.push(field);
    }
    Ok(fields)
}
fn cursor_allowed_values(registry: &CursorRegistry, path: &str) -> Vec<String> {
    match path {
        CURSOR_TRAIL_PATH => registry
            .enabled_cursors
            .iter()
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
    ConfigUiFieldSpec {
        edit_behavior: ConfigUiEditBehavior::OrderedStringList,
        ..ConfigUiFieldSpec::new(
            SOURCE_CONFIG,
            BAR_WIDGETS_PATH,
            TAB_CONFIG,
            "Top bar widgets, left to right.",
            string_values(BAR_WIDGET_VALUES),
            "known widget ids",
            apply_status(
                "next launch",
                "bar",
                "Saved widget order applies to newly launched Yazelix sessions.",
            ),
        )
    }
    .build_string_list(current, Some(default))
    .map_err(error)
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
