#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ./test_yzx_helpers.nu [setup_managed_config_fixture]
use ../integrations/yazi.nu [get_ya_command, get_yazi_command, resolve_managed_editor_open_strategy]

def test_managed_editor_open_strategy_routes_missing_and_existing_states [] {
    print "🧪 Testing managed editor open strategy routes both missing and existing states correctly..."

    try {
        let cases = [
            {
                status: "missing"
                expected_action: "open_new_managed"
            }
            {
                status: "ok"
                expected_action: "reuse_managed"
            }
        ]

        let failures = (
            $cases
            | where {|case|
                let result = (resolve_managed_editor_open_strategy $case.status)
                $result.action != $case.expected_action
            }
        )

        if ($failures | is-empty) {
            print "  ✅ managed editor routing stays correct for missing and existing pane states"
            true
        } else {
            print $"  ❌ Unexpected routing failures: ($failures | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yazi_command_resolvers_honor_defaults_and_overrides [] {
    print "🧪 Testing Yazi command resolvers honor defaults and explicit overrides..."

    let cases = [
        {
            label: "defaults"
            raw_toml: '[core]
recommended_deps = true
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

export def run_yazi_canonical_tests [] {
    [
        # Defends: managed editor open strategy routes missing and existing states correctly.
        (test_managed_editor_open_strategy_routes_missing_and_existing_states)
        # Defends: Yazi command resolution honors defaults and user overrides.
        (test_yazi_command_resolvers_honor_defaults_and_overrides)
    ]
}

export def run_yazi_tests [] {
    run_yazi_canonical_tests
}
