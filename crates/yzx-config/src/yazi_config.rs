use std::{collections::BTreeSet, fs, path::Path};

use ratconfig::{
    ConfigUiApplyStatus, ConfigUiCapability, ConfigUiChoice, ConfigUiField, ConfigUiOverride,
    ConfigUiResolvedValue, ConfigUiTomlDocumentSpec, build_toml_document_fields,
    toml_adapter::{set_toml_value_text, unset_toml_value_text},
};
use serde_json::Value as JsonValue;

use crate::{catalog::*, common::*, paths::ConfigPaths};

pub(crate) fn build_yazi_fields(paths: &ConfigPaths) -> Result<Vec<ConfigUiField>> {
    let packaged = fs::read_to_string(paths.packaged_yazi.join("yazi.toml"))?;
    let current = read_optional_text(&paths.yazi_config)?;
    let settings = build_toml_document_fields(document(
        SOURCE_YAZI_CONFIG,
        "Yazi settings",
        &current,
        &packaged,
    ))
    .map_err(|source| error(source.to_string()))?;
    let theme = read_optional_text(&paths.yazi_theme)?;
    let mut appearance = build_toml_document_fields(document(
        SOURCE_YAZI_THEME,
        "Appearance",
        &theme,
        YAZI_THEME_STARTER,
    ))
    .map_err(|source| error(source.to_string()))?;
    let flavors = discovered_flavors(paths)?;
    for field in &mut appearance.fields {
        let label = match field.path.as_str() {
            "flavor.dark" => "Dark flavor",
            "flavor.light" => "Light flavor",
            _ => continue,
        };
        field.display_label = label.to_string();
        field.type_label = Some("string".to_string());
        field.capability = if flavors.is_empty() {
            ConfigUiCapability::ReadOnly {
                reason: "No installed Yazi flavors were discovered.".to_string(),
                file_action_id: Some(ACTION_YAZI_THEME.to_string()),
            }
        } else {
            ConfigUiCapability::Choice {
                choices: flavors
                    .iter()
                    .cloned()
                    .map(|value| ConfigUiChoice::new(JsonValue::String(value)))
                    .collect(),
            }
        };
        field.can_unset = matches!(field.snapshot.intent, ConfigUiOverride::Explicit(_));
        if let ConfigUiOverride::Explicit(value) = &field.snapshot.intent {
            if value
                .as_str()
                .is_some_and(|value| flavors.iter().any(|flavor| flavor == value))
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
        if let Some(baseline) = &mut field.snapshot.baseline {
            baseline.origin = Some("Yazi default theme".to_string());
        }
        if matches!(field.snapshot.intent, ConfigUiOverride::Absent) {
            field
                .snapshot
                .effective
                .clone_from(&field.snapshot.baseline);
        }
        field.validation = "installed packaged or user flavor".to_string();
        field.description =
            format!("{label} from native yazi/theme.toml. Reset uses Yazi's default theme.");
    }
    appearance.fields.extend(settings.fields);
    retain_toml_leaf_fields(&mut appearance.fields);
    Ok(appearance.fields)
}

pub(crate) fn write_yazi_field(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let path = yazi_source_path(paths, source_id)?;
    paths.reject_mutation(path, source_id)?;
    if matches!(field_path, "flavor.dark" | "flavor.light") {
        let flavors = discovered_flavors(paths)?;
        if !value
            .as_str()
            .is_some_and(|value| flavors.iter().any(|flavor| flavor == value))
        {
            return Err(error(format!(
                "{field_path} must name an installed flavor: {}",
                flavors.join(", ")
            )));
        }
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

fn yazi_source_path<'a>(paths: &'a ConfigPaths, source_id: &str) -> Result<&'a Path> {
    match source_id {
        SOURCE_YAZI_CONFIG => Ok(&paths.yazi_config),
        SOURCE_YAZI_THEME => Ok(&paths.yazi_theme),
        _ => Err(error(format!("unknown Yazi config source: {source_id}"))),
    }
}

fn discovered_flavors(paths: &ConfigPaths) -> Result<Vec<String>> {
    let mut names = BTreeSet::new();
    for directory in [
        paths.packaged_yazi.join("flavors"),
        paths.yazi_config.with_file_name("flavors"),
    ] {
        if !directory.is_dir() {
            continue;
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
    }
    Ok(names.into_iter().collect())
}
