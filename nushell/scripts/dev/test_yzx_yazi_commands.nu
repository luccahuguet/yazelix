#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [setup_managed_config_fixture]
use ../integrations/managed_editor.nu [get_managed_editor_kind]
use ../integrations/yazi.nu [get_ya_command, get_yazi_command, refresh_active_sidebar_yazi]
use ../integrations/yazi_sidebar_state.nu get_active_sidebar_state
use ../integrations/zellij.nu toggle_editor_sidebar_focus

def write_executable_fixture_file [path: string, lines: list<string>] {
    $lines | str join "\n" | save --force --raw $path
    ^chmod +x $path
}

# Defends: Yazi command resolution honors defaults and user overrides.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yazi_command_resolvers_honor_defaults_and_overrides [] {
    print "🧪 Testing Yazi command resolvers honor defaults and explicit overrides..."

    let cases = [
        {
            label: "defaults"
            raw_toml: '[core]
skip_welcome_screen = true
'
            expected_yazi: "yazi"
            expected_ya: "ya"
        }
        {
            label: "overrides"
            raw_toml: '[yazi]
command = "/opt/custom/yazi"
ya_command = "/opt/custom/ya"
'
            expected_yazi: "/opt/custom/yazi"
            expected_ya: "/opt/custom/ya"
        }
    ]

    try {
        let failures = (
            $cases
            | each {|case|
                let fixture = (setup_managed_config_fixture $"yazelix_yazi_command_($case.label)" $case.raw_toml)

                try {
                    let resolved = (with-env {
                        HOME: $fixture.tmp_home
                        YAZELIX_CONFIG_DIR: $fixture.config_dir
                        YAZELIX_RUNTIME_DIR: $fixture.repo_root
                    } {
                        {
                            yazi: (get_yazi_command)
                            ya: (get_ya_command)
                        }
                    })

                    if ($resolved.yazi == $case.expected_yazi) and ($resolved.ya == $case.expected_ya) {
                        null
                    } else {
                        {
                            label: $case.label
                            resolved: $resolved
                            expected_yazi: $case.expected_yazi
                            expected_ya: $case.expected_ya
                        }
                    }
                } catch {|err|
                    {
                        label: $case.label
                        error: $err.msg
                    }
                } finally {
                    rm -rf $fixture.tmp_home
                }
            }
            | where {|item| $item != null}
        )

        if ($failures | is-empty) {
            print "  ✅ Yazi command config falls back to PATH by default and honors explicit overrides"
            true
        } else {
            print $"  ❌ Unexpected resolver failures: ($failures | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Regression: managed Helix sessions expose a wrapper path in EDITOR, so Yazi-side editor detection must still resolve Helix.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_get_managed_editor_kind_accepts_managed_helix_wrapper_env [] {
    print "🧪 Testing managed editor detection accepts the Yazelix Helix wrapper env..."

    let fixture = (setup_managed_config_fixture
        "yazelix_yazi_managed_helix_wrapper"
        '[core]
skip_welcome_screen = true
'
    )

    let result = (try {
        let repo_root = $fixture.repo_root
        let detected = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            EDITOR: ($repo_root | path join "shells" "posix" "yazelix_hx.sh")
            YAZELIX_MANAGED_HELIX_BINARY: ($repo_root | path join "bin" "hx")
        } {
            get_managed_editor_kind
        })

        if $detected == "helix" {
            print "  ✅ Managed editor detection now treats the Yazelix Helix wrapper as a Helix session"
            true
        } else {
            print $"  ❌ Unexpected detected editor kind: ($detected | to nuon)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: sidebar refresh must use the pane-orchestrator-owned sidebar Yazi identity instead of the cache.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_refresh_active_sidebar_yazi_emits_refresh_to_plugin_sidebar_instance [] {
    print "🧪 Testing active sidebar Yazi refresh uses the plugin-owned sidebar instance and ignores stale cache state..."

    let fixture = (setup_managed_config_fixture
        "yazelix_yazi_sidebar_refresh"
        '[yazi]
ya_command = "ya"
'
    )

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        let state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix" "state" "yazi" "sidebar")
        let ya_log = ($fixture.tmp_home | path join "ya.log")
        mkdir $fake_bin
        mkdir $state_dir

        write_executable_fixture_file ($fake_bin | path join "ya") [
            "#!/bin/sh"
            "printf '%s\\n' \"$*\" >> \"$YAZI_TEST_LOG\""
            "exit 0"
        ]
        write_executable_fixture_file ($fake_bin | path join "zellij") [
            "#!/bin/sh"
            "if [ \"$6\" = \"get_active_sidebar_yazi_state\" ]; then"
            "  printf '%s\\n' '{\"pane_id\":\"terminal:5\",\"yazi_id\":\"plugin-sidebar-yazi-123\",\"cwd\":\"/home/test/workspace\"}'"
            "  exit 0"
            "fi"
            "printf '%s\\n' \"unexpected zellij args: $*\" >&2"
            "exit 1"
        ]

        "stale-cache-yazi-id\n/home/stale\n" | save --force --raw ($state_dir | path join "testsession__terminal_5.txt")

        let refresh_result = (with-env {
            HOME: $fixture.tmp_home
            PATH: ($env.PATH | prepend $fake_bin)
            ZELLIJ: "1"
            ZELLIJ_SESSION_NAME: "testsession"
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
            YAZI_TEST_LOG: $ya_log
        } {
            refresh_active_sidebar_yazi
        })
        let ya_args = if ($ya_log | path exists) {
            open --raw $ya_log | lines
        } else {
            []
        }

        if (
            ($refresh_result.status == "ok")
            and ($ya_args == [
                "emit-to plugin-sidebar-yazi-123 refresh",
                "emit-to plugin-sidebar-yazi-123 plugin git refresh-sidebar",
                "emit-to plugin-sidebar-yazi-123 plugin starship /home/test/workspace",
            ])
        ) {
            print "  ✅ active sidebar Yazi refresh now uses the plugin-owned sidebar identity and ignores stale cache entries"
            true
        } else {
            print $"  ❌ Unexpected sidebar refresh result: result=($refresh_result | to json -r) ya_args=($ya_args | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: active sidebar state lookup must use the pane-orchestrator-owned sidebar identity instead of filesystem cache selection.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_get_active_sidebar_state_uses_plugin_owned_sidebar_identity_instead_of_cache [] {
    print "🧪 Testing active sidebar Yazi lookup uses the plugin-owned sidebar identity instead of stale cache files..."

    let fixture = (setup_managed_config_fixture
        "yazelix_yazi_sidebar_state_plugin_owned"
        '[yazi]
ya_command = "ya"
'
    )

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        let state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix" "state" "yazi" "sidebar")
        mkdir $fake_bin
        mkdir $state_dir

        write_executable_fixture_file ($fake_bin | path join "zellij") [
            "#!/bin/sh"
            "if [ \"$6\" = \"get_active_sidebar_yazi_state\" ]; then"
            "  printf '%s\\n' '{\"pane_id\":\"terminal:0\",\"yazi_id\":\"plugin-yazi-id\",\"cwd\":\"/home/plugin\"}'"
            "  exit 0"
            "fi"
            "printf '%s\\n' \"unexpected zellij args: $*\" >&2"
            "exit 1"
        ]

        let current_state = ($state_dir | path join "current-session__terminal_0.txt")
        let foreign_state = ($state_dir | path join "foreign-session__terminal_0.txt")
        "current-yazi-id\n/home/current\n" | save --force --raw $current_state
        "foreign-yazi-id\n/home/foreign\n" | save --force --raw $foreign_state

        # Make the foreign session file newer to defend the exact regression:
        # same pane id, wrong session, higher mtime.
        sleep 50ms
        "foreign-yazi-id\n/home/foreign\n" | save --force --raw $foreign_state

        let resolved = (with-env {
            HOME: $fixture.tmp_home
            PATH: ($env.PATH | prepend $fake_bin)
            ZELLIJ: "1"
            ZELLIJ_SESSION_NAME: "current-session"
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
        } {
            get_active_sidebar_state
        })

        if (
            (($resolved.yazi_id? | default null) == "plugin-yazi-id")
            and (($resolved.cwd? | default null) == "/home/plugin")
        ) {
            print "  ✅ active sidebar lookup now uses the plugin-owned sidebar identity instead of cache selection"
            true
        } else {
            print $"  ❌ Unexpected active sidebar state: ($resolved | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: sidebar focus toggles must expose when the sidebar gained focus so refresh hooks can stay exact.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_toggle_editor_sidebar_focus_reports_sidebar_target_from_plugin_response [] {
    print "🧪 Testing sidebar focus toggles report when the sidebar gained focus..."

    let fixture = (setup_managed_config_fixture
        "yazelix_yazi_sidebar_focus_target"
        '[core]
skip_welcome_screen = true
'
    )

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        mkdir $fake_bin

        write_executable_fixture_file ($fake_bin | path join "zellij") [
            "#!/bin/sh"
            "printf '%s\\n' \"$YAZELIX_TEST_PANE_RESULT\""
            "exit 0"
        ]

        let sidebar_result = (with-env {
            HOME: $fixture.tmp_home
            PATH: ($env.PATH | prepend $fake_bin)
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
            YAZELIX_TEST_PANE_RESULT: "focused_sidebar"
        } {
            toggle_editor_sidebar_focus
        })

        let editor_result = (with-env {
            HOME: $fixture.tmp_home
            PATH: ($env.PATH | prepend $fake_bin)
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
            YAZELIX_TEST_PANE_RESULT: "focused_editor"
        } {
            toggle_editor_sidebar_focus
        })

        if (
            ($sidebar_result.status == "ok")
            and (($sidebar_result.target? | default "") == "sidebar")
            and ($editor_result.status == "ok")
            and (($editor_result.target? | default "") == "editor")
        ) {
            print "  ✅ sidebar focus toggles now preserve the focus target so refresh hooks can stay exact"
            true
        } else {
            print $"  ❌ Unexpected focus-toggle parse results: sidebar=($sidebar_result | to json -r) editor=($editor_result | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

export def run_yazi_canonical_tests [] {
    [
        (test_yazi_command_resolvers_honor_defaults_and_overrides)
        (test_get_managed_editor_kind_accepts_managed_helix_wrapper_env)
        (test_refresh_active_sidebar_yazi_emits_refresh_to_plugin_sidebar_instance)
        (test_get_active_sidebar_state_uses_plugin_owned_sidebar_identity_instead_of_cache)
        (test_toggle_editor_sidebar_focus_reports_sidebar_target_from_plugin_response)
    ]
}
