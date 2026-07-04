use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    catalog::*,
    common::*,
    root_config::ensure_config_file_at,
    zellij_sidecar::{ZellijSidecar, render_zellij_sidecar},
};

pub(crate) struct ConfigPaths {
    pub(crate) root: PathBuf,
    pub(crate) mars: PathBuf,
    pub(crate) zellij: PathBuf,
    pub(crate) helix_dir: PathBuf,
    pub(crate) helix_config: PathBuf,
    pub(crate) helix_languages: PathBuf,
    pub(crate) helix_module: PathBuf,
    pub(crate) helix_init: PathBuf,
    pub(crate) nu_env: PathBuf,
    pub(crate) nu_config: PathBuf,
    pub(crate) starship: PathBuf,
    pub(crate) yazi_init: PathBuf,
    pub(crate) yazi_keymap: PathBuf,
    pub(crate) zellij_plugins: PathBuf,
}
pub(crate) fn ensure_config_sources() -> Result<ConfigPaths> {
    let paths = config_paths()?;
    ensure_config_file_at(paths.root.clone())?;
    ensure_plain_config_file_at(&paths.mars, DEFAULT_MARS_CONFIG_TOML)?;
    ensure_plain_config_file_at(
        &paths.zellij,
        &render_zellij_sidecar(&ZellijSidecar::default()),
    )?;
    ensure_plain_config_file_at(&paths.starship, DEFAULT_STARSHIP_CONFIG_TOML)?;
    Ok(paths)
}
pub(crate) fn ensure_plain_config_file_at(path: &Path, default: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    atomic_write(path, default)
}
pub(crate) fn config_paths() -> Result<ConfigPaths> {
    let home = config_home()?;
    Ok(ConfigPaths {
        root: home.join("config.toml"),
        mars: home.join("mars/config.toml"),
        zellij: home.join("zellij/config.kdl"),
        helix_dir: home.join("helix"),
        helix_config: home.join("helix/config.toml"),
        helix_languages: home.join("helix/languages.toml"),
        helix_module: home.join("helix/helix.scm"),
        helix_init: home.join("helix/init.scm"),
        nu_env: home.join("nu/env.nu"),
        nu_config: home.join("nu/config.nu"),
        starship: home.join("starship.toml"),
        yazi_init: home.join("yazi/init.lua"),
        yazi_keymap: home.join("yazi/keymap.toml"),
        zellij_plugins: home.join("zellij/plugins.kdl"),
    })
}
pub(crate) fn config_home() -> Result<PathBuf> {
    if let Some(path) = env::var_os("YAZELIX_NEXT_CONFIG_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("yazelix-next"));
    }
    let home = env::var_os("HOME").ok_or_else(|| error("HOME is required"))?;
    Ok(PathBuf::from(home).join(".config/yazelix-next"))
}
