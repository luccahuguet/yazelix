#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/setup/generate_zellij_config.nu

# Generate Zellij configuration using built-in commands
# This creates a minimal config with Yazelix-specific settings

def main [yazelix_dir: string] {
    print "🔧 Generating Zellij configuration..."

    let config_path = $"($yazelix_dir)/zellij/config.kdl"
    let overrides_path = $"($yazelix_dir)/zellij/yazelix_overrides.kdl"

    # Get the default config from Zellij
    # Note: You can replace this with your own existing config file:
    #   let default_config = (open ~/.config/zellij/config.kdl)
    let default_config = try {
        zellij setup --dump-config
    } catch {
        print "⚠️  Could not get default Zellij config, using minimal fallback"
        "// Minimal Zellij configuration\n"
    }

    # Read Yazelix-specific overrides
    # Note: You can edit the yazelix_overrides.kdl file to add your own settings as well
    let yazelix_overrides = if ($overrides_path | path exists) {
        try {
            open $overrides_path
        } catch {
            print "⚠️  Could not read yazelix_overrides.kdl, skipping overrides"
            ""
        }
    } else {
        print "⚠️  yazelix_overrides.kdl not found, skipping overrides"
        ""
    }

    # Combine default config with Yazelix overrides
    let combined_config = $default_config + "\n" + $yazelix_overrides

    # Write the combined configuration
    try {
        $combined_config | save $config_path
        print "✅ Zellij configuration generated successfully!"
        print $"   📁 Config saved to: ($config_path)"
        print "   - Combined Zellij defaults with Yazelix overrides from yazelix_overrides.kdl"
        print "   - Edit yazelix_overrides.kdl to customize Yazelix-specific settings"
    } catch {|err|
        print $"❌ Failed to write config: ($err.msg)"
        exit 1
    }
}

# Export the main function
export def generate_zellij_config [yazelix_dir: string] {
    main $yazelix_dir
} 