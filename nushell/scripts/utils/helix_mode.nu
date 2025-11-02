#!/usr/bin/env nu
# Helix mode detection utility for Yazelix

# Get the current Helix mode from yazelix.toml configuration (falls back to legacy yazelix.nix)
export def get_helix_mode [] {
    let toml_config = $"($env.HOME)/.config/yazelix/yazelix.toml"
    let legacy_config = $"($env.HOME)/.config/yazelix/yazelix.nix"
    let default_toml = $"($env.HOME)/.config/yazelix/yazelix_default.toml"

    if ($toml_config | path exists) {
        try {
            open $toml_config | get helix.mode
        } catch {
            "release"
        }
    } else if ($legacy_config | path exists) {
        try {
            let config_content = (open $legacy_config)
            let helix_mode_line = ($config_content | lines | where $it | str contains "helix_mode")

            if not ($helix_mode_line | is-empty) {
                $helix_mode_line | first | str replace "helix_mode = " "" | str replace "\"" "" | str replace ";" "" | str trim
            } else {
                "release"
            }
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
