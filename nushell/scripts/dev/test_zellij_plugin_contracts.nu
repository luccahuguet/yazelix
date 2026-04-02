#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu get_repo_root
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../setup/zellij_plugin_paths.nu [get_tracked_zjstatus_wasm_path get_zjstatus_wasm_path]

# Strength: 7/10
# Defends: generated Zellij layouts load zjstatus from a stable Yazelix plugin path instead of a store path.
def test_generate_merged_zellij_layouts_use_stable_zjstatus_plugin_path [] {
    print "🧪 Testing generated Zellij layouts load zjstatus from the stable Yazelix plugin path..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_stable_zjstatus_layouts_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let out_dir = ($tmp_home | path join "out")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    let result = (try {
        let generated = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            generate_merged_zellij_config $repo_root $out_dir | ignore
            {
                layout: (open --raw ($out_dir | path join "layouts" "yzx_side.kdl"))
                stable_plugin_path: ($state_dir | path join "configs" "zellij" "plugins" "zjstatus.wasm")
            }
        })

        let expected_plugin_url = $"file:($generated.stable_plugin_path)"

        if (
            ($generated.layout | str contains $"plugin location=\"($expected_plugin_url)\"")
            and not ($generated.layout | str contains "/nix/store/")
            and ($generated.stable_plugin_path | path exists)
        ) {
            print "  ✅ Generated Zellij layouts now point zjstatus at the stable Yazelix plugin path instead of a runtime store path"
            true
        } else {
            print $"  ❌ Unexpected zjstatus plugin layout output: (($generated | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: 7/10
# Regression: zjstatus permission grants migrate onto tracked and stable Yazelix plugin paths.
def test_zjstatus_permission_cache_migrates_to_tracked_and_stable_paths [] {
    print "🧪 Testing zjstatus permission grants migrate onto the tracked and stable Yazelix plugin paths..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_zjstatus_permission_cache_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let cache_path = ($tmp_home | path join ".cache" "zellij" "permissions.kdl")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir ($cache_path | path dirname)

    let result = (try {
        '"/tmp/legacy/zjstatus.wasm" {
    ReadApplicationState
    ChangeApplicationState
    RunCommands
}
' | save --force --raw $cache_path

        let migrated = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            let stable_path = (get_zjstatus_wasm_path $repo_root)
            let tracked_path = (get_tracked_zjstatus_wasm_path $repo_root)
            {
                stable_path: $stable_path
                tracked_path: $tracked_path
                cache: (open --raw $cache_path)
            }
        })

        if (
            ($migrated.stable_path | path exists)
            and ($migrated.cache | str contains $"\"($migrated.tracked_path)\"")
            and ($migrated.cache | str contains $"\"($migrated.stable_path)\"")
            and ($migrated.cache | str contains "RunCommands")
        ) {
            print "  ✅ zjstatus permission grants now migrate onto both the tracked runtime path and the stable Yazelix plugin path"
            true
        } else {
            print $"  ❌ Unexpected zjstatus permission cache state: (($migrated | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

export def run_zellij_plugin_contract_tests [] {
    [
        (test_generate_merged_zellij_layouts_use_stable_zjstatus_plugin_path)
        (test_zjstatus_permission_cache_migrates_to_tracked_and_stable_paths)
    ]
}

export def main [] {
    let results = (run_zellij_plugin_contract_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All Zellij plugin contract tests passed \(($passed)/($total)\)"
    } else {
        error make { msg: $"Zellij plugin contract tests failed \(($passed)/($total)\)" }
    }
}
