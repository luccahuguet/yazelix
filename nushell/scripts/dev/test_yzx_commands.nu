#!/usr/bin/env nu
# Test script for yzx CLI commands

use ../core/yazelix.nu *

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
            "yzx test"
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

def test_yzx_test_exists [] {
    print "🧪 Testing yzx test command exists..."

    try {
        # Just check that help mentions the test command
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx test") {
            print "  ✅ yzx test command is documented in help"
            true
        } else {
            print "  ❌ yzx test command not found in help"
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

def main [] {
    print "=== Testing yzx Commands ==="
    print ""

    let results = [
        (test_yzx_help),
        (test_yzx_status),
        (test_yzx_status_versions),
        (test_yzx_why),
        (test_yzx_status_verbose),
        (test_yzx_test_exists),
        (test_yzx_doctor_exists),
        (test_yzx_menu_exists),
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
