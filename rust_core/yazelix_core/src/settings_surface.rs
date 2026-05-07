// Test lane: default
//! Canonical `settings.jsonc` surface and one-time old-format migration.

use crate::bridge::{CoreError, ErrorClass};
use crate::user_config_paths;
use jsonc_parser::ParseOptions;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;
use yazelix_cursors::{CursorRegistry, render_cursor_settings_jsonc};

pub const SETTINGS_SCHEMA_FILENAME: &str = "yazelix_settings.schema.json";
const SETTINGS_TOP_LEVEL_ORDER: &[&str] = &[
    "core", "helix", "editor", "shell", "terminal", "zellij", "yazi",
];

#[derive(Debug, Clone)]
pub struct SettingsSurfacePaths {
    pub settings_config: PathBuf,
    pub shared_cursor_config: PathBuf,
    pub old_main_config: PathBuf,
    pub old_nested_main_config: PathBuf,
    pub old_cursor_config: PathBuf,
    pub old_nested_cursor_config: PathBuf,
}

pub fn settings_surface_paths(config_dir: &Path) -> SettingsSurfacePaths {
    SettingsSurfacePaths {
        settings_config: user_config_paths::main_config(config_dir),
        shared_cursor_config: user_config_paths::shared_cursor_config(config_dir),
        old_main_config: user_config_paths::old_main_config(config_dir),
        old_nested_main_config: user_config_paths::legacy_main_config(config_dir),
        old_cursor_config: user_config_paths::cursor_config(config_dir),
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

    if paths.settings_config.exists() {
        ensure_no_old_main_inputs_next_to_settings(&paths)?;
        if cursor_component_enabled {
            migrate_embedded_cursor_settings(&paths)?;
            ensure_shared_cursor_settings_config(&paths, default_cursor_config)?;
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
            "Reinstall Yazelix so the runtime includes yazelix_default.toml.",
            json!({ "path": default_main_config.display().to_string() }),
        ));
    }
    if cursor_component_enabled {
        ensure_default_cursor_config_exists(default_cursor_config)?;
    }

    let main_source = migration_source(
        &paths.old_main_config,
        &paths.old_nested_main_config,
        "main Yazelix settings",
    )?;

    let main_table = read_toml_source_or_default(
        main_source.as_ref().map(|source| source.path.as_path()),
        default_main_config,
        "main Yazelix settings",
    )?;
    let rendered = render_settings_jsonc(&main_table)?;

    if let Some(parent) = paths.settings_config.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            io_err(
                "create_settings_config_parent",
                parent,
                "Could not create the Yazelix settings directory",
                source,
            )
        })?;
    }
    fs::write(&paths.settings_config, rendered).map_err(|source| {
        io_err(
            "write_settings_config",
            &paths.settings_config,
            "Could not write ~/.config/yazelix/settings.jsonc",
            source,
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::Permissions::from_mode(0o644);
        let _ = fs::set_permissions(&paths.settings_config, mode);
    }

    if let Some(source) = main_source {
        move_migrated_input(&source.path)?;
    }
    if cursor_component_enabled {
        ensure_shared_cursor_settings_config(&paths, default_cursor_config)?;
    }

    Ok(paths.settings_config)
}

pub fn render_default_settings_jsonc(
    default_main_config: &Path,
    _default_cursor_config: &Path,
) -> Result<String, CoreError> {
    let main_table = read_toml_table(
        default_main_config,
        "read_default_main_config",
        "Could not parse the default Yazelix main config",
    )?;
    render_settings_jsonc(&main_table)
}

pub fn read_config_table(path: &Path, code: &'static str) -> Result<toml::Table, CoreError> {
    if is_settings_config_path(path) {
        let value = read_settings_jsonc_value(path)?;
        json_value_to_toml_table(&value, path)
    } else {
        read_toml_table(path, code, "Could not parse Yazelix TOML input")
    }
}

pub fn read_settings_jsonc_value(path: &Path) -> Result<JsonValue, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        io_err(
            "read_settings_jsonc",
            path,
            "Could not read Yazelix settings.jsonc",
            source,
        )
    })?;
    parse_jsonc_value(path, &raw)
}

