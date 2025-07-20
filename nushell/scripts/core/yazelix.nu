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
    print "  yzx config_status [shell]      - Show status of all shell configurations"
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

# Show configuration status (canonical, no aliases)
export def "yzx config_status" [shell?: string] {
    if ($shell | is-empty) {
        show_config_status ~/.config/yazelix
    } else {
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

# Restart yazelix
export def "yzx restart" [] {
    print "Attempting to kill the current Zellij session..."
    let current_session = (zellij list-sessions | lines | where $it =~ 'current' | first | split row " " | first)
    let clean_session = ($current_session | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' '')
    print "Restarting Yazelix..."
    yzx launch
    print "Waiting for Zellij to shut down..."
    sleep 1sec
    if ($clean_session | is-empty) {
        print "No current Zellij session detected. Skipping kill step."
    } else {
        print $"Killing Zellij session: ($clean_session)"
        try { zellij kill-session $clean_session } catch { print $"Failed to kill session: ($clean_session)" }
    }
}

