#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/specs/floating_tui_panes.md

use ../yzx/popup.nu [resolve_yzx_popup_command resolve_yzx_popup_cwd]
use ../integrations/zellij_runtime_wrappers.nu [build_floating_wrapper_env_args get_floating_wrapper_env get_new_editor_pane_launch_env open_floating_runtime_wrapper]
use ../utils/config_parser.nu [parse_yazelix_config]
use ../../../configs/zellij/scripts/yzx_toggle_popup.nu [resolve_popup_toggle_action]

def write_executable_fixture_file [path: string, lines: list<string>] {
    $lines | str join "\n" | save --force --raw $path
    ^chmod +x $path
}

def setup_runtime_wrapper_fixture [label: string] {
    let tmpdir = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmpdir | path join "runtime")
    let integrations_dir = ($runtime_dir | path join "nushell" "scripts" "integrations")
    let wrapper_dir = ($runtime_dir | path join "nushell" "scripts" "zellij_wrappers")
    let fake_bin = ($tmpdir | path join "bin")
    let refresh_log = ($tmpdir | path join "refresh.log")
    let real_nu = (which nu | get -o 0.path)

    mkdir $integrations_dir
    mkdir $wrapper_dir
    mkdir $fake_bin
    "" | save --force --raw ($runtime_dir | path join "yazelix_default.toml")

    {
        tmpdir: $tmpdir
        runtime_dir: $runtime_dir
        integrations_dir: $integrations_dir
        wrapper_dir: $wrapper_dir
        fake_bin: $fake_bin
        refresh_log: $refresh_log
        real_nu: $real_nu
    }
}

def install_runtime_wrapper_script [fixture: record, script_name: string] {
    cp ($env.PWD | path join "nushell" "scripts" "zellij_wrappers" $script_name) ($fixture.wrapper_dir | path join $script_name)
}

# Defends: popup command resolution prefers the configured default program.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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

# Defends: popup cwd resolution prefers the workspace root.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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

# Defends: popup size parsing accepts valid percentages and rejects invalid ones.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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

# Regression: popup toggle wrapper surfaces permission denials instead of failing silently.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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