pub fn parse_jsonc_value(path: &Path, raw: &str) -> Result<JsonValue, CoreError> {
    jsonc_parser::parse_to_serde_value::<JsonValue>(raw, &jsonc_parse_options()).map_err(|source| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_settings_jsonc",
            format!(
                "Could not parse Yazelix settings JSONC at {}: {source}.",
                path.display(),
            ),
            "Fix the JSONC syntax in settings.jsonc and retry. Comments must use `//` or `/* ... */`, not `#`.",
            json!({
                "path": path.display().to_string(),
                "error": source.to_string(),
            }),
        )
    })
}

pub(crate) fn jsonc_parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

#[derive(Debug, Clone)]
struct MigrationSource {
    path: PathBuf,
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

fn ensure_no_old_main_inputs_next_to_settings(
    paths: &SettingsSurfacePaths,
) -> Result<(), CoreError> {
    if !paths.settings_config.exists() {
        return Ok(());
    }

    for path in [&paths.old_main_config, &paths.old_nested_main_config] {
        if optional_symlink_metadata(path)?.is_some() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "stale_old_settings_input",
                format!(
                    "Yazelix found old settings input {} next to canonical settings.jsonc.",
                    path.display()
                ),
                "Move the old TOML file aside after confirming settings.jsonc contains the migrated values, then retry.",
                json!({
                    "settings": paths.settings_config.display().to_string(),
                    "old_input": path.display().to_string(),
                }),
            ));
        }
    }

    Ok(())
}

fn ensure_shared_cursor_settings_config(
    paths: &SettingsSurfacePaths,
    default_cursor_config: &Path,
) -> Result<(), CoreError> {
    ensure_default_cursor_config_exists(default_cursor_config)?;
    if paths.shared_cursor_config.exists() {
        ensure_no_old_cursor_inputs_next_to_shared(paths)?;
        return Ok(());
    }

    let cursor_source = migration_source(
        &paths.old_cursor_config,
        &paths.old_nested_cursor_config,
        "Yazelix cursor settings",
    )?;
    let source_path = cursor_source
        .as_ref()
        .map(|source| source.path.as_path())
        .unwrap_or(default_cursor_config);
    let raw = fs::read_to_string(source_path).map_err(|source| {
        io_err(
            "read_cursor_settings_source",
            source_path,
            "Could not read Yazelix cursor settings input",
            source,
        )
    })?;
    let registry = CursorRegistry::parse_str(source_path, &raw)?;
    write_shared_cursor_settings(paths, &registry)?;

    if let Some(source) = cursor_source {
        move_migrated_input(&source.path)?;
    }

    Ok(())
}

fn ensure_no_old_cursor_inputs_next_to_shared(
    paths: &SettingsSurfacePaths,
) -> Result<(), CoreError> {
    for path in [&paths.old_cursor_config, &paths.old_nested_cursor_config] {
        if optional_symlink_metadata(path)?.is_some() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "stale_old_cursor_settings_input",
                format!(
                    "Yazelix found old cursor settings input {} next to canonical cursor settings.",
                    path.display()
                ),
                "Move the old cursor TOML file aside after confirming ~/.config/yazelix_cursors/settings.jsonc contains the migrated values, then retry.",
                json!({
                    "shared_cursor_config": paths.shared_cursor_config.display().to_string(),
                    "old_input": path.display().to_string(),
                }),
            ));
        }
    }
    Ok(())
}

