// Test lane: default
//! Canonical `config.toml` surface and one-time Classic JSONC migration.

use crate::atomic_fs::write_text_atomic;
use crate::backup_timestamp::compact_utc_backup_timestamp;
use crate::bridge::{CoreError, ErrorClass};
use crate::native_config_status::{path_owned_by_home_manager, path_present};
use crate::settings_contract::reconcile_settings_contract_text;
use crate::user_config_paths;
use ratconfig::jsonc::jsonc_parse_options;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;
use yazelix_cursors::{import_cursor_settings_jsonc, initialize_cursor_config, load_cursor_config};
use yazelix_zellij_config_pack::{
    DEFAULT_ZELLIJ_CONFIG_SIDECAR, DEFAULT_ZELLIJ_PLUGINS_SIDECAR, add_zellij_native_preferences,
    split_zellij_sidecars, validate_zellij_config_sidecar, validate_zellij_plugins_sidecar,
};

pub const SETTINGS_SCHEMA_FILENAME: &str = "yazelix_settings.schema.json";
pub const DEFAULT_MAIN_CONFIG_FILENAME: &str = "config_default.toml";
const LEGACY_ZELLIJ_CONFIG_SIDECAR: &str =
    "// Native Zellij preferences used by Yazelix\nscroll_buffer_size 5000\n";
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

    if path_present(&paths.settings_config) && path_present(&paths.legacy_settings_config) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "root_config_migration_conflict",
            "Both ~/.config/yazelix/config.toml and the retired settings.jsonc exist.",
            "Keep the intended values in config.toml, then move settings.jsonc aside and retry.",
            json!({
                "config": paths.settings_config.display().to_string(),
                "legacy": paths.legacy_settings_config.display().to_string(),
            }),
        ));
    }

    if path_present(&paths.settings_config) {
        read_config_table(&paths.settings_config, "invalid_main_config_toml")?;
        ensure_zellij_sidecars(config_dir, DEFAULT_ZELLIJ_CONFIG_SIDECAR)?;
        if cursor_component_enabled {
            ensure_no_embedded_cursor_settings(&paths)?;
            ensure_cursor_config(&paths, default_cursor_config)?;
        }
        return Ok(paths.settings_config);
    }

    if path_present(&paths.legacy_settings_config) {
        migrate_legacy_settings_config(config_dir, &paths, default_main_config)?;
        if cursor_component_enabled {
            ensure_cursor_config(&paths, default_cursor_config)?;
        }
        return Ok(paths.settings_config);
    }

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
    if cursor_component_enabled {
        ensure_default_cursor_config_exists(default_cursor_config)?;
    }

    render_default_config(default_main_config)?;
    ensure_zellij_sidecars(config_dir, DEFAULT_ZELLIJ_CONFIG_SIDECAR)?;

    if cursor_component_enabled {
        ensure_cursor_config(&paths, default_cursor_config)?;
    }

    Ok(paths.settings_config)
}

