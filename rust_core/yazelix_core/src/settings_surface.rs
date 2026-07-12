// Test lane: default
//! Canonical `config.toml` surface and one-time Classic JSONC migration.

use crate::atomic_fs::write_text_atomic_create_new;
use crate::backup_timestamp::compact_utc_backup_timestamp;
use crate::bridge::{CoreError, ErrorClass};
use crate::classic_nova_root_migration::{
    ClassicNovaMigrationRequest, migrate_classic_root_to_nova, remove_file_if_unchanged,
};
use crate::native_config_status::{path_owned_by_home_manager, path_present};
use crate::user_config_paths;
use ratconfig::jsonc::jsonc_parse_options;
use serde_json::{Value as JsonValue, json};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;
use yazelix_cursors::{import_cursor_settings_jsonc, initialize_cursor_config, load_cursor_config};
use yazelix_zellij_config_pack::{
    DEFAULT_ZELLIJ_CONFIG_SIDECAR, DEFAULT_ZELLIJ_PLUGINS_SIDECAR, split_zellij_sidecars,
    validate_zellij_config_sidecar, validate_zellij_plugins_sidecar,
};

const FIRST_NESTED_ZELLIJ_CONFIG_SIDECAR: &str = r#"// Native Zellij preferences used by Yazelix
scroll_buffer_size 5000
"#;
const FLAT_SPLIT_EMPTY_ZELLIJ_PLUGINS_SIDECAR: &str = "plugins {\n}\n\nload_plugins {\n}\n";

pub const SETTINGS_SCHEMA_FILENAME: &str = "yazelix_settings.schema.json";
pub const DEFAULT_MAIN_CONFIG_FILENAME: &str = "config_default.toml";
pub const CLASSIC_MAIN_CONFIG_FILENAME: &str = "classic_config_default.toml";
pub const CLASSIC_MAIN_CONTRACT_FILENAME: &str = "classic_main_config_contract.toml";
#[derive(Debug, Clone)]
pub struct SettingsSurfacePaths {
    pub settings_config: PathBuf,
    pub legacy_settings_config: PathBuf,
    pub cursor_config: PathBuf,
    pub legacy_shared_cursor_config: PathBuf,
    pub old_main_config: PathBuf,
    pub old_nested_main_config: PathBuf,
    pub old_nested_cursor_config: PathBuf,
}

pub fn settings_surface_paths(config_dir: &Path) -> SettingsSurfacePaths {
    SettingsSurfacePaths {
        settings_config: user_config_paths::main_config(config_dir),
        legacy_settings_config: user_config_paths::legacy_settings_config(config_dir),
        cursor_config: user_config_paths::cursor_config(config_dir),
        legacy_shared_cursor_config: user_config_paths::legacy_shared_cursor_config(config_dir),
        old_main_config: user_config_paths::old_main_config(config_dir),
        old_nested_main_config: user_config_paths::legacy_main_config(config_dir),
        old_nested_cursor_config: user_config_paths::legacy_cursor_config(config_dir),
    }
}

pub fn settings_schema_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir
        .join("config_metadata")
        .join(SETTINGS_SCHEMA_FILENAME)
}

pub fn is_settings_config_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == user_config_paths::SETTINGS_CONFIG)
        .unwrap_or(false)
}

pub fn is_jsonc_config_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "jsonc")
}

pub fn ensure_settings_config(
    config_dir: &Path,
    default_main_config: &Path,
    default_cursor_config: &Path,
) -> Result<PathBuf, CoreError> {
    ensure_settings_config_with_cursor_component(
        config_dir,
        default_main_config,
        default_cursor_config,
        true,
    )
}

