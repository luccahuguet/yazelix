#!/usr/bin/env nu

use ../core/yazelix.nu *

def test_yzx_status [] {
    print "🧪 Testing yzx status..."

    try {
        yzx status | ignore
        print "  ✅ yzx status runs successfully"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_view [] {
    print "🧪 Testing yzx config..."

    try {
        let output = (
            ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config | columns | str join ','" | complete
        ).stdout | str trim

        if ($output | str contains "core") and ($output | str contains "terminal") and not ($output | str contains "packs") {
            print "  ✅ yzx config hides packs by default"
            true
        } else {
            print $"  ❌ Unexpected output: ($output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_sections [] {
    print "🧪 Testing yzx config section views..."

    try {
        ^nu -c 'use ~/.config/yazelix/nushell/scripts/setup/yazi_config_merger.nu *; let root = ($env.HOME | path join ".config" "yazelix"); generate_merged_yazi_config $root --quiet | ignore' | complete | ignore
        let hx_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config hx | columns | str join ','" | complete).stdout | str trim
        let yazi_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config yazi | columns | str join ','" | complete).stdout | str trim
        if (which zellij | is-empty) {
            if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") {
                print "  ℹ️  Skipping zellij config section check because zellij is not available"
                print "  ✅ yzx config section commands return focused sections"
                return true
            }
        }

        ^nu -c 'use ~/.config/yazelix/nushell/scripts/setup/zellij_config_merger.nu *; let root = ($env.HOME | path join ".config" "yazelix"); generate_merged_zellij_config $root | ignore' | complete | ignore
        let zellij_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config zellij" | complete).stdout | str trim

        if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") and ($zellij_output | str contains "default_layout") {
            print "  ✅ yzx config section commands return focused sections"
            true
        } else {
            print $"  ❌ Unexpected section output: hx=($hx_output) yazi=($yazi_output) zellij=($zellij_output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_core_tests [] {
    [
        (test_yzx_status)
        (test_yzx_config_view)
        (test_yzx_config_sections)
    ]
}
