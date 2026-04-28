// Test lane: default
//! User-owned managed config surface scaffolding.

use crate::bridge::CoreError;
use std::fs;
use std::path::Path;

const ZELLIJ_README: &str = r#"# Yazelix-managed Zellij overrides

Create `config.kdl` here when you want Zellij settings that apply only inside Yazelix

Yazelix intentionally does not create `config.kdl` automatically, so native `~/.config/zellij/config.kdl` fallback keeps working until you choose this managed surface
"#;

const HELIX_README: &str = r#"# Yazelix-managed Helix overrides

Create `config.toml` here when you want Helix settings that apply only inside Yazelix

Yazelix intentionally does not create `config.toml` automatically, so `yzx import helix` can still detect native Helix config before you choose this managed surface
"#;

const YAZI_README: &str = r#"# Yazelix-managed Yazi overrides

Create `yazi.toml`, `keymap.toml`, or `init.lua` here for Yazi settings that apply only inside Yazelix

Yazelix merges these files into the generated Yazi runtime config when they exist
"#;

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

const GHOSTTY_OVERRIDE: &str = r#"# Yazelix-managed Ghostty overrides
# Add terminal-native Ghostty settings for Yazelix windows here
"#;

const KITTY_OVERRIDE: &str = r#"# Yazelix-managed Kitty overrides
# Add terminal-native Kitty settings for Yazelix windows here
"#;

const ALACRITTY_OVERRIDE: &str = r#"# Yazelix-managed Alacritty overrides
# Add terminal-native Alacritty settings for Yazelix windows here
"#;

pub(crate) fn ensure_zellij_surface_stub(config_dir: &Path) -> Result<(), CoreError> {
    write_stub_if_missing(
        &config_dir
            .join("user_configs")
            .join("zellij")
            .join("README.md"),
        ZELLIJ_README,
    )
}

pub(crate) fn ensure_helix_surface_stub(config_dir: &Path) -> Result<(), CoreError> {
    write_stub_if_missing(
        &config_dir
            .join("user_configs")
            .join("helix")
            .join("README.md"),
        HELIX_README,
    )
}

pub(crate) fn ensure_yazi_surface_stub(config_dir: &Path) -> Result<(), CoreError> {
    write_stub_if_missing(
        &config_dir
            .join("user_configs")
            .join("yazi")
            .join("README.md"),
        YAZI_README,
    )
}

pub(crate) fn ensure_shell_hook_stubs(
    config_dir: &Path,
    shells_to_configure: &[String],
) -> Result<(), CoreError> {
    for shell in shells_to_configure {
        match shell.as_str() {
            "bash" => write_stub_if_missing(
                &config_dir
                    .join("user_configs")
                    .join("shells")
                    .join("bash.sh"),
                BASH_HOOK,
            )?,
            "zsh" => write_stub_if_missing(
                &config_dir
                    .join("user_configs")
                    .join("shells")
                    .join("zsh.zsh"),
                ZSH_HOOK,
            )?,
            "fish" => write_stub_if_missing(
                &config_dir
                    .join("user_configs")
                    .join("shells")
                    .join("fish.fish"),
                FISH_HOOK,
            )?,
            "nu" => write_stub_if_missing(
                &config_dir.join("user_configs").join("shells").join("nu.nu"),
                NU_HOOK,
            )?,
            _ => {}
        }
    }

    Ok(())
}

pub(crate) fn ensure_terminal_override_stubs(
    config_dir: &Path,
    terminals: &[String],
) -> Result<(), CoreError> {
    for terminal in terminals {
        match terminal.as_str() {
            "ghostty" => write_stub_if_missing(
                &config_dir
                    .join("user_configs")
                    .join("terminal")
                    .join("ghostty"),
                GHOSTTY_OVERRIDE,
            )?,
            "kitty" => write_stub_if_missing(
                &config_dir
                    .join("user_configs")
                    .join("terminal")
                    .join("kitty.conf"),
                KITTY_OVERRIDE,
            )?,
            "alacritty" => write_stub_if_missing(
                &config_dir
                    .join("user_configs")
                    .join("terminal")
                    .join("alacritty.toml"),
                ALACRITTY_OVERRIDE,
            )?,
            _ => {}
        }
    }

    Ok(())
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
                "Check permissions for ~/.config/yazelix/user_configs and retry.",
                parent.to_string_lossy(),
                source,
            )
        })?;
    }

    fs::write(path, content).map_err(|source| {
        CoreError::io(
            "write_user_config_stub",
            "Could not write Yazelix managed user config stub",
            "Check permissions for ~/.config/yazelix/user_configs and retry.",
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

    // Defends: discovery stubs for fallback-sensitive surfaces do not create live config files that would change source selection.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn readme_stubs_preserve_zellij_and_helix_behavior_owned_files_absent() {
        let config = tempdir().expect("config");

        ensure_zellij_surface_stub(config.path()).unwrap();
        ensure_helix_surface_stub(config.path()).unwrap();

        assert!(config.path().join("user_configs/zellij/README.md").exists());
        assert!(config.path().join("user_configs/helix/README.md").exists());
        assert!(
            !config
                .path()
                .join("user_configs/zellij/config.kdl")
                .exists()
        );
        assert!(
            !config
                .path()
                .join("user_configs/helix/config.toml")
                .exists()
        );
    }

    // Defends: shell hook scaffolding follows the configured shell set instead of dumping every supported shell hook.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn shell_hook_stubs_are_limited_to_requested_shells() {
        let config = tempdir().expect("config");

        ensure_shell_hook_stubs(config.path(), &["bash".to_string(), "nu".to_string()]).unwrap();

        let shells = config.path().join("user_configs/shells");
        assert!(shells.join("bash.sh").exists());
        assert!(shells.join("nu.nu").exists());
        assert!(!shells.join("fish.fish").exists());
        assert!(!shells.join("zsh.zsh").exists());
    }

    // Defends: terminal override scaffolding only creates files for terminals with a live managed override contract.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn terminal_override_stubs_follow_supported_override_surfaces() {
        let config = tempdir().expect("config");

        ensure_terminal_override_stubs(
            config.path(),
            &[
                "ghostty".to_string(),
                "kitty".to_string(),
                "wezterm".to_string(),
                "alacritty".to_string(),
            ],
        )
        .unwrap();

        let terminal = config.path().join("user_configs/terminal");
        assert!(terminal.join("ghostty").exists());
        assert!(terminal.join("kitty.conf").exists());
        assert!(terminal.join("alacritty.toml").exists());
        assert!(!terminal.join("wezterm.lua").exists());
        assert!(
            fs::read_to_string(terminal.join("ghostty"))
                .unwrap()
                .contains("Yazelix-managed Ghostty overrides")
        );
    }
}