pub fn ensure_settings_config_with_cursor_component(
    config_dir: &Path,
    default_main_config: &Path,
    default_cursor_config: &Path,
    cursor_component_enabled: bool,
) -> Result<PathBuf, CoreError> {
    let paths = settings_surface_paths(config_dir);
    ensure_no_old_main_inputs(&paths)?;

    let runtime_dir = default_main_config.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_default_config_path",
            "The packaged config_default.toml path has no runtime parent.",
            "Reinstall Yazelix so its packaged config paths are complete.",
            json!({ "path": default_main_config.display().to_string() }),
        )
    })?;
    migrate_classic_root_to_nova(&ClassicNovaMigrationRequest {
        config_dir: config_dir.to_path_buf(),
        classic_default_config: runtime_dir
            .join("config_metadata")
            .join(CLASSIC_MAIN_CONFIG_FILENAME),
        classic_contract: runtime_dir
            .join("config_metadata")
            .join(CLASSIC_MAIN_CONTRACT_FILENAME),
    })?;

    let settings_present = path_present(&paths.settings_config);
    if settings_present {
        read_config_table(&paths.settings_config, "invalid_main_config_toml")?;
    } else {
        if !default_main_config.exists() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "missing_default_config",
                format!(
                    "Yazelix runtime is missing the default main config at {}.",
                    default_main_config.display()
                ),
                "Reinstall Yazelix so the runtime includes config_default.toml.",
                json!({ "path": default_main_config.display().to_string() }),
            ));
        }
        render_default_config(default_main_config)?;
    }
    ensure_zellij_sidecars(config_dir)?;

    if cursor_component_enabled {
        if settings_present {
            ensure_no_embedded_cursor_settings(&paths)?;
        }
        ensure_cursor_config(&paths, default_cursor_config)?;
    }

    Ok(paths.settings_config)
}

fn ensure_zellij_sidecars(config_dir: &Path) -> Result<(), CoreError> {
    let flat = user_config_paths::flat_zellij_config(config_dir);
    let config = user_config_paths::zellij_config(config_dir);
    let plugins = user_config_paths::zellij_plugins(config_dir);
    let flat_present = path_present(&flat);
    if flat_present && (path_present(&config) || path_present(&plugins)) {
        return Err(zellij_migration_failure(
            "zellij_sidecar_migration_conflict",
            "Both the retired flat Zellij sidecar and a nested Zellij sidecar exist.",
            "Keep the intended content in zellij/config.kdl and zellij/plugins.kdl, then move zellij.kdl aside.",
            &flat,
        ));
    }
    if !flat_present {
        let timestamp = compact_utc_backup_timestamp();
        cleanup_or_validate_zellij_sidecar(&config, validate_zellij_config_sidecar, &timestamp)?;
        return cleanup_or_validate_zellij_sidecar(
            &plugins,
            validate_zellij_plugins_sidecar,
            &timestamp,
        );
    }
    let removable = fs::symlink_metadata(&flat)
        .is_ok_and(|metadata| metadata.file_type().is_file() && !metadata.permissions().readonly());
    if path_owned_by_home_manager(&flat) || !removable {
        return Err(zellij_migration_failure(
            "unsupported_flat_zellij_migration_source",
            "The retired flat Zellij sidecar is declarative, read-only, symlinked, or not a regular file.",
            "Split it declaratively into programs.yazelix.config.zellij and zellij/plugins.kdl, then move zellij.kdl aside.",
            &flat,
        ));
    }
    let flat_text = fs::read_to_string(&flat).map_err(|source| {
        io_err(
            "read_flat_zellij_migration_source",
            &flat,
            "Could not read the retired flat Zellij sidecar",
            source,
        )
    })?;
    let split =
        split_zellij_sidecars(&flat_text).map_err(|error| zellij_migration_error(&flat, error))?;
    let timestamp = compact_utc_backup_timestamp();
    backup_zellij_migration_source(&flat, &flat_text, &timestamp)?;
    let config_written = !split.config.is_empty() && !is_generated_zellij_sidecar(&split.config);
    if config_written {
        write_text_atomic_create_new(&config, &split.config)?;
    }
    let plugins_written = !split.plugins.is_empty() && !is_generated_zellij_sidecar(&split.plugins);
    if plugins_written && let Err(error) = write_text_atomic_create_new(&plugins, &split.plugins) {
        if config_written {
            let _ = remove_file_if_unchanged(&config, &split.config);
        }
        return Err(error);
    }
    if let Err(source) = remove_file_if_unchanged(&flat, &flat_text) {
        if config_written {
            let _ = remove_file_if_unchanged(&config, &split.config);
        }
        if plugins_written {
            let _ = remove_file_if_unchanged(&plugins, &split.plugins);
        }
        return Err(io_err(
            "retire_flat_zellij_config",
            &flat,
            "Could not retire the flat Zellij sidecar after migration",
            source,
        ));
    }
    Ok(())
}

