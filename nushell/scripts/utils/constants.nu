#!/usr/bin/env nu
# Yazelix Constants
# Small live constants plus narrow accessors for static metadata assets

const CONSTANTS_DATA_PATH = ((path self | path dirname) | path join "constants_data.json")

def load_constants_data [] {
    open $CONSTANTS_DATA_PATH
}

# ============================================================================
# VERSION INFORMATION
# ============================================================================

export const YAZELIX_VERSION = "v15.4"
export const PINNED_NIX_VERSION = "2.34.5"
export const PINNED_NUSHELL_VERSION = "0.111.0"

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

export def get_terminal_config_paths [] {
    (load_constants_data).terminal_config_paths
}

export def get_terminal_metadata [] {
    (load_constants_data).terminal_metadata
}

export def get_cursor_trail_shaders [] {
    (load_constants_data).cursor_trail_shaders
}

export def get_ghostty_trail_effects [] {
    (load_constants_data).ghostty_trail_effects
}

export def get_ghostty_mode_effects [] {
    (load_constants_data).ghostty_mode_effects
}

# Common terminal configuration values
export const YAZELIX_WINDOW_CLASS = "com.yazelix.Yazelix"
export const YAZELIX_X11_INSTANCE = "yazelix"

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

export def get_zellij_config_paths [] {
    (load_constants_data).zellij_config_paths
}

# ============================================================================
# SHELL CONFIGURATION PATHS
# ============================================================================

export def get_shell_initializer_dirs [] {
    (load_constants_data).shell_initializer_dirs
}

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================
# Helper functions have been moved to dedicated modules:
# - cursor_trail_helpers.nu - Cursor trail management
# - environment_detection.nu - Environment detection
