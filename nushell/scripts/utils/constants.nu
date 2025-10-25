#!/usr/bin/env nu
# Yazelix Constants
# Centralized constants for yazelix configuration and management

# Configuration section markers
# v1 markers (for detecting old hooks)
export const YAZELIX_START_MARKER_V1 = "# YAZELIX START - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER_V1 = "# YAZELIX END - Yazelix managed configuration (do not modify this comment)"
# v2 markers (current version with conditional loading)
export const YAZELIX_START_MARKER = "# YAZELIX START v2 - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER = "# YAZELIX END v2 - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_REGENERATE_COMMENT = "# delete this whole section to re-generate the config, if needed"

# Version information
export const YAZELIX_VERSION = "v10"
export const YAZELIX_DESCRIPTION = "Yazi + Zellij + Helix integrated terminal environment"

# Default configuration values
export const DEFAULT_SHELL = "nu"
export const DEFAULT_TERMINAL = "ghostty"
export const DEFAULT_HELIX_MODE = "release"

# Supported terminal emulators (fallback priority order)
export const SUPPORTED_TERMINALS = ["ghostty", "wezterm", "kitty", "alacritty", "foot"]

# Terminal configuration paths
export const TERMINAL_CONFIG_PATHS = {
    ghostty: {
        yazelix: "~/.local/share/yazelix/configs/terminal_emulators/ghostty/config"
        user: "~/.config/ghostty/config"
    }
    kitty: {
        yazelix: "~/.local/share/yazelix/configs/terminal_emulators/kitty/kitty.conf"
        user: "~/.config/kitty/kitty.conf"
    }
    wezterm: {
        yazelix: "~/.local/share/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua"
        user_main: "~/.wezterm.lua"
        user_alt: "~/.config/wezterm/wezterm.lua"
    }
    alacritty: {
        yazelix: "~/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty.toml"
        user: "~/.config/alacritty/alacritty.toml"
    }
    foot: {
        yazelix: "~/.local/share/yazelix/configs/terminal_emulators/foot/foot.ini"
        user: "~/.config/foot/foot.ini"
    }
}

# Terminal display names and wrappers
export const TERMINAL_METADATA = {
    ghostty: {name: "Ghostty", wrapper: "yazelix-ghostty"}
    kitty: {name: "Kitty", wrapper: "yazelix-kitty"}
    wezterm: {name: "WezTerm", wrapper: "yazelix-wezterm"}
    alacritty: {name: "Alacritty", wrapper: "yazelix-alacritty"}
    foot: {name: "Foot", wrapper: "yazelix-foot"}
}

# Common terminal configuration values
export const YAZELIX_WINDOW_CLASS = "com.yazelix.Yazelix"
export const YAZELIX_X11_INSTANCE = "yazelix"
export const YAZELIX_SHELL_COMMAND = "bash -l -c 'nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu'"
export const YAZELIX_THEME = "Abernathy"

# Transparency opacity mapping
export const TRANSPARENCY_VALUES = {
    none: "1.0"
    low: "0.95"
    medium: "0.9"
    high: "0.8"
}

# Cursor trail shader paths for Ghostty
export const CURSOR_TRAIL_SHADERS = {
    blaze: "./shaders/cursor_smear.glsl"
    snow: "./shaders/cursor_trail_white.glsl"
    cosmic: "./shaders/cursor_trail_cosmic.glsl"
    ocean: "./shaders/cursor_trail_ocean.glsl"
    forest: "./shaders/cursor_trail_forest.glsl"
    sunset: "./shaders/cursor_trail_sunset.glsl"
    neon: "./shaders/cursor_trail_neon.glsl"
    party: "./shaders/cursor_trail_party.glsl"
    prism: "./shaders/cursor_trail_prism.glsl"
    orchid: "./shaders/cursor_trail_orchid.glsl"
    reef: "./shaders/cursor_trail_reef.glsl"
    none: ""
}

# Common config sections
export const SHELL_ARGS_BASH = '["bash", "-l", "-c", "nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu"]'
export const SHELL_ARGS_STRING = '["-l", "-c", "nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu"]'

# Font configurations
export const FONT_FIRACODE = "FiraCode Nerd Font"

# Cursor trail presets documentation
export const CURSOR_TRAIL_PRESETS_COMMENT = "# Alternative presets (uncomment to try)
# snow:  custom-shader = ./shaders/cursor_trail_white.glsl
# blaze (fire):  custom-shader = ./shaders/cursor_smear.glsl
# cosmic (violet): custom-shader = ./shaders/cursor_trail_cosmic.glsl
# ocean (blue):  custom-shader = ./shaders/cursor_trail_ocean.glsl
# forest (green): custom-shader = ./shaders/cursor_trail_forest.glsl
# sunset (orange/pink): custom-shader = ./shaders/cursor_trail_sunset.glsl
# neon (cyan/magenta): custom-shader = ./shaders/cursor_trail_neon.glsl
# party (multi-hue): custom-shader = ./shaders/cursor_trail_party.glsl
# prism (purple/magenta): custom-shader = ./shaders/cursor_trail_prism.glsl
# orchid (tri-color gradient): custom-shader = ./shaders/cursor_trail_orchid.glsl
# reef (aqua tri-color): custom-shader = ./shaders/cursor_trail_reef.glsl"

