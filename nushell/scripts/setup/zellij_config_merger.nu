#!/usr/bin/env nu
# Dynamic Zellij Configuration Merger
# Merges three layers: Zellij defaults + Yazelix overrides + User config

use ../utils/constants.nu [YAZELIX_STATE_DIR, ZELLIJ_CONFIG_PATHS]
use ../utils/config_parser.nu parse_yazelix_config

# Get modification time of a file, or 0 if file doesn't exist
def get_mtime [path: string] {
    if ($path | path exists) {
        let expanded_path = ($path | path expand)
        let file_info = (ls $expanded_path | get 0)
        ($file_info.modified | into int)
    } else {
        0
    }
}

# Check if merged config is up-to-date
def is_config_current [
    merged_path: string,
    yazelix_overrides_path: string, 
    user_config_path: string
] {
    let merged_mtime = get_mtime $merged_path
    let yazelix_mtime = get_mtime $yazelix_overrides_path
    let user_mtime = get_mtime $user_config_path
    
    # Config is current if merged file is newer than all source files
    ($merged_mtime > $yazelix_mtime) and ($merged_mtime > $user_mtime)
}

# Fetch Zellij default configuration
def get_zellij_defaults [] {
    let result = (try { zellij setup --dump-config } catch {|err| 
        print $"❌ CRITICAL ERROR: Cannot fetch Zellij defaults: ($err.msg)"
        print "   Zellij must be available in PATH for Yazelix to work properly."
        print "   This indicates the merger is running outside the Nix environment."
        print "   Yazelix cannot function without proper Zellij configuration."
        exit 1
    })
    $result
}

# Read config file with error handling
def read_config_file [path: string, name: string] {
    if ($path | path exists) {
        try {
            open $path
        } catch {|err|
            print $"⚠️  Could not read ($name): ($err.msg)"
            ""
        }
    } else {
        ""
    }
}

# Generate dynamic Yazelix overrides based on yazelix.nix config
export def get_dynamic_yazelix_overrides [yazelix_dir: string] {
    let config = (try {
        # Don't override the existing YAZELIX_CONFIG_OVERRIDE if it's set
        parse_yazelix_config
    } catch {
        # If config parsing fails, use defaults
        {disable_zellij_tips: "false", zellij_rounded_corners: "true"}
    })

    mut overrides = []

    # Add tips disable setting if enabled (handle both "true" and "true  # comment" formats)
    # Use default of "true" if field doesn't exist (backwards compatibility)
    let disable_tips = ($config | get -i disable_zellij_tips | default "true")
    if ($disable_tips | str starts-with "true") {
        $overrides = ($overrides | append "// Disable startup tips (set via disable_zellij_tips in yazelix.nix)")
        $overrides = ($overrides | append "show_startup_tips false")
        $overrides = ($overrides | append "")
    }

    # Add rounded corners setting (handle both "true" and "true  # comment" formats)
    # Use default of "true" if field doesn't exist (backwards compatibility)
    let rounded_corners = ($config | get -i zellij_rounded_corners | default "true")
    if ($rounded_corners | str starts-with "true") {
        $overrides = ($overrides | append "// Enable rounded corners for pane frames (set via zellij_rounded_corners in yazelix.nix)")
        $overrides = ($overrides | append "ui {")
        $overrides = ($overrides | append "    pane_frames {")
        $overrides = ($overrides | append "        rounded_corners true")
        $overrides = ($overrides | append "    }")
        $overrides = ($overrides | append "}")
        $overrides = ($overrides | append "")
    } else if ($rounded_corners | str starts-with "false") {
        $overrides = ($overrides | append "// Disable rounded corners for pane frames (set via zellij_rounded_corners in yazelix.nix)")
        $overrides = ($overrides | append "ui {")
        $overrides = ($overrides | append "    pane_frames {")
        $overrides = ($overrides | append "        rounded_corners false")
        $overrides = ($overrides | append "    }")
        $overrides = ($overrides | append "}")
        $overrides = ($overrides | append "")
    }

    $overrides | str join "\n"
}

