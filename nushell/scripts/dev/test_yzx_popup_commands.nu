#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/specs/floating_tui_panes.md

use ../yzx/popup.nu [resolve_yzx_popup_command resolve_yzx_popup_cwd]
use ../integrations/zellij_runtime_wrappers.nu [build_floating_wrapper_env_args get_floating_wrapper_env get_new_editor_pane_launch_env open_floating_runtime_script]
use ../utils/config_parser.nu [parse_yazelix_config]

def write_executable_fixture_file [path: string, lines: list<string>] {
    $lines | str join "\n" | save --force --raw $path
    ^chmod +x $path
}

def setup_runtime_wrapper_fixture [label: string] {
    let tmpdir = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmpdir | path join "runtime")
    let integrations_dir = ($runtime_dir | path join "nushell" "scripts" "integrations")
    let yzx_dir = ($runtime_dir | path join "nushell" "scripts" "yzx")
    let wrapper_dir = ($runtime_dir | path join "nushell" "scripts" "zellij_wrappers")
    let fake_bin = ($tmpdir | path join "bin")
    let refresh_log = ($tmpdir | path join "refresh.log")
    let real_nu = (which nu | get -o 0.path)

    mkdir $integrations_dir
    mkdir $yzx_dir
    mkdir $wrapper_dir
    mkdir $fake_bin
    cp ($env.PWD | path join "yazelix_default.toml") ($runtime_dir | path join "yazelix_default.toml")
    cp ($env.PWD | path join ".taplo.toml") ($runtime_dir | path join ".taplo.toml")
    ^ln -s ($env.PWD | path join "config_metadata") ($runtime_dir | path join "config_metadata")

    {
        tmpdir: $tmpdir
        runtime_dir: $runtime_dir
        integrations_dir: $integrations_dir
        yzx_dir: $yzx_dir
        wrapper_dir: $wrapper_dir
        fake_bin: $fake_bin
        refresh_log: $refresh_log
        real_nu: $real_nu
    }
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

