#!/usr/bin/env nu
# Yazelix Constants
# Pure constant definitions with no imports or function dependencies

# ============================================================================
# VERSION INFORMATION
# ============================================================================

export const YAZELIX_VERSION = "v15"
export const YAZELIX_DESCRIPTION = "Yazi + Zellij + Helix integrated terminal environment"
export const PINNED_NIX_VERSION = "2.34.5"
export const PINNED_NUSHELL_VERSION = "0.111.0"

# ============================================================================
# CONFIGURATION SECTION MARKERS (Shell Hook Management)
# ============================================================================
# These markers are used to manage the current yazelix section in user shell configs.
export const YAZELIX_START_MARKER = "# YAZELIX START v4 - Yazelix managed configuration (do not modify this comment)"
export const YAZELIX_END_MARKER = "# YAZELIX END v4 - Yazelix managed configuration (do not modify this comment)"

export const YAZELIX_REGENERATE_COMMENT = "# delete this whole section to re-generate the config, if needed"

# ============================================================================
# CONFIGURATION DEFAULTS
# ============================================================================

export const DEFAULT_SHELL = "nu"
export const DEFAULT_TERMINAL = "ghostty"

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

# Terminal display names
export const TERMINAL_METADATA = {
    ghostty: {name: "Ghostty"}
    kitty: {name: "Kitty"}
    wezterm: {name: "WezTerm"}
    alacritty: {name: "Alacritty"}
    foot: {name: "Foot"}
}

# Common terminal configuration values
export const YAZELIX_WINDOW_CLASS = "com.yazelix.Yazelix"
export const YAZELIX_X11_INSTANCE = "yazelix"
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

# ============================================================================
# CURSOR TRAIL CONFIGURATION (Ghostty)
# ============================================================================

# Cursor trail shader paths
# Note: Use get_cursor_trail_random_pool() to get the pool for random selection
export const CURSOR_TRAIL_SHADERS = {
    blaze: "./shaders/cursor_trail_blaze.glsl"
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

export const CURSOR_TRAIL_COLOR_LITERALS = {
    blaze: "vec4(1.0, 0.725, 0.161, 1.0)"
    snow: "vec4(1.0, 1.0, 1.0, 1.0)"
    cosmic: "vec4(0.78, 0.38, 0.96, 1.0)"
    ocean: "vec4(0.37, 0.66, 1.00, 1.0)"
    forest: "vec4(0.23, 0.82, 0.48, 1.0)"
    sunset: "vec4(1.00, 0.48, 0.35, 1.0)"
    neon: "vec4(0.00, 0.565, 1.00, 1.0)"
    party: "vec4(1.00, 0.00, 1.00, 1.0)"
    eclipse: "vec4(1.000, 0.831, 0.000, 1.0)"
    dusk: "vec4(0.914, 0.271, 0.376, 1.0)"
    orchid: "vec4(1.0, 0.420, 0.0, 1.0)"
    reef: "vec4(0.0, 0.902, 1.0, 1.0)"
    inferno: "vec4(1.0, 0.086, 0.0, 1.0)"
}

export const CURSOR_TRAIL_COLOR_HEX = {
    blaze: "#ffb929"
    snow: "#ffffff"
    cosmic: "#c761f5"
    ocean: "#5ea8ff"
    forest: "#3bd17a"
    sunset: "#ff7a59"
    neon: "#0090ff"
    party: "#ff00ff"
    eclipse: "#ffd400"
    dusk: "#e94560"
    orchid: "#ff6b00"
    reef: "#00e6ff"
    inferno: "#ff1600"
}

export const GHOSTTY_TRAIL_EFFECTS = [
    "tail"
    "warp"
    "sweep"
]

export const GHOSTTY_MODE_EFFECTS = [
    "ripple"
    "sonic_boom"
    "rectangle_boom"
    "ripple_rectangle"
]

export const GHOSTTY_TRAIL_GLOW_LEVELS = [
    "none"
    "low"
    "medium"
    "high"
]

export const GHOSTTY_CURSOR_EFFECT_TEMPLATE_FILES = {
    tail: "cursor_tail.glsl"
    warp: "cursor_warp.glsl"
    sweep: "cursor_sweep.glsl"
    ripple: "ripple_cursor.glsl"
    sonic_boom: "sonic_boom_cursor.glsl"
    rectangle_boom: "rectangle_boom_cursor.glsl"
    ripple_rectangle: "ripple_rectangle_cursor.glsl"
}

# Cursor trail presets documentation
export const CURSOR_TRAIL_PRESETS_COMMENT = "# Alternative presets (uncomment to try)
# snow:  custom-shader = ./shaders/cursor_trail_white.glsl
# blaze (fire):  custom-shader = ./shaders/cursor_trail_blaze.glsl
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
    bash: "shells/bash/yazelix_bash_config.sh"
    nushell: "nushell/config/config.nu"
    fish: "shells/fish/yazelix_fish_config.fish"
    zsh: "shells/zsh/yazelix_zsh_config.zsh"
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
