#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/specs/floating_tui_panes.md

use ../yzx/popup.nu [resolve_yzx_popup_command resolve_yzx_popup_cwd]
use ../integrations/zellij.nu [get_floating_wrapper_env]
use ../utils/config_parser.nu [parse_yazelix_config]
use ../../../configs/zellij/scripts/yzx_toggle_popup.nu [resolve_popup_toggle_action]

def test_popup_command_prefers_configured_default [] {
    print "🧪 Testing yzx popup uses the configured popup_program by default..."

    try {
        let configured_program = ["lazygit"]
        let result = (resolve_yzx_popup_command $configured_program)

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

def test_popup_size_parser_accepts_valid_and_rejects_invalid_percentages [] {
    print "🧪 Testing popup size config accepts valid percentages and rejects invalid ones..."

    let cases = [
        {
            label: "valid"
            raw_toml: ([
                "[zellij]"
                "popup_program = [\"lazygit\"]"
                "popup_width_percent = 1"
                "popup_height_percent = 100"
            ] | str join "\n")
            expect_ok: true
        }
        {
            label: "invalid"
            raw_toml: ([
                "[zellij]"
                "popup_program = [\"lazygit\"]"
                "popup_width_percent = 0"
                "popup_height_percent = 101"
            ] | str join "\n")
            expect_ok: false
        }
    ]

    try {
        let failures = (
            $cases
            | each {|case|
                let tmpdir = (^mktemp -d $"/tmp/yazelix_popup_size_($case.label)_XXXXXX" | str trim)

                try {
                    let config_path = ($tmpdir | path join "yazelix.toml")
                    $case.raw_toml | save --force --raw $config_path

                    let parse_result = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
                        try {
                            let parsed = (parse_yazelix_config)
                            { ok: true, parsed: $parsed }
                        } catch {|err|
                            { ok: false, msg: $err.msg }
                        }
                    })

                    if $case.expect_ok {
                        if (
                            $parse_result.ok
                            and ($parse_result.parsed.popup_width_percent == 1)
                            and ($parse_result.parsed.popup_height_percent == 100)
                        ) {
                            null
                        } else {
                            {
                                label: $case.label
                                result: $parse_result
                            }
                        }
                    } else if (not $parse_result.ok) and ($parse_result.msg | str contains "zellij.popup_width_percent") {
                        null
                    } else {
                        {
                            label: $case.label
                            result: $parse_result
                        }
                    }
                } finally {
                    rm -rf $tmpdir
                }
            }
            | where {|item| $item != null}
        )

        if ($failures | is-empty) {
            print "  ✅ popup size config accepts the full valid range and fails fast on out-of-range values"
            true
        } else {
            print $"  ❌ Unexpected popup size parser failures: ($failures | to json -r)"
            false
        }
    } catch {|err|
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

def test_popup_wrapper_uses_canonical_editor_for_current_profile [] {
    print "🧪 Testing popup wrappers derive EDITOR from the canonical launch env, not a stale shell value..."

    try {
        let tmpdir = (^mktemp -d /tmp/yazelix_popup_env_XXXXXX | str trim)
        mut success = false

        try {
            let profile_path = ($tmpdir | path join "profile")
            let profile_bin = ($profile_path | path join "bin")
            let profile_nvim = ($profile_bin | path join "nvim")
            mkdir $profile_bin
            "" | save --force --raw $profile_nvim
            ^chmod +x $profile_nvim

            let config_path = ($tmpdir | path join "yazelix.toml")
            [
                "[editor]"
                "command = \"nvim\""
            ] | str join "\n" | save --force --raw $config_path

            let result = (with-env {
                YAZELIX_CONFIG_OVERRIDE: $config_path
                DEVENV_PROFILE: $profile_path
                YAZELIX_RUNTIME_DIR: $env.PWD
                YAZELIX_DIR: $env.PWD
                PATH: $"($profile_bin):/usr/bin"
                EDITOR: "/tmp/wrong-editor"
            } {
                get_floating_wrapper_env
            })
            let raw_path = ($result.PATH? | default [])
            let path_entries = if (($raw_path | describe) | str starts-with "list") {
                $raw_path | each {|entry| $entry | into string }
            } else {
                let path_text = ($raw_path | into string | str trim)
                if ($path_text | is-empty) {
                    []
                } else {
                    $path_text | split row (char esep)
                }
            }

            let conditions = [
                (($result.EDITOR? | default "") == $profile_nvim)
                (($result.DEVENV_PROFILE? | default "") == $profile_path)
                ($path_entries | any {|entry| $entry == $profile_bin })
            ]

            if ($conditions | all {|item| $item }) {
                print "  ✅ popup wrappers resolve EDITOR from the current Yazelix profile instead of a stale shell value"
                $success = true
            } else {
                print $"  ❌ Unexpected popup wrapper env: ($result | to json -r)"
                $success = false
            }
        } finally {
            rm -rf $tmpdir
        }

        $success
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_popup_canonical_tests [] {
    [
        # Defends: popup command resolution prefers the configured default program.
        (test_popup_command_prefers_configured_default)
        # Defends: popup cwd resolution prefers the workspace root.
        (test_popup_cwd_prefers_workspace_root)
        # Defends: popup size parsing accepts valid percentages and rejects invalid ones.
        (test_popup_size_parser_accepts_valid_and_rejects_invalid_percentages)
        # Regression: popup toggle wrapper surfaces permission denials instead of failing silently.
        (test_popup_toggle_wrapper_surfaces_permission_denials)
        # Regression: popup wrappers use the canonical editor for the current launch profile.
        (test_popup_wrapper_uses_canonical_editor_for_current_profile)
    ]
}

export def run_popup_tests [] {
    run_popup_canonical_tests
}

def main [] {
    let results = (run_popup_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx popup tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some yzx popup tests failed \(($passed)/($total)\)"
        error make { msg: "yzx popup tests failed" }
    }
}