fn cleanup_or_validate_zellij_sidecar(
    path: &Path,
    validate: fn(&str) -> Result<(), yazelix_zellij_config_pack::ZellijSidecarError>,
    timestamp: &str,
) -> Result<(), CoreError> {
    if !path_present(path) {
        return Ok(());
    }
    let raw = fs::read_to_string(path).map_err(|source| {
        io_err(
            "read_zellij_sidecar",
            path,
            "Could not read a managed Zellij sidecar",
            source,
        )
    })?;
    validate(&raw).map_err(|error| zellij_migration_error(path, error))?;
    let removable = fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.file_type().is_file() && !metadata.permissions().readonly());
    if !is_generated_zellij_sidecar(&raw) || !removable || path_owned_by_home_manager(path) {
        return Ok(());
    }
    backup_zellij_migration_source(path, &raw, timestamp)?;
    remove_file_if_unchanged(path, &raw).map_err(|source| {
        io_err(
            "retire_generated_zellij_sidecar",
            path,
            "Could not retire the unchanged generated Zellij sidecar after backing it up",
            source,
        )
    })
}

fn is_generated_zellij_sidecar(raw: &str) -> bool {
    [
        FIRST_NESTED_ZELLIJ_CONFIG_SIDECAR,
        DEFAULT_ZELLIJ_CONFIG_SIDECAR,
        DEFAULT_ZELLIJ_PLUGINS_SIDECAR,
        FLAT_SPLIT_EMPTY_ZELLIJ_PLUGINS_SIDECAR,
    ]
    .contains(&raw)
}

fn migration_backup_path(path: &Path, timestamp: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config");
    path.with_file_name(format!("{name}.backup-{timestamp}"))
}

fn backup_zellij_migration_source(
    path: &Path,
    raw: &str,
    timestamp: &str,
) -> Result<(), CoreError> {
    write_text_atomic_create_new(&migration_backup_path(path, timestamp), raw)
}

fn zellij_migration_error(
    path: &Path,
    error: yazelix_zellij_config_pack::ZellijSidecarError,
) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        error.code,
        format!("{}: {}:{}", error.message(), path.display(), error.line),
        error.remediation(),
        json!({
            "path": path.display().to_string(),
            "line": error.line,
            "node": error.node,
        }),
    )
}

fn zellij_migration_failure(
    code: &'static str,
    message: impl Into<String>,
    remediation: &'static str,
    path: &Path,
) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        code,
        message,
        remediation,
        json!({ "path": path.display().to_string() }),
    )
}

pub fn render_default_config(default_main_config: &Path) -> Result<String, CoreError> {
    let raw = fs::read_to_string(default_main_config).map_err(|source| {
        io_err(
            "read_default_main_config",
            default_main_config,
            "Could not read the default Yazelix config",
            source,
        )
    })?;
    let value = read_toml_table(
        default_main_config,
        "invalid_default_main_config",
        "Could not parse the default Yazelix config",
    )?;
    if value.contains_key("cursors") {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "embedded_default_cursor_settings_unsupported",
            "Yazelix default main config must not contain cursor settings.",
            "Keep cursor defaults in the child-owned cursor registry instead.",
            json!({ "path": default_main_config.display().to_string() }),
        ));
    }
    Ok(ensure_trailing_newline(raw))
}

pub fn read_config_table(path: &Path, code: &'static str) -> Result<toml::Table, CoreError> {
    if is_jsonc_config_path(path) {
        let value = read_config_value(path)?;
        json_value_to_toml_table(&value, path)
    } else {
        read_toml_table(path, code, "Could not parse Yazelix TOML input")
    }
}

pub fn read_sparse_config_table(path: &Path, code: &'static str) -> Result<toml::Table, CoreError> {
    match fs::read_to_string(path) {
        Ok(raw) => toml::from_str::<toml::Table>(&raw).map_err(|source| {
            CoreError::toml(
                "invalid_toml",
                "Could not parse Yazelix TOML input",
                "Fix the TOML syntax in the reported file and retry.",
                path.to_string_lossy(),
                source,
            )
        }),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(toml::Table::new()),
        Err(source) => Err(io_err(
            code,
            path,
            "Could not read Yazelix config input",
            source,
        )),
    }
}

pub fn sparse_config_is_semantically_empty(path: &Path, raw: &str) -> Result<bool, CoreError> {
    let table = toml::from_str::<toml::Table>(raw).map_err(|source| {
        CoreError::toml(
            "invalid_toml",
            "Could not parse Yazelix TOML input",
            "Fix the TOML syntax in the reported file and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    Ok(!table.values().any(toml_value_has_semantic_value))
}

fn toml_value_has_semantic_value(value: &TomlValue) -> bool {
    match value {
        TomlValue::Table(table) => table.values().any(toml_value_has_semantic_value),
        _ => true,
    }
}

pub fn read_config_value(path: &Path) -> Result<JsonValue, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        io_err(
            "read_settings_jsonc",
            path,
            "Could not read Yazelix config",
            source,
        )
    })?;
    parse_config_value(path, &raw)
}

