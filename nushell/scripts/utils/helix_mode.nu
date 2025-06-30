#!/usr/bin/env nu
# Helix mode detection utility for Yazelix

# Get the current Helix mode from yazelix.nix configuration
export def get_helix_mode [] {
    let yazelix_config = $"($env.HOME)/.config/yazelix/yazelix.nix"
    let default_config = $"($env.HOME)/.config/yazelix/yazelix_default.nix"
    
    let config_file = if ($yazelix_config | path exists) { $yazelix_config } else { $default_config }
    
    if ($config_file | path exists) {
        try {
            let config_content = (open $config_file)
            let helix_mode_line = ($config_content | lines | where $it | str contains "helix_mode")
            
            if not ($helix_mode_line | is-empty) {
                $helix_mode_line | first | str replace "helix_mode = " "" | str replace "\"" "" | str replace ";" "" | str trim
            } else {
                "default"
            }
        } catch {
            "default"
        }
    } else {
        "default"
    }
}

# Get the appropriate Helix binary path based on mode
export def get_helix_binary [] {
    let mode = get_helix_mode
    let custom_path = $"($env.HOME)/.config/yazelix/helix_patchy/target/release/hx"
    
    if $mode in ["steel", "patchy", "source"] and ($custom_path | path exists) {
        $custom_path
    } else {
        "hx"
    }
}

# Set environment variables for Helix mode
export def set_helix_env [] {
    let mode = get_helix_mode
    $env.YAZELIX_HELIX_MODE = $mode
    
    if $mode in ["steel", "patchy", "source"] {
        $env.YAZELIX_PATCHY_HX = $"($env.HOME)/.config/yazelix/helix_patchy/target/release/hx"
    }
}

# Export environment variables as shell-compatible format
export def export_helix_env [] {
    let mode = get_helix_mode
    let exports = if $mode in ["steel", "patchy", "source"] {
        [
            $"export YAZELIX_HELIX_MODE=\"($mode)\""
            $"export YAZELIX_PATCHY_HX=\"($env.HOME)/.config/yazelix/helix_patchy/target/release/hx\""
        ]
    } else {
        [
            $"export YAZELIX_HELIX_MODE=\"($mode)\""
        ]
    }
    
    $exports | str join "\n"
} 