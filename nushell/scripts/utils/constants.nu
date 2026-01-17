#!/usr/bin/env nu
# Yazelix Constants
# Pure constant definitions with no imports or function dependencies

# ============================================================================
# VERSION INFORMATION
# ============================================================================

export const YAZELIX_VERSION = "v11"
export const YAZELIX_DESCRIPTION = "Yazi + Zellij + Helix integrated terminal environment"

# ============================================================================
# CONFIGURATION SECTION MARKERS (Shell Hook Management)
# ============================================================================
# These markers are used to manage yazelix sections in user shell configs
# Multiple versions exist to detect and migrate old configurations

# v1 markers (for detecting old hooks)
export const YAZELIX_START_MARKER_V1 = "# YAZELIX START - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER_V1 = "# YAZELIX END - Yazelix managed configuration (do not modify this comment)"

# v2 markers (for detecting old hooks with bash wrapper alias)
export const YAZELIX_START_MARKER_V2 = "# YAZELIX START v2 - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER_V2 = "# YAZELIX END v2 - Yazelix managed configuration (do not modify this comment)"

# v3 markers (yzx function in shell configs)
export const YAZELIX_START_MARKER_V3 = "# YAZELIX START v3 - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER_V3 = "# YAZELIX END v3 - Yazelix managed configuration (do not modify this comment)"

# v4 markers (current version - same as v3, version bump for yzx profile)
export const YAZELIX_START_MARKER = "# YAZELIX START v4 - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER = "# YAZELIX END v4 - Yazelix managed configuration (do not modify this comment)"

export const YAZELIX_REGENERATE_COMMENT = "# delete this whole section to re-generate the config, if needed"

# ============================================================================
# CONFIGURATION DEFAULTS
# ============================================================================

export const DEFAULT_SHELL = "nu"
export const DEFAULT_TERMINAL = "ghostty"
export const DEFAULT_HELIX_MODE = "release"

# Supported terminal emulators (fallback priority order)
export const SUPPORTED_TERMINALS = ["ghostty", "wezterm", "kitty", "alacritty", "foot"]

# ============================================================================
# TERMINAL EMULATOR CONFIGURATION
# ============================================================================

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
export const YAZELIX_SHELL_COMMAND = "sh -c 'PATH=\"$HOME/.local/state/nix/profile/bin:$HOME/.nix-profile/bin:$PATH\" exec nu \"$HOME/.config/yazelix/nushell/scripts/core/start_yazelix.nu\"'"
export const YAZELIX_THEME = "Abernathy"
export const FONT_FIRACODE = "FiraCode Nerd Font"

# Transparency opacity mapping
export const TRANSPARENCY_VALUES = {
    none: "1.0"
    very_low: "0.95"
    low: "0.90"
    medium: "0.85"
    high: "0.80"
    very_high: "0.70"
    super_high: "0.60"
}

# Shell argument templates for terminal configs
export const SHELL_ARGS_BASH = '["bash", "-l", "-c", "nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu"]'
export const SHELL_ARGS_STRING = '["-l", "-c", "nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu"]'

# ============================================================================
# CURSOR TRAIL CONFIGURATION (Ghostty)
# ============================================================================

# Cursor trail shader paths
# Note: Use get_cursor_trail_random_pool() to get the pool for random selection
export const CURSOR_TRAIL_SHADERS = {
    blaze: "./shaders/cursor_smear.glsl"
    snow: "./shaders/cursor_trail_white.glsl"
    cosmic: "./shaders/cursor_trail_cosmic.glsl"
    ocean: "./shaders/cursor_trail_ocean.glsl"
    forest: "./shaders/cursor_trail_forest.glsl"
    sunset: "./shaders/cursor_trail_sunset.glsl"
    neon: "./shaders/cursor_trail_neon.glsl"
    party: "./shaders/cursor_trail_party.glsl"
    eclipse: "./shaders/cursor_trail_eclipse.glsl"
    dusk: "./shaders/cursor_trail_dusk.glsl"
    orchid: "./shaders/cursor_trail_orchid.glsl"
    reef: "./shaders/cursor_trail_reef.glsl"
    inferno: "./shaders/cursor_trail_inferno.glsl"
    none: ""
}

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
# eclipse (indigo/gold): custom-shader = ./shaders/cursor_trail_eclipse.glsl
# dusk (blue/coral): custom-shader = ./shaders/cursor_trail_dusk.glsl
# orchid (amber/cobalt): custom-shader = ./shaders/cursor_trail_orchid.glsl
# reef (cyan/green): custom-shader = ./shaders/cursor_trail_reef.glsl
# inferno (crimson/silver): custom-shader = ./shaders/cursor_trail_inferno.glsl
# random (pick on generate): custom-shader = ./shaders/cursor_trail_<random>.glsl"