pub fn parse_config_value(path: &Path, raw: &str) -> Result<JsonValue, CoreError> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "toml")
    {
        let value = toml::from_str::<toml::Table>(raw).map_err(|source| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_main_config_toml",
                format!(
                    "Could not parse Yazelix TOML at {}: {source}.",
                    path.display()
                ),
                "Fix the TOML syntax in the reported config file and retry.",
                json!({ "path": path.display().to_string(), "error": source.to_string() }),
            )
        })?;
        return serde_json::to_value(TomlValue::Table(value)).map_err(|source| {
            CoreError::classified(
                ErrorClass::Internal,
                "convert_main_config_toml",
                format!(
                    "Could not convert Yazelix TOML at {}: {source}.",
                    path.display()
                ),
                "Report this as a Yazelix internal error.",
                json!({ "path": path.display().to_string(), "error": source.to_string() }),
            )
        });
    }
    jsonc_parser::parse_to_serde_value::<JsonValue>(raw, &jsonc_parse_options()).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_settings_jsonc",
            format!(
                "Could not parse Yazelix settings JSONC at {}: {source}.",
                path.display(),
            ),
            "Fix the JSONC syntax in the reported settings file and retry. Comments must use `//` or `/* ... */`, not `#`.",
            json!({
                "path": path.display().to_string(),
                "error": source.to_string(),
            }),
        )
    })
}

fn ensure_default_cursor_config_exists(default_cursor_config: &Path) -> Result<(), CoreError> {
    if default_cursor_config.exists() {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Config,
        "missing_default_cursor_config",
        format!(
            "Yazelix runtime is missing the default cursor registry at {}.",
            default_cursor_config.display()
        ),
        "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml.",
        json!({ "path": default_cursor_config.display().to_string() }),
    ))
}

fn ensure_no_old_main_inputs(paths: &SettingsSurfacePaths) -> Result<(), CoreError> {
    for path in [&paths.old_main_config, &paths.old_nested_main_config] {
        ensure_old_input_absent(
            path,
            &paths.settings_config,
            "stale_old_settings_input",
            "old settings input",
            "Move the old yazelix.toml file aside and keep config.toml as the only Yazelix settings source.",
        )?;
    }
    Ok(())
}

fn ensure_cursor_config(
    paths: &SettingsSurfacePaths,
    default_cursor_config: &Path,
) -> Result<(), CoreError> {
    ensure_default_cursor_config_exists(default_cursor_config)?;
    ensure_old_input_absent(
        &paths.old_nested_cursor_config,
        &paths.cursor_config,
        "stale_old_cursor_settings_input",
        "old cursor settings input",
        "Move the old nested cursor TOML file aside; ~/.config/yazelix/cursors.toml is the only cursor settings source.",
    )?;

    if path_present(&paths.cursor_config) && path_present(&paths.legacy_shared_cursor_config) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "cursor_config_migration_conflict",
            "Both ~/.config/yazelix/cursors.toml and the retired cursor settings.jsonc exist.",
            "Keep the intended values in cursors.toml, then move ~/.config/yazelix_cursors/settings.jsonc aside and retry.",
            json!({
                "config": paths.cursor_config.display().to_string(),
                "legacy": paths.legacy_shared_cursor_config.display().to_string(),
            }),
        ));
    }
    if path_present(&paths.cursor_config) {
        load_cursor_config(&paths.cursor_config)?;
        return Ok(());
    }
    if path_present(&paths.legacy_shared_cursor_config) {
        let backup =
            import_cursor_settings_jsonc(&paths.legacy_shared_cursor_config, &paths.cursor_config)?;
        fs::remove_file(&paths.legacy_shared_cursor_config).map_err(|source| {
            io_err(
                "retire_legacy_cursor_settings_jsonc",
                &paths.legacy_shared_cursor_config,
                "Could not retire the imported cursor settings.jsonc",
                source,
            )
        })?;
        eprintln!(
            "Migrated cursor settings to {} and backed up the legacy file at {}.",
            paths.cursor_config.display(),
            backup.display()
        );
        return Ok(());
    }
    initialize_cursor_config(&paths.cursor_config)?;
    Ok(())
}

