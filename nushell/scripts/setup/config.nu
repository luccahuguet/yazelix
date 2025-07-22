#!/usr/bin/env nu
# Zellij configuration generator for Yazelix

def main [yazelix_dir: string] {
    use ../utils/constants.nu YAZELIX_CACHE_DIR
    print "🔧 Generating Zellij configuration..."

    # Store generated config in cache directory (XDG-compliant)
    let cache_dir = ($YAZELIX_CACHE_DIR | str replace "~" $env.HOME)
    mkdir $cache_dir
    let config_path = $"($cache_dir)/zellij_config.kdl"
    let template = $"
default_shell \"nu\"
    "

    # Generate basic config if it doesn't exist
    if not ($config_path | path exists) {
        $template | save $config_path
        print $"✅ Generated Zellij config at ($config_path)"
    } else {
        print $"✅ Zellij config already exists at ($config_path)"
    }
}