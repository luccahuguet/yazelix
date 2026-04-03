#!/usr/bin/env nu
# Helix mode detection utility for Yazelix

use common.nu get_yazelix_runtime_dir
# Get the current Helix mode from the managed TOML surfaces.
use config_surfaces.nu get_main_user_config_path
export def get_helix_mode [] {
    let toml_config = (get_main_user_config_path)
    let default_toml = ((get_yazelix_runtime_dir) | path join "yazelix_default.toml")

    if ($toml_config | path exists) {
        try {
            open $toml_config | get helix.mode
        } catch {
            "release"
        }
    } else if ($default_toml | path exists) {
        try {
            open $default_toml | get helix.mode
        } catch {
            "release"
        }
    } else {
        "release"
    }
}

# Get the appropriate Helix binary path from environment
# Note: This assumes EDITOR is set to a Helix binary
export def get_helix_binary [] {
    let managed_binary = ($env.YAZELIX_MANAGED_HELIX_BINARY? | default "" | into string | str trim)
    if ($managed_binary | is-not-empty) {
        return $managed_binary
    }

    # Only return EDITOR if it's actually Helix, fallback to 'hx' for safety
    let editor = $env.EDITOR
    let is_helix = ($editor | str ends-with "/hx") or ($editor == "hx") or ($editor | str ends-with "/helix") or ($editor == "helix")
    if $is_helix {
        $editor
    } else {
        "hx"  # Fallback for non-Helix editors
    }
}