# Regression: the runtime popup pane wrapper must run the resolved argv directly, refresh the sidebar, and close its own transient pane after success.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_popup_program_wrapper_runs_resolved_argv_directly [] {
    print "🧪 Testing popup program wrapper runs the resolved popup argv directly and closes its transient pane..."

    let fixture = (setup_runtime_wrapper_fixture "yazelix_popup_direct_wrapper")

    let result = (try {
        write_executable_fixture_file ($fixture.fake_bin | path join "zellij") [
            "#!/bin/sh"
            "if [ -f \"$YAZELIX_TEST_ZELLIJ_LOG\" ]; then"
            "  printf '%s\\n' \"$*\" >> \"$YAZELIX_TEST_ZELLIJ_LOG\""
            "else"
            "  printf '%s\\n' \"$*\" > \"$YAZELIX_TEST_ZELLIJ_LOG\""
            "fi"
            "exit 0"
        ]
        write_executable_fixture_file ($fixture.fake_bin | path join "fake-popup") [
            "#!/bin/sh"
            "printf 'args=%s\\n' \"$*\" > \"$YAZELIX_TEST_POPUP_LOG\""
            "printf 'pane-env=%s\\n' \"${YAZELIX_POPUP_PANE-unset}\" >> \"$YAZELIX_TEST_POPUP_LOG\""
            "exit 0"
        ]
        [
            "export def refresh_active_sidebar_yazi [] {"
            "    'refresh' | save --force --raw $env.YAZELIX_TEST_REFRESH_LOG"
            "    {status: 'ok'}"
            "}"
        ] | str join "\n" | save --force --raw ($fixture.integrations_dir | path join "yazi.nu")
        cp ($env.PWD | path join "nushell" "scripts" "zellij_wrappers" "yzx_popup_program.nu") ($fixture.wrapper_dir | path join "yzx_popup_program.nu")

        let wrapper_script = ($fixture.wrapper_dir | path join "yzx_popup_program.nu")
        let output = (with-env {
            PATH: ([$fixture.fake_bin] | append $env.PATH)
            ZELLIJ: "1"
            YAZELIX_TEST_POPUP_LOG: ($fixture.tmpdir | path join "popup_program.log")
            YAZELIX_TEST_ZELLIJ_LOG: ($fixture.tmpdir | path join "zellij.log")
            YAZELIX_TEST_REFRESH_LOG: $fixture.refresh_log
        } {
            ^nu $wrapper_script fake-popup "--flag" "value" | complete
        })

        let popup_log = ($fixture.tmpdir | path join "popup_program.log")
        let zellij_log = ($fixture.tmpdir | path join "zellij.log")
        let popup_invocation = if ($popup_log | path exists) {
            open --raw $popup_log | lines
        } else {
            []
        }
        let zellij_invocation = if ($zellij_log | path exists) {
            open --raw $zellij_log | lines
        } else {
            []
        }
        let refresh_log = if ($fixture.refresh_log | path exists) {
            open --raw $fixture.refresh_log | str trim
        } else {
            ""
        }

        if (
            ($output.exit_code == 0)
            and ($popup_invocation == ["args=--flag value" "pane-env=unset"])
            and ($zellij_invocation == ["action rename-pane yzx_popup" "action close-pane"])
            and ($refresh_log == "refresh")
        ) {
            print "  ✅ popup runtime wrapper now runs the resolved argv directly, refreshes the sidebar, and closes the transient pane after success"
            true
        } else {
            print $"  ❌ Unexpected popup wrapper behavior: exit=($output.exit_code) popup=($popup_invocation | to json -r) zellij=($zellij_invocation | to json -r) refresh=($refresh_log | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: the runtime menu popup wrapper must mark popup mode, rename itself, and close its own transient pane after success.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_menu_popup_wrapper_marks_popup_mode_and_closes_transient_pane [] {
    print "🧪 Testing menu popup wrapper sets popup mode and closes its transient pane..."

    let fixture = (setup_runtime_wrapper_fixture "yazelix_menu_popup_wrapper")

    let result = (try {
        let zellij_log = ($fixture.tmpdir | path join "zellij.log")
        let menu_log = ($fixture.tmpdir | path join "menu.log")

        write_executable_fixture_file ($fixture.fake_bin | path join "zellij") [
            "#!/bin/sh"
            "if [ -f \"$YAZELIX_TEST_ZELLIJ_LOG\" ]; then"
            "  printf '%s\\n' \"$*\" >> \"$YAZELIX_TEST_ZELLIJ_LOG\""
            "else"
            "  printf '%s\\n' \"$*\" > \"$YAZELIX_TEST_ZELLIJ_LOG\""
            "fi"
            "exit 0"
        ]
        [
            "export def \"yzx menu\" [] {"
            "    let value = ($env.YAZELIX_MENU_POPUP? | default \"unset\")"
            "    $\"YAZELIX_MENU_POPUP=($value)\" | save --force --raw $env.YAZELIX_TEST_MENU_LOG"
            "}"
        ] | str join "\n" | save --force --raw ($fixture.yzx_dir | path join "menu.nu")
        cp ($env.PWD | path join "nushell" "scripts" "zellij_wrappers" "yzx_menu_popup.nu") ($fixture.wrapper_dir | path join "yzx_menu_popup.nu")

        let wrapper_script = ($fixture.wrapper_dir | path join "yzx_menu_popup.nu")
        let output = (with-env {
            PATH: ([$fixture.fake_bin] | append $env.PATH)
            ZELLIJ: "1"
            YAZELIX_TEST_ZELLIJ_LOG: $zellij_log
            YAZELIX_TEST_MENU_LOG: $menu_log
        } {
            ^nu $wrapper_script | complete
        })

        let menu_env = if ($menu_log | path exists) {
            open --raw $menu_log | str trim
        } else {
            ""
        }
        let zellij_invocation = if ($zellij_log | path exists) {
            open --raw $zellij_log | lines
        } else {
            []
        }

        if (
            ($output.exit_code == 0)
            and ($menu_env == "YAZELIX_MENU_POPUP=true")
            and ($zellij_invocation == ["action rename-pane yzx_menu" "action close-pane"])
        ) {
            print "  ✅ menu popup wrapper now marks popup mode, renames itself, and closes the transient pane after success"
            true
        } else {
            print $"  ❌ Unexpected menu wrapper behavior: exit=($output.exit_code) menu_env=($menu_env | to json -r) zellij=($zellij_invocation | to json -r) stderr=(($output.stderr | str trim))"
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
        let wrapper_root = ($fixture.runtime_dir | path join "nushell" "scripts" "zellij_wrappers")
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
            PATH: ([$fixture.fake_bin] | append $env.PATH)
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_TEST_ZELLIJ_LOG: $zellij_log
        } {
            open_floating_runtime_script "proof_popup" "nushell/scripts/zellij_wrappers/proof_popup.nu" "/tmp/workspace"
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
        (test_popup_program_wrapper_runs_resolved_argv_directly)
        (test_menu_popup_wrapper_marks_popup_mode_and_closes_transient_pane)
        (test_popup_wrapper_env_falls_back_to_runtime_env)
        (test_popup_wrapper_serializes_path_list_for_env_command)
        (test_popup_wrapper_falls_back_to_host_nu_without_runtime_owned_nu)
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
