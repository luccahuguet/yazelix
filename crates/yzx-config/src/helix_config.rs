use std::{fs, path::Path};

use ratconfig::{
    ConfigUiApplyStatus, ConfigUiCapability, ConfigUiDiagnostic, ConfigUiResolvedValue,
    ConfigUiTomlDocumentError, ConfigUiTomlDocumentRows, ConfigUiTomlDocumentSpec,
    build_toml_document_fields,
};
use toml::{Value as TomlValue, map::Map as TomlMap};

use crate::{catalog::*, common::*, paths::ConfigPaths};

const REVEAL_KEY: &str = "A-r";
const REVEAL_COMMAND: &str = r#":sh yzx reveal "%{buffer_name}""#;
pub(crate) const HELIX_REVEAL_PATH: &str = "keys.normal.A-r";

pub(crate) const HELIX_RECOMMENDED_PATHS: &[&str] = &[
    "theme",
    "editor.auto-format",
    "editor.bufferline",
    "editor.cursorline",
    "editor.cursor-shape.insert",
    "editor.file-picker.hidden",
    "editor.soft-wrap.enable",
    HELIX_REVEAL_PATH,
];

pub(crate) fn build_helix_fields(
    paths: &ConfigPaths,
) -> Result<(ConfigUiTomlDocumentRows, Vec<ConfigUiDiagnostic>)> {
    let (mut config, config_diagnostic) = build_helix_document_fields(
        &read_optional_text(&paths.helix_config)?,
        Some(DEFAULT_HELIX_CONFIG_TOML),
        SOURCE_HELIX_CONFIG,
        "helix/config.toml",
        "Config",
        ACTION_HELIX_CONFIG,
    )?;
    let (languages, language_diagnostic) = build_helix_document_fields(
        &read_optional_text(&paths.helix_languages)?,
        None,
        SOURCE_HELIX_LANGUAGES,
        "helix/languages.toml",
        "Languages",
        ACTION_HELIX_LANGUAGES,
    )?;
    config.fields.extend(languages.fields);
    Ok((
        config,
        [config_diagnostic, language_diagnostic]
            .into_iter()
            .flatten()
            .collect(),
    ))
}

fn build_helix_document_fields(
    current: &str,
    baseline: Option<&str>,
    source_id: &str,
    display_path: &str,
    section_label: &str,
    file_action_id: &str,
) -> Result<(ConfigUiTomlDocumentRows, Option<ConfigUiDiagnostic>)> {
    let document = |current_toml| ConfigUiTomlDocumentSpec {
        source_id,
        tab: TAB_HELIX,
        section_label,
        current_toml,
        baseline_toml: baseline,
        validation: "TOML syntax here; Helix validates native settings at launch",
        rebuild_required: false,
        apply_status: ConfigUiApplyStatus {
            summary: "next Helix".to_string(),
            label: "helix".to_string(),
            detail: "Native values apply when the next managed Helix process starts.".to_string(),
            pending: false,
        },
    };
    let (rows, diagnostic) = match build_toml_document_fields(document(current)) {
        Ok(rows) => (rows, None),
        Err(ConfigUiTomlDocumentError::Current { message }) => (
            build_toml_document_fields(document(""))
                .map_err(|source| error(format!("invalid packaged Helix baseline: {source}")))?,
            Some(invalid_source_diagnostic(
                display_path,
                source_id,
                format!("invalid TOML: {message}"),
            )),
        ),
        Err(ConfigUiTomlDocumentError::Baseline { message }) => {
            return Err(error(format!("invalid packaged Helix baseline: {message}")));
        }
    };
    let mut rows = rows;
    remove_toml_parent_fields(
        &mut rows.fields,
        (source_id == SOURCE_HELIX_CONFIG).then_some(HELIX_REVEAL_PATH),
    );
    for field in &mut rows.fields {
        field.capability = ConfigUiCapability::ReadOnly {
            reason: format!(
                "Helix publishes no machine-readable validation contract; edit {display_path} directly."
            ),
            file_action_id: Some(file_action_id.to_string()),
        };
        field.can_unset = false;
        field.description = format!(
            "Native Helix value surfaced through {display_path}; Ratconfig does not infer edit authority from TOML shape."
        );
        if source_id == SOURCE_HELIX_CONFIG {
            for resolved in [&mut field.snapshot.baseline, &mut field.snapshot.effective]
                .into_iter()
                .flatten()
            {
                resolved.origin = Some("Yazelix packaged Helix config".to_string());
            }
            if field.path == HELIX_REVEAL_PATH {
                let reserved = ConfigUiResolvedValue {
                    value: serde_json::Value::String(REVEAL_COMMAND.to_string()),
                    origin: Some("Yazelix reserved Helix reveal binding".to_string()),
                };
                field.snapshot.baseline = Some(reserved.clone());
                field.snapshot.effective = Some(reserved);
                field.capability = ConfigUiCapability::ReadOnly {
                    reason: "Reserved by Yazelix for editor reveal; edit other Helix values in helix/config.toml."
                        .to_string(),
                    file_action_id: Some(ACTION_HELIX_CONFIG.to_string()),
                };
                field.description =
                    "Yazelix always restores Alt r to reveal the active editor file in the persistent Yazi popup."
                        .to_string();
            }
        }
    }
    Ok((rows, diagnostic))
}

pub(crate) fn write_effective_helix_config(
    packaged_path: &Path,
    user_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let mut config = read_toml_config(packaged_path, "packaged Helix config")?;
    if user_path.is_file() {
        let user_config = read_toml_config(user_path, "user Helix config")?;
        deep_merge_toml(&mut config, &user_config);
    }
    enforce_reveal_binding(&mut config)?;
    let output = toml::to_string_pretty(&config)
        .map_err(|err| error(format!("could not serialize effective Helix config: {err}")))?;
    atomic_write(output_path, &output)
}

fn read_toml_config(path: &Path, label: &str) -> Result<TomlValue> {
    let raw = fs::read_to_string(path)
        .map_err(|err| error(format!("could not read {label} {}: {err}", path.display())))?;
    toml::from_str(&raw)
        .map_err(|err| error(format!("could not parse {label} {}: {err}", path.display())))
}

fn enforce_reveal_binding(config: &mut TomlValue) -> Result<()> {
    let root = config
        .as_table_mut()
        .ok_or_else(|| error("effective Helix config root must be a TOML table"))?;
    let keys = table_entry(root, "keys", "[keys]")?;
    let normal = table_entry(keys, "normal", "[keys.normal]")?;
    normal.insert(
        REVEAL_KEY.to_string(),
        TomlValue::String(REVEAL_COMMAND.to_string()),
    );
    Ok(())
}

fn table_entry<'a>(
    table: &'a mut TomlMap<String, TomlValue>,
    key: &str,
    label: &str,
) -> Result<&'a mut TomlMap<String, TomlValue>> {
    table
        .entry(key.to_string())
        .or_insert_with(|| TomlValue::Table(TomlMap::new()))
        .as_table_mut()
        .ok_or_else(|| error(format!("{label} must be a TOML table")))
}
