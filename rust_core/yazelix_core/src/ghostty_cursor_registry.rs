//! Yazelix-owned loader for the reusable cursor registry.

pub use yazelix_cursors::{
    CursorColor, CursorDefinition, CursorFamily, CursorRegistry, CursorSettings,
    DEFAULT_CURSOR_CONFIG_FILENAME, DEFAULT_GHOSTTY_TRAIL_DURATION, GHOSTTY_TRAIL_DURATION_MAX,
    GHOSTTY_TRAIL_DURATION_MIN, ResolvedCursorRegistryState, SplitDivider, SplitTransition,
    format_ghostty_trail_duration, write_ghostty_cursor_effect_shaders,
    write_ghostty_cursor_palette_shaders,
};
use yazelix_cursors::{load_cursor_settings_jsonc, persist_migrated_cursor_settings_jsonc};

use crate::bridge::{CoreError, ErrorClass};
use crate::settings_surface::{is_settings_config_path, read_settings_jsonc_value};
use crate::user_config_paths;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub trait YazelixCursorRegistryExt: Sized {
    fn load(path: &Path) -> Result<Self, CoreError>;
    fn load_from_cursor_settings_jsonc(path: &Path) -> Result<Self, CoreError>;
    fn load_from_settings_jsonc(path: &Path) -> Result<Self, CoreError>;
    fn user_config_path(config_dir: &Path) -> PathBuf;
    fn default_config_path(runtime_dir: &Path) -> PathBuf;
}

impl YazelixCursorRegistryExt for CursorRegistry {
    fn load(path: &Path) -> Result<Self, CoreError> {
        if user_config_paths::is_shared_cursor_config_path(path) {
            return CursorRegistry::load_from_cursor_settings_jsonc(path);
        }
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
        CursorRegistry::parse_str(path, &raw).map_err(CoreError::from)
    }

    fn load_from_cursor_settings_jsonc(path: &Path) -> Result<Self, CoreError> {
        let (registry, migration) = load_cursor_settings_jsonc(path)
            .map_err(|error| cursor_settings_jsonc_error(path, error))?;
        persist_migrated_cursor_settings_jsonc(path, &migration).map_err(CoreError::from)?;
        Ok(registry)
    }

    fn load_from_settings_jsonc(path: &Path) -> Result<Self, CoreError> {
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
                    "Fix the embedded cursors object in settings.jsonc or move it to ~/.config/yazelix_ghostty_cursors/settings.jsonc.",
                    json!({
                        "path": path.display().to_string(),
                        "error": format!("{error:?}"),
                    }),
                )
            } else {
                CoreError::from(error)
            }
        })
    }

    fn user_config_path(config_dir: &Path) -> PathBuf {
        user_config_paths::shared_cursor_config(config_dir)
    }

    fn default_config_path(runtime_dir: &Path) -> PathBuf {
        runtime_dir.join(DEFAULT_CURSOR_CONFIG_FILENAME)
    }
}

fn cursor_settings_jsonc_error(path: &Path, error: yazelix_cursors::CursorError) -> CoreError {
    if error.code() == "invalid_cursor_registry_json" {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_cursor_settings_jsonc",
            format!(
                "Could not parse Yazelix cursor settings in {}.",
                path.display()
            ),
            "Fix ~/.config/yazelix_ghostty_cursors/settings.jsonc or run `yzc init` after moving the broken file aside.",
            json!({
                "path": path.display().to_string(),
                "error": format!("{error:?}"),
            }),
        )
    } else {
        CoreError::from(error)
    }
}

#[cfg(test)]
mod tests {
    // Test lane: default

    use super::*;
    use tempfile::tempdir;

    // Regression: the main runtime loader must run child-owned cursor settings migrations before strict validation.
    #[test]
    fn shared_cursor_settings_loader_migrates_retired_neon_config() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("settings.jsonc");
        fs::write(
            &path,
            r##"{
  "schema_version": 1,
  "enabled_cursors": ["neon"],
  "settings": {
    "trail": "neon",
    "trail_effect": "tail",
    "mode_effect": "ripple",
    "glow": "medium",
    "duration": 1.0,
    "kitty_enable_cursor": true
  },
  "cursor": [
    {
      "name": "neon",
      "family": "curated_template",
      "template": "neon",
      "cursor_color": "#0090ff"
    }
  ]
}
"##,
        )
        .unwrap();

        let registry = CursorRegistry::load_from_cursor_settings_jsonc(&path).unwrap();

        assert_eq!(registry.enabled_cursors, vec!["cosmic".to_string()]);
        assert_eq!(registry.settings.trail, "cosmic");
        assert!(!fs::read_to_string(&path).unwrap().contains(r#""neon""#));

        let backup_path = fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains("backup_before_yazelix_cursors_v2"))
            })
            .expect("migration backup");
        assert!(
            fs::read_to_string(backup_path)
                .unwrap()
                .contains(r#""neon""#)
        );
    }
}