# ============================================================================
# XDG-COMPLIANT DIRECTORIES
# ============================================================================
# Separation between static config (potentially managed by home-manager)
# and runtime state (always writable, never managed by home-manager)

# Static configuration (potentially managed by home-manager)
export const YAZELIX_CONFIG_DIR = "~/.config/yazelix"

# Runtime state (always writable, never managed by home-manager)
export const YAZELIX_STATE_DIR = "~/.local/share/yazelix"
export const YAZELIX_LOGS_DIR = "~/.local/share/yazelix/logs"
export const YAZELIX_INITIALIZERS_DIR = "~/.local/share/yazelix/initializers"
export const YAZELIX_CACHE_DIR = "~/.local/share/yazelix/cache"
export const YAZELIX_GENERATED_CONFIGS_DIR = "~/.local/share/yazelix/configs"

# ============================================================================
# INTEGRATED TOOL CONFIGURATION PATHS
# ============================================================================

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

# ============================================================================
# SHELL CONFIGURATION PATHS
# ============================================================================

# Shell-specific initializer directories (in state, not config)
export const SHELL_INITIALIZER_DIRS = {
    bash: "~/.local/share/yazelix/initializers/bash"
    nushell: "~/.local/share/yazelix/initializers/nushell"
    fish: "~/.local/share/yazelix/initializers/fish"
    zsh: "~/.local/share/yazelix/initializers/zsh"
}

# User shell configuration files
export const SHELL_CONFIGS = {
    bash: "~/.bashrc"
    nushell: "~/.config/nushell/config.nu"
    fish: "~/.config/fish/config.fish"
    zsh: "~/.zshrc"
}

# Yazelix shell configuration files
export const YAZELIX_CONFIG_FILES = {
    bash: "~/.config/yazelix/shells/bash/yazelix_bash_config.sh"
    nushell: "~/.config/yazelix/nushell/config/config.nu"
    fish: "~/.config/yazelix/shells/fish/yazelix_fish_config.fish"
    zsh: "~/.config/yazelix/shells/zsh/yazelix_zsh_config.zsh"
}

# ============================================================================
# ENVIRONMENT VARIABLES
# ============================================================================

export const YAZELIX_ENV_VARS = {
    YAZELIX_DIR: "~/.config/yazelix"
    YAZELIX_DEFAULT_SHELL: "nu"
    YAZELIX_PREFERRED_TERMINAL: "ghostty"
    YAZELIX_HELIX_MODE: "release"
    YAZI_CONFIG_HOME: "~/.local/share/yazelix/configs/yazi"
    ZELLIJ_DEFAULT_LAYOUT: "yzx_side"
}

# ============================================================================
# TEMPLATE STRINGS
# ============================================================================

# Ghostty config header template
export const GHOSTTY_CONFIG_HEADER = "# This is the configuration file for Ghostty.
#
# This template file has been automatically created at the following
# path since Ghostty couldn't find any existing config files on your system:
#
#   <USER_CONFIG_PATH>
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

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================
# Helper functions have been moved to dedicated modules:
# - cursor_trail_helpers.nu - Cursor trail management
# - environment_detection.nu - Environment detection
# - shell_config_generation.nu - Shell configuration generation
#
# To use all constants AND helpers in one import, use:
#   use constants_with_helpers.nu *