fn ensure_zellij_sidecars(config_dir: &Path, config_default: &str) -> Result<(), CoreError> {
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
        ensure_zellij_sidecar(&config, config_default, validate_zellij_config_sidecar)?;
        return ensure_zellij_sidecar(
            &plugins,
            DEFAULT_ZELLIJ_PLUGINS_SIDECAR,
            validate_zellij_plugins_sidecar,
        );
    }
    if flat_present && (path_owned_by_home_manager(&flat) || settings_path_is_read_only(&flat)) {
        return Err(zellij_migration_failure(
            "read_only_flat_zellij_migration",
            "The retired flat Zellij sidecar is read-only.",
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
    backup_zellij_migration_source(&flat, &timestamp)?;
    write_text_atomic(&config, &split.config)?;
    if let Err(error) = write_text_atomic(&plugins, &split.plugins) {
        let _ = fs::remove_file(&config);
        return Err(error);
    }
    if let Err(source) = fs::remove_file(&flat) {
        let _ = fs::remove_file(&config);
        let _ = fs::remove_file(&plugins);
        return Err(io_err(
            "retire_flat_zellij_config",
            &flat,
            "Could not retire the flat Zellij sidecar after migration",
            source,
        ));
    }
    Ok(())
}

fn ensure_zellij_sidecar(
    path: &Path,
    default: &str,
    validate: fn(&str) -> Result<(), yazelix_zellij_config_pack::ZellijSidecarError>,
) -> Result<(), CoreError> {
    if !path_present(path) {
        return write_text_atomic(path, default);
    }
    let raw = fs::read_to_string(path).map_err(|source| {
        io_err(
            "read_zellij_sidecar",
            path,
            "Could not read a managed Zellij sidecar",
            source,
        )
    })?;
    validate(&raw).map_err(|error| zellij_migration_error(path, error))
}

fn migration_backup_path(path: &Path, timestamp: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config");
    path.with_file_name(format!("{name}.backup-{timestamp}"))
}

fn backup_zellij_migration_source(path: &Path, timestamp: &str) -> Result<(), CoreError> {
    fs::copy(path, migration_backup_path(path, timestamp))
        .map(|_| ())
        .map_err(|error| {
            io_err(
                "backup_zellij_migration_source",
                path,
                "Could not back up a Zellij migration source",
                error,
            )
        })
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

fn migrate_legacy_settings_config(
    config_dir: &Path,
    paths: &SettingsSurfacePaths,
    default_main_config: &Path,
) -> Result<(), CoreError> {
    let legacy = &paths.legacy_settings_config;
    if path_owned_by_home_manager(legacy) || settings_path_is_read_only(legacy) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "read_only_root_config_migration",
            "The retired settings.jsonc is read-only and cannot be migrated safely.",
            "Update Home Manager to generate programs.yazelix config.toml, remove its settings.jsonc owner, then run home-manager switch.",
            json!({ "path": legacy.display().to_string() }),
        ));
    }
    let flat_zellij = user_config_paths::flat_zellij_config(config_dir);
    if path_present(&flat_zellij) {
        return Err(zellij_migration_failure(
            "root_config_requires_nested_zellij_sidecar",
            "The retired flat zellij.kdl sidecar still exists.",
            "Start the current Classic runtime once to migrate it to zellij/config.kdl and zellij/plugins.kdl, then retry the root config migration.",
            &flat_zellij,
        ));
    }

    let raw = fs::read_to_string(legacy).map_err(|source| {
        io_err(
            "read_legacy_settings_migration_source",
            legacy,
            "Could not read the retired settings.jsonc",
            source,
        )
    })?;
    let reconciled = reconcile_settings_contract_text(legacy, &raw, default_main_config)?;
    let mut value = parse_config_value(legacy, &reconciled.text)?;
    let root = value.as_object_mut().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Config,
            "legacy_settings_not_object",
            "The retired settings.jsonc does not contain a JSON object.",
            "Replace it with a valid Yazelix settings object or restore a working backup, then retry.",
            json!({ "path": legacy.display().to_string() }),
        )
    })?;
    if root.contains_key("cursors") {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "embedded_cursor_settings_unsupported",
            "Yazelix found cursor settings embedded in the retired settings.jsonc.",
            "Move cursor settings to ~/.config/yazelix/cursors.toml, then retry.",
            json!({ "path": legacy.display().to_string() }),
        ));
    }
    root.remove("ratconfig");

    let zellij = root
        .get_mut("zellij")
        .and_then(JsonValue::as_object_mut)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "missing_legacy_zellij_settings",
                "The retired settings.jsonc is missing its zellij object.",
                "Restore a valid Classic settings backup, then retry.",
                json!({ "path": legacy.display().to_string() }),
            )
        })?;
    let disable_tips = take_legacy_bool(zellij, "disable_tips", true, legacy)?;
    let pane_frames = take_legacy_bool(zellij, "pane_frames", true, legacy)?;
    let rounded_corners = take_legacy_bool(zellij, "rounded_corners", true, legacy)?;
    let default_mode = take_legacy_string(zellij, "default_mode", "normal", legacy)?;
    if !matches!(default_mode.as_str(), "normal" | "locked") {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_legacy_zellij_default_mode",
            format!("Cannot migrate zellij.default_mode value {default_mode:?}."),
            "Use normal or locked, then retry.",
            json!({ "path": legacy.display().to_string() }),
        ));
    }

    let config = json_value_to_toml_table(&value, legacy)?;
    let rendered = toml::to_string_pretty(&config).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "render_migrated_config_toml",
            format!("Could not render migrated config.toml: {source}"),
            "Report this as a Yazelix migration error.",
            json!({ "path": legacy.display().to_string() }),
        )
    })?;
    toml::from_str::<toml::Table>(&rendered).map_err(|source| {
        CoreError::toml(
            "verify_migrated_config_toml",
            "Could not verify the migrated Yazelix config",
            "Restore the settings.jsonc backup and report this migration error.",
            paths.settings_config.to_string_lossy(),
            source,
        )
    })?;

    let zellij_path = user_config_paths::zellij_config(config_dir);
    let zellij_existed = path_present(&zellij_path);
    if zellij_existed
        && (path_owned_by_home_manager(&zellij_path) || settings_path_is_read_only(&zellij_path))
    {
        return Err(zellij_migration_failure(
            "read_only_zellij_native_preference_migration",
            "The Zellij config sidecar is read-only and cannot receive migrated native preferences.",
            "Declare show_startup_tips, pane_frames, default_mode, and ui.pane_frames.rounded_corners in programs.yazelix.config.zellij, then run home-manager switch.",
            &zellij_path,
        ));
    }
    let original_zellij = if zellij_existed {
        fs::read_to_string(&zellij_path).map_err(|source| {
            io_err(
                "read_zellij_native_preference_migration_target",
                &zellij_path,
                "Could not read the Zellij config sidecar",
                source,
            )
        })?
    } else {
        LEGACY_ZELLIJ_CONFIG_SIDECAR.to_string()
    };
    let migrated_zellij = add_zellij_native_preferences(
        &original_zellij,
        disable_tips,
        pane_frames,
        rounded_corners,
        &default_mode,
    )
    .map_err(|error| zellij_migration_error(&zellij_path, error))?;

    let timestamp = compact_utc_backup_timestamp();
    fs::copy(legacy, migration_backup_path(legacy, &timestamp)).map_err(|source| {
        io_err(
            "backup_legacy_settings_config",
            legacy,
            "Could not back up settings.jsonc before migration",
            source,
        )
    })?;
    if zellij_existed {
        backup_zellij_migration_source(&zellij_path, &timestamp)?;
    }

    write_text_atomic(&zellij_path, &migrated_zellij)?;
    if let Err(error) = write_text_atomic(&paths.settings_config, &rendered) {
        restore_zellij_after_failed_root_migration(&zellij_path, zellij_existed, &original_zellij);
        return Err(error);
    }
    if let Err(source) = fs::remove_file(legacy) {
        let _ = fs::remove_file(&paths.settings_config);
        restore_zellij_after_failed_root_migration(&zellij_path, zellij_existed, &original_zellij);
        return Err(io_err(
            "retire_legacy_settings_config",
            legacy,
            "Could not retire settings.jsonc after writing config.toml",
            source,
        ));
    }
    ensure_zellij_sidecar(
        &user_config_paths::zellij_plugins(config_dir),
        DEFAULT_ZELLIJ_PLUGINS_SIDECAR,
        validate_zellij_plugins_sidecar,
    )
}

