#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/version_info.nu *

# =============================================================================
# YAZELIX COMMANDS WITH NATIVE SUBCOMMAND SUPPORT
# =============================================================================

# Main yzx command - default shows help
export def yzx [] {
    yzx help
}

# Help subcommand
export def "yzx help" [] {
    print "=== Yazelix Command Suite ==="
    print ""
    print "CONFIGURATION MANAGEMENT:"
print "  yzx get_config [shell]         - Show status of all shell configurations"
print "  yzx check_config               - Check if configurations are up to date"
print "  yzx config_status [shell]      - Same as get_config (alias)"
print "  yzx config_schema              - Show the configuration schema"
    print ""
    print "VERSION AND SYSTEM:"
    print "  yzx versions                   - Show version info for all tools"
    print "  yzx version                    - Show yazelix version"
    print "  yzx info                       - Show yazelix system information"
    print ""
    print "LAUNCHER:"
    print "  yzx launch                     - Launch yazelix via terminal"
    print "  yzx start                      - Start yazelix directly"
    print ""
    print "HELP:"
    print "  yzx help                       - Show this help message"
    print ""
    print "Supported shells: bash, nushell, fish, zsh"
    print "=========================================="
}

# Get configuration details
export def "yzx get_config" [shell?: string] {
    if ($shell | is-empty) {
        # Show all configurations
        show_config_status ~/.config/yazelix
    } else {
        # Show specific shell configuration
        let config_file = ($SHELL_CONFIGS | get $shell | str replace "~" $env.HOME)

        if not ($config_file | path exists) {
            print $"❌ Config file not found: ($config_file)"
            return
        }

        let section = extract_yazelix_section $config_file

        if $section.exists {
            print $"=== Yazelix Section in ($shell) ==="
            print $section.content
            print "=================================="
        } else {
            print $"❌ No yazelix section found in ($config_file)"
        }

        $section
    }
}

# Check configuration validity
export def "yzx check_config" [] {
    let status = check_config_versions ~/.config/yazelix
    let outdated = ($status | where status == "outdated")
    let missing = ($status | where status == "missing")

    if ($outdated | is-empty) and ($missing | is-empty) {
        print "✅ All yazelix configurations are current!"
    } else {
        if not ($outdated | is-empty) {
            print "⚠️  Outdated configurations:"
            for $config in $outdated {
                print $"   ($config.shell): ($config.file)"
            }
        }
        if not ($missing | is-empty) {
            print "❌ Missing configurations:"
            for $config in $missing {
                print $"   ($config.shell): ($config.file)"
            }
        }
    }

    $status
}

# Show configuration status (alias for get_config)
export def "yzx config_status" [shell?: string] {
    yzx get_config $shell
}

# List available versions
export def "yzx versions" [] {
    nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu
}

# Show current version
export def "yzx version" [] {
    print $"Yazelix ($YAZELIX_VERSION)"
    print $YAZELIX_DESCRIPTION
}

# Show system info
export def "yzx info" [] {
    print "=== Yazelix Information ==="
    print $"Version: ($YAZELIX_VERSION)"
    print $"Description: ($YAZELIX_DESCRIPTION)"
    print $"Directory: ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME)"
    print $"Logs: ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)"
    print $"Default Shell: ($DEFAULT_SHELL)"
    print $"Preferred Terminal: ($DEFAULT_TERMINAL)"
    print $"Helix Mode: ($DEFAULT_HELIX_MODE)"
    print "=========================="
}

# Launch yazelix
export def "yzx launch" [] {
    nu ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
}

# Start yazelix
export def "yzx start" [] {
    use ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu main
    main
}

# Show configuration schema
export def "yzx config_schema" [] {
    use ../utils/config_schema.nu get_config_schema

    let schema = get_config_schema

    print "=== Yazelix Configuration Schema ==="
    print ""

    for field in ($schema | columns) {
        let field_schema = ($schema | get $field)
        print $"📝 ($field):"
        print $"   Type: ($field_schema.type)"
        print $"   Default: ($field_schema.default)"
        print $"   Description: ($field_schema.description)"

        if ($field_schema | get valid_values? | default [] | length) > 0 {
            let valid_values = ($field_schema | get valid_values | str join ", ")
            print $"   Valid values: ($valid_values)"
        }

        if ($field_schema.type == "object") {
            print "   Fields:"
            for nested_field in ($field_schema | get fields | columns) {
                let nested_schema = ($field_schema | get fields | get $nested_field)
                print $"     - ($nested_field): ($nested_schema.type) (default: ($nested_schema.default))"
            }
        }

        print ""
    }

    print "====================================="
}

