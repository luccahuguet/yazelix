#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let yazelix_dir = "~/.config/yazelix" | path expand

    # Check for config override first (for testing)
    let config_to_read = if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) {
        $env.YAZELIX_CONFIG_OVERRIDE
    } else {
        # Determine which config file to use
        let toml_file = ($yazelix_dir | path join "yazelix.toml")
        let default_toml = ($yazelix_dir | path join "yazelix_default.toml")

        if ($toml_file | path exists) {
            $toml_file
        } else if ($default_toml | path exists) {
            # Auto-create yazelix.toml from default (copy raw to preserve comments)
            print "📝 Creating yazelix.toml from yazelix_default.toml..."
            open --raw $default_toml | save $toml_file
            print "✅ yazelix.toml created\n"
            $toml_file
        } else {
            error make {msg: "No yazelix configuration file found (yazelix_default.toml is missing)"}
        }
    }

    # Parse TOML configuration (Nushell auto-parses TOML files)
    let raw_config = open $config_to_read

    # Extract and return values
    {
        persistent_sessions: ($raw_config.zellij?.persistent_sessions? | default false | into string),
        session_name: ($raw_config.zellij?.session_name? | default "yazelix"),
        preferred_terminal: ($raw_config.terminal?.preferred_terminal? | default "ghostty"),
        extra_terminals: ($raw_config.terminal?.extra_terminals? | default [] | into string),
        enable_atuin: ($raw_config.shell?.enable_atuin? | default false | into string),
        terminal_config_mode: ($raw_config.terminal?.config_mode? | default "yazelix"),
        cursor_trail: ($raw_config.terminal?.cursor_trail? | default "random"),
        transparency: ($raw_config.terminal?.transparency? | default "low"),
        default_shell: ($raw_config.shell?.default_shell? | default "nu"),
        helix_mode: ($raw_config.helix?.mode? | default "release"),
        disable_zellij_tips: ($raw_config.zellij?.disable_tips? | default true | into string),
        zellij_rounded_corners: ($raw_config.zellij?.rounded_corners? | default true | into string),
        config_file: $config_to_read
    }
}
