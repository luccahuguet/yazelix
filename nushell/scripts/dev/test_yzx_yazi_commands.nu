#!/usr/bin/env nu
# Defends: docs/specs/test_suite_governance.md

use ./test_yzx_helpers.nu [setup_managed_config_fixture]
use ../integrations/yazi.nu [get_ya_command, get_yazi_command, resolve_managed_editor_open_strategy]

def test_missing_managed_editor_opens_new_managed_pane [] {
    print "🧪 Testing shell-opened editors do not get adopted as the managed editor pane..."

    try {
        let result = (resolve_managed_editor_open_strategy "missing")

        if $result.action == "open_new_managed" {
            print "  ✅ missing managed-editor state routes Yazi opens to a new managed editor pane"
            true
        } else {
            print $"  ❌ Unexpected routing result: ($result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_existing_managed_editor_is_reused [] {
    print "🧪 Testing existing managed editor panes are reused..."

    try {
        let result = (resolve_managed_editor_open_strategy "ok")

        if $result.action == "reuse_managed" {
            print "  ✅ existing managed-editor state reuses the managed editor pane"
            true
        } else {
            print $"  ❌ Unexpected routing result: ($result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yazi_command_resolvers_default_to_path_binaries [] {
    print "🧪 Testing Yazi command resolvers default to PATH binaries when unset..."

    let fixture = (setup_managed_config_fixture "yazelix_yazi_command_defaults" '[core]
recommended_deps = true
')

    let result = (try {
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

        if ($resolved.yazi == "yazi") and ($resolved.ya == "ya") {
            print "  ✅ Unset Yazi command config falls back to PATH binaries"
            true
        } else {
            print $"  ❌ Unexpected resolved commands: ($resolved | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yazi_command_resolvers_honor_custom_config [] {
    print "🧪 Testing Yazi command resolvers honor custom configured binaries..."

    let fixture = (setup_managed_config_fixture "yazelix_yazi_command_overrides" '[yazi]
command = "/opt/custom/yazi"
ya_command = "/opt/custom/ya"
')

    let result = (try {
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

        if ($resolved.yazi == "/opt/custom/yazi") and ($resolved.ya == "/opt/custom/ya") {
            print "  ✅ Custom Yazi command config is respected"
            true
        } else {
            print $"  ❌ Unexpected resolved commands: ($resolved | to json -r)"
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
        (test_missing_managed_editor_opens_new_managed_pane)
        (test_existing_managed_editor_is_reused)
        (test_yazi_command_resolvers_default_to_path_binaries)
        (test_yazi_command_resolvers_honor_custom_config)
    ]
}

export def run_yazi_tests [] {
    run_yazi_canonical_tests
}
