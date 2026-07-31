use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use ratconfig::{
    ConfigUiApplyStatus, ConfigUiCapability, ConfigUiChoice, ConfigUiField, ConfigUiFieldSpec,
    ConfigUiOverride, ConfigUiResolvedValue, ConfigUiSchemaField, ConfigUiTextEncoding,
    ConfigUiTomlDocumentSpec, build_toml_document_fields, collect_config_ui_schema_fields,
    toml_adapter::{set_toml_value_text, unset_toml_value_text},
};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use crate::{catalog::*, common::*, paths::ConfigPaths};

const YAZI_NATIVE_DARK_LABEL: &str = "default";
const YAZI_SCHEMA_DESCRIPTION: &str = "Setting published by the pinned Yazi schema.";

pub(crate) const YAZI_RECOMMENDED_FIELDS: &[(&str, &str)] = &[
    (SOURCE_YAZI_CONFIG, "mgr.ratio"),
    (SOURCE_YAZI_CONFIG, "mgr.sort_by"),
    (SOURCE_YAZI_CONFIG, "mgr.sort_reverse"),
    (SOURCE_YAZI_CONFIG, "mgr.sort_dir_first"),
    (SOURCE_YAZI_CONFIG, "mgr.linemode"),
    (SOURCE_YAZI_CONFIG, "mgr.show_hidden"),
    (SOURCE_YAZI_CONFIG, "mgr.show_symlink"),
    (SOURCE_YAZI_CONFIG, "preview.wrap"),
    (SOURCE_YAZI_THEME, "flavor.dark"),
    (SOURCE_YAZI_THEME, "flavor.light"),
];

pub(crate) fn build_yazi_fields(paths: &ConfigPaths, light: bool) -> Result<Vec<ConfigUiField>> {
    let mut baseline = parse_packaged_toml(paths, "yazi-default.toml")?;
    deep_merge_toml(&mut baseline, &parse_packaged_toml(paths, "yazi.toml")?);
    let baseline = toml::to_string(&baseline)
        .map_err(|source| boxed_debug("could not render Yazi baseline", source))?;
    let current = read_optional_text(&paths.yazi_config)?;
    let settings = build_schema_document_fields(
        paths,
        SOURCE_YAZI_CONFIG,
        "yazi-schema.json",
        "Yazi settings",
        &current,
        &baseline,
        ACTION_YAZI_CONFIG,
    )?;
    let theme = read_optional_text(&paths.yazi_theme)?;
    let theme_baseline = fs::read_to_string(paths.packaged_yazi.join(if light {
        "theme-light.toml"
    } else {
        "theme-dark.toml"
    }))?;
    let mut appearance = build_schema_document_fields(
        paths,
        SOURCE_YAZI_THEME,
        "theme-schema.json",
        "Appearance",
        &theme,
        &theme_baseline,
        ACTION_YAZI_THEME,
    )?;
    let flavors = discovered_flavors(paths)?;
    for field in &mut appearance {
        let (label, choices, native_default) = match field.path.as_str() {
            "flavor.dark" => ("Dark flavor", &flavors.dark, true),
            "flavor.light" => ("Light flavor", &flavors.light, false),
            _ => continue,
        };
        let mut available = Vec::with_capacity(choices.len() + usize::from(native_default));
        let patchable = !matches!(field.capability, ConfigUiCapability::ReadOnly { .. });
        if native_default {
            available.push(ConfigUiChoice {
                value: JsonValue::Null,
                label: Some(YAZI_NATIVE_DARK_LABEL.to_string()),
            });
        }
        available.extend(choices.iter().cloned().map(|value| {
            ConfigUiChoice {
                label: (native_default && value == YAZI_NATIVE_DARK_LABEL)
                    .then(|| "default (installed flavor)".to_string()),
                value: JsonValue::String(value),
            }
        }));
        field.display_label = label.to_string();
        field.type_label = Some("string".to_string());
        field.capability = if !patchable {
            read_only(ACTION_YAZI_THEME)
        } else if available.is_empty() {
            ConfigUiCapability::ReadOnly {
                reason: format!("No installed {label} choices were discovered."),
                file_action_id: Some(ACTION_YAZI_THEME.to_string()),
            }
        } else {
            ConfigUiCapability::Choice { choices: available }
        };
        if let ConfigUiOverride::Explicit(value) = &field.snapshot.intent {
            if value
                .as_str()
                .is_some_and(|value| choices.iter().any(|flavor| flavor == value))
            {
                field.snapshot.effective = Some(ConfigUiResolvedValue {
                    value: value.clone(),
                    origin: Some("User yazi/theme.toml".to_string()),
                });
            } else {
                field.snapshot.intent = ConfigUiOverride::Invalid {
                    input: value.to_string(),
                };
                field.snapshot.effective = None;
            }
        }
        field.snapshot.baseline = match field.path.as_str() {
            "flavor.dark" => Some(ConfigUiResolvedValue {
                value: JsonValue::Null,
                origin: Some("Yazi native preset".to_string()),
            }),
            "flavor.light" => flavors
                .default_light
                .as_ref()
                .map(|flavor| ConfigUiResolvedValue {
                    value: JsonValue::String(flavor.clone()),
                    origin: Some("Yazelix appearance default".to_string()),
                }),
            _ => None,
        };
        if matches!(field.snapshot.intent, ConfigUiOverride::Absent) {
            field
                .snapshot
                .effective
                .clone_from(&field.snapshot.baseline);
        }
        field.validation = if native_default {
            "default (Yazi native preset) or an installed dark flavor"
        } else {
            "installed light flavor"
        }
        .to_string();
        field.description = if native_default {
            "Dark flavor from native yazi/theme.toml. Packaged choices follow Yazi Bistro's classification; user-installed flavors appear in both pools. Selecting default removes flavor.dark and uses Yazi's native preset."
        } else {
            "Light flavor from native yazi/theme.toml. Packaged choices follow Yazi Bistro's classification; user-installed flavors appear in both pools. Reset inherits Yazelix's packaged light default."
        }
        .to_string();
    }
    appearance.extend(settings);
    Ok(appearance)
}