fn migrate_embedded_cursor_settings(paths: &SettingsSurfacePaths) -> Result<(), CoreError> {
    let value = read_settings_jsonc_value(&paths.settings_config)?;
    let Some(cursors) = value.get("cursors").cloned() else {
        return Ok(());
    };
    if paths.shared_cursor_config.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "embedded_cursor_settings_after_sidecar",
            "Yazelix found cursor settings embedded in settings.jsonc after the shared cursor config already exists.",
            "Move or delete the stale cursors object from ~/.config/yazelix/settings.jsonc; cursor settings now live in ~/.config/yazelix_cursors/settings.jsonc.",
            json!({
                "settings_config": paths.settings_config.display().to_string(),
                "shared_cursor_config": paths.shared_cursor_config.display().to_string(),
            }),
        ));
    }
    if optional_symlink_metadata(&paths.old_cursor_config)?.is_some()
        || optional_symlink_metadata(&paths.old_nested_cursor_config)?.is_some()
    {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "duplicate_cursor_settings_sources",
            "Yazelix found embedded cursor settings and old cursor TOML inputs at the same time.",
            "Keep one cursor settings source, move the others aside, then retry.",
            json!({
                "settings_config": paths.settings_config.display().to_string(),
                "old_cursor_config": paths.old_cursor_config.display().to_string(),
                "old_nested_cursor_config": paths.old_nested_cursor_config.display().to_string(),
            }),
        ));
    }

    let registry = CursorRegistry::parse_json_value(&paths.settings_config, cursors)?;
    write_shared_cursor_settings(paths, &registry)?;

    let mut updated = value;
    if let Some(object) = updated.as_object_mut() {
        object.remove("cursors");
    }
    let rendered = render_settings_jsonc_value(&updated)?;
    fs::write(&paths.settings_config, rendered).map_err(|source| {
        io_err(
            "remove_embedded_cursor_settings",
            &paths.settings_config,
            "Could not remove migrated cursor settings from settings.jsonc",
            source,
        )
    })
}

fn write_shared_cursor_settings(
    paths: &SettingsSurfacePaths,
    registry: &CursorRegistry,
) -> Result<(), CoreError> {
    if let Some(parent) = paths.shared_cursor_config.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            io_err(
                "create_shared_cursor_config_parent",
                parent,
                "Could not create the Yazelix cursor settings directory",
                source,
            )
        })?;
    }
    fs::write(
        &paths.shared_cursor_config,
        render_cursor_settings_jsonc(registry),
    )
    .map_err(|source| {
        io_err(
            "write_shared_cursor_settings",
            &paths.shared_cursor_config,
            "Could not write ~/.config/yazelix_cursors/settings.jsonc",
            source,
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::Permissions::from_mode(0o644);
        let _ = fs::set_permissions(&paths.shared_cursor_config, mode);
    }
    Ok(())
}

fn migration_source(
    flat: &Path,
    nested: &Path,
    label: &str,
) -> Result<Option<MigrationSource>, CoreError> {
    let flat_meta = optional_symlink_metadata(flat)?;
    let nested_meta = optional_symlink_metadata(nested)?;

    let flat_regular = validate_old_input_metadata(flat, flat_meta.as_ref(), label)?;
    let nested_regular = validate_old_input_metadata(nested, nested_meta.as_ref(), label)?;

    match (flat_regular, nested_regular) {
        (false, false) => Ok(None),
        (true, false) => Ok(Some(MigrationSource {
            path: flat.to_path_buf(),
        })),
        (false, true) => Ok(Some(MigrationSource {
            path: nested.to_path_buf(),
        })),
        (true, true) => {
            let flat_raw = fs::read(flat).map_err(|source| {
                io_err(
                    "read_duplicate_old_settings_input",
                    flat,
                    "Could not read old Yazelix settings input",
                    source,
                )
            })?;
            let nested_raw = fs::read(nested).map_err(|source| {
                io_err(
                    "read_duplicate_old_settings_input",
                    nested,
                    "Could not read old Yazelix settings input",
                    source,
                )
            })?;
            if flat_raw == nested_raw {
                Ok(Some(MigrationSource {
                    path: flat.to_path_buf(),
                }))
            } else {
                Err(CoreError::classified(
                    ErrorClass::Config,
                    "conflicting_old_settings_inputs",
                    format!("Yazelix found conflicting old {label} inputs."),
                    "Keep one old input file or migrate the values into settings.jsonc manually, then retry.",
                    json!({
                        "flat": flat.display().to_string(),
                        "nested": nested.display().to_string(),
                    }),
                ))
            }
        }
    }
}

fn validate_old_input_metadata(
    path: &Path,
    metadata: Option<&fs::Metadata>,
    label: &str,
) -> Result<bool, CoreError> {
    let Some(metadata) = metadata else {
        return Ok(false);
    };
    if metadata.file_type().is_symlink() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "old_settings_symlink_requires_manual_migration",
            format!("Yazelix found old {label} symlink {}.", path.display()),
            "Update the symlink owner, such as Home Manager, to write ~/.config/yazelix/settings.jsonc.",
            json!({ "path": path.display().to_string() }),
        ));
    }
    if !metadata.file_type().is_file() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "old_settings_input_not_regular_file",
            format!(
                "Yazelix found old {label} input that is not a regular file: {}.",
                path.display()
            ),
            "Move the old path aside or replace it with a regular file, then retry.",
            json!({ "path": path.display().to_string() }),
        ));
    }
    Ok(true)
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

