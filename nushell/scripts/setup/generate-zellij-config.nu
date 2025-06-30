#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/setup/generate-zellij-config.nu

# Generate Zellij configuration using built-in commands
# This creates a minimal config with Yazelix-specific settings

def main [yazelix_dir: string] {
    print "🔧 Generating Zellij configuration..."

    let config_path = $"($yazelix_dir)/zellij/config.kdl"
    let overrides_path = $"($yazelix_dir)/zellij/yazelix-overrides.kdl"

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
    # Note: You can edit the yazelix-overrides.kdl file to add your own settings as well
    let yazelix_overrides = if ($overrides_path | path exists) {
        try {
            open $overrides_path
        } catch {
            print "⚠️  Could not read yazelix-overrides.kdl, skipping overrides"
            ""
        }
    } else {
        print "⚠️  yazelix-overrides.kdl not found, skipping overrides"
        ""
    }

    # Combine default config with Yazelix overrides
    let yazelix_config = if ($yazelix_overrides | str length) > 0 {
        $default_config + "\n\n// === Yazelix-specific overrides ===\n" + $yazelix_overrides + "\n\n// User-specific settings: add below\n"
    } else {
        $default_config
    }

    # Generate config file
    $yazelix_config | save --force $config_path

    print $"✅ Zellij configuration generated successfully at: ($config_path)"
    print "   - Combined Zellij defaults with Yazelix overrides from yazelix-overrides.kdl"
    print "   - Edit yazelix-overrides.kdl to customize Yazelix-specific settings"
    print "   - Add personal settings at the bottom of config.kdl"
} 