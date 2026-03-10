#!/usr/bin/env nu
# Test script for yzx CLI commands

use ../core/yazelix.nu *

const clean_zellij_env_prefix = "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID -u ZELLIJ_TAB_NAME -u ZELLIJ_TAB_POSITION"

def test_yzx_help [] {
    print "🧪 Testing yzx help..."

    try {
        let output = (yzx | str join "\n")

        # Check for key elements in auto-generated help output
        let required_elements = [
            "Usage:",
            "Subcommands:",
            "yzx doctor",
            "yzx launch",
            "yzx dev"
        ]

        for element in $required_elements {
            if not ($output | str contains $element) {
                print $"  ❌ Missing element: ($element)"
                return false
            }
        }

        print "  ✅ Help output contains all required elements"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

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

def test_yzx_status_versions [] {
    print "🧪 Testing yzx status --versions..."

    try {
        let output = (
            ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx status --versions" | complete
        ).stdout

        # Check for core tools
        let expected_tools = [
            "zellij",
            "yazi",
            "helix",
            "nushell"
        ]

        for tool in $expected_tools {
            if not ($output | str contains $tool) {
                print $"  ❌ Missing tool: ($tool)"
                return false
            }
        }

        print "  ✅ Versions output contains expected tools"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_why [] {
    print "🧪 Testing yzx why..."

    try {
        # Just verify the command runs without error
        # (yzx why uses print, which doesn't produce pipeline output)
        yzx why | ignore
        print "  ✅ yzx why runs successfully"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_status_verbose [] {
    print "🧪 Testing yzx status --verbose..."

    try {
        let output = (
            ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx status --verbose" | complete
        ).stdout

        # Check for shell entries
        let shells = ["bash", "nushell", "fish", "zsh"]

        for shell in $shells {
            if not ($output | str contains $shell) {
                print $"  ⚠️  Missing shell in output: ($shell)"
            }
        }

        print "  ✅ Status verbose output generated"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_dev_exists [] {
    print "🧪 Testing yzx dev command exists..."

    try {
        # Just check that help mentions the dev command surface
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx dev") {
            print "  ✅ yzx dev command is documented in help"
            true
        } else {
            print "  ❌ yzx dev command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_doctor_exists [] {
    print "🧪 Testing yzx doctor command exists..."

    try {
        # Just check that help mentions the doctor command
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx doctor") {
            print "  ✅ yzx doctor command is documented in help"
            true
        } else {
            print "  ❌ yzx doctor command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_menu_exists [] {
    print "🧪 Testing yzx menu command exists..."

    try {
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx menu") {
            print "  ✅ yzx menu command is documented in help"
            true
        } else {
            print "  ❌ yzx menu command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_cwd_exists [] {
    print "🧪 Testing yzx cwd command exists..."

    try {
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx cwd") {
            print "  ✅ yzx cwd command is documented in help"
            true
        } else {
            print "  ❌ yzx cwd command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_cwd_requires_zellij [] {
    print "🧪 Testing yzx cwd outside Zellij..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx cwd .'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 1) and ($stdout | str contains "only works inside Zellij") {
            print "  ✅ yzx cwd fails clearly outside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_cwd_resolves_zoxide_query [] {
    print "🧪 Testing yzx cwd zoxide resolution..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; resolve_yzx_cwd_target yazelix" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "/home/lucca/.config/yazelix") {
            print "  ✅ yzx cwd resolves zoxide queries before updating the tab directory"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sidebar_yazi_state_path_normalization [] {
    print "🧪 Testing sidebar Yazi state path normalization..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; get_sidebar_yazi_state_path main 2" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str ends-with "main__terminal_2.txt") {
            print "  ✅ Sidebar Yazi state paths normalize pane ids consistently"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sidebar_state_plugin_generated [] {
    print "🧪 Testing generated Yazi init includes sidebar-state..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/setup/yazi_config_merger.nu *; let root = ($env.HOME | path join '.config' 'yazelix'); generate_merged_yazi_config $root --quiet; open --raw ($env.HOME | path join '.local' 'share' 'yazelix' 'configs' 'yazi' 'init.lua')" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains 'require("sidebar-state"):setup()') {
            print "  ✅ Generated Yazi init loads the sidebar-state core plugin"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sidebar_yazi_sync_skips_outside_zellij [] {
    print "🧪 Testing sidebar Yazi sync skips outside Zellij..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; sync_active_sidebar_yazi_to_directory . | to json -r'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and (($stdout | str contains '"status":"skipped"') and ($stdout | str contains '"reason":"outside_zellij"')) {
            print "  ✅ Sidebar Yazi sync stays non-fatal outside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_managed_editor_sync_skips_outside_zellij [] {
    print "🧪 Testing managed editor cwd sync skips outside Zellij..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; sync_managed_editor_cwd . | to json -r'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and (($stdout | str contains '"status":"skipped"') and ($stdout | str contains '"reason":"outside_zellij"')) {
            print "  ✅ Managed editor cwd sync stays non-fatal outside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_sponsor_exists [] {
    print "🧪 Testing yzx sponsor command exists..."

    try {
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx sponsor") {
            print "  ✅ yzx sponsor command is documented in help"
            true
        } else {
            print "  ❌ yzx sponsor command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_open_print [] {
    print "🧪 Testing yzx config open --print..."

    try {
        let output = (yzx config open --print | into string | str trim)

        if ($output | str ends-with ".toml") and ($output | path exists) {
            print $"  ✅ Config path resolved: ($output)"
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
        let hx_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config hx | columns | str join ','" | complete).stdout | str trim
        let yazi_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config yazi | columns | str join ','" | complete).stdout | str trim
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

def main [] {
    print "=== Testing yzx Commands ==="
    print ""

    let results = [
        (test_yzx_help),
        (test_yzx_status),
        (test_yzx_status_versions),
        (test_yzx_why),
        (test_yzx_status_verbose),
        (test_yzx_dev_exists),
        (test_yzx_doctor_exists),
        (test_yzx_menu_exists),
        (test_yzx_cwd_exists),
        (test_yzx_cwd_requires_zellij),
        (test_yzx_cwd_resolves_zoxide_query),
        (test_sidebar_yazi_state_path_normalization),
        (test_sidebar_state_plugin_generated),
        (test_sidebar_yazi_sync_skips_outside_zellij),
        (test_managed_editor_sync_skips_outside_zellij),
        (test_yzx_sponsor_exists),
        (test_yzx_config_view),
        (test_yzx_config_sections),
        (test_yzx_config_open_print)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx command tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some tests failed \(($passed)/($total)\)"
        error make { msg: "yzx command tests failed" }
    }
}