fn read_toml_source_or_default(
    source: Option<&Path>,
    default: &Path,
    label: &str,
) -> Result<toml::Table, CoreError> {
    let path = source.unwrap_or(default);
    read_toml_table(
        path,
        "read_settings_migration_input",
        &format!("Could not parse {label} TOML"),
    )
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

fn render_settings_jsonc(main_table: &toml::Table) -> Result<String, CoreError> {
    let mut root = toml_value_to_json(&TomlValue::Table(main_table.clone()))?;
    let Some(root_object) = root.as_object_mut() else {
        return Err(CoreError::classified(
            ErrorClass::Internal,
            "settings_render_root_not_object",
            "Could not render settings.jsonc from the main config table.",
            "Report this as a Yazelix internal error.",
            json!({}),
        ));
    };
    root_object.remove("cursors");
    render_settings_jsonc_value(&root)
}

pub fn render_settings_jsonc_value(value: &JsonValue) -> Result<String, CoreError> {
    let body = match value.as_object() {
        Some(object) => render_ordered_settings_root(object)?,
        None => serialize_settings_jsonc_fragment(value)?,
    };
    Ok(format!(
        "// Yazelix settings. Edit with `yzx config`/your editor; schema metadata powers future UI discovery.\n{body}\n"
    ))
}

fn serialize_settings_jsonc_fragment(value: &JsonValue) -> Result<String, CoreError> {
    serde_json::to_string_pretty(value).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_settings_jsonc",
            format!("Could not serialize settings.jsonc: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })
}

fn render_ordered_settings_root(object: &JsonMap<String, JsonValue>) -> Result<String, CoreError> {
    let mut ordered_keys = Vec::new();
    for key in SETTINGS_TOP_LEVEL_ORDER {
        if object.contains_key(*key) {
            ordered_keys.push((*key).to_string());
        }
    }
    for key in object.keys() {
        if key != "cursors" && !SETTINGS_TOP_LEVEL_ORDER.contains(&key.as_str()) {
            ordered_keys.push(key.clone());
        }
    }
    if object.contains_key("cursors") {
        ordered_keys.push("cursors".to_string());
    }

    let mut entries = Vec::with_capacity(ordered_keys.len());
    for key in ordered_keys {
        let rendered_key = serde_json::to_string(&key).map_err(|source| {
            CoreError::classified(
                ErrorClass::Internal,
                "serialize_settings_jsonc",
                format!("Could not serialize settings.jsonc key: {source}"),
                "Report this as a Yazelix internal error.",
                json!({ "key": key }),
            )
        })?;
        let rendered_value = serialize_settings_jsonc_fragment(&object[&key])?;
        let indented_value = rendered_value
            .lines()
            .enumerate()
            .map(|(index, line)| {
                if index == 0 {
                    line.to_string()
                } else {
                    format!("  {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        entries.push(format!("  {rendered_key}: {indented_value}"));
    }

    Ok(format!("{{\n{}\n}}", entries.join(",\n")))
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

fn json_value_to_toml_table(value: &JsonValue, path: &Path) -> Result<toml::Table, CoreError> {
    let JsonValue::Object(object) = value else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "settings_jsonc_not_object",
            "Yazelix settings.jsonc must contain a JSON object.",
            "Replace settings.jsonc with a valid object, then retry.",
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
            "Yazelix settings.jsonc contains null where a concrete value is required.",
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
                    "Yazelix settings.jsonc contains a number that cannot be represented.",
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

fn move_migrated_input(path: &Path) -> Result<(), CoreError> {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Ok(());
    };
    let backup = path.with_file_name(format!("{file_name}.migrated-{}", timestamp()));
    fs::rename(path, &backup).map_err(|source| {
        io_err(
            "move_migrated_settings_input",
            path,
            "Could not move old Yazelix settings input after migration",
            source,
        )
    })
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
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
    use crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME;
    use std::fs;
    use tempfile::tempdir;

    fn write_defaults(root: &Path) -> (PathBuf, PathBuf) {
        let main = root.join("yazelix_default.toml");
        let cursor = root.join(DEFAULT_CURSOR_CONFIG_FILENAME);
        fs::write(
            &main,
            "[editor]\ncommand = \"hx\"\nhide_sidebar_on_file_open = true\n",
        )
        .unwrap();
        fs::write(
            &cursor,
            "schema_version = 1\nenabled_cursors = [\"snow\"]\n[settings]\ntrail = \"snow\"\ntrail_effect = \"tail\"\nmode_effect = \"ripple\"\nglow = \"medium\"\nduration = 1.0\nkitty_enable_cursor = true\n[[cursor]]\nname = \"snow\"\nfamily = \"mono\"\ncolor = \"#ffffff\"\n",
        )
        .unwrap();
        (main, cursor)
    }

    // Defends: new installs create settings.jsonc instead of keeping the old main/cursor TOML surfaces alive.
    #[test]
    fn creates_settings_jsonc_from_defaults() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let (main, cursor) = write_defaults(runtime.path());

        let path = ensure_settings_config(config.path(), &main, &cursor).unwrap();
        assert_eq!(path, config.path().join("settings.jsonc"));

        let value = read_settings_jsonc_value(&path).unwrap();
        assert_eq!(
            value["editor"]["hide_sidebar_on_file_open"].as_bool(),
            Some(true)
        );
        assert!(value.get("cursors").is_none());
        let cursor_value =
            read_settings_jsonc_value(&config.path().join("yazelix_cursors/settings.jsonc"))
                .unwrap();
        assert_eq!(cursor_value["settings"]["trail"].as_str(), Some("snow"));
        assert!(!config.path().join("yazelix.toml").exists());
        assert!(!config.path().join("cursors.toml").exists());
    }

    // Regression: JSONC parse errors should explain that TOML/Nix-style # comments are not valid settings comments.
    #[test]
    fn invalid_jsonc_error_mentions_supported_comment_syntax() {
        let err = parse_jsonc_value(Path::new("settings.jsonc"), "{\n  # comment\n}\n")
            .expect_err("invalid jsonc");

        assert_eq!(err.code(), "invalid_settings_jsonc");
        assert!(err.remediation().contains("not `#`"));
    }

    // Defends: generated settings.jsonc stays focused on main settings while cursors use their shared sidecar.
    #[test]
    fn renders_default_settings_without_embedded_cursors() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let rendered = render_default_settings_jsonc(
            &repo.join("yazelix_default.toml"),
            &repo.join(DEFAULT_CURSOR_CONFIG_FILENAME),
        )
        .unwrap();

        assert!(rendered.contains("\"yazi\""));
        assert!(!rendered.contains("\"cursors\""));
    }

    // Defends: old flat TOML config inputs are one-time migration inputs, not long-lived runtime alternatives.
    #[test]
    fn auto_migrates_old_flat_inputs_and_moves_them_aside() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let (main, cursor) = write_defaults(runtime.path());
        fs::write(
            config.path().join("yazelix.toml"),
            "[editor]\ncommand = \"nvim\"\n",
        )
        .unwrap();
        fs::write(
            config.path().join("cursors.toml"),
            "schema_version = 1\nenabled_cursors = [\"snow\"]\n[settings]\ntrail = \"snow\"\ntrail_effect = \"tail\"\nmode_effect = \"ripple\"\nglow = \"high\"\nduration = 1.0\nkitty_enable_cursor = true\n[[cursor]]\nname = \"snow\"\nfamily = \"mono\"\ncolor = \"#ffffff\"\n",
        )
        .unwrap();

        let path = ensure_settings_config(config.path(), &main, &cursor).unwrap();
        let value = read_settings_jsonc_value(&path).unwrap();

        assert_eq!(value["editor"]["command"].as_str(), Some("nvim"));
        assert!(value.get("cursors").is_none());
        let cursor_value =
            read_settings_jsonc_value(&config.path().join("yazelix_cursors/settings.jsonc"))
                .unwrap();
        assert_eq!(cursor_value["settings"]["glow"].as_str(), Some("high"));
        assert!(!config.path().join("yazelix.toml").exists());
        assert!(!config.path().join("cursors.toml").exists());
        assert!(fs::read_dir(config.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("migrated-")
        }));
    }

    // Regression: embedded settings.jsonc cursors migrate once to the shared cursor config, then are removed from the main file.
    #[test]
    fn migrates_embedded_cursors_to_shared_cursor_settings_once() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let (main, cursor) = write_defaults(runtime.path());
        let embedded = render_default_settings_jsonc(&main, &cursor).unwrap();
        let mut value =
            parse_jsonc_value(&config.path().join("settings.jsonc"), &embedded).unwrap();
        let cursor_table = read_toml_table(&cursor, "read_cursor", "read cursor").unwrap();
        value.as_object_mut().unwrap().insert(
            "cursors".to_string(),
            toml_value_to_json(&TomlValue::Table(cursor_table)).unwrap(),
        );
        fs::write(
            config.path().join("settings.jsonc"),
            render_settings_jsonc_value(&value).unwrap(),
        )
        .unwrap();

        ensure_settings_config(config.path(), &main, &cursor).unwrap();

        let main_value = read_settings_jsonc_value(&config.path().join("settings.jsonc")).unwrap();
        assert!(main_value.get("cursors").is_none());
        let cursor_value =
            read_settings_jsonc_value(&config.path().join("yazelix_cursors/settings.jsonc"))
                .unwrap();
        assert_eq!(cursor_value["settings"]["trail"].as_str(), Some("snow"));
    }

    // Defends: settings.jsonc plus stale old-format inputs fails fast instead of mixing config owners.
    #[test]
    fn hard_errors_when_settings_and_old_input_coexist() {
        let runtime = tempdir().unwrap();
        let config = tempdir().unwrap();
        let (main, cursor) = write_defaults(runtime.path());
        ensure_settings_config(config.path(), &main, &cursor).unwrap();
        fs::write(config.path().join("yazelix.toml"), "[core]\n").unwrap();

        let err = ensure_settings_config(config.path(), &main, &cursor).unwrap_err();
        assert_eq!(err.code(), "stale_old_settings_input");
    }
}
