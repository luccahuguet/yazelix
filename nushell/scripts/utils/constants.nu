#!/usr/bin/env nu
# Yazelix Constants
# Centralized constants for yazelix configuration and management

# Configuration section markers
export const YAZELIX_START_MARKER = "# YAZELIX START - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER = "# YAZELIX END - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_REGENERATE_COMMENT = "# delete this whole section to re-generate the config, if needed"

# Version information
export const YAZELIX_VERSION = "v7"
export const YAZELIX_DESCRIPTION = "Yazi + Zellij + Helix integrated terminal environment"

# Default configuration values
export const DEFAULT_SHELL = "nu"
export const DEFAULT_TERMINAL = "wezterm"
export const DEFAULT_HELIX_MODE = "release"

# File paths and directories
export const YAZELIX_CONFIG_DIR = "~/.config/yazelix"
export const YAZELIX_LOGS_DIR = "~/.config/yazelix/logs"
export const YAZELIX_INITIALIZERS_DIR = "~/.config/yazelix/initializers"

# Shell configuration files
export const SHELL_CONFIGS = {
    bash: "~/.bashrc"
    nushell: "~/.config/nushell/config.nu"
    fish: "~/.config/fish/config.fish"
    zsh: "~/.zshrc"
}

# Yazelix configuration files
export const YAZELIX_CONFIG_FILES = {
    bash: "~/.config/yazelix/bash/yazelix_bash_config.sh"
    nushell: "~/.config/yazelix/nushell/config/config.nu"
    fish: "~/.config/yazelix/fish/yazelix_fish_config.fish"
    zsh: "~/.config/yazelix/zsh/yazelix_zsh_config.zsh"
}

# Environment variables
export const YAZELIX_ENV_VARS = {
    YAZELIX_DIR: "~/.config/yazelix"
    YAZELIX_DEFAULT_SHELL: "nu"
    YAZELIX_PREFERRED_TERMINAL: "wezterm"
    YAZELIX_HELIX_MODE: "release"
    YAZI_CONFIG_HOME: "~/.config/yazelix/yazi"
    ZELLIJ_DEFAULT_LAYOUT: "yazelix"
}

# Get the full start comment with regeneration instruction
export def get_yazelix_start_comment [] {
    $YAZELIX_START_MARKER + "\n" + $YAZELIX_REGENERATE_COMMENT
}

# Get the complete yazelix section content for a shell
export def get_yazelix_section_content [shell: string, yazelix_dir: string] {
    let config_file = $YAZELIX_CONFIG_FILES | get $shell
    let source_line = $"source \"($config_file)\""

    (get_yazelix_start_comment) + "\n" + $source_line + "\n" + $YAZELIX_END_MARKER
}