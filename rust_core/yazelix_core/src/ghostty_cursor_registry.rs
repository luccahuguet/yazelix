//! Yazelix-owned loader for the reusable cursor registry.

pub use crate::yazelix_cursors::{
    CursorColor, CursorDefinition, CursorFamily, CursorRegistry, CursorSettings,
    DEFAULT_CURSOR_CONFIG_FILENAME, DEFAULT_GHOSTTY_TRAIL_DURATION, GHOSTTY_TRAIL_DURATION_MAX,
    GHOSTTY_TRAIL_DURATION_MIN, ResolvedCursorRegistryState, SplitDivider, SplitTransition,
    format_ghostty_trail_duration, write_ghostty_cursor_palette_shaders,
};

use crate::bridge::{CoreError, ErrorClass};
use crate::settings_surface::{is_settings_config_path, read_settings_jsonc_value};
use crate::user_config_paths;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

impl CursorRegistry {
    pub fn load(path: &Path) -> Result<Self, CoreError> {
        if is_settings_config_path(path) {
            return CursorRegistry::load_from_settings_jsonc(path);
        }

        let raw = fs::read_to_string(path).map_err(|source| {
            CoreError::io(
                "read_cursor_config",
                "Could not read Yazelix cursor config",
                "Restore settings.jsonc with `yzx reset config --yes`, then retry.",
                path.to_string_lossy(),
                source,
            )
        })?;
        CursorRegistry::parse_str(path, &raw)
    }

    pub fn load_from_settings_jsonc(path: &Path) -> Result<Self, CoreError> {
        let value = read_settings_jsonc_value(path)?;
        let Some(cursors) = value.get("cursors").cloned() else {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "missing_cursor_settings",
                "Yazelix settings.jsonc is missing its cursors section.",
                "Restore settings.jsonc with `yzx reset config --yes`, then retry.",
                json!({ "path": path.display().to_string() }),
            ));
        };
        CursorRegistry::parse_json_value(path, cursors).map_err(|error| {
            if error.code() == "invalid_cursor_registry_json" {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_cursor_settings_jsonc",
                    format!(
                        "Could not parse Yazelix cursor settings in {}.",
                        path.display()
                    ),
                    "Fix the cursors object in settings.jsonc or run `yzx reset config --yes` as a blunt fallback.",
                    json!({
                        "path": path.display().to_string(),
                        "error": format!("{error:?}"),
                    }),
                )
            } else {
                error
            }
        })
    }

    pub fn user_config_path(config_dir: &Path) -> PathBuf {
        user_config_paths::main_config(config_dir)
    }

    pub fn default_config_path(runtime_dir: &Path) -> PathBuf {
        runtime_dir.join(DEFAULT_CURSOR_CONFIG_FILENAME)
    }
}
