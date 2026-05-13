//! Resolve active Yazelix config paths for control-plane commands (Rust-only path).

use crate::bridge::{CoreError, ErrorClass};
use crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME;
use crate::runtime_component_enabled;
use crate::settings_surface::{
    DEFAULT_SETTINGS_CONFIG_FILENAME, ensure_settings_config_with_cursor_component,
    settings_schema_path, settings_surface_paths,
};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const TOML_TOOLING_CONFIG_FILENAME: &str = "tombi.toml";

#[derive(Debug, Clone, Serialize)]
pub struct ActiveConfigPaths {
    pub user_config_dir: PathBuf,
    pub user_config: PathBuf,
    pub user_cursor_config: PathBuf,
    pub legacy_user_config: PathBuf,
    pub managed_toml_tooling_config: PathBuf,
    pub config_file: PathBuf,
    pub default_config_path: PathBuf,
    pub default_cursor_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub settings_schema_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PrimaryConfigPaths {
    pub user_config_dir: PathBuf,
    pub user_config: PathBuf,
    pub user_cursor_config: PathBuf,
    pub legacy_user_config: PathBuf,
    pub old_flat_user_config: PathBuf,
    pub default_config_path: PathBuf,
    pub default_cursor_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub settings_schema_path: PathBuf,
    pub runtime_toml_tooling_config: PathBuf,
    pub managed_toml_tooling_config: PathBuf,
}

fn io_err(path: &Path, source: io::Error) -> CoreError {
    CoreError::io(
        "control_config_io",
        "Could not access a Yazelix config path",
        "Fix permissions or restore the missing path, then retry.",
        path.display().to_string(),
        source,
    )
}

pub fn primary_config_paths(runtime_dir: &Path, config_dir: &Path) -> PrimaryConfigPaths {
    let user_config_dir = config_dir.to_path_buf();
    let settings_paths = settings_surface_paths(config_dir);
    let user_config = settings_paths.settings_config;
    let user_cursor_config = settings_paths.shared_cursor_config;
    let old_flat_user_config = settings_paths.old_main_config;
    let legacy_user_config = settings_paths.old_nested_main_config;
    let default_config_path = runtime_dir.join(DEFAULT_SETTINGS_CONFIG_FILENAME);
    let default_cursor_config_path = runtime_dir.join(DEFAULT_CURSOR_CONFIG_FILENAME);
    let contract_path = runtime_dir
        .join("config_metadata")
        .join("main_config_contract.toml");
    let settings_schema_path = settings_schema_path(runtime_dir);
    let runtime_toml_tooling_config = runtime_dir.join(TOML_TOOLING_CONFIG_FILENAME);
    let managed_toml_tooling_config = config_dir.join(TOML_TOOLING_CONFIG_FILENAME);

    PrimaryConfigPaths {
        user_config_dir,
        user_config,
        user_cursor_config,
        legacy_user_config,
        old_flat_user_config,
        default_config_path,
        default_cursor_config_path,
        contract_path,
        settings_schema_path,
        runtime_toml_tooling_config,
        managed_toml_tooling_config,
    }
}

pub fn validate_primary_config_surface(paths: &PrimaryConfigPaths) -> Result<(), CoreError> {
    if old_config_entry_exists(&paths.old_flat_user_config)
        || old_config_entry_exists(&paths.legacy_user_config)
    {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "stale_old_settings_input",
            "Yazelix found old settings input next to the canonical config surface.",
            "Move the old TOML config aside and keep settings.jsonc as the only Yazelix settings source.",
            json!({
                "user_config": paths.user_config.display().to_string(),
                "old_flat_user_config": paths.old_flat_user_config.display().to_string(),
                "legacy_user_config": paths.legacy_user_config.display().to_string(),
            }),
        ));
    }

    Ok(())
}