pub(crate) fn write_yazi_field(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let path = yazi_source_path(paths, source_id)?;
    paths.reject_mutation(path, source_id)?;
    if source_id == SOURCE_YAZI_THEME && field_path == "flavor.dark" && value.is_null() {
        return unset_yazi_path(path, field_path);
    }
    if source_id == SOURCE_YAZI_THEME && matches!(field_path, "flavor.dark" | "flavor.light") {
        let flavors = discovered_flavors(paths)?;
        let choices = if field_path == "flavor.dark" {
            &flavors.dark
        } else {
            &flavors.light
        };
        if !value
            .as_str()
            .is_some_and(|value| choices.iter().any(|flavor| flavor == value))
        {
            return Err(error(format!(
                "{field_path} must name an installed flavor: {}",
                choices.join(", ")
            )));
        }
    } else {
        validate_schema_write(paths, source_id, field_path, value)?;
    }
    let text = set_toml_value_text(&read_optional_text(path)?, field_path, value)
        .map_err(|error| boxed_debug("could not update native Yazi TOML", error))?
        .text;
    atomic_write(path, &text)
}

pub(crate) fn unset_yazi_field(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
) -> Result<()> {
    let path = yazi_source_path(paths, source_id)?;
    paths.reject_mutation(path, source_id)?;
    unset_yazi_path(path, field_path)
}

fn unset_yazi_path(path: &Path, field_path: &str) -> Result<()> {
    if !path_entry_exists(path)? {
        return Ok(());
    }
    let text = unset_toml_value_text(&fs::read_to_string(path)?, field_path)
        .map_err(|error| boxed_debug("could not reset native Yazi TOML", error))?
        .text;
    if text.trim().is_empty() {
        fs::remove_file(path)?;
        Ok(())
    } else {
        atomic_write(path, &text)
    }
}

