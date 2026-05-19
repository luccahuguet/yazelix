//! Canonical paths for user-editable Yazelix-owned config surfaces.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const SETTINGS_CONFIG: &str = "settings.jsonc";
pub const OLD_MAIN_CONFIG: &str = "yazelix.toml";
pub const CURSOR_CONFIG: &str = "cursors.toml";
pub const SHARED_CURSOR_CONFIG_DIR: &str = "yazelix_ghostty_cursors";
pub const SHARED_CURSOR_SETTINGS_CONFIG: &str = "settings.jsonc";
pub const HELIX_CONFIG: &str = "helix.toml";
pub const ZELLIJ_CONFIG: &str = "zellij.kdl";
pub const YAZI_CONFIG_DIR: &str = "yazi";
pub const YAZI_CONFIG: &str = "yazi/yazi.toml";
pub const YAZI_KEYMAP: &str = "yazi/keymap.toml";
pub const YAZI_INIT: &str = "yazi/init.lua";
pub const YAZI_PACKAGE: &str = "yazi/package.toml";
pub const YAZI_PLUGINS_DIR: &str = "yazi/plugins";
pub const YAZI_FLAVORS_DIR: &str = "yazi/flavors";
pub const TERMINAL_GHOSTTY_CONFIG: &str = "terminal_ghostty.conf";
pub const TERMINAL_KITTY_CONFIG: &str = "terminal_kitty.conf";
pub const TERMINAL_ALACRITTY_CONFIG: &str = "terminal_alacritty.toml";
pub const TERMINAL_FOOT_CONFIG: &str = "terminal_foot.ini";
pub const SHELL_BASH_HOOK: &str = "shell_bash.sh";
pub const SHELL_ZSH_HOOK: &str = "shell_zsh.zsh";
pub const SHELL_FISH_HOOK: &str = "shell_fish.fish";
pub const SHELL_NU_HOOK: &str = "shell_nu.nu";

pub const CURRENT_MANAGED_CONFIG_FILE_NAMES: &[&str] = &[
    SETTINGS_CONFIG,
    HELIX_CONFIG,
    ZELLIJ_CONFIG,
    YAZI_CONFIG,
    YAZI_KEYMAP,
    YAZI_INIT,
    YAZI_PACKAGE,
    YAZI_PLUGINS_DIR,
    YAZI_FLAVORS_DIR,
    TERMINAL_GHOSTTY_CONFIG,
    TERMINAL_KITTY_CONFIG,
    TERMINAL_ALACRITTY_CONFIG,
    TERMINAL_FOOT_CONFIG,
    SHELL_BASH_HOOK,
    SHELL_ZSH_HOOK,
    SHELL_FISH_HOOK,
    SHELL_NU_HOOK,
];

pub const LEGACY_CONFIG_ENTRY_NAMES: &[&str] = &[OLD_MAIN_CONFIG, CURSOR_CONFIG, "user_configs"];

pub fn main_config(config_dir: &Path) -> PathBuf {
    config_dir.join(SETTINGS_CONFIG)
}

pub fn old_main_config(config_dir: &Path) -> PathBuf {
    config_dir.join(OLD_MAIN_CONFIG)
}

pub fn legacy_main_config(config_dir: &Path) -> PathBuf {
    config_dir.join("user_configs").join(OLD_MAIN_CONFIG)
}

pub fn cursor_config(config_dir: &Path) -> PathBuf {
    config_dir.join(CURSOR_CONFIG)
}

pub fn shared_cursor_config_dir(config_dir: &Path) -> PathBuf {
    if config_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "yazelix")
    {
        return config_dir
            .parent()
            .map(|parent| parent.join(SHARED_CURSOR_CONFIG_DIR))
            .unwrap_or_else(|| PathBuf::from(SHARED_CURSOR_CONFIG_DIR));
    }
    config_dir.join(SHARED_CURSOR_CONFIG_DIR)
}

pub fn shared_cursor_config(config_dir: &Path) -> PathBuf {
    shared_cursor_config_dir(config_dir).join(SHARED_CURSOR_SETTINGS_CONFIG)
}

pub fn is_shared_cursor_config_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == SHARED_CURSOR_SETTINGS_CONFIG)
        && path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == SHARED_CURSOR_CONFIG_DIR)
}

pub fn legacy_cursor_config(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("yazelix_ghostty_cursors.toml")
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

pub fn yazi_config_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_CONFIG_DIR)
}

pub fn yazi_config(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_CONFIG)
}

pub fn flat_yazi_config(config_dir: &Path) -> PathBuf {
    config_dir.join("yazi.toml")
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

pub fn flat_yazi_keymap(config_dir: &Path) -> PathBuf {
    config_dir.join("yazi_keymap.toml")
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

pub fn flat_yazi_init(config_dir: &Path) -> PathBuf {
    config_dir.join("yazi_init.lua")
}

pub fn yazi_package(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_PACKAGE)
}

pub fn yazi_plugins_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_PLUGINS_DIR)
}

pub fn flat_yazi_plugins_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("yazi_plugins")
}

pub fn yazi_flavors_dir(config_dir: &Path) -> PathBuf {
    config_dir.join(YAZI_FLAVORS_DIR)
}

pub fn legacy_yazi_init(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("yazi")
        .join("init.lua")
}

pub fn terminal_config(config_dir: &Path, terminal: &str) -> Option<PathBuf> {
    match terminal {
        "ghostty" => Some(config_dir.join(TERMINAL_GHOSTTY_CONFIG)),
        "kitty" => Some(config_dir.join(TERMINAL_KITTY_CONFIG)),
        "alacritty" => Some(config_dir.join(TERMINAL_ALACRITTY_CONFIG)),
        "foot" => Some(config_dir.join(TERMINAL_FOOT_CONFIG)),
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
        "bash" => Some(config_dir.join(SHELL_BASH_HOOK)),
        "zsh" => Some(config_dir.join(SHELL_ZSH_HOOK)),
        "fish" => Some(config_dir.join(SHELL_FISH_HOOK)),
        "nu" => Some(config_dir.join(SHELL_NU_HOOK)),
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
        "Could not inspect an old Yazelix config path",
        "Fix permissions or move the reported file manually, then retry.",
        path.display().to_string(),
        source,
    )
}

fn optional_symlink_metadata(path: &Path) -> Result<Option<fs::Metadata>, CoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io_err("stat_old_config_path", path, source)),
    }
}

pub fn resolve_current_config_file(
    current_path: &Path,
    legacy_path: &Path,
    label: &str,
) -> Result<PathBuf, CoreError> {
    if optional_symlink_metadata(legacy_path)?.is_some() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "legacy_config_surface",
            format!(
                "Yazelix found an old {label} config surface at {}.",
                legacy_path.display()
            ),
            "Move the old user_configs path aside or import it explicitly; Yazelix no longer relocates legacy config files automatically.",
            json!({
                "label": label,
                "current_path": current_path.display().to_string(),
                "legacy_path": legacy_path.display().to_string(),
            }),
        ));
    }

    Ok(current_path.to_path_buf())
}
