#!/usr/bin/env nu
# Test script for yzx CLI commands

use ../core/yazelix.nu *

def test_yzx_help [] {
    print "🧪 Testing yzx help..."

    try {
        let output = (yzx help | str join "\n")

        # Check for key sections in help output
        let required_sections = [
            "Yazelix Command Suite",
            "DIAGNOSTICS",
            "VERSION AND SYSTEM",
            "LAUNCHER",
            "HELP"
        ]

        for section in $required_sections {
            if not ($output | str contains $section) {
                print $"  ❌ Missing section: ($section)"
                return false
            }
        }

        print "  ✅ Help output contains all required sections"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_info [] {
    print "🧪 Testing yzx info..."

    try {
        let output = (yzx info | str join "\n")

        # Check for key information
        let required_fields = [
            "Version",
            "Description",
            "Directory",
            "Default Shell",
            "Preferred Terminal",
            "Helix Mode"
        ]

        for field in $required_fields {
            if not ($output | str contains $field) {
                print $"  ❌ Missing field: ($field)"
                return false
            }
        }

        print "  ✅ Info output contains all required fields"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_versions [] {
    print "🧪 Testing yzx versions..."

    try {
        let output = (yzx versions | str join "\n")

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
        let output = (yzx why | str join "\n")

        # Check for key phrases in elevator pitch
        let key_phrases = [
            "Yazelix",
            "terminal IDE",
            "Yazi",
            "Zellij",
            "Helix"
        ]

        for phrase in $key_phrases {
            if not ($output | str contains $phrase) {
                print $"  ❌ Missing phrase: ($phrase)"
                return false
            }
        }

        print "  ✅ Why output contains elevator pitch"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_status [] {
    print "🧪 Testing yzx config_status..."

    try {
        # Test without arguments (shows all shells)
        let output = (yzx config_status | str join "\n")

        # Check for shell entries
        let shells = ["bash", "nushell", "fish", "zsh"]

        for shell in $shells {
            if not ($output | str contains $shell) {
                print $"  ⚠️  Missing shell in output: ($shell)"
            }
        }

        print "  ✅ Config status output generated"
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
        let output = (yzx help | str join "\n")

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
        let output = (yzx help | str join "\n")

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

def main [] {
    print "=== Testing yzx Commands ==="
    print ""

    let results = [
        (test_yzx_help),
        (test_yzx_info),
        (test_yzx_versions),
        (test_yzx_why),
        (test_yzx_config_status),
        (test_yzx_test_exists),
        (test_yzx_doctor_exists)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx command tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some tests failed \(($passed)/($total)\)"
    }
}
