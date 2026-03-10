#!/usr/bin/env nu
# Test script for configuration parser

use ../utils/config_parser.nu parse_yazelix_config

def test_config_exists [] {
    print "🧪 Testing config file detection..."

    let yazelix_dir = "~/.config/yazelix" | path expand
    let config_file = ($yazelix_dir | path join "yazelix.toml")
    let default_config = ($yazelix_dir | path join "yazelix_default.toml")
    let legacy_config = ($yazelix_dir | path join "yazelix.nix")

    if ($config_file | path exists) {
        print $"  ✅ yazelix.toml exists at ($config_file)"
    } else if ($default_config | path exists) {
        print $"  ✅ yazelix_default.toml exists at ($default_config)"
    } else if ($legacy_config | path exists) {
        print $"  ⚠️  Legacy yazelix.nix exists at ($legacy_config)"
    } else {
        print "  ❌ No config file found"
        return false
    }

    true
}

def test_parse_config [] {
    print "🧪 Testing config parsing..."

    try {
        let config = parse_yazelix_config

        # Check that we got a config object
        if ($config | is-empty) {
            print "  ❌ Config is empty"
            return false
        }

        print "  ✅ Config parsed successfully"

        # Validate expected fields exist
        let required_fields = [
            "persistent_sessions",
            "session_name",
            "zellij_default_mode",
            "terminals",
            "default_shell",
            "helix_mode",
            "config_file"
        ]

        for field in $required_fields {
            if ($config | get -o $field | is-empty) {
                print $"  ❌ Missing required field: ($field)"
                return false
            }
        }

        print "  ✅ All required fields present"
        true
    } catch { |err|
        print $"  ❌ Parse failed: ($err.msg)"
        false
    }
}

def test_config_values [] {
    print "🧪 Testing config values..."

    try {
        let config = parse_yazelix_config

        # Check that values are reasonable
        let valid_shells = ["nu", "bash", "fish", "zsh"]
        if not ($config.default_shell in $valid_shells) {
            print $"  ⚠️  Unusual shell: ($config.default_shell)"
        } else {
            print $"  ✅ Valid default_shell: ($config.default_shell)"
        }

        let valid_terminals = ["ghostty", "wezterm", "kitty", "alacritty", "foot"]
        let terminals = ($config.terminals? | default [])
        if ($terminals | is-empty) {
            print "  ❌ terminals is empty"
            return false
        }
        if (not ($terminals | all {|t| $t in $valid_terminals })) {
            print $"  ⚠️  Unusual terminals: (($terminals | str join ', '))"
        } else {
            print $"  ✅ Valid terminals: (($terminals | str join ', '))"
        }

        let valid_helix_modes = ["release", "source"]
        if not ($config.helix_mode in $valid_helix_modes) {
            print $"  ⚠️  Unusual helix_mode: ($config.helix_mode)"
        } else {
            print $"  ✅ Valid helix_mode: ($config.helix_mode)"
        }

        let valid_bool = ["true", "false"]
        if not ($config.persistent_sessions in $valid_bool) {
            print $"  ❌ Invalid persistent_sessions: ($config.persistent_sessions)"
            return false
        } else {
            print $"  ✅ Valid persistent_sessions: ($config.persistent_sessions)"
        }

        let valid_zellij_modes = ["normal", "locked"]
        if not ($config.zellij_default_mode in $valid_zellij_modes) {
            print $"  ❌ Invalid zellij_default_mode: ($config.zellij_default_mode)"
            return false
        } else {
            print $"  ✅ Valid zellij_default_mode: ($config.zellij_default_mode)"
        }

        true
    } catch { |err|
        print $"  ❌ Value validation failed: ($err.msg)"
        false
    }
}

def test_config_file_path [] {
    print "🧪 Testing config_file path..."

    try {
        let config = parse_yazelix_config

        if not ($config.config_file | path exists) {
            print $"  ❌ config_file path doesn't exist: ($config.config_file)"
            return false
        }

        print $"  ✅ config_file exists: ($config.config_file)"
        true
    } catch { |err|
        print $"  ❌ Failed: ($err.msg)"
        false
    }
}

def main [] {
    print "=== Testing Config Parser ==="
    print ""

    let results = [
        (test_config_exists),
        (test_parse_config),
        (test_config_values),
        (test_config_file_path)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All config parser tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some tests failed \(($passed)/($total)\)"
    }
}