# Regression: popup toggle refreshes sidebar Yazi only after closing the popup back into the workspace.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_popup_toggle_wrapper_refreshes_sidebar_only_after_close [] {
    print "🧪 Testing popup toggle refreshes sidebar Yazi only after popup close..."

    let fixture = (setup_runtime_wrapper_fixture "yazelix_popup_toggle_refresh")

    let result = (try {
        write_executable_fixture_file ($fixture.fake_bin | path join "zellij") [
            "#!/bin/sh"
            "printf '%s\\n' \"$YAZELIX_TEST_POPUP_RESULT\""
            "exit 0"
        ]

        [
            "export def refresh_active_sidebar_yazi [] {"
            "    if ($env.YAZELIX_TEST_REFRESH_LOG | path exists) {"
            "        'refresh' | save --append --raw $env.YAZELIX_TEST_REFRESH_LOG"
            "    } else {"
            "        'refresh' | save --force --raw $env.YAZELIX_TEST_REFRESH_LOG"
            "    }"
            "    {status: 'ok'}"
            "}"
        ] | str join "\n" | save --force --raw ($fixture.integrations_dir | path join "yazi.nu")
        install_runtime_wrapper_script $fixture "popup_refresh_active_sidebar_yazi.nu"

        let wrapper_script = ($env.PWD | path join "configs" "zellij" "scripts" "yzx_toggle_popup.nu")
        let closed_output = (with-env {
            PATH: ($env.PATH | prepend $fixture.fake_bin)
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_NU_BIN: $fixture.real_nu
            YAZELIX_TEST_REFRESH_LOG: $fixture.refresh_log
            YAZELIX_TEST_POPUP_RESULT: "closed"
        } {
            ^nu $wrapper_script | complete
        })
        let closed_refresh = if ($fixture.refresh_log | path exists) {
            open --raw $fixture.refresh_log | str trim
        } else {
            ""
        }

        rm -f $fixture.refresh_log

        let focused_output = (with-env {
            PATH: ($env.PATH | prepend $fixture.fake_bin)
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_NU_BIN: $fixture.real_nu
            YAZELIX_TEST_REFRESH_LOG: $fixture.refresh_log
            YAZELIX_TEST_POPUP_RESULT: "focused"
        } {
            ^nu $wrapper_script | complete
        })
        let focused_refresh_exists = ($fixture.refresh_log | path exists)

        if (
            ($closed_output.exit_code == 0)
            and ($closed_refresh == "refresh")
            and ($focused_output.exit_code == 0)
            and (not $focused_refresh_exists)
        ) {
            print "  ✅ popup toggle now refreshes sidebar Yazi only after closing the popup"
            true
        } else {
            print $"  ❌ Unexpected popup-toggle refresh behavior: closed_exit=($closed_output.exit_code) closed_refresh=($closed_refresh | to json -r) focused_exit=($focused_output.exit_code) focused_refresh_exists=($focused_refresh_exists) closed_stderr=(($closed_output.stderr | str trim)) focused_stderr=(($focused_output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: popup wrappers should fall back to the runtime env contract when the current shell has no wrapper env to reuse.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_popup_wrapper_env_falls_back_to_runtime_env [] {
    print "🧪 Testing popup wrappers fall back to the runtime env contract when wrapper vars are absent..."

    try {
        let tmpdir = (^mktemp -d /tmp/yazelix_popup_runtime_env_XXXXXX | str trim)
        mut success = false

        try {
            let runtime_dir = ($env.PWD | path expand)
            let profile_bin = ($tmpdir | path join "profile" "bin")
            let config_path = ($tmpdir | path join "yazelix.toml")
            mkdir $profile_bin
            "" | save --force --raw ($profile_bin | path join "nvim")
            ^chmod +x ($profile_bin | path join "nvim")

            [
                "[editor]"
                "command = \"nvim\""
            ] | str join "\n" | save --force --raw $config_path

            let resolved = (with-env {
                YAZELIX_CONFIG_OVERRIDE: $config_path
                YAZELIX_RUNTIME_DIR: $runtime_dir
                PATH: $"($profile_bin):/usr/bin"
                EDITOR: ""
                YAZELIX_MANAGED_HELIX_BINARY: ""
                YAZELIX_NU_BIN: ""
                YAZELIX_TERMINAL_CONFIG_MODE: ""
            } {
                {
                    wrapper: (get_floating_wrapper_env)
                    pane: (get_new_editor_pane_launch_env "1234")
                }
            })

            let wrapper_env = $resolved.wrapper
            let pane_env = $resolved.pane
            let raw_path = ($wrapper_env.PATH? | default [])
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
            let runtime_bin = ($runtime_dir | path join "bin")
            let runtime_bin_ok = if ($runtime_bin | path exists) {
                $path_entries | any {|entry| $entry == $runtime_bin }
            } else {
                true
            }

            if (
                (($wrapper_env.EDITOR? | default "") == "nvim")
                and (($pane_env.EDITOR? | default "") == "nvim")
                and (($pane_env.YAZI_ID? | default "") == "1234")
                and ($path_entries | any {|entry| $entry == $profile_bin })
                and $runtime_bin_ok
                and (not ($wrapper_env | columns | any {|column| $column == "YAZELIX_NU_BIN" }))
                and (not ($wrapper_env | columns | any {|column| $column == "YAZELIX_TERMINAL_CONFIG_MODE" }))
            ) {
                print "  ✅ popup wrappers now derive their fallback env from the trimmed runtime contract and still tag new editor panes with YAZI_ID"
                $success = true
            } else {
                print $"  ❌ Unexpected popup wrapper env: wrapper=($wrapper_env | to json -r) pane=($pane_env | to json -r)"
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

# Regression: popup wrappers serialize PATH lists into real env strings for zellij run.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_popup_wrapper_serializes_path_list_for_env_command [] {
    print "🧪 Testing popup wrappers serialize PATH lists into real env strings..."

    try {
        let result = (build_floating_wrapper_env_args {
            PATH: ["/tmp/profile/bin", "/usr/bin", "/bin"]
            EDITOR: "/tmp/profile/bin/hx"
        })
        let path_arg = ($result | where {|entry| $entry | str starts-with "PATH=" } | first)
        let editor_arg = ($result | where {|entry| $entry | str starts-with "EDITOR=" } | first)

        if (
            ($path_arg == $"PATH=/tmp/profile/bin(char esep)/usr/bin(char esep)/bin")
            and ($editor_arg == "EDITOR=/tmp/profile/bin/hx")
            and (not ($path_arg | str contains "["))
            and (not ($path_arg | str contains "]"))
        ) {
            print "  ✅ popup wrapper env args serialize PATH lists correctly for zellij run"
            true
        } else {
            print $"  ❌ Unexpected popup wrapper env args: ($result | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Regression: popup wrappers must fall back to a host-provided Nushell binary when the runtime root does not ship bin/nu.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_popup_wrapper_falls_back_to_host_nu_without_runtime_owned_nu [] {
    print "🧪 Testing popup wrappers fall back to host-provided nu without a runtime-owned bin/nu..."

    let fixture = (setup_runtime_wrapper_fixture "yazelix_popup_host_nu")
    let zellij_log = ($fixture.tmpdir | path join "zellij_args.log")

    let result = (try {
        let wrapper_root = ($fixture.runtime_dir | path join "configs" "zellij" "scripts")
        let wrapper_path = ($wrapper_root | path join "proof_popup.nu")
        let fake_path_nu = ($fixture.fake_bin | path join "nu")
        let host_nu_candidates = [
            $fake_path_nu
            ($nu.current-exe? | default "")
            (which nu | get -o 0.path | default "")
        ] | where {|candidate| ($candidate | str trim | is-not-empty) } | uniq

        mkdir $wrapper_root
        "" | save --force --raw $wrapper_path

        write_executable_fixture_file ($fixture.fake_bin | path join "nu") [
            "#!/bin/sh"
            "exit 0"
        ]
        write_executable_fixture_file ($fixture.fake_bin | path join "zellij") [
            "#!/bin/sh"
            ": > \"$YAZELIX_TEST_ZELLIJ_LOG\""
            "for arg in \"$@\"; do"
            "  printf '%s\\n' \"$arg\" >> \"$YAZELIX_TEST_ZELLIJ_LOG\""
            "done"
            "exit 0"
        ]

        with-env {
            PATH: ($env.PATH | prepend $fixture.fake_bin)
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_TEST_ZELLIJ_LOG: $zellij_log
        } {
            open_floating_runtime_wrapper "proof_popup" "proof_popup.nu" "/tmp/workspace"
        }

        let invocation = (open --raw $zellij_log | lines)

        if (
            ($invocation | any {|arg| $arg in $host_nu_candidates })
            and ($invocation | any {|arg| $arg == $wrapper_path })
            and not ($invocation | any {|arg| $arg == ($fixture.runtime_dir | path join "bin" "nu") })
        ) {
            print "  ✅ popup wrappers now fall back to host-provided nu when the runtime root does not ship one"
            true
        } else {
            print $"  ❌ Unexpected popup wrapper invocation: ($invocation | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

export def run_popup_canonical_tests [] {
    [
        (test_popup_command_prefers_configured_default)
        (test_popup_cwd_prefers_workspace_root)
        (test_popup_size_parser_accepts_valid_and_rejects_invalid_percentages)
        (test_popup_toggle_wrapper_surfaces_permission_denials)
        (test_popup_toggle_wrapper_refreshes_sidebar_only_after_close)
        (test_popup_wrapper_env_falls_back_to_runtime_env)
        (test_popup_wrapper_serializes_path_list_for_env_command)
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