fn old_config_entry_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Canonical Rust owner for active managed-config surface resolution.
pub fn resolve_active_config_paths(
    runtime_dir: &Path,
    config_dir: &Path,
    config_override: Option<&str>,
) -> Result<ActiveConfigPaths, CoreError> {
    let paths = primary_config_paths(runtime_dir, config_dir);
    let cursor_component_enabled = runtime_component_enabled(runtime_dir, "cursors")?;

    ensure_settings_config_with_cursor_component(
        &paths.user_config_dir,
        &paths.default_config_path,
        &paths.default_cursor_config_path,
        cursor_component_enabled,
    )?;
    ensure_managed_toml_tooling_config(
        &paths.runtime_toml_tooling_config,
        &paths.managed_toml_tooling_config,
    )?;

    let config_file = match config_override {
        Some(raw) if !raw.trim().is_empty() => PathBuf::from(raw.trim()),
        _ if paths.user_config.exists() => paths.user_config.clone(),
        _ => {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "missing_default_config",
                "No Yazelix settings file found.",
                "Restore settings.jsonc with `yzx reset config`, or reinstall Yazelix if the shipped defaults are missing from the runtime.",
                json!({}),
            ));
        }
    };

    if !paths.contract_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_config_contract",
            format!(
                "Yazelix runtime is missing the config contract at {}.",
                paths.contract_path.display()
            ),
            "Reinstall Yazelix so the runtime includes config_metadata/main_config_contract.toml.",
            json!({ "path": paths.contract_path.display().to_string() }),
        ));
    }

    Ok(ActiveConfigPaths {
        user_config_dir: paths.user_config_dir,
        user_config: paths.user_config,
        user_cursor_config: paths.user_cursor_config,
        legacy_user_config: paths.legacy_user_config,
        managed_toml_tooling_config: paths.managed_toml_tooling_config,
        config_file,
        default_config_path: paths.default_config_path,
        default_cursor_config_path: paths.default_cursor_config_path,
        contract_path: paths.contract_path,
        settings_schema_path: paths.settings_schema_path,
    })
}

