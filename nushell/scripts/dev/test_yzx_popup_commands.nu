#!/usr/bin/env nu

use ../yzx/popup.nu [resolve_yzx_popup_command resolve_yzx_popup_cwd]
use ../utils/config_parser.nu [parse_yazelix_config]
use ../../../configs/zellij/scripts/yzx_toggle_popup.nu [resolve_popup_toggle_action]

def test_popup_command_prefers_configured_default [] {
    print "🧪 Testing yzx popup uses the configured popup_program by default..."

    try {
        let result = (resolve_yzx_popup_command ["lazygit"])

        if $result == ["lazygit"] {
            print "  ✅ yzx popup defaults to the configured popup_program"
            true
        } else {
            print $"  ❌ Unexpected result: ($result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_popup_cwd_prefers_workspace_root [] {
    print "🧪 Testing yzx popup uses the tab workspace root for cwd..."

    try {
        let result = (resolve_yzx_popup_cwd "/tmp/workspace" "/tmp/current")

        if $result == "/tmp/workspace" {
            print "  ✅ yzx popup prefers the tab workspace root over the incidental shell cwd"
            true
        } else {
            print $"  ❌ Unexpected result: ($result)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_popup_size_parser_accepts_valid_percentages [] {
    print "🧪 Testing popup size config accepts percentages from 1 to 100..."

    let tmpdir = (^mktemp -d /tmp/yazelix_popup_size_valid_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")

        [
            "[zellij]"
            "popup_program = [\"lazygit\"]"
            "popup_width_percent = 1"
            "popup_height_percent = 100"
        ] | str join "\n" | save --force --raw $config_path

        let parsed = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            parse_yazelix_config
        })

        if ($parsed.popup_width_percent == 1) and ($parsed.popup_height_percent == 100) {
            print "  ✅ popup size config accepts the full 1..100 range"
            true
        } else {
            print $"  ❌ Unexpected parsed values: width=($parsed.popup_width_percent) height=($parsed.popup_height_percent)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_popup_size_parser_rejects_out_of_range_percentages [] {
    print "🧪 Testing popup size config rejects percentages outside 1 to 100..."

    let tmpdir = (^mktemp -d /tmp/yazelix_popup_size_invalid_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")

        [
            "[zellij]"
            "popup_program = [\"lazygit\"]"
            "popup_width_percent = 0"
            "popup_height_percent = 101"
        ] | str join "\n" | save --force --raw $config_path

        let parse_result = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            try {
                parse_yazelix_config
                { ok: true }
            } catch { |err|
                { ok: false, msg: $err.msg }
            }
        })

        if (not $parse_result.ok) and ($parse_result.msg | str contains "zellij.popup_width_percent") {
            print "  ✅ popup size config fails fast on out-of-range values"
            true
        } else {
            print $"  ❌ Unexpected parser result: ($parse_result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_popup_toggle_wrapper_surfaces_permission_denials [] {
    print "🧪 Testing popup toggle wrapper surfaces popup-plugin permission denials..."

    try {
        let result = (resolve_popup_toggle_action "permissions_denied")

        if ($result.action == "error") and ($result.message | str contains "popup-runner plugin permissions") {
            print "  ✅ popup toggle wrapper reports popup-plugin permission denials clearly"
            true
        } else {
            print $"  ❌ Unexpected result: ($result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_popup_canonical_tests [] {
    [
        (test_popup_command_prefers_configured_default)
        (test_popup_cwd_prefers_workspace_root)
        (test_popup_size_parser_accepts_valid_percentages)
        (test_popup_size_parser_rejects_out_of_range_percentages)
        (test_popup_toggle_wrapper_surfaces_permission_denials)
    ]
}

export def run_popup_tests [] {
    run_popup_canonical_tests
}

def main [] {
    let results = (run_popup_tests)
    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx popup tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some yzx popup tests failed \(($passed)/($total)\)"
        error make { msg: "yzx popup tests failed" }
    }
}
