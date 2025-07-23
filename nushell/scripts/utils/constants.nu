#!/usr/bin/env nu
# Yazelix Constants
# Centralized constants for yazelix configuration and management

# Configuration section markers
export const YAZELIX_START_MARKER = "# YAZELIX START - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER = "# YAZELIX END - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_REGENERATE_COMMENT = "# delete this whole section to re-generate the config, if needed"

# Version information
export const YAZELIX_VERSION = "v7.5"
export const YAZELIX_DESCRIPTION = "Yazi + Zellij + Helix integrated terminal environment"

# Default configuration values
export const DEFAULT_SHELL = "nu"
export const DEFAULT_TERMINAL = "ghostty"
export const DEFAULT_HELIX_MODE = "release"

# File paths and directories - XDG-compliant separation
# Static configuration (potentially managed by home-manager)
export const YAZELIX_CONFIG_DIR = "~/.config/yazelix"
# Runtime state (always writable, never managed by home-manager)
export const YAZELIX_STATE_DIR = "~/.local/share/yazelix"
export const YAZELIX_LOGS_DIR = "~/.local/share/yazelix/logs"
export const YAZELIX_INITIALIZERS_DIR = "~/.local/share/yazelix/initializers"
export const YAZELIX_CACHE_DIR = "~/.local/share/yazelix/cache"

# Shell-specific initializer directories (in state, not config)
export const SHELL_INITIALIZER_DIRS = {
    bash: "~/.local/share/yazelix/initializers/bash"
    nushell: "~/.local/share/yazelix/initializers/nushell"
    fish: "~/.local/share/yazelix/initializers/fish"
    zsh: "~/.local/share/yazelix/initializers/zsh"
}

# Shell configuration files
export const SHELL_CONFIGS = {
    bash: "~/.bashrc"
    nushell: "~/.config/nushell/config.nu"
    fish: "~/.config/fish/config.fish"
    zsh: "~/.zshrc"
}

# Yazelix configuration files
export const YAZELIX_CONFIG_FILES = {
    bash: "~/.config/yazelix/shells/bash/yazelix_bash_config.sh"
    nushell: "~/.config/yazelix/nushell/config/config.nu"
    fish: "~/.config/yazelix/shells/fish/yazelix_fish_config.fish"
    zsh: "~/.config/yazelix/shells/zsh/yazelix_zsh_config.zsh"
}

# Environment variables
export const YAZELIX_ENV_VARS = {
    YAZELIX_DIR: "~/.config/yazelix"
    YAZELIX_DEFAULT_SHELL: "nu"
    YAZELIX_PREFERRED_TERMINAL: "ghostty"
    YAZELIX_HELIX_MODE: "release"
    YAZI_CONFIG_HOME: "~/.config/yazelix/configs/yazi"
    ZELLIJ_DEFAULT_LAYOUT: "yazelix"
}

# Get the full start comment with regeneration instruction
export def get_yazelix_start_comment [] {
    $YAZELIX_START_MARKER + "\n" + $YAZELIX_REGENERATE_COMMENT
}

# Environment detection functions
export def is_read_only_config [] {
    let config_dir = ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME)
    try {
        # Test write access by trying to create a temporary file
        let test_file = $"($config_dir)/.yazelix_write_test"
        touch $test_file
        rm $test_file
        false
    } catch {
        true
    }
}

export def is_home_manager_environment [] {
    # Check for common home-manager indicators
    let home_manager_indicators = [
        ($env.HOME + "/.local/state/nix/profiles/home-manager")
        ($env.HOME + "/.nix-profile/etc/profile.d/hm-session-vars.sh")
        $env.NIX_PROFILE?
    ]
    $home_manager_indicators | any { |path| $path | path exists }
}

export def detect_environment [] {
    let is_readonly = (is_read_only_config)
    let is_hm = (is_home_manager_environment)

    {
        read_only_config: $is_readonly
        home_manager: $is_hm
        environment_type: (
            if $is_hm { "home-manager" }
            else if $is_readonly { "read-only" }
            else { "standard" }
        )
    }
}

# Get the complete yazelix section content for a shell
export def get_yazelix_section_content [shell: string, yazelix_dir: string] {
    let config_file = $YAZELIX_CONFIG_FILES | get $shell
    let source_line = if $shell in ["bash", "zsh", "fish"] {
        # Use $HOME for POSIX shells
        let home_file = ($config_file | str replace "~" "$HOME")
        $"source \"($home_file)\""
    } else {
        $"source \"($config_file)\""
    }

    (get_yazelix_start_comment) + "\n" + $source_line + "\n" + $YAZELIX_END_MARKER
}