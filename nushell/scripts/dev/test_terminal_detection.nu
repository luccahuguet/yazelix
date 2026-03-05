#!/usr/bin/env nu
# Test script for terminal detection

use ../utils/terminal_launcher.nu *
use ../utils/constants.nu [SUPPORTED_TERMINALS, TERMINAL_METADATA]

def test_command_exists [] {
    print "🧪 Testing command_exists utility..."

    # Test with a command that should exist
    if (command_exists "nu") {
        print "  ✅ Detected 'nu' command"
    } else {
        print "  ❌ Failed to detect 'nu' command"
        return false
    }

    # Test with a command that shouldn't exist
    if not (command_exists "this-definitely-does-not-exist-12345") {
        print "  ✅ Correctly reported non-existent command"
    } else {
        print "  ❌ False positive on non-existent command"
        return false
    }

    true
}

def test_supported_terminals [] {
    print "🧪 Testing supported terminals list..."

    let expected_terminals = ["ghostty", "wezterm", "kitty", "alacritty", "foot"]

    for term in $expected_terminals {
        if ($term in $SUPPORTED_TERMINALS) {
            print $"  ✅ ($term) is supported"
        } else {
            print $"  ❌ ($term) missing from SUPPORTED_TERMINALS"
            return false
        }
    }

    true
}

def test_terminal_metadata [] {
    print "🧪 Testing terminal metadata..."

    for term in $SUPPORTED_TERMINALS {
        let meta = $TERMINAL_METADATA | get -o $term
        if ($meta | is-empty) {
            print $"  ❌ Missing metadata for ($term)"
            return false
        }

        # Check required fields
        if ($meta.name | is-empty) {
            print $"  ❌ Missing name for ($term)"
            return false
        }

        if ($meta.wrapper | is-empty) {
            print $"  ❌ Missing wrapper for ($term)"
            return false
        }

        print $"  ✅ ($term): ($meta.name) \(wrapper: ($meta.wrapper)\)"
    }

    true
}

def test_terminal_detection [] {
    print "🧪 Testing terminal detection..."

    # Try detecting with each supported terminal as preferred
    for term in $SUPPORTED_TERMINALS {
        let result = detect_terminal $term true

        if ($result | is-empty) {
            print $"  ℹ️  No terminal detected for preferred: ($term)"
        } else {
            print $"  ✅ Detected: ($result.name) \(command: ($result.command), wrapper: ($result.use_wrapper)\)"
        }
    }

    # Try detecting with wrapper preference off
    let result_no_wrapper = detect_terminal "ghostty" false
    if not ($result_no_wrapper | is-empty) {
        if $result_no_wrapper.use_wrapper {
            print "  ❌ Wrapper used when prefer_wrappers=false"
            return false
        } else {
            print "  ✅ Direct terminal used when prefer_wrappers=false"
        }
    }

    true
}

def test_config_path_resolution [] {
    print "🧪 Testing config path resolution..."

    let modes = ["yazelix", "user", "auto"]

    for term in $SUPPORTED_TERMINALS {
        for mode in $modes {
            try {
                let path = resolve_terminal_config $term $mode
                if ($path | is-empty) {
                    print $"  ❌ Empty path for ($term) in ($mode) mode"
                    return false
                }
                print $"  ✅ ($term) \(($mode)\): ($path)"
            } catch { |err|
                print $"  ❌ Error resolving ($term) \(($mode)\): ($err.msg)"
                return false
            }
        }
    }

    true
}

