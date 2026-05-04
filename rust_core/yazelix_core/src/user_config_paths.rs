//! Canonical paths for user-editable Yazelix-owned config surfaces.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const MAIN_CONFIG: &str = "yazelix.toml";
pub const CURSOR_CONFIG: &str = "cursors.toml";
pub const HELIX_CONFIG: &str = "helix.toml";
pub const ZELLIJ_CONFIG: &str = "zellij.kdl";
pub const YAZI_CONFIG: &str = "yazi.toml";
pub const YAZI_KEYMAP: &str = "yazi_keymap.toml";
pub const YAZI_INIT: &str = "yazi_init.lua";

pub fn main_config(config_dir: &Path) -> PathBuf {
    config_dir.join(MAIN_CONFIG)
}

pub fn legacy_main_config(config_dir: &Path) -> PathBuf {
    config_dir.join("user_configs").join(MAIN_CONFIG)
}

pub fn cursor_config(config_dir: &Path) -> PathBuf {
    config_dir.join(CURSOR_CONFIG)
}

pub fn legacy_cursor_config(config_dir: &Path) -> PathBuf {
    config_dir.join("user_configs").join("yazelix_cursors.toml")
}

pub fn helix_config(config_dir: &Path) -> PathBuf {
    config_dir.join(HELIX_CONFIG)
}

pub fn legacy_helix_config(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("helix")
        .join("config.toml")
}

pub fn zellij_config(config_dir: &Path) -> PathBuf {
    config_dir.join(ZELLIJ_CONFIG)
}

pub fn legacy_zellij_config(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("zellij")
        .join("config.kdl")
}

pub fn yazi_config(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_CONFIG)
}

pub fn legacy_yazi_config(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("yazi")
        .join("yazi.toml")
}

pub fn yazi_keymap(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_KEYMAP)
}

pub fn legacy_yazi_keymap(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("yazi")
        .join("keymap.toml")
}

pub fn yazi_init(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_INIT)
}

pub fn legacy_yazi_init(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("yazi")
        .join("init.lua")
}

pub fn terminal_config(config_dir: &Path, terminal: &str) -> Option<PathBuf> {
    match terminal {
        "ghostty" => Some(config_dir.join("terminal_ghostty.conf")),
        "kitty" => Some(config_dir.join("terminal_kitty.conf")),
        "alacritty" => Some(config_dir.join("terminal_alacritty.toml")),
        "foot" => Some(config_dir.join("terminal_foot.ini")),
        _ => None,
    }
}

pub fn legacy_terminal_config(config_dir: &Path, terminal: &str) -> Option<PathBuf> {
    let root = config_dir.join("user_configs").join("terminal");
    match terminal {
        "ghostty" => Some(root.join("ghostty")),
        "kitty" => Some(root.join("kitty.conf")),
        "alacritty" => Some(root.join("alacritty.toml")),
        "foot" => Some(root.join("foot.ini")),
        _ => None,
    }
}

pub fn shell_hook(config_dir: &Path, shell: &str) -> Option<PathBuf> {
    match shell {
        "bash" => Some(config_dir.join("shell_bash.sh")),
        "zsh" => Some(config_dir.join("shell_zsh.zsh")),
        "fish" => Some(config_dir.join("shell_fish.fish")),
        "nu" => Some(config_dir.join("shell_nu.nu")),
        _ => None,
    }
}

pub fn legacy_shell_hook(config_dir: &Path, shell: &str) -> Option<PathBuf> {
    let root = config_dir.join("user_configs").join("shells");
    match shell {
        "bash" => Some(root.join("bash.sh")),
        "zsh" => Some(root.join("zsh.zsh")),
        "fish" => Some(root.join("fish.fish")),
        "nu" => Some(root.join("nu.nu")),
        _ => None,
    }
}

fn io_err(code: &'static str, path: &Path, source: io::Error) -> CoreError {
    CoreError::io(
        code,
        "Could not migrate a Yazelix config path",
        "Fix permissions or move the reported file manually, then retry.",
        path.display().to_string(),
        source,
    )
}

fn optional_symlink_metadata(path: &Path) -> Result<Option<fs::Metadata>, CoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io_err("stat_config_path_for_migration", path, source)),
    }
}

pub fn resolve_flat_config_file(
    current_path: &Path,
    legacy_path: &Path,
    label: &str,
) -> Result<PathBuf, CoreError> {
    let current_exists = current_path.exists();
    let legacy_metadata = optional_symlink_metadata(legacy_path)?;
    let legacy_exists = legacy_metadata.is_some();

    if let Some(metadata) = legacy_metadata.as_ref() {
        if metadata.file_type().is_symlink() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "legacy_config_symlink_requires_manual_migration",
                format!(
                    "Yazelix found an old {label} config symlink at {}.",
                    legacy_path.display()
                ),
                "Update your Home Manager or external symlink owner to write the new flat ~/.config/yazelix path, then retry.",
                json!({
                    "label": label,
                    "current_path": current_path.display().to_string(),
                    "legacy_path": legacy_path.display().to_string(),
                }),
            ));
        }

        if !metadata.file_type().is_file() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "legacy_config_not_regular_file",
                format!(
                    "Yazelix found an old {label} config path that is not a regular file: {}.",
                    legacy_path.display()
                ),
                "Move the old path aside or replace it with a regular file, then retry.",
                json!({
                    "label": label,
                    "current_path": current_path.display().to_string(),
                    "legacy_path": legacy_path.display().to_string(),
                }),
            ));
        }
    }

    if current_exists && legacy_exists {
        let current = fs::read(current_path).map_err(|source| {
            io_err(
                "read_current_flat_config_for_migration",
                current_path,
                source,
            )
        })?;
        let legacy = fs::read(legacy_path)
            .map_err(|source| io_err("read_legacy_config_for_migration", legacy_path, source))?;
        if current == legacy {
            eprintln!(
                "ℹ️  {} already exists at {}. The old path {} has identical content and can be removed.",
                label,
                current_path.display(),
                legacy_path.display()
            );
            return Ok(current_path.to_path_buf());
        }

        return Err(CoreError::classified(
            ErrorClass::Config,
            "duplicate_flat_config_surface",
            format!("Yazelix found duplicate {label} config surfaces."),
            "Keep the flat ~/.config/yazelix file or move the old user_configs file aside, then retry.",
            json!({
                "label": label,
                "current_path": current_path.display().to_string(),
                "legacy_path": legacy_path.display().to_string(),
            }),
        ));
    }

    if !current_exists && legacy_exists {
        if let Some(parent) = current_path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                io_err("create_flat_config_parent_for_migration", parent, source)
            })?;
        }
        fs::rename(legacy_path, current_path)
            .map_err(|source| io_err("rename_legacy_config_to_flat_path", legacy_path, source))?;
        eprintln!(
            "✅ Migrated {} from {} to {}",
            label,
            legacy_path.display(),
            current_path.display()
        );
    }

    Ok(current_path.to_path_buf())
}
