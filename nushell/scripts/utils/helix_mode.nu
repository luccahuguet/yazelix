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

# Detect the actual running Helix mode by checking for Steel artifacts
export def detect_actual_helix_mode [] {
    let helix_config_dir = $"($env.HOME)/.config/helix"
    let helix_scm = $"($helix_config_dir)/helix.scm"
    let init_scm = $"($helix_config_dir)/init.scm"
    let helix_patchy_dir = $"($env.HOME)/.config/yazelix/helix_patchy"
    
    # Check for Steel configuration files
    let has_steel_config = ($helix_scm | path exists) or ($init_scm | path exists)
    
    # Check for Steel build
    let has_steel_build = ($helix_patchy_dir | path exists) and (try { 
        cd $helix_patchy_dir
        git branch --show-current 
    } catch { "unknown" } | str contains "steel")
    
    # Check for Steel dependencies in binary
    let has_steel_binary = try {
        let binary_path = $"($helix_patchy_dir)/target/release/hx"
        if ($binary_path | path exists) {
            # Check if binary has Steel dependencies (simplified check)
            let ldd_output = (ldd $binary_path 2>/dev/null | str join " ")
            $ldd_output | str contains "steel"
        } else {
            false
        }
    } catch {
        false
    }
    
    # Determine actual mode
    if $has_steel_config or $has_steel_build or $has_steel_binary {
        "steel"
    } else if ($helix_patchy_dir | path exists) and (try { 
        cd $helix_patchy_dir
        git branch --show-current 
    } catch { "unknown" } | str contains "patchy") {
        "patchy"
    } else if ($helix_patchy_dir | path exists) {
        # If there's a local build directory but not steel/patchy, it's likely from a previous setup
        # The actual mode depends on what's configured
        get_helix_mode
    } else {
        "default"
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
        has_steel_files: (try { ls $"($env.HOME)/.config/helix/*.scm" | length } catch { 0 })
        custom_binary_exists: ($"($env.HOME)/.config/yazelix/helix_patchy/target/release/hx" | path exists)
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
    
    print $"Steel files in config: ($info.has_steel_files)"
    print $"Custom binary exists: ($info.custom_binary_exists)"
    
    if $info.has_steel_files > 0 {
        print "🔧 Steel files found:"
        try {
            ls $"($env.HOME)/.config/helix/*.scm" | get name | each { |file| print $"   • ($file)" }
        } catch {
            print "   (Could not list Steel files)"
        }
    }
    
    print "========================"
} 