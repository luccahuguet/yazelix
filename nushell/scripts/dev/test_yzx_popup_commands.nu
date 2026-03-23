#!/usr/bin/env nu

use ../yzx/popup.nu [resolve_yzx_popup_command resolve_yzx_popup_cwd]
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

def test_popup_command_allows_inline_override [] {
    print "🧪 Testing yzx popup allows an inline command override..."

    try {
        let result = (resolve_yzx_popup_command ["lazygit"] "claude-code" "--continue")

        if $result == ["claude-code", "--continue"] {
            print "  ✅ yzx popup overrides the configured popup program when explicit args are passed"
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
        let result = (resolve_popup_toggle_action false)

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
        let result = (resolve_popup_toggle_action true "ok")

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
        let result = (resolve_popup_toggle_action true "permissions_denied")

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

export def run_popup_tests [] {
    [
        (test_popup_command_prefers_configured_default)
        (test_popup_command_allows_inline_override)
        (test_popup_cwd_prefers_workspace_root)
        (test_popup_launch_uses_shared_floating_runner)
        (test_popup_wrapper_runs_inline_without_pane_id)
        (test_popup_toggle_wrapper_opens_when_popup_is_missing)
        (test_popup_toggle_wrapper_treats_ok_as_handled)
        (test_popup_toggle_wrapper_surfaces_permission_denials)
    ]
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
