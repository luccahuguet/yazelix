//! Resolve active Yazelix config paths for control-plane commands (Rust-only path).

use crate::bridge::{CoreError, ErrorClass};
use crate::ghostty_cursor_registry::{CursorRegistry, DEFAULT_CURSOR_CONFIG_FILENAME};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct ActiveConfigPaths {
    pub user_config_dir: PathBuf,
    pub user_config: PathBuf,
    pub user_cursor_config: PathBuf,
    pub legacy_user_config: PathBuf,
    pub managed_taplo: PathBuf,
    pub config_file: PathBuf,
    pub default_config_path: PathBuf,
    pub default_cursor_config_path: PathBuf,
    pub contract_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PrimaryConfigPaths {
    pub user_config_dir: PathBuf,
    pub user_config: PathBuf,
    pub user_cursor_config: PathBuf,
    pub legacy_user_config: PathBuf,
    pub default_config_path: PathBuf,
    pub default_cursor_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_taplo: PathBuf,
    pub managed_taplo: PathBuf,
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
    let user_config_dir = config_dir.join("user_configs");
    let user_config = user_config_dir.join("yazelix.toml");
    let user_cursor_config = CursorRegistry::user_config_path(config_dir);
    let legacy_user_config = config_dir.join("yazelix.toml");
    let default_config_path = runtime_dir.join("yazelix_default.toml");
    let default_cursor_config_path = runtime_dir.join(DEFAULT_CURSOR_CONFIG_FILENAME);
    let contract_path = runtime_dir
        .join("config_metadata")
        .join("main_config_contract.toml");
    let runtime_taplo = runtime_dir.join(".taplo.toml");
    let managed_taplo = config_dir.join(".taplo.toml");

    PrimaryConfigPaths {
        user_config_dir,
        user_config,
        user_cursor_config,
        legacy_user_config,
        default_config_path,
        default_cursor_config_path,
        contract_path,
        runtime_taplo,
        managed_taplo,
    }
}

pub fn validate_primary_config_surface(paths: &PrimaryConfigPaths) -> Result<(), CoreError> {
    if paths.user_config.exists() && paths.legacy_user_config.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "duplicate_config_surfaces",
            "Yazelix found duplicate config surfaces in both the repo root and user_configs.",
            "Keep only the user_configs copies. Move or delete the legacy root-level config files so Yazelix has one clear config owner.",
            json!({
                "user_config": paths.user_config.display().to_string(),
                "legacy_user_config": paths.legacy_user_config.display().to_string(),
            }),
        ));
    }

    if paths.legacy_user_config.exists() && !paths.user_config.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "legacy_root_config_surface",
            "Yazelix found an unsupported legacy root-level config surface.",
            "Move your current Yazelix config to user_configs/yazelix.toml manually, or run `yzx config reset` to create a fresh v15 config template.",
            json!({
                "legacy_main": paths.legacy_user_config.display().to_string(),
                "current_main": paths.user_config.display().to_string(),
            }),
        ));
    }

    Ok(())
}

/// Canonical Rust owner for active managed-config surface resolution.
pub fn resolve_active_config_paths(
    runtime_dir: &Path,
    config_dir: &Path,
    config_override: Option<&str>,
) -> Result<ActiveConfigPaths, CoreError> {
    let paths = primary_config_paths(runtime_dir, config_dir);

    validate_primary_config_surface(&paths)?;
    ensure_managed_taplo(&paths.runtime_taplo, &paths.managed_taplo)?;
    ensure_user_cursor_config(&paths.default_cursor_config_path, &paths.user_cursor_config)?;

    let config_file = match config_override {
        Some(raw) if !raw.trim().is_empty() => PathBuf::from(raw.trim()),
        _ if paths.user_config.exists() => paths.user_config.clone(),
        _ if paths.default_config_path.exists() => {
            eprintln!("📝 Creating yazelix.toml from yazelix_default.toml...");
            fs::create_dir_all(&paths.user_config_dir)
                .map_err(|e| io_err(&paths.user_config_dir, e))?;
            fs::copy(&paths.default_config_path, &paths.user_config)
                .map_err(|e| io_err(&paths.user_config, e))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::Permissions::from_mode(0o644);
                let _ = fs::set_permissions(&paths.user_config, mode);
            }
            eprintln!("✅ yazelix.toml created\n");
            paths.user_config.clone()
        }
        _ => {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "missing_default_config",
                "No yazelix configuration file found.",
                "Restore yazelix_default.toml, or reinstall Yazelix if the default config is missing from the runtime.",
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
        managed_taplo: paths.managed_taplo,
        config_file,
        default_config_path: paths.default_config_path,
        default_cursor_config_path: paths.default_cursor_config_path,
        contract_path: paths.contract_path,
    })
}