fn document<'a>(
    source_id: &'a str,
    section_label: &'a str,
    current_toml: &'a str,
    default_toml: &'a str,
) -> ConfigUiTomlDocumentSpec<'a> {
    ConfigUiTomlDocumentSpec {
        source_id,
        tab: TAB_YAZI,
        section_label,
        current_toml,
        baseline_toml: Some(default_toml),
        validation: "native TOML value of the existing type",
        rebuild_required: false,
        apply_status: ConfigUiApplyStatus {
            summary: "next Yazi".to_string(),
            label: "yazi".to_string(),
            detail: "Saved native values apply on the next managed Yazi launch or sidebar reopen."
                .to_string(),
            pending: false,
        },
    }
}

fn build_schema_document_fields(
    paths: &ConfigPaths,
    source_id: &str,
    schema_name: &str,
    section_label: &str,
    current: &str,
    baseline: &str,
    file_action_id: &str,
) -> Result<Vec<ConfigUiField>> {
    let schema = schema_fields(paths, schema_name)?;
    let document =
        build_toml_document_fields(document(source_id, section_label, current, baseline))
            .map_err(|source| error(source.to_string()))?;
    let mut observed = document
        .fields
        .into_iter()
        .map(|field| (field.path.clone(), field))
        .collect::<BTreeMap<_, _>>();
    let mut fields = Vec::new();

    for spec in &schema {
        if spec.path == "$schema" {
            if let Some(mut field) = observed.remove(r#""$schema""#) {
                field.display_label = spec.path.clone();
                configure_schema_field(&mut field, spec, current, file_action_id);
                fields.push(field);
            }
            continue;
        }
        if spec.kind == "object" {
            let prefix = format!("{}.", spec.path);
            let dynamic = observed
                .extract_if(.., |path, _| path.starts_with(&prefix))
                .map(|(_, mut field)| {
                    field.section_label = schema_section(section_label, &field.path);
                    field.capability = read_only(file_action_id);
                    field.can_unset =
                        matches!(field.snapshot.intent, ConfigUiOverride::Explicit(_))
                            && unset_toml_value_text(current, &field.path).is_ok();
                    field
                });
            fields.extend(dynamic);
            continue;
        }

        let mut field = observed
            .remove(&spec.path)
            .unwrap_or_else(|| absent_schema_field(source_id, section_label, spec, file_action_id));
        configure_schema_field(&mut field, spec, current, file_action_id);
        fields.push(field);
    }

    for (_, mut field) in observed {
        if schema
            .iter()
            .any(|spec| spec.kind != "object" && field.path.starts_with(&format!("{}.", spec.path)))
        {
            continue;
        }
        let ConfigUiOverride::Explicit(value) = &field.snapshot.intent else {
            continue;
        };
        let input = value.to_string();
        field.section_label = schema_section(section_label, &field.path);
        field.snapshot.intent = ConfigUiOverride::Invalid { input };
        field.snapshot.effective = None;
        field.capability = read_only(file_action_id);
        field.can_unset = unset_toml_value_text(current, &field.path).is_ok();
        fields.push(field);
    }
    remove_toml_parent_fields(&mut fields, None);
    fields.sort_by(|left, right| {
        left.section_label
            .cmp(&right.section_label)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(fields)
}

fn configure_schema_field(
    field: &mut ConfigUiField,
    spec: &ConfigUiSchemaField,
    current: &str,
    file_action_id: &str,
) {
    field.section_label = schema_section(&field.section_label, &field.path);
    let capability = schema_capability(spec);
    let explicit = matches!(field.snapshot.intent, ConfigUiOverride::Explicit(_));
    let path_is_patchable = unset_toml_value_text(current, &field.path).is_ok();
    field.can_unset = explicit && path_is_patchable;
    field.description = YAZI_SCHEMA_DESCRIPTION.to_string();
    field.validation = schema_validation(spec);
    if let (Some(capability), ConfigUiOverride::Explicit(value)) =
        (&capability, &field.snapshot.intent)
    {
        if capability_accepts(capability, value) {
            field.snapshot.effective = Some(ConfigUiResolvedValue {
                value: value.clone(),
                origin: Some("User native Yazi TOML".to_string()),
            });
        } else {
            field.snapshot.intent = ConfigUiOverride::Invalid {
                input: value.to_string(),
            };
            field.snapshot.effective = None;
        }
    }
    field.capability = capability
        .filter(|_| path_is_patchable)
        .unwrap_or_else(|| read_only(file_action_id));
    if let Some(baseline) = &mut field.snapshot.baseline {
        baseline.origin = Some("Pinned Yazi preset".to_string());
    }
    if matches!(field.snapshot.intent, ConfigUiOverride::Absent)
        && let Some(effective) = &mut field.snapshot.effective
    {
        effective.origin = Some("Pinned Yazi preset".to_string());
    }
}

fn absent_schema_field(
    source_id: &str,
    section_label: &str,
    spec: &ConfigUiSchemaField,
    file_action_id: &str,
) -> ConfigUiField {
    let path = spec.path.clone();
    let mut field = ConfigUiFieldSpec::new(
        source_id,
        &path,
        TAB_YAZI,
        YAZI_SCHEMA_DESCRIPTION,
        schema_capability(spec).unwrap_or_else(|| read_only(file_action_id)),
        schema_validation(spec),
        document(source_id, section_label, "", "").apply_status,
    )
    .build(schema_type_label(spec), None, None);
    field.display_label.clone_from(&path);
    field.section_label = section_label.to_string();
    field
}

pub(crate) fn schema_fields(
    paths: &ConfigPaths,
    schema_name: &str,
) -> Result<Vec<ConfigUiSchemaField>> {
    let schema: JsonValue =
        serde_json::from_str(&fs::read_to_string(paths.packaged_yazi.join(schema_name))?)
            .map_err(|source| boxed_debug("invalid packaged Yazi schema", source))?;
    Ok(collect_config_ui_schema_fields(&schema, "root")
        .into_iter()
        .map(|mut field| {
            field.path = field
                .path
                .strip_prefix("root.")
                .unwrap_or(&field.path)
                .to_string();
            field
        })
        .collect())
}

fn schema_capability(spec: &ConfigUiSchemaField) -> Option<ConfigUiCapability> {
    if !spec.allowed_values.is_empty() {
        return Some(ConfigUiCapability::Choice {
            choices: spec
                .allowed_values
                .iter()
                .cloned()
                .map(JsonValue::String)
                .map(ConfigUiChoice::new)
                .collect(),
        });
    }
    match spec.kind.as_str() {
        "boolean" => Some(ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(JsonValue::Bool(false)),
            on: ConfigUiChoice::new(JsonValue::Bool(true)),
        }),
        "string" => Some(ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        }),
        _ => None,
    }
}

