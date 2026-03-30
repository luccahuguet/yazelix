#!/usr/bin/env nu

use ../yzx/popup.nu [resolve_yzx_popup_command resolve_yzx_popup_cwd]
use ../utils/config_parser.nu [parse_yazelix_config]
use ../../../configs/zellij/scripts/yzx_toggle_popup.nu [resolve_popup_toggle_action]
use ./test_yzx_helpers.nu [repo_path]

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

def test_popup_launch_uses_shared_floating_runner [] {
    print "🧪 Testing yzx popup routes through the shared floating Zellij runner..."

    let tmpdir = (^mktemp -d /tmp/yazelix_popup_runner_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let fake_bin = ($tmpdir | path join "bin")
        let log_path = ($tmpdir | path join "zellij.log")
        let config_path = ($tmpdir | path join "yazelix.toml")
        mkdir $fake_home
        mkdir $fake_bin

        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        let zellij_script = ($fake_bin | path join "zellij")
        [
            "#!/bin/sh"
            $"printf '%s\\n' \"$@\" > \"($log_path)\""
        ] | str join "\n" | save --force --raw $zellij_script
        chmod +x $zellij_script

        [
            "[zellij]"
            "popup_program = [\"lazygit\"]"
            "popup_width_percent = 61"
            "popup_height_percent = 73"
        ] | str join "\n" | save --force --raw $config_path

        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let output = (with-env {
            HOME: $fake_home
            PATH: $fake_bin
            ZELLIJ: "0"
            YAZELIX_CONFIG_OVERRIDE: $config_path
            YAZELIX_RUNTIME_DIR: (repo_path)
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx popup" | complete
        })

        let logged = (open --raw $log_path | str trim)

        if (
            ($output.exit_code == 0)
            and ($logged | str contains "--name")
            and ($logged | str contains "yzx_popup")
            and ($logged | str contains "--floating")
            and ($logged | str contains "61%")
            and ($logged | str contains "73%")
            and ($logged | str contains "yzx_popup_program.nu")
        ) {
            print "  ✅ yzx popup launches through the shared floating runner"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) logged=($logged) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
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

def test_popup_wrapper_runs_inline_without_pane_id [] {
    print "🧪 Testing popup wrapper panes run inline even without ZELLIJ_PANE_ID..."

    let tmpdir = (^mktemp -d /tmp/yazelix_popup_wrapper_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let marker_path = ($tmpdir | path join "wrapper_ok")
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")

        [
            "[zellij]"
            $"popup_program = [\"sh\", \"-lc\", \"printf inline > ($marker_path)\"]"
        ] | str join "\n" | save --force --raw $config_path

        let output = (with-env {
            HOME: $env.HOME
            PATH: $env.PATH
            ZELLIJ: "0"
            YAZELIX_POPUP_PANE: "true"
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx popup" | complete
        })

        if (($output.exit_code == 0) and ($marker_path | path exists)) {
            print "  ✅ popup wrapper panes execute the configured program inline without a pane id"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_popup_toggle_wrapper_opens_when_popup_is_missing [] {
    print "🧪 Testing popup toggle wrapper opens when the popup is missing..."

    try {
        let result = (resolve_popup_toggle_action "missing")

        if $result == { action: "open" } {
            print "  ✅ popup toggle wrapper opens the popup when none exists"
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

def test_popup_toggle_wrapper_treats_ok_as_handled [] {
    print "🧪 Testing popup toggle wrapper treats an existing popup toggle as handled..."

    try {
        let result = (resolve_popup_toggle_action "ok")

        if $result == { action: "handled" } {
            print "  ✅ popup toggle wrapper leaves popup focus/close behavior to the plugin when it already exists"
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