fn ensure_user_cursor_config(runtime_src: &Path, managed: &Path) -> Result<(), CoreError> {
    if !runtime_src.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_default_cursor_config",
            format!(
                "Yazelix runtime is missing the default cursor registry at {}.",
                runtime_src.display()
            ),
            "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml.",
            json!({ "path": runtime_src.display().to_string() }),
        ));
    }

    if managed.exists() {
        return Ok(());
    }

    if let Some(parent) = managed.parent() {
        fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
    }
    fs::copy(runtime_src, managed).map_err(|e| io_err(managed, e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::Permissions::from_mode(0o644);
        let _ = fs::set_permissions(managed, mode);
    }

    Ok(())
}

pub fn ensure_managed_taplo(runtime_src: &Path, managed: &Path) -> Result<(), CoreError> {
    if !runtime_src.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_runtime_taplo",
            format!(
                "Yazelix runtime is missing the Taplo formatter config at {}.",
                runtime_src.display()
            ),
            "Reinstall Yazelix so the runtime includes the managed Taplo formatter config.",
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

    fn write_runtime_layout(runtime_dir: &Path) {
        fs::write(
            runtime_dir.join("yazelix_default.toml"),
            "[core]\nwelcome_style = \"minimal\"\n",
        )
        .expect("write default config");
        fs::write(
            runtime_dir.join(DEFAULT_CURSOR_CONFIG_FILENAME),
            include_str!("../../../yazelix_cursors_default.toml"),
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
            runtime_dir.join(".taplo.toml"),
            "array_auto_expand = true\n",
        )
        .expect("write taplo");
    }

    // Defends: Rust active-config-surface resolution bootstraps the managed main config and Taplo support when the canonical surface is missing.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn bootstraps_missing_managed_config_and_taplo_support() {
        let runtime = tempdir().expect("runtime dir");
        let config = tempdir().expect("config dir");
        write_runtime_layout(runtime.path());

        let resolved = resolve_active_config_paths(runtime.path(), config.path(), None).unwrap();

        assert_eq!(
            resolved.user_config,
            config.path().join("user_configs").join("yazelix.toml")
        );
        assert_eq!(resolved.config_file, resolved.user_config);
        assert_eq!(
            fs::read_to_string(&resolved.config_file).unwrap(),
            fs::read_to_string(runtime.path().join("yazelix_default.toml")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(&resolved.managed_taplo).unwrap(),
            fs::read_to_string(runtime.path().join(".taplo.toml")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(&resolved.user_cursor_config).unwrap(),
            fs::read_to_string(runtime.path().join(DEFAULT_CURSOR_CONFIG_FILENAME)).unwrap()
        );
    }

    // Defends: Rust active-config-surface resolution rejects duplicate canonical and legacy managed config surfaces instead of guessing.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rejects_duplicate_canonical_and_legacy_surfaces() {
        let runtime = tempdir().expect("runtime dir");
        let config = tempdir().expect("config dir");
        write_runtime_layout(runtime.path());

        let user_config_dir = config.path().join("user_configs");
        fs::create_dir_all(&user_config_dir).expect("user config dir");
        fs::write(
            user_config_dir.join("yazelix.toml"),
            "[core]\nwelcome_style = \"minimal\"\n",
        )
        .expect("write canonical config");
        fs::write(
            config.path().join("yazelix.toml"),
            "[core]\nwelcome_style = \"random\"\n",
        )
        .expect("write legacy config");

        let error = resolve_active_config_paths(runtime.path(), config.path(), None).unwrap_err();
        assert_eq!(error.code(), "duplicate_config_surfaces");
    }
}