fn take_legacy_bool(
    zellij: &mut JsonMap<String, JsonValue>,
    key: &str,
    default: bool,
    path: &Path,
) -> Result<bool, CoreError> {
    match zellij.remove(key) {
        None => Ok(default),
        Some(JsonValue::Bool(value)) => Ok(value),
        Some(value) => Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_legacy_zellij_boolean",
            format!("Cannot migrate zellij.{key}; expected a boolean."),
            "Fix the value in settings.jsonc, then retry.",
            json!({ "path": path.display().to_string(), "value": value }),
        )),
    }
}

fn take_legacy_string(
    zellij: &mut JsonMap<String, JsonValue>,
    key: &str,
    default: &str,
    path: &Path,
) -> Result<String, CoreError> {
    match zellij.remove(key) {
        None => Ok(default.to_string()),
        Some(JsonValue::String(value)) => Ok(value),
        Some(value) => Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_legacy_zellij_string",
            format!("Cannot migrate zellij.{key}; expected a string."),
            "Fix the value in settings.jsonc, then retry.",
            json!({ "path": path.display().to_string(), "value": value }),
        )),
    }
}

fn restore_zellij_after_failed_root_migration(path: &Path, existed: bool, original: &str) {
    if existed {
        let _ = write_text_atomic(path, original);
    } else {
        let _ = fs::remove_file(path);
    }
}