# Merge three configuration layers
def merge_zellij_configs [
    yazelix_dir: string
] {
    print "🔧 Merging Zellij configuration layers..."

    # Layer 1: Zellij defaults
    print "   📥 Fetching Zellij defaults..."
    let defaults = get_zellij_defaults

    # Layer 2: Yazelix overrides
    let yazelix_overrides_path = $"($yazelix_dir)/configs/zellij/yazelix_overrides.kdl"
    print "   📥 Reading Yazelix overrides..."
    let yazelix_overrides = read_config_file $yazelix_overrides_path "Yazelix overrides"

    # Layer 2.5: Dynamic Yazelix settings based on config
    print "   ⚙️  Applying dynamic configuration..."
    let dynamic_overrides = get_dynamic_yazelix_overrides $yazelix_dir

    # Layer 3: User config
    let user_config_path = $"($yazelix_dir)/configs/zellij/personal/user_config.kdl"
    print "   📥 Reading personal configuration..."
    let user_config = read_config_file $user_config_path "personal configuration"
    
    # Combine all layers with clear separation
    # NOTE: This uses simple concatenation. Zellij processes all sections
    # and uses the last occurrence of any setting. For nested blocks like
    # ui, keybinds, themes - this means the user's entire block overrides
    # Yazelix's block (which is the desired behavior for most cases).
    let merged = [
        "// ========================================",
        "// DYNAMICALLY GENERATED ZELLIJ CONFIG",
        "// ========================================",
        "// This file is automatically generated by Yazelix.",
        "// Do not edit directly - changes will be lost!",
        "//",
        "// To customize Zellij, edit:",
        "//   - configs/zellij/personal/user_config.kdl (your personal settings)",
        "//   - configs/zellij/yazelix_overrides.kdl (Yazelix defaults)",
        "//   - yazelix.nix (global Yazelix options)",
        "//",
        $"// Generated: (date now | format date '%Y-%m-%d %H:%M:%S')",
        "// ========================================",
        "",
        "// === LAYER 1: ZELLIJ DEFAULTS ===",
        $defaults,
        "",
        "// === LAYER 2: YAZELIX OVERRIDES ===",
        $yazelix_overrides,
        "",
        "// === LAYER 2.5: DYNAMIC YAZELIX SETTINGS ===",
        $dynamic_overrides,
        "",
        "// === LAYER 3: USER CONFIGURATION ===",
        $user_config,
        ""
    ] | str join "\n"
    
    $merged
}

# Ensure directory exists
def ensure_dir [path: string] {
    let dir = ($path | path dirname)
    if not ($dir | path exists) {
        mkdir $dir
    }
}

# Main function: Generate merged Zellij configuration
export def generate_merged_zellij_config [yazelix_dir: string] {
    # Define paths using constants
    let merged_config_dir = ($ZELLIJ_CONFIG_PATHS.merged_config_dir | path expand)
    let merged_config_path = ($ZELLIJ_CONFIG_PATHS.merged_config | path expand)
    let yazelix_overrides_path = $"($yazelix_dir)/($ZELLIJ_CONFIG_PATHS.yazelix_overrides)"
    let user_config_path = $"($yazelix_dir)/($ZELLIJ_CONFIG_PATHS.user_config)"
    
    # Always regenerate configs to avoid stale config issues
    # if (is_config_current $merged_config_path $yazelix_overrides_path $user_config_path) {
    #     print "✅ Zellij config is current, skipping regeneration"
    #     return $merged_config_path
    # }
    
    print "🔄 Regenerating Zellij configuration..."
    
    # Ensure output directory exists
    ensure_dir $merged_config_path
    
    # Copy layouts directory to merged config location
    let source_layouts_dir = $"($yazelix_dir)/($ZELLIJ_CONFIG_PATHS.layouts_dir)"
    let target_layouts_dir = $"($merged_config_dir)/layouts"
    if ($source_layouts_dir | path exists) {
        if not ($target_layouts_dir | path exists) {
            mkdir $target_layouts_dir
        }
        # Copy all layout files
        let layout_files = (ls $source_layouts_dir | where type == file | get name)
        for file in $layout_files {
            let filename = ($file | path basename)
            cp $file $"($target_layouts_dir)/($filename)"
        }
    }
    
    # Generate merged configuration
    let merged_config = merge_zellij_configs $yazelix_dir
    
    # Write atomically (write to temp file, then move)
    let temp_path = $"($merged_config_path).tmp"
    try {
        $merged_config | save $temp_path
        mv $temp_path $merged_config_path
        print $"✅ Zellij configuration generated successfully!"
        print $"   📁 Config saved to: ($merged_config_path)"
        print "   🔄 Config will auto-regenerate when source files change"
    } catch {|err|
        print $"❌ Failed to write merged config: ($err.msg)"
        # Clean up temp file if it exists
        if ($temp_path | path exists) {
            rm $temp_path
        }
        exit 1
    }
    
    $merged_config_path
}

# Export main function for external use
export def main [yazelix_dir: string] {
    generate_merged_zellij_config $yazelix_dir | ignore
}