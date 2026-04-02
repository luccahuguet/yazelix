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

# Set environment variables for Helix mode
export def set_helix_env [] {
    let mode = get_helix_mode
    $env.YAZELIX_HELIX_MODE = $mode
}

# Export environment variables as shell-compatible format
export def export_helix_env [] {
    let mode = get_helix_mode
    $"export YAZELIX_HELIX_MODE=\"($mode)\""
}

# Detect the actual running Helix mode
export def detect_actual_helix_mode [] {
    let helix_custom_dir = $"($env.HOME)/.config/yazelix/helix_custom"

    if ($helix_custom_dir | path exists) {
        # If there's a local build directory, it's likely from source mode
        get_helix_mode
    } else {
        "release"
    }
}

# Compare configured mode vs actual running mode
export def compare_helix_modes [] {
    let configured_mode = get_helix_mode
    let actual_mode = detect_actual_helix_mode

    {
        configured: $configured_mode
        actual: $actual_mode
        mismatch: ($configured_mode != $actual_mode)
        helix_config_dir: $"($env.HOME)/.config/helix"
        custom_binary_exists: ($"($env.HOME)/.config/yazelix/helix_custom/target/release/hx" | path exists)
    }
}

# Show detailed Helix mode information
export def show_helix_mode_info [] {
    let info = compare_helix_modes

    print "=== Helix Mode Analysis ==="
    print $"Configured mode: ($info.configured)"
    print $"Actual running mode: ($info.actual)"

    if $info.mismatch {
        print "⚠️  MODE MISMATCH DETECTED!"
        print "   The configured mode differs from the actual running mode."
        print "   This usually happens when switching between modes without proper cleanup."
    } else {
        print "✅ Mode consistency: OK"
    }

    print $"Custom binary exists: ($info.custom_binary_exists)"

    print "========================"
}
