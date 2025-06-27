#!/usr/bin/env nu
# Zellij configuration generator for Yazelix

def main [yazelix_dir: string] {
    print "🔧 Generating Zellij configuration..."

    let config_path = $"($yazelix_dir)/zellij/config.kdl"
    let template = $"
default_shell \"nu\"
theme \"catppuccin-mocha\"
    "

    # Generate basic config if it doesn't exist
    if not ($config_path | path exists) {
        $template | save $config_path
        print $"✅ Generated Zellij config at ($config_path)"
    } else {
        print $"✅ Zellij config already exists at ($config_path)"
    }
}