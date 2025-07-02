#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ./utils/config_manager.nu *
use ./utils/constants.nu *
use ./utils/version-info.nu *

# =============================================================================
# CONFIGURATION MANAGEMENT COMMANDS
# =============================================================================

# Get configuration status - can show all or specific shell
export def get_config [shell?: string] {
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

# Check if configurations are up to date
export def check_config [] {
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

# Alias for get_config
export def config_status [shell?: string] {
    get_config $shell
}

# =============================================================================
# VERSION AND SYSTEM COMMANDS
# =============================================================================

# Show version information for all yazelix tools
export def versions [] {
    nu nushell/scripts/utils/version-info.nu
}

# Show yazelix version and description
export def version [] {
    print $"Yazelix ($YAZELIX_VERSION)"
    print $YAZELIX_DESCRIPTION
}

# =============================================================================
# LAUNCHER COMMANDS
# =============================================================================

# Launch yazelix using the preferred terminal
export def launch [] {
    nu ~/.config/yazelix/nushell/scripts/launch-yazelix.nu
}

# Start yazelix directly (bypasses terminal launcher)
export def start [] {
    bash ~/.config/yazelix/bash/start-yazelix.sh
}

# =============================================================================
# UTILITY COMMANDS
# =============================================================================

# Show yazelix directory and important paths
export def info [] {
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

# Show help for all yazelix commands
export def help [] {
    print "=== Yazelix Command Suite ==="
    print ""
    print "CONFIGURATION MANAGEMENT:"
    print "  yazelix get_config           - Show status of all shell configurations"
    print "  yazelix get_config <shell>   - Show yazelix section from specific shell"
    print "  yazelix check_config         - Check if configurations are up to date"
    print "  yazelix config_status        - Same as get_config (alias)"
    print ""
    print "VERSION AND SYSTEM:"
    print "  yazelix versions             - Show version info for all tools"
    print "  yazelix version              - Show yazelix version"
    print "  yazelix info                 - Show yazelix system information"
    print ""
    print "LAUNCHER:"
    print "  yazelix launch               - Launch yazelix via terminal"
    print "  yazelix start                - Start yazelix directly"
    print ""
    print "HELP:"
    print "  yazelix help                 - Show this help message"
    print ""
    print "Supported shells: bash, nushell, fish, zsh"
    print "=========================================="
}