fn capability_accepts(capability: &ConfigUiCapability, value: &JsonValue) -> bool {
    match capability {
        ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        } => value.is_string(),
        ConfigUiCapability::Toggle { off, on } => value == &off.value || value == &on.value,
        ConfigUiCapability::Choice { choices } => {
            choices.iter().any(|choice| value == &choice.value)
        }
        _ => false,
    }
}

fn validate_schema_write(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let schema_name = match source_id {
        SOURCE_YAZI_CONFIG => "yazi-schema.json",
        SOURCE_YAZI_THEME => "theme-schema.json",
        _ => return Err(error(format!("unknown Yazi config source: {source_id}"))),
    };
    let fields = schema_fields(paths, schema_name)?;
    let spec = fields
        .iter()
        .find(|field| field.path == field_path)
        .ok_or_else(|| error(format!("{field_path} is not a finite Yazi schema field")))?;
    let capability = schema_capability(spec)
        .ok_or_else(|| error(format!("{field_path} requires native Yazi file editing")))?;
    if !capability_accepts(&capability, value) {
        return Err(error(format!(
            "{field_path} must satisfy the pinned Yazi schema"
        )));
    }
    Ok(())
}

fn schema_validation(spec: &ConfigUiSchemaField) -> String {
    if spec.allowed_values.is_empty() {
        format!("pinned Yazi schema {}", schema_type_label(spec))
    } else {
        format!("one of: {}", spec.allowed_values.join(", "))
    }
}

