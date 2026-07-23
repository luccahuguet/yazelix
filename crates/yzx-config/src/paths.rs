use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::common::*;
use yazelix_cursors::initialize_cursor_config;

pub(crate) struct ConfigPaths {
    pub(crate) store_root: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) cursors: PathBuf,
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
    pub(crate) yazi_config: PathBuf,
    pub(crate) yazi_init: PathBuf,
    pub(crate) yazi_keymap: PathBuf,
    pub(crate) yazi_package: PathBuf,
    pub(crate) yazi_theme: PathBuf,
    pub(crate) packaged_yazi: PathBuf,
    pub(crate) zellij_plugins: PathBuf,
}
impl ConfigPaths {
    fn home_manager_files(&self) -> [(&Path, &'static str); 16] {
        [
            (&self.root, "settings"),
            (&self.cursors, "cursors"),
            (&self.mars, "mars"),
            (&self.zellij, "zellij"),
            (&self.starship, "starship"),
            (&self.helix_config, "helix.config"),
            (&self.helix_languages, "helix.languages"),
            (&self.helix_module, "helix.module"),
            (&self.helix_init, "helix.init"),
            (&self.yazi_config, "yazi.config"),
            (&self.yazi_init, "yazi.init"),
            (&self.yazi_keymap, "yazi.keymap"),
            (&self.yazi_package, "yazi.package"),
            (&self.yazi_theme, "yazi.theme"),
            (&self.nu_env, "nu.env"),
            (&self.nu_config, "nu.config"),
        ]
    }

    pub(crate) fn is_home_manager_owned(&self, path: &Path) -> bool {
        self.home_manager_option(path).is_some()
            && resolved_target(path).is_some_and(|path| path.starts_with(&self.store_root))
    }

    pub(crate) fn home_manager_guidance(&self, path: &Path) -> Option<String> {
        self.is_home_manager_owned(path).then(|| {
            format!(
                "Managed by Home Manager through `programs.yazelix.config.{}`; edit that option and run your normal Home Manager switch.",
                self.home_manager_option(path).expect("mapped path")
            )
        })
    }

    pub(crate) fn reject_mutation(&self, path: &Path, source_id: &str) -> Result<()> {
        if let Some(guidance) = self.home_manager_guidance(path) {
            return Err(error(guidance));
        }
        reject_read_only_source(path, source_id)
    }

    fn home_manager_option(&self, path: &Path) -> Option<&'static str> {
        self.home_manager_files()
            .into_iter()
            .find_map(|(candidate, option)| (candidate == path).then_some(option))
    }
}
pub(crate) fn ensure_config_sources() -> Result<ConfigPaths> {
    ensure_config_sources_at(config_paths()?)
}
pub(crate) fn ensure_config_sources_at(paths: ConfigPaths) -> Result<ConfigPaths> {
    initialize_cursor_config(&paths.cursors)?;
    Ok(paths)
}
pub(crate) fn config_paths() -> Result<ConfigPaths> {
    let home = config_home()?;
    Ok(ConfigPaths {
        store_root: option_env!("YAZELIX_NIX_STORE_ROOT")
            .map(PathBuf::from)
            .ok_or_else(|| error("yzx-config is missing its packaged Nix store root"))?,
        root: home.join("config.toml"),
        cursors: home.join("cursors.toml"),
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
        yazi_config: home.join("yazi/yazi.toml"),
        yazi_init: home.join("yazi/init.lua"),
        yazi_keymap: home.join("yazi/keymap.toml"),
        yazi_package: home.join("yazi/package.toml"),
        yazi_theme: home.join("yazi/theme.toml"),
        packaged_yazi: option_env!("YAZELIX_PACKAGED_YAZI")
            .map(PathBuf::from)
            .ok_or_else(|| error("yzx-config is missing its packaged Yazi config"))?,
        zellij_plugins: home.join("zellij/plugins.kdl"),
    })
}
fn resolved_target(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok().or_else(|| {
        let target = fs::read_link(path).ok()?;
        Some(if target.is_absolute() {
            target
        } else {
            path.parent()?.join(target)
        })
    })
}
pub(crate) fn config_home() -> Result<PathBuf> {
    if let Some(path) = env::var_os("YAZELIX_CONFIG_HOME").filter(|path| !path.is_empty()) {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_CONFIG_HOME").filter(|path| !path.is_empty()) {
        return Ok(PathBuf::from(path).join("yazelix"));
    }
    let home = env::var_os("HOME")
        .filter(|path| !path.is_empty())
        .ok_or_else(|| error("HOME is required"))?;
    Ok(PathBuf::from(home).join(".config/yazelix"))
}