# Ghostty config header template
export const GHOSTTY_CONFIG_HEADER = "# This is the configuration file for Ghostty.
#
# This template file has been automatically created at the following
# path since Ghostty couldn't find any existing config files on your system:
#
#   /home/lucca/.config/ghostty/config
#
# The template does not set any default options, since Ghostty ships
# with sensible defaults for all options. Users should only need to set
# options that they want to change from the default.
#
# Run `ghostty +show-config --default --docs` to view a list of
# all available config options and their default values.
#
# Additionally, each config option is also explained in detail
# on Ghostty's website, at https://ghostty.org/docs/config.

# Config syntax crash course
# ==========================
# # The config file consists of simple key-value pairs,
# # separated by equals signs.
# font-family = Iosevka
# window-padding-x = 2
#
# # Spacing around the equals sign does not matter.
# # All of these are identical:
# key=value
# key= value
# key =value
# key = value
#
# # Any line beginning with a # is a comment. It's not possible to put
# # a comment after a config option, since it would be interpreted as a
# # part of the value. For example, this will have a value of \"#123abc\":
# background = #123abc
#
# # Empty values are used to reset config keys to default.
# key =
#
# # Some config options have unique syntaxes for their value,
# # which is explained in the docs for that config option.
# # Just for example:
# resize-overlay-duration = 4s 200ms
"

# File paths and directories - XDG-compliant separation
# Static configuration (potentially managed by home-manager)
export const YAZELIX_CONFIG_DIR = "~/.config/yazelix"
# Runtime state (always writable, never managed by home-manager)
export const YAZELIX_STATE_DIR = "~/.local/share/yazelix"
export const YAZELIX_LOGS_DIR = "~/.local/share/yazelix/logs"
export const YAZELIX_INITIALIZERS_DIR = "~/.local/share/yazelix/initializers"
export const YAZELIX_CACHE_DIR = "~/.local/share/yazelix/cache"
export const YAZELIX_GENERATED_CONFIGS_DIR = "~/.local/share/yazelix/configs"

# Zellij configuration paths
export const ZELLIJ_CONFIG_PATHS = {
    # Source configuration files (in config dir, tracked by git)
    yazelix_overrides: "configs/zellij/yazelix_overrides.kdl"
    user_config: "configs/zellij/personal/user_config.kdl"
    layouts_dir: "configs/zellij/layouts"
    
    # Generated/merged configuration (in state dir, not tracked)
    merged_config_dir: "~/.local/share/yazelix/configs/zellij"
    merged_config: "~/.local/share/yazelix/configs/zellij/config.kdl"
}

# Yazi configuration paths
export const YAZI_CONFIG_PATHS = {
    # Generated/merged configuration (in state dir, not tracked)
    merged_config_dir: "~/.local/share/yazelix/configs/yazi"
}

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
    YAZELIX_PREFERRED_TERMINAL: "wezterm"
    YAZELIX_HELIX_MODE: "release"
    YAZI_CONFIG_HOME: "~/.local/share/yazelix/configs/yazi"
    ZELLIJ_DEFAULT_LAYOUT: "yzx_side"
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
    $home_manager_indicators | where ($it != null) | any { |path| $path | path exists }
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

    # Generate shell-specific conditional loading + yzx availability
    let section_body = if $shell == "bash" or $shell == "zsh" {
        let home_file = ($config_file | str replace "~" "$HOME")
        let yzx_path = $"($yazelix_dir)/shells/bash/yzx"
        [
            $"if [ -n \"$IN_YAZELIX_SHELL\" ]; then"
            $"  source \"($home_file)\""
            "fi"
            $"alias yzx=\"($yzx_path)\""
        ] | str join "\n"
    } else if $shell == "fish" {
        let home_file = ($config_file | str replace "~" "$HOME")
        let yzx_path = $"($yazelix_dir)/shells/bash/yzx"
        [
            "if test -n \"$IN_YAZELIX_SHELL\""
            $"  source \"($home_file)\""
            "end"
            $"alias yzx=\"($yzx_path)\""
        ] | str join "\n"
    } else {
        # Nushell
        [
            "if ($env.IN_YAZELIX_SHELL? == \"true\") {"
            $"  source \"($config_file)\""
            "}"
            "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *"
        ] | str join "\n"
    }

    (get_yazelix_start_comment) + "\n" + $section_body + "\n" + $YAZELIX_END_MARKER
}
