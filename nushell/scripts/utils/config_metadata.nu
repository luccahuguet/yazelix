# Configuration metadata for Yazelix
# Defines which config settings require a devenv shell rebuild

# Settings that require 'devenv shell' rebuild when changed
# These settings affect package installation, nix evaluation, or build process
export const REBUILD_REQUIRED_KEYS = [
    # Core packages and build settings
    "core.recommended_deps",
    "core.yazi_extensions",
    "core.yazi_media",
    "core.build_cores",

    # Helix editor build configuration
    "helix.mode",
    "helix.runtime_path",

    # Editor command (may require package installation)
    "editor.command",

    # Shell packages
    "shell.extra_shells",
    "shell.enable_atuin",

    # Terminal packages
    "terminal.terminals",

    # Language and tool packs
    "packs.language",
    "packs.tools",
    "packs.user_packages"
]

# All other settings are runtime settings that apply immediately
# or on next session without requiring a devenv rebuild