def test_launch_command_building [] {
    print "🧪 Testing launch command building..."

    # Test with a mock terminal info
    let terminal_info = {
        terminal: "ghostty",
        name: "Ghostty",
        command: "ghostty",
        use_wrapper: false
    }

    let config_path = "/tmp/test.conf"
    let working_dir = "/tmp"

    try {
        let launch_cmd = build_launch_command $terminal_info $config_path $working_dir

        if ($launch_cmd | is-empty) {
            print "  ❌ Empty launch command"
            return false
        }

        if not ($launch_cmd | str contains "ghostty") {
            print "  ❌ Launch command doesn't contain terminal name"
            return false
        }

        if not ($launch_cmd | str contains $config_path) {
            print "  ❌ Launch command doesn't contain config path"
            return false
        }

        if not ($launch_cmd | str contains $working_dir) {
            print "  ❌ Launch command doesn't contain working directory"
            return false
        }

        print $"  ✅ Launch command: ($launch_cmd)"
        true
    } catch { |err|
        print $"  ❌ Failed to build launch command: ($err.msg)"
        false
    }
}

def test_launch_command_detachment [] {
    print "🧪 Testing launch command detachment..."

    let terminal_info = {
        terminal: "ghostty",
        name: "Ghostty",
        command: "ghostty",
        use_wrapper: false
    }

    let cold_cmd = build_launch_command $terminal_info "/tmp/test.conf" "/tmp" true
    let warm_cmd = build_launch_command $terminal_info "/tmp/test.conf" "/tmp" false

    let required_segments = [
        "nohup "
        "env -u ZELLIJ"
        "-u ZELLIJ_SESSION_NAME"
        "-u ZELLIJ_PANE_ID"
        "-u ZELLIJ_TAB_NAME"
        "-u ZELLIJ_TAB_POSITION"
        "< /dev/null &"
    ]

    for segment in $required_segments {
        if not ($cold_cmd | str contains $segment) {
            print $"  ❌ Missing required detachment segment: ($segment)"
            return false
        }
    }

    if (which setsid | is-not-empty) and not ($cold_cmd | str contains "setsid ") {
        print "  ❌ setsid expected but missing from launch command"
        return false
    }

    if not ($cold_cmd | str contains "-u IN_YAZELIX_SHELL") or not ($cold_cmd | str contains "-u IN_NIX_SHELL") {
        print "  ❌ Cold launch should unset IN_YAZELIX_SHELL and IN_NIX_SHELL"
        return false
    }

    if ($warm_cmd | str contains "-u IN_YAZELIX_SHELL") or ($warm_cmd | str contains "-u IN_NIX_SHELL") {
        print "  ❌ Warm launch should not unset IN_YAZELIX_SHELL/IN_NIX_SHELL"
        return false
    }

    print "  ✅ Launch command detachment rules verified"
    true
}

def test_display_name [] {
    print "🧪 Testing display name generation..."

    let terminal_info_wrapper = {
        terminal: "ghostty",
        name: "Ghostty",
        command: "yazelix-ghostty",
        use_wrapper: true
    }

    let terminal_info_direct = {
        terminal: "ghostty",
        name: "Ghostty",
        command: "ghostty",
        use_wrapper: false
    }

    let name_wrapper = get_terminal_display_name $terminal_info_wrapper
    let name_direct = get_terminal_display_name $terminal_info_direct

    if ($name_wrapper | str contains "GPU acceleration") {
        print $"  ✅ Wrapper display name: ($name_wrapper)"
    } else {
        print $"  ❌ Wrapper display name missing GPU acceleration hint: ($name_wrapper)"
        return false
    }

    if not ($name_direct | str contains "GPU acceleration") {
        print $"  ✅ Direct display name: ($name_direct)"
    } else {
        print $"  ❌ Direct display name incorrectly mentions GPU acceleration: ($name_direct)"
        return false
    }

    true
}

def main [] {
    print "=== Testing Terminal Detection ==="
    print ""

    let results = [
        (test_command_exists),
        (test_supported_terminals),
        (test_terminal_metadata),
        (test_terminal_detection),
        (test_config_path_resolution),
        (test_launch_command_building),
        (test_launch_command_detachment),
        (test_display_name)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All terminal detection tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some tests failed \(($passed)/($total)\)"
        error make { msg: "terminal detection tests failed" }
    }
}
