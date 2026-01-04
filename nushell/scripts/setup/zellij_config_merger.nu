#!/usr/bin/env nu
# Zellij Configuration Merger
# Uses the user's Zellij config when available, falls back to Zellij defaults

use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS]
use ../utils/config_parser.nu parse_yazelix_config

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

# Read the user's native Zellij config if it exists
def read_user_zellij_config [] {
    let user_config_path = ("~/.config/zellij/config.kdl" | path expand)
    if ($user_config_path | path exists) {
        try {
            print "📥 Using existing Zellij config from ~/.config/zellij/config.kdl"
            open $user_config_path
        } catch {|err|
            print $"⚠️  Could not read user config: ($err.msg)"
            ""
        }
    } else {
        ""
    }
}

# Choose the base config: user config if present, otherwise Zellij defaults
def get_base_config [] {
    let user_config = read_user_zellij_config
    if ($user_config | is-not-empty) {
        $user_config
    } else {
        print "📥 No user Zellij config found, fetching defaults..."
        get_zellij_defaults
    }
}

# Dynamic overrides sourced from yazelix.toml (takes precedence over user config)
def get_dynamic_overrides [] {
    let config = (try {
        parse_yazelix_config
    } catch {
        {zellij_rounded_corners: "true", zellij_theme: "default", disable_zellij_tips: "true"}
    })

    let rounded = ($config | get -o zellij_rounded_corners | default "true")
    let rounded_value = if ($rounded | str starts-with "false") {
        "false"
    } else {
        "true"
    }

    # Zellij built-in themes (37 total: 28 dark + 9 light)
    let zellij_themes = [
        "ansi", "ao", "atelier-sulphurpool", "ayu_mirage", "ayu_dark",
        "catppuccin-frappe", "catppuccin-macchiato", "cyber-noir", "blade-runner",
        "retro-wave", "dracula", "everforest-dark", "gruvbox-dark", "iceberg-dark",
        "kanagawa", "lucario", "menace", "molokai-dark", "night-owl", "nightfox",
        "nord", "one-half-dark", "onedark", "solarized-dark", "tokyo-night-dark",
        "tokyo-night-storm", "tokyo-night", "vesper",
        "ayu_light", "catppuccin-latte", "everforest-light", "gruvbox-light",
        "iceberg-light", "dayfox", "pencil-light", "solarized-light", "tokyo-night-light"
    ]

    let theme_config = ($config | get -o zellij_theme | default "default")
    let theme = if $theme_config == "random" {
        $zellij_themes | shuffle | first
    } else {
        $theme_config
    }

    # disable_tips in yazelix.toml → show_startup_tips in Zellij config (inverted logic)
    let disable_tips = ($config | get -o disable_zellij_tips | default "true")
    let show_tips_value = if ($disable_tips | str starts-with "false") {
        "true"
    } else {
        "false"
    }

    [
        "// === YAZELIX DYNAMIC SETTINGS (from yazelix.toml) ===",
        $"theme \"($theme)\"",
        $"show_startup_tips ($show_tips_value)",
        "ui {",
        "    pane_frames {",
        $"        rounded_corners ($rounded_value)",
        "    }",
        "}"
    ] | str join "\n"
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
    let yazelix_layout_dir = $"($merged_config_dir)/layouts"
    let config = parse_yazelix_config
    let widget_tray = ($config.zellij_widget_tray? | default ["layout", "editor", "shell", "term", "cpu", "ram"])
    
    print "🔄 Regenerating Zellij configuration..."
    
    # Ensure output directory exists
    ensure_dir $merged_config_path
    
    # Copy layouts directory to merged config
    let source_layouts_dir = $"($yazelix_dir)/($ZELLIJ_CONFIG_PATHS.layouts_dir)"
    let target_layouts_dir = $"($merged_config_dir)/layouts"
    if ($source_layouts_dir | path exists) {
        # Copy layouts to merged config directory
        use ../utils/layout_generator.nu
        layout_generator generate_all_layouts $source_layouts_dir $target_layouts_dir $widget_tray
    }
    
    # Generate configuration from user config or defaults
    let base_config_raw = get_base_config

    # Remove any settings we control from base config (yazelix.toml takes precedence)
    # This prevents conflicts when multiple declarations of the same setting exist
    let base_config = ($base_config_raw | lines | where {|line|
        let trimmed = ($line | str trim)
        not (
            ($trimmed | str starts-with "theme ") or
            ($trimmed | str starts-with "pane_frames ") or
            ($trimmed | str starts-with "default_layout ") or
            ($trimmed | str starts-with "layout_dir ") or
            ($trimmed | str starts-with "on_force_close ") or
            ($trimmed | str starts-with "show_startup_tips ")
        )
    } | str join "\n")

    let merged_config = [
        "// ========================================",
        "// GENERATED ZELLIJ CONFIG (YAZELIX)",
        "// ========================================",
        "// Source preference:",
        "//   1) ~/.config/zellij/config.kdl (user-managed)",
        "//   2) zellij setup --dump-config (defaults)",
        "//",
        $"// Generated: (date now | format date '%Y-%m-%d %H:%M:%S')",
        "// ========================================",
        "",
        $base_config,
        "",
        (get_dynamic_overrides),
        "",
        "// === YAZELIX ENFORCED SETTINGS ===",
        "pane_frames false",
        $"default_layout \"($yazelix_layout_dir)/yzx_side.kdl\"",
        $"layout_dir \"($yazelix_layout_dir)\""
    ] | str join "\n"
    
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