fn ensure_no_embedded_cursor_settings(paths: &SettingsSurfacePaths) -> Result<(), CoreError> {
    let value = read_config_value(&paths.settings_config)?;
    if value.get("cursors").is_none() {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Config,
        "embedded_cursor_settings_unsupported",
        "Yazelix found cursor settings embedded in config.toml.",
        "Move cursor settings to ~/.config/yazelix/cursors.toml; config.toml does not own cursor settings.",
        json!({
            "settings_config": paths.settings_config.display().to_string(),
            "cursor_config": paths.cursor_config.display().to_string(),
        }),
    ))
}

fn ensure_old_input_absent(
    path: &Path,
    current_path: &Path,
    code: &'static str,
    label: &str,
    remediation: &'static str,
) -> Result<(), CoreError> {
    if optional_symlink_metadata(path)?.is_none() {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Config,
        code,
        format!("Yazelix found {label} at {}.", path.display()),
        remediation,
        json!({
            "current": current_path.display().to_string(),
            "old_input": path.display().to_string(),
        }),
    ))
}

fn optional_symlink_metadata(path: &Path) -> Result<Option<fs::Metadata>, CoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io_err(
            "stat_settings_input",
            path,
            "Could not inspect a Yazelix settings path",
            source,
        )),
    }
}

fn read_toml_table(
    path: &Path,
    code: &'static str,
    message: &str,
) -> Result<toml::Table, CoreError> {
    let raw = fs::read_to_string(path)
        .map_err(|source| io_err(code, path, "Could not read Yazelix config input", source))?;
    toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_toml",
            message,
            "Fix the TOML syntax in the reported file and retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

pub fn render_config_value(value: &JsonValue) -> Result<String, CoreError> {
    let table = json_value_to_toml_table(value, Path::new("config.toml"))?;
    toml::to_string_pretty(&table).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_main_config_toml",
            format!("Could not serialize config.toml: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })
}

fn ensure_trailing_newline(mut raw: String) -> String {
    if !raw.ends_with('\n') {
        raw.push('\n');
    }
    raw
}

pub(crate) fn json_value_to_toml_table(
    value: &JsonValue,
    path: &Path,
) -> Result<toml::Table, CoreError> {
    let JsonValue::Object(object) = value else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "config_not_object",
            "Yazelix config must contain an object/table.",
            "Replace the config with a valid root object/table, then retry.",
            json!({ "path": path.display().to_string() }),
        ));
    };
    let mut table = toml::Table::new();
    for (key, value) in object {
        if value.is_null() {
            continue;
        }
        table.insert(key.clone(), json_value_to_toml(value, path)?);
    }
    Ok(table)
}

fn json_value_to_toml(value: &JsonValue, path: &Path) -> Result<TomlValue, CoreError> {
    match value {
        JsonValue::Null => Err(CoreError::classified(
            ErrorClass::Config,
            "unsupported_nested_settings_null",
            "The retired settings.jsonc contains null where TOML requires a concrete value.",
            "Remove the field to use the default, or replace null with a supported value.",
            json!({ "path": path.display().to_string() }),
        )),
        JsonValue::Bool(value) => Ok(TomlValue::Boolean(*value)),
        JsonValue::String(value) => Ok(TomlValue::String(value.clone())),
        JsonValue::Number(value) => {
            if let Some(integer) = value.as_i64() {
                Ok(TomlValue::Integer(integer))
            } else if let Some(float) = value.as_f64() {
                Ok(TomlValue::Float(float))
            } else {
                Err(CoreError::classified(
                    ErrorClass::Config,
                    "unsupported_settings_number",
                    "The retired settings.jsonc contains a number that cannot be represented in TOML.",
                    "Use an integer or finite float value.",
                    json!({ "path": path.display().to_string(), "value": value.to_string() }),
                ))
            }
        }
        JsonValue::Array(values) => values
            .iter()
            .map(|value| json_value_to_toml(value, path))
            .collect::<Result<Vec<_>, _>>()
            .map(TomlValue::Array),
        JsonValue::Object(object) => {
            let mut table = toml::Table::new();
            for (key, value) in object {
                if value.is_null() {
                    continue;
                }
                table.insert(key.clone(), json_value_to_toml(value, path)?);
            }
            Ok(TomlValue::Table(table))
        }
    }
}

fn io_err(code: &'static str, path: &Path, message: &str, source: io::Error) -> CoreError {
    CoreError::io(
        code,
        message,
        "Fix permissions or move the reported file manually, then retry.",
        path.display().to_string(),
        source,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    type Validator = fn(&str) -> Result<(), yazelix_zellij_config_pack::ZellijSidecarError>;

    fn reconcile_test_sidecar(path: &Path, text: &str, validate: Validator) {
        fs::write(path, text).unwrap();
        cleanup_or_validate_zellij_sidecar(path, validate, "test").unwrap();
    }

    // Defends: missing sidecars stay absent, while every released generated artifact retires backup-first and idempotently.
    #[test]
    fn inherits_missing_and_retires_generated_sidecars() {
        let absent = tempdir().unwrap();
        ensure_zellij_sidecars(absent.path()).unwrap();
        assert_eq!(fs::read_dir(absent.path()).unwrap().count(), 0);

        for text in [
            FIRST_NESTED_ZELLIJ_CONFIG_SIDECAR,
            DEFAULT_ZELLIJ_CONFIG_SIDECAR,
        ] {
            let dir = tempdir().unwrap();
            let path = dir.path().join("config.kdl");
            reconcile_test_sidecar(&path, text, validate_zellij_config_sidecar);
            assert!(!path.exists());
            assert_eq!(
                fs::read_to_string(migration_backup_path(&path, "test")).unwrap(),
                text
            );
            cleanup_or_validate_zellij_sidecar(&path, validate_zellij_config_sidecar, "test")
                .unwrap();
        }
        let dir = tempdir().unwrap();
        let path = dir.path().join("plugins.kdl");
        reconcile_test_sidecar(
            &path,
            DEFAULT_ZELLIJ_PLUGINS_SIDECAR,
            validate_zellij_plugins_sidecar,
        );
        assert!(!path.exists());
    }

    // Defends: edits, plugins, symlinks, and read-only files remain whole instead of being inferred or reduced.
    #[test]
    fn preserves_customized_and_declarative_sidecars() {
        let dir = tempdir().unwrap();
        let edited = format!("{DEFAULT_ZELLIJ_CONFIG_SIDECAR}// keep this comment\n");
        let plugin =
            "plugins {\n    compact-bar location=\"https://example.invalid/compact.wasm\"\n}\n";
        let config = dir.path().join("config.kdl");
        reconcile_test_sidecar(&config, &edited, validate_zellij_config_sidecar);
        assert_eq!(fs::read_to_string(config).unwrap(), edited);
        let plugins = dir.path().join("plugins.kdl");
        reconcile_test_sidecar(&plugins, plugin, validate_zellij_plugins_sidecar);
        assert_eq!(fs::read_to_string(plugins).unwrap(), plugin);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let target = dir.path().join("declarative.kdl");
            let path = dir.path().join("linked.kdl");
            fs::write(&target, DEFAULT_ZELLIJ_CONFIG_SIDECAR).unwrap();
            symlink(&target, &path).unwrap();
            cleanup_or_validate_zellij_sidecar(&path, validate_zellij_config_sidecar, "linked")
                .unwrap();
            let kind = fs::symlink_metadata(&path).unwrap().file_type();
            assert!(kind.is_symlink());

            let path = dir.path().join("read_only.kdl");
            fs::write(&path, DEFAULT_ZELLIJ_CONFIG_SIDECAR).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
            cleanup_or_validate_zellij_sidecar(&path, validate_zellij_config_sidecar, "read_only")
                .unwrap();
            assert!(path.exists());
        }
    }

    // Regression: the flat migration backs up its source, omits generated-empty outputs, and writes only real sparse content.
    #[test]
    fn flat_zellij_migration_stays_sparse_after_backup() {
        for flat in ["", "scroll_buffer_size 9000\n"] {
            let expected_config = (!flat.is_empty()).then_some(flat);
            let config = tempdir().unwrap();
            let flat_path = config.path().join("zellij.kdl");
            fs::write(&flat_path, flat).unwrap();

            ensure_zellij_sidecars(config.path()).unwrap();

            assert!(!flat_path.exists());
            let backup = fs::read_dir(config.path())
                .unwrap()
                .filter_map(Result::ok)
                .find(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("zellij.kdl.backup-")
                })
                .unwrap()
                .path();
            assert_eq!(fs::read_to_string(backup).unwrap(), flat);
            let config_path = config.path().join("zellij/config.kdl");
            assert_eq!(config_path.exists(), expected_config.is_some());
            if let Some(expected) = expected_config {
                assert_eq!(fs::read_to_string(config_path).unwrap(), expected);
            }
            assert!(!config.path().join("zellij/plugins.kdl").exists());
        }
    }
}
