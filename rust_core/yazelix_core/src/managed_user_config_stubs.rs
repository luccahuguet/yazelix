// Test lane: default
//! User-owned managed config surface scaffolding.

use crate::bridge::CoreError;
use crate::user_config_paths;
use std::fs;
use std::path::Path;

const BASH_HOOK: &str = r#"# Yazelix-managed Bash hook
# Add Bash-only commands for Yazelix sessions here
"#;

const ZSH_HOOK: &str = r#"# Yazelix-managed Zsh hook
# Add Zsh-only commands for Yazelix sessions here
"#;

const FISH_HOOK: &str = r#"# Yazelix-managed Fish hook
# Add Fish-only commands for Yazelix sessions here
"#;

const NU_HOOK: &str = r#"# Yazelix-managed Nushell hook
# Add Nushell-only commands for Yazelix sessions here
"#;

const XONSH_HOOK: &str = r#"# Yazelix-managed Xonsh hook
# Source this file from ~/.xonshrc or ~/.config/xonsh/rc.xsh
# Add Xonsh-only commands for Yazelix sessions below

import os as _yazelix_os

_yazelix_init = _yazelix_os.path.expanduser("~/.local/share/yazelix/initializers/xonsh/yazelix_init.xsh")
if _yazelix_os.path.isfile(_yazelix_init) and _yazelix_os.path.getsize(_yazelix_init) > 0:
    execx(open(_yazelix_init, encoding="utf-8").read(), "exec", __xonsh__.ctx, filename=_yazelix_init)

del _yazelix_init
del _yazelix_os
"#;

const HELIX_STEEL_PLUGINS_README: &str = r#"# Yazelix-managed Helix Steel plugins

Place custom Steel plugin source files below this directory, then declare them in
helix.steel_plugins.extra inside ~/.config/yazelix/config.toml.

Bundled Yazelix Steel plugins live in the packaged runtime and are selected with
helix.steel_plugins.enabled.
"#;

pub(crate) fn ensure_helix_surface_stub(config_dir: &Path) -> Result<(), CoreError> {
    write_stub_if_missing(
        &config_dir
            .join("helix")
            .join("steel_plugins")
            .join("README.md"),
        HELIX_STEEL_PLUGINS_README,
    )
}

pub(crate) fn ensure_yazi_surface_stub(_config_dir: &Path) -> Result<(), CoreError> {
    Ok(())
}

pub(crate) fn ensure_shell_hook_stubs(
    config_dir: &Path,
    shells_to_configure: &[String],
) -> Result<(), CoreError> {
    for shell in shells_to_configure {
        match shell.as_str() {
            "bash" => ensure_stub_with_legacy(config_dir, shell, BASH_HOOK)?,
            "zsh" => ensure_stub_with_legacy(config_dir, shell, ZSH_HOOK)?,
            "fish" => ensure_stub_with_legacy(config_dir, shell, FISH_HOOK)?,
            "nu" => ensure_stub_with_legacy(config_dir, shell, NU_HOOK)?,
            "xonsh" => ensure_current_shell_stub(config_dir, shell, XONSH_HOOK)?,
            _ => {}
        }
    }

    Ok(())
}

fn ensure_stub_with_legacy(config_dir: &Path, shell: &str, content: &str) -> Result<(), CoreError> {
    let current = user_config_paths::shell_hook(config_dir, shell).expect("supported shell");
    let legacy = user_config_paths::legacy_shell_hook(config_dir, shell).expect("supported shell");
    let path = user_config_paths::resolve_current_config_file(
        &current,
        &legacy,
        &format!("Yazelix {shell} shell hook"),
    )?;
    write_stub_if_missing(&path, content)
}

fn ensure_current_shell_stub(
    config_dir: &Path,
    shell: &str,
    content: &str,
) -> Result<(), CoreError> {
    let current = user_config_paths::shell_hook(config_dir, shell).expect("supported shell");
    write_stub_if_missing(&current, content)
}

fn write_stub_if_missing(path: &Path, content: &str) -> Result<(), CoreError> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "create_user_config_stub_dir",
                "Could not create Yazelix managed user config stub directory",
                "Check permissions for ~/.config/yazelix and retry.",
                parent.to_string_lossy(),
                source,
            )
        })?;
    }

    fs::write(path, content).map_err(|source| {
        CoreError::io(
            "write_user_config_stub",
            "Could not write Yazelix managed user config stub",
            "Check permissions for ~/.config/yazelix and retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Defends: fallback-sensitive surfaces do not create live flat files that would change source selection.
    #[test]
    fn readme_stubs_preserve_behavior_owned_files_absent() {
        let config = tempdir().expect("config");

        ensure_helix_surface_stub(config.path()).unwrap();

        assert!(!config.path().join("helix.toml").exists());
        assert!(!config.path().join("helix/config.toml").exists());
        assert!(
            fs::read_to_string(config.path().join("helix/steel_plugins/README.md"))
                .unwrap()
                .contains("helix.steel_plugins.extra")
        );
    }

    // Defends: shell hook scaffolding follows the configured shell set instead of dumping every supported shell hook.
    #[test]
    fn shell_hook_stubs_are_limited_to_requested_shells() {
        let config = tempdir().expect("config");

        ensure_shell_hook_stubs(
            config.path(),
            &["bash".to_string(), "nu".to_string(), "xonsh".to_string()],
        )
        .unwrap();

        assert!(config.path().join("shell_bash.sh").exists());
        assert!(config.path().join("shell_nu.nu").exists());
        assert!(
            fs::read_to_string(config.path().join("shell_xonsh.xsh"))
                .unwrap()
                .contains("yazelix_init.xsh")
        );
        assert!(!config.path().join("shell_fish.fish").exists());
        assert!(!config.path().join("shell_zsh.zsh").exists());
    }
}