fn settings_path_is_read_only(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().readonly())
        .unwrap_or(false)
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

fn json_value_to_toml_table(value: &JsonValue, path: &Path) -> Result<toml::Table, CoreError> {
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

    fn default_main_config() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config_default.toml")
    }

    fn write_cursor_default(root: &Path) -> PathBuf {
        let path = root.join("yazelix_cursors_default.toml");
        fs::write(
            &path,
            "schema_version = 1\nenabled_cursors = [\"snow\"]\n[settings]\ntrail = \"snow\"\ntrail_effect = \"tail\"\nmode_effect = \"ripple\"\nglow = \"medium\"\nduration = 1.0\nkitty_enable_cursor = true\n[[cursor]]\nname = \"snow\"\nfamily = \"mono\"\ncolor = \"#ffffff\"\n",
        )
        .unwrap();
        path
    }

    fn write_current_legacy_settings(path: &Path, zellij: &str) {
        fs::write(
            path,
            format!(
                "{{\n  \"editor\": {{ \"command\": \"nvim\" }},\n  \"zellij\": {{ {zellij} }},\n  \"ratconfig\": {{ \"contract\": {{ \"schema_version\": 1, \"contract_id\": \"yazelix.settings\", \"version\": 16, \"applied_change_ids\": {} }} }}\n}}\n",
                serde_json::to_string(&crate::settings_contract::SETTINGS_CONTRACT_APPLIED_CHANGE_IDS)
                    .unwrap()
            ),
        )
        .unwrap();
    }

    // Defends: fresh runtimes inherit semantic defaults without creating config.toml.
    #[test]
    fn leaves_fresh_config_toml_absent() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let cursor = write_cursor_default(runtime.path());

        let path = ensure_settings_config(config.path(), &default_main_config(), &cursor).unwrap();

        assert_eq!(path, config.path().join("config.toml"));
        assert!(!path.exists());
        let zellij = fs::read_to_string(config.path().join("zellij/config.kdl")).unwrap();
        assert!(zellij.contains("show_startup_tips false"));
        assert!(zellij.contains("rounded_corners true"));
        assert!(!config.path().join("settings.jsonc").exists());
    }

    // Regression: Classic imports the released JSONC registry once, preserves custom cursor semantics, and retires the old owner.
    #[test]
    fn migrates_legacy_cursor_jsonc_to_canonical_toml_once() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let default_cursor = write_cursor_default(runtime.path());
        let paths = settings_surface_paths(config.path());
        fs::create_dir_all(paths.legacy_shared_cursor_config.parent().unwrap()).unwrap();
        fs::write(
            &paths.legacy_shared_cursor_config,
            r##"{
  "schema_version": 1,
  "enabled_cursors": ["local_split", "blaze"],
  "settings": { "trail": "local_split", "trail_effect": "sweep", "mode_effect": "none", "glow": "high", "duration": 1.5, "kitty_enable_cursor": false },
  "cursor": [
    { "name": "local_split", "family": "split", "colors": ["#112233", "#aabbcc"], "divider": "horizontal", "transition": "hard", "cursor_color": "#aabbcc" },
    { "name": "blaze", "family": "mono", "color": "#ffb929" }
  ]
}
"##,
        )
        .unwrap();

        ensure_settings_config(config.path(), &default_main_config(), &default_cursor).unwrap();
        let registry = load_cursor_config(&paths.cursor_config).unwrap();
        assert_eq!(registry.enabled_cursors, ["local_split", "blaze"]);
        assert_eq!(registry.settings.trail, "local_split");
        assert_eq!(registry.settings.trail_effect, "sweep");
        assert!(!registry.settings.kitty_enable_cursor);
        assert_eq!(
            registry.definitions["local_split"].split_secondary_color_hex(),
            Some("#aabbcc")
        );
        assert!(!paths.legacy_shared_cursor_config.exists());
        assert!(
            fs::read_dir(paths.legacy_shared_cursor_config.parent().unwrap())
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry.file_name().to_string_lossy().contains("backup"))
        );

        let migrated = fs::read_to_string(&paths.cursor_config).unwrap();
        ensure_settings_config(config.path(), &default_main_config(), &default_cursor).unwrap();
        assert_eq!(fs::read_to_string(paths.cursor_config).unwrap(), migrated);
    }

    // Regression: the one-time transaction backs up JSONC, preserves Yazelix values, and moves native Zellij values exactly once.
    #[test]
    fn migrates_legacy_jsonc_to_toml_and_zellij_sidecar() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let cursor = write_cursor_default(runtime.path());
        let legacy = config.path().join("settings.jsonc");
        write_current_legacy_settings(
            &legacy,
            "\"disable_tips\": false, \"pane_frames\": false, \"rounded_corners\": false, \"default_mode\": \"locked\", \"support_kitty_keyboard_protocol\": true",
        );
        fs::create_dir_all(config.path().join("zellij")).unwrap();
        fs::write(
            config.path().join("zellij/config.kdl"),
            LEGACY_ZELLIJ_CONFIG_SIDECAR,
        )
        .unwrap();

        let path = ensure_settings_config(config.path(), &default_main_config(), &cursor).unwrap();
        let value = read_config_table(&path, "test").unwrap();
        assert_eq!(value["editor"]["command"].as_str(), Some("nvim"));
        let zellij = value["zellij"].as_table().unwrap();
        for removed in [
            "disable_tips",
            "pane_frames",
            "rounded_corners",
            "default_mode",
        ] {
            assert!(!zellij.contains_key(removed));
        }
        let sidecar = fs::read_to_string(config.path().join("zellij/config.kdl")).unwrap();
        assert!(sidecar.contains("show_startup_tips true"));
        assert!(sidecar.contains("pane_frames false"));
        assert!(sidecar.contains("default_mode \"locked\""));
        assert!(sidecar.contains("rounded_corners false"));
        assert!(!legacy.exists());
        assert!(
            fs::read_dir(config.path())
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("settings.jsonc.backup-")
                })
        );

        let before = fs::read_to_string(&path).unwrap();
        ensure_settings_config(config.path(), &default_main_config(), &cursor).unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), before);
    }

    // Defends: two root owners never trigger precedence selection or mutation.
    #[test]
    fn rejects_root_config_migration_conflict() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let cursor = write_cursor_default(runtime.path());
        fs::write(
            config.path().join("config.toml"),
            "[core]\ndebug_mode = false\n",
        )
        .unwrap();
        fs::write(config.path().join("settings.jsonc"), "{}\n").unwrap();

        let error =
            ensure_settings_config(config.path(), &default_main_config(), &cursor).unwrap_err();

        assert_eq!(error.code(), "root_config_migration_conflict");
        assert_eq!(
            fs::read_to_string(config.path().join("settings.jsonc")).unwrap(),
            "{}\n"
        );
    }

    // Defends: declarative/read-only ownership is reported instead of bypassed.
    #[cfg(unix)]
    #[test]
    fn rejects_read_only_legacy_settings() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let cursor = write_cursor_default(runtime.path());
        let legacy = config.path().join("settings.jsonc");
        fs::write(&legacy, "{}\n").unwrap();
        fs::set_permissions(&legacy, fs::Permissions::from_mode(0o444)).unwrap();

        let error =
            ensure_settings_config(config.path(), &default_main_config(), &cursor).unwrap_err();
        fs::set_permissions(&legacy, fs::Permissions::from_mode(0o644)).unwrap();

        assert_eq!(error.code(), "read_only_root_config_migration");
        assert!(!config.path().join("config.toml").exists());
    }

    // Regression: a dangling declarative owner must fail instead of being replaced by bootstrap.
    #[cfg(unix)]
    #[test]
    fn rejects_dangling_canonical_config_symlink() {
        use std::os::unix::fs::symlink;

        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let cursor = write_cursor_default(runtime.path());
        let path = config.path().join("config.toml");
        symlink(config.path().join("missing-home-manager-config"), &path).unwrap();

        let error = ensure_settings_config(config.path(), &default_main_config(), &cursor)
            .expect_err("dangling config owner");

        assert_eq!(error.code(), "invalid_main_config_toml");
        assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    }
}
