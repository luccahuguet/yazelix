use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use ratconfig::toml_adapter::{get_toml_path, parse_toml_value};
use ratconfig::{
    ConfigUiApplyStatus, ConfigUiEditBehavior, ConfigUiFieldRowSpec, ConfigUiListColumn,
    ConfigUiListTable, ConfigUiModel, ConfigUiPathOwner, ConfigUiSource,
    ConfigUiStringListChoiceSpec, ConfigUiTheme, ConfigUiThemeMapping, ConfigUiThemeSwitcher,
    build_config_ui_field, build_string_list_choice_field,
};
use serde_json::Value as JsonValue;

use crate::{
    catalog::*,
    common::*,
    file_actions::build_file_actions,
    native_config::{validate_mars_field, validate_starship_field},
    paths::ConfigPaths,
    root_config::{
        bar_widgets, default_config, default_config_path_value, popup_keybinding_spec,
        read_toml_file_value, validate_config_value, validate_popup_keybindings,
    },
    zellij_sidecar::{ZellijSidecar, parse_zellij_sidecar, zellij_field_value},
};

pub(crate) fn build_model(paths: &ConfigPaths) -> Result<ConfigUiModel> {
    let config_active = read_toml_file_value(&paths.root, "config.toml")?;
    let config_default = default_config()?;
    let mars_active = read_toml_file_value(&paths.mars, "invalid mars/config.toml")?;
    let mars_default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Mars config", error))?;
    let starship_active = read_toml_file_value(&paths.starship, "invalid starship.toml")?;
    let starship_default = parse_toml_value(DEFAULT_STARSHIP_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Starship config", error))?;
    let (zellij_active, diagnostics) = parse_zellij_sidecar(&fs::read_to_string(&paths.zellij)?);
    let zellij_default = ZellijSidecar::default();
    let zellij_blocking = diagnostics.iter().any(|diagnostic| diagnostic.blocking);

    let mut fields: Vec<_> = CONFIG_FIELDS
        .iter()
        .map(|spec| build_root_config_field(&config_active, &config_default, spec))
        .collect::<Result<_>>()?;
    fields.push(build_bar_widgets_field(&config_active, &config_default)?);
    fields.extend(KEY_BINDINGS.iter().map(build_key_binding_field));
    for spec in MARS_FIELDS {
        fields.push(build_config_field(
            SOURCE_MARS,
            TAB_MARS,
            spec,
            get_toml_path(&mars_active, spec.path),
            get_toml_path(&mars_default, spec.path),
            mars_apply_status(spec.path),
            get_toml_path(&mars_active, spec.path)
                .is_some_and(|value| validate_mars_field(spec, value).is_err()),
        ));
    }
    for spec in STARSHIP_FIELDS {
        fields.push(build_config_field(
            SOURCE_STARSHIP,
            TAB_STARSHIP,
            spec,
            get_toml_path(&starship_active, spec.path),
            get_toml_path(&starship_default, spec.path),
            ConfigUiApplyStatus {
                summary: "new prompts".to_string(),
                label: "starship".to_string(),
                detail: "Saved values apply to newly rendered managed Nu prompts.".to_string(),
                pending: false,
            },
            get_toml_path(&starship_active, spec.path)
                .is_some_and(|value| validate_starship_field(spec, value).is_err()),
        ));
    }
    for spec in ZELLIJ_FIELDS {
        let current = zellij_field_value(&zellij_active, spec.path);
        let default = zellij_field_value(&zellij_default, spec.path);
        fields.push(build_config_field(
            SOURCE_ZELLIJ,
            TAB_ZELLIJ,
            spec,
            Some(&current),
            Some(&default),
            ConfigUiApplyStatus {
                summary: "session".to_string(),
                label: "zellij".to_string(),
                detail: "Inside a session, saves update the active config; many scalars apply live, some need a new session.".to_string(),
                pending: false,
            },
            zellij_blocking,
        ));
    }

    Ok(ConfigUiModel {
        active_config_path: paths.root.clone(),
        cursor_config_path: PathBuf::new(),
        default_cursor_config_path: PathBuf::new(),
        active_config_exists: paths.root.exists(),
        config_owner: ConfigUiPathOwner::User,
        config_read_only: path_read_only(&paths.root),
        sources: vec![
            build_config_source(SOURCE_CONFIG, TAB_CONFIG, "config.toml", &paths.root),
            build_config_source(SOURCE_MARS, TAB_MARS, "mars/config.toml", &paths.mars),
            build_config_source(
                SOURCE_ZELLIJ,
                TAB_ZELLIJ,
                "zellij/config.kdl",
                &paths.zellij,
            ),
            build_config_source(
                SOURCE_STARSHIP,
                TAB_STARSHIP,
                "starship.toml",
                &paths.starship,
            ),
            build_config_source(SOURCE_HELIX, TAB_HELIX, "helix", &paths.helix_dir),
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
            TAB_MARS.to_string(),
            TAB_ZELLIJ.to_string(),
            TAB_STARSHIP.to_string(),
            TAB_HELIX.to_string(),
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
        fields,
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
        apply_status: ConfigUiApplyStatus {
            summary: "read-only".to_string(),
            label: "read-only".to_string(),
            detail: KEY_READ_ONLY_REASON.to_string(),
            pending: false,
        },
        edit_behavior: ConfigUiEditBehavior::StructuredOnly {
            notice: KEY_READ_ONLY_REASON.to_string(),
        },
    }
}
fn build_config_source(id: &str, tab: &str, label: &str, path: &Path) -> ConfigUiSource {
    ConfigUiSource {
        id: id.to_string(),
        tab: tab.to_string(),
        label: label.to_string(),
        path: path.to_path_buf(),
        exists: path.exists(),
        owner: ConfigUiPathOwner::User,
        read_only: path_read_only(path),
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
        TAB_CONFIG,
        &spec.field,
        current,
        Some(&default),
        ConfigUiApplyStatus {
            summary: spec.apply_summary.to_string(),
            label: "runtime".to_string(),
            detail: spec.apply_detail.to_string(),
            pending: false,
        },
        current.is_some_and(|value| validate_config_value(spec.field.path, value).is_err())
            || (popup_keybinding_spec(spec.field.path).is_some()
                && validate_popup_keybindings(active).is_err()),
    ))
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
    build_config_ui_field(ConfigUiFieldRowSpec {
        source_id,
        path: spec.path,
        display_label: String::new(),
        list_cells: Vec::new(),
        tab,
        kind: spec.kind,
        current,
        default,
        description: spec.description.to_string(),
        allowed_values: string_values(spec.allowed_values),
        validation: spec.validation.to_string(),
        rebuild_required: false,
        apply_status,
        has_blocking_diagnostic,
        edit_behavior: ConfigUiEditBehavior::Default,
    })
}
fn build_bar_widgets_field(
    active: &JsonValue,
    defaults: &JsonValue,
) -> Result<ratconfig::ConfigUiField> {
    let current = get_toml_path(active, BAR_WIDGETS_PATH)
        .map(bar_widgets)
        .transpose();
    let has_blocking_diagnostic = current.is_err();
    let default = bar_widgets(&default_config_path_value(defaults, BAR_WIDGETS_PATH)?)?;
    build_string_list_choice_field(ConfigUiStringListChoiceSpec {
        source_id: SOURCE_CONFIG.to_string(),
        path: BAR_WIDGETS_PATH.to_string(),
        display_label: String::new(),
        list_cells: Vec::new(),
        tab: TAB_CONFIG.to_string(),
        current: current.ok().flatten(),
        default: Some(default),
        description: "Top bar widgets, left to right.".to_string(),
        allowed_values: string_values(BAR_WIDGET_VALUES),
        validation: "known widget ids".to_string(),
        rebuild_required: false,
        apply_status: ConfigUiApplyStatus {
            summary: "next launch".to_string(),
            label: "bar".to_string(),
            detail: "Saved widget order applies to newly launched Yazelix sessions.".to_string(),
            pending: false,
        },
        has_blocking_diagnostic,
        edit_behavior: ConfigUiEditBehavior::OrderedStringList,
    })
    .map_err(error)
}
fn next_launch_apply_status(label: &str, detail: &str) -> ConfigUiApplyStatus {
    ConfigUiApplyStatus {
        summary: "next launch".to_string(),
        label: label.to_string(),
        detail: detail.to_string(),
        pending: false,
    }
}

fn mars_apply_status(path: &str) -> ConfigUiApplyStatus {
    if path == MARS_APPEARANCE_PRESET_PATH {
        ConfigUiApplyStatus {
            summary: "live".to_string(),
            label: "mars/ui".to_string(),
            detail: "Saved appearance changes apply to Mars and this config UI immediately."
                .to_string(),
            pending: false,
        }
    } else {
        next_launch_apply_status("mars", "Saved values apply to newly launched Mars windows.")
    }
}