pub fn ensure_managed_toml_tooling_config(
    runtime_src: &Path,
    managed: &Path,
) -> Result<(), CoreError> {
    if !runtime_src.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_runtime_toml_tooling_config",
            format!(
                "Yazelix runtime is missing the TOML tooling config at {}.",
                runtime_src.display()
            ),
            "Reinstall Yazelix so the runtime includes the managed TOML tooling config.",
            json!({ "path": runtime_src.display().to_string() }),
        ));
    }

    let source_content = fs::read_to_string(runtime_src).map_err(|e| io_err(runtime_src, e))?;

    let should_write = match fs::read_to_string(managed) {
        Ok(existing) => existing != source_content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => true,
        Err(e) => return Err(io_err(managed, e)),
    };

    if should_write {
        if let Some(parent) = managed.parent() {
            fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
        }
        let mut f = fs::File::create(managed).map_err(|e| io_err(managed, e))?;
        f.write_all(source_content.as_bytes())
            .map_err(|e| io_err(managed, e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::Permissions::from_mode(0o644);
            let _ = fs::set_permissions(managed, mode);
        }
    }

    Ok(())
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::tempdir;

    const DEFAULT_SETTINGS_FIXTURE: &str = r#"{
  "core": {
    "welcome_style": "minimal"
  }
}
"#;

    fn write_runtime_layout(runtime_dir: &Path) {
        fs::write(
            runtime_dir.join("settings_default.jsonc"),
            DEFAULT_SETTINGS_FIXTURE,
        )
        .expect("write default config");
        fs::write(
            runtime_dir.join(DEFAULT_CURSOR_CONFIG_FILENAME),
            include_str!("../../../yazelix_ghostty_cursors_default.toml"),
        )
        .expect("write cursor config");
        fs::create_dir_all(runtime_dir.join("config_metadata")).expect("contract dir");
        fs::write(
            runtime_dir
                .join("config_metadata")
                .join("main_config_contract.toml"),
            "[fields]\n",
        )
        .expect("write contract");
        fs::write(
            runtime_dir.join(TOML_TOOLING_CONFIG_FILENAME),
            "array_auto_expand = true\n",
        )
        .expect("write TOML tooling config");
        fs::write(
            runtime_dir.join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": true, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .expect("write runtime component manifest");
    }

    // Defends: Rust active-config-surface resolution bootstraps settings.jsonc and TOML tooling support when the canonical surface is missing.
    #[test]
    fn bootstraps_missing_managed_config_and_toml_tooling_support() {
        let runtime = tempdir().expect("runtime dir");
        let config = tempdir().expect("config dir");
        write_runtime_layout(runtime.path());

        let resolved = resolve_active_config_paths(runtime.path(), config.path(), None).unwrap();

        assert_eq!(resolved.user_config, config.path().join("settings.jsonc"));
        assert_eq!(resolved.config_file, resolved.user_config);
        let rendered = fs::read_to_string(&resolved.config_file).unwrap();
        assert!(rendered.contains("\"core\""));
        assert!(!rendered.contains("\"cursors\""));
        assert!(resolved.user_cursor_config.exists());
        assert_eq!(
            fs::read_to_string(&resolved.managed_toml_tooling_config).unwrap(),
            fs::read_to_string(runtime.path().join(TOML_TOOLING_CONFIG_FILENAME)).unwrap()
        );
        assert_eq!(
            resolved.user_cursor_config,
            config.path().join("yazelix_ghostty_cursors/settings.jsonc")
        );
    }

    // Defends: disabling the cursor component does not require or generate the shared cursor sidecar during config bootstrap.
    #[test]
    fn disabled_cursor_component_bootstraps_without_cursor_sidecar() {
        let runtime = tempdir().expect("runtime dir");
        let config = tempdir().expect("config dir");
        fs::write(
            runtime.path().join("settings_default.jsonc"),
            DEFAULT_SETTINGS_FIXTURE,
        )
        .expect("write default config");
        fs::create_dir_all(runtime.path().join("config_metadata")).expect("contract dir");
        fs::write(
            runtime
                .path()
                .join("config_metadata")
                .join("main_config_contract.toml"),
            "[fields]\n",
        )
        .expect("write contract");
        fs::write(
            runtime.path().join(TOML_TOOLING_CONFIG_FILENAME),
            "array_auto_expand = true\n",
        )
        .expect("write TOML tooling config");
        fs::write(
            runtime.path().join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": false, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .expect("write runtime component manifest");

        let resolved = resolve_active_config_paths(runtime.path(), config.path(), None).unwrap();

        assert!(resolved.user_config.exists());
        assert!(!resolved.user_cursor_config.exists());
    }

    // Defends: Rust active-config-surface resolution rejects stale old-format inputs when settings.jsonc already exists.
    #[test]
    fn rejects_settings_jsonc_with_old_inputs() {
        let runtime = tempdir().expect("runtime dir");
        let config = tempdir().expect("config dir");
        write_runtime_layout(runtime.path());

        fs::write(config.path().join("settings.jsonc"), "{}").expect("write settings config");
        fs::write(config.path().join("yazelix.toml"), "[core]\n").expect("write old config");

        let error = resolve_active_config_paths(runtime.path(), config.path(), None).unwrap_err();
        assert_eq!(error.code(), "stale_old_settings_input");
    }

    // Defends: old-only regular main configs fail fast instead of being silently migrated.
    #[test]
    fn rejects_old_only_regular_main_config() {
        let runtime = tempdir().expect("runtime dir");
        let config = tempdir().expect("config dir");
        write_runtime_layout(runtime.path());

        let user_config_dir = config.path().join("user_configs");
        let legacy_config = user_config_dir.join("yazelix.toml");
        fs::create_dir_all(&user_config_dir).expect("user config dir");
        fs::write(&legacy_config, "[core]\nwelcome_style = \"minimal\"\n")
            .expect("write legacy config");

        let error = resolve_active_config_paths(runtime.path(), config.path(), None).unwrap_err();
        assert_eq!(error.code(), "stale_old_settings_input");
        assert!(!config.path().join("settings.jsonc").exists());
        assert!(legacy_config.exists());
    }

    // Regression: dangling old nested symlinks still block startup instead of being ignored as missing paths.
    #[cfg(unix)]
    #[test]
    fn rejects_dangling_legacy_config_symlink() {
        use std::os::unix::fs::symlink;

        let runtime = tempdir().expect("runtime dir");
        let config = tempdir().expect("config dir");
        write_runtime_layout(runtime.path());

        let user_config_dir = config.path().join("user_configs");
        fs::create_dir_all(&user_config_dir).expect("user config dir");
        symlink(
            config.path().join("missing_home_manager_target.toml"),
            user_config_dir.join("yazelix.toml"),
        )
        .expect("legacy symlink");

        let error = resolve_active_config_paths(runtime.path(), config.path(), None).unwrap_err();
        assert_eq!(error.code(), "stale_old_settings_input");
    }
}
