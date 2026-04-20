//! Resolve active Yazelix config paths for control-plane commands (Rust-only path).

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct ActiveConfigPaths {
    pub config_file: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
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

/// Match `nushell/scripts/utils/config_surfaces.nu` `resolve_active_config_paths` + reconciliation.
pub fn resolve_active_config_paths(
    runtime_dir: &Path,
    config_dir: &Path,
    config_override: Option<&str>,
) -> Result<ActiveConfigPaths, CoreError> {
    let user_config_dir = config_dir.join("user_configs");
    let user_config = user_config_dir.join("yazelix.toml");
    let legacy_user_config = config_dir.join("yazelix.toml");
    let default_config_path = runtime_dir.join("yazelix_default.toml");
    let contract_path = runtime_dir
        .join("config_metadata")
        .join("main_config_contract.toml");
    let runtime_taplo = runtime_dir.join(".taplo.toml");
    let managed_taplo = config_dir.join(".taplo.toml");

    if user_config.exists() && legacy_user_config.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "duplicate_config_surfaces",
            "Yazelix found duplicate config surfaces in both the repo root and user_configs.",
            "Keep only the user_configs copies. Move or delete the legacy root-level config files so Yazelix has one clear config owner.",
            json!({
                "user_config": user_config.display().to_string(),
                "legacy_user_config": legacy_user_config.display().to_string(),
            }),
        ));
    }

    if legacy_user_config.exists() && !user_config.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "legacy_root_config_surface",
            "Yazelix found an unsupported legacy root-level config surface.",
            "Move your current Yazelix config to user_configs/yazelix.toml manually, or run `yzx config reset` to create a fresh v15 config template.",
            json!({
                "legacy_main": legacy_user_config.display().to_string(),
                "current_main": user_config.display().to_string(),
            }),
        ));
    }

    ensure_managed_taplo(&runtime_taplo, &managed_taplo)?;

    let config_file = match config_override {
        Some(raw) if !raw.trim().is_empty() => PathBuf::from(raw.trim()),
        _ if user_config.exists() => user_config.clone(),
        _ if default_config_path.exists() => {
            eprintln!("📝 Creating yazelix.toml from yazelix_default.toml...");
            fs::create_dir_all(&user_config_dir).map_err(|e| io_err(&user_config_dir, e))?;
            fs::copy(&default_config_path, &user_config).map_err(|e| io_err(&user_config, e))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::Permissions::from_mode(0o644);
                let _ = fs::set_permissions(&user_config, mode);
            }
            eprintln!("✅ yazelix.toml created\n");
            user_config
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

    if !contract_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_config_contract",
            format!(
                "Yazelix runtime is missing the config contract at {}.",
                contract_path.display()
            ),
            "Reinstall Yazelix so the runtime includes config_metadata/main_config_contract.toml.",
            json!({ "path": contract_path.display().to_string() }),
        ));
    }

    Ok(ActiveConfigPaths {
        config_file,
        default_config_path,
        contract_path,
    })
}

fn ensure_managed_taplo(runtime_src: &Path, managed: &Path) -> Result<(), CoreError> {
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