fn schema_type_label(spec: &ConfigUiSchemaField) -> &str {
    match spec.kind.as_str() {
        "unknown" if !spec.allowed_values.is_empty() => "string",
        "unknown" => "owner-defined",
        kind => kind,
    }
}

fn schema_section(label: &str, path: &str) -> String {
    if path.starts_with('"') {
        label.to_string()
    } else {
        format!("{label} · {}", path.split('.').next().unwrap_or(path))
    }
}

fn read_only(file_action_id: &str) -> ConfigUiCapability {
    ConfigUiCapability::ReadOnly {
        reason: "This Yazi value cannot be edited safely inline.".to_string(),
        file_action_id: Some(file_action_id.to_string()),
    }
}

fn parse_packaged_toml(paths: &ConfigPaths, name: &str) -> Result<TomlValue> {
    toml::from_str(&fs::read_to_string(paths.packaged_yazi.join(name))?)
        .map_err(|source| boxed_debug("invalid packaged Yazi TOML", source))
}

fn yazi_source_path<'a>(paths: &'a ConfigPaths, source_id: &str) -> Result<&'a Path> {
    match source_id {
        SOURCE_YAZI_CONFIG => Ok(&paths.yazi_config),
        SOURCE_YAZI_THEME => Ok(&paths.yazi_theme),
        _ => Err(error(format!("unknown Yazi config source: {source_id}"))),
    }
}

struct FlavorPools {
    dark: Vec<String>,
    light: Vec<String>,
    default_light: Option<String>,
}

fn discovered_flavors(paths: &ConfigPaths) -> Result<FlavorPools> {
    let packaged = flavor_names(&paths.packaged_yazi.join("flavors"))?;
    let user = flavor_names(&paths.yazi_config.with_file_name("flavors"))?;
    let mut dark = user.clone();
    let mut light = user;
    let mut default_light = None;
    if !packaged.is_empty() {
        let catalog_path = paths.packaged_yazi.join("catalog.toml");
        let catalog: TomlValue = toml::from_str(&fs::read_to_string(&catalog_path)?)
            .map_err(|source| error(format!("invalid Yazi Bistro catalog: {source}")))?;
        let light_default = catalog
            .get("default_light")
            .and_then(TomlValue::as_str)
            .ok_or_else(|| error("Yazi Bistro catalog is missing its light default"))?
            .to_string();
        let flavors = catalog
            .get("flavors")
            .and_then(TomlValue::as_table)
            .ok_or_else(|| error("Yazi Bistro catalog is missing its flavors table"))?;
        for name in packaged {
            let mode = flavors
                .get(&name)
                .and_then(|flavor| flavor.get("mode"))
                .and_then(TomlValue::as_str)
                .ok_or_else(|| error(format!("Yazi Bistro catalog does not classify {name}")))?;
            match mode {
                "dark" => {
                    dark.insert(name);
                }
                "light" => {
                    light.insert(name);
                }
                _ => {
                    return Err(error(format!(
                        "Yazi Bistro catalog has invalid mode for {name}"
                    )));
                }
            }
        }
        if !light.contains(&light_default) {
            return Err(error(
                "Yazi Bistro light default is not an installed light flavor",
            ));
        }
        default_light = Some(light_default);
    }
    Ok(FlavorPools {
        dark: dark.into_iter().collect(),
        light: light.into_iter().collect(),
        default_light,
    })
}

fn flavor_names(directory: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    if !directory.is_dir() {
        return Ok(names);
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let Some(name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".yazi").filter(|name| !name.is_empty()))
        else {
            continue;
        };
        if path.join("flavor.toml").is_file() {
            names.insert(name.to_string());
        }
    }
    Ok(names)
}
