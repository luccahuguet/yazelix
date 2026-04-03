#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_root]
use ../setup/helix_config_merger.nu [generate_managed_helix_config get_helix_import_notice_marker_path]
use ../utils/launch_state.nu [get_launch_env]

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: managed Helix config preserves user overrides while enforcing the Yazelix reveal binding.
def test_generate_managed_helix_config_merges_user_config_and_enforces_reveal [] {
    print "🧪 Testing managed Helix config generation keeps user settings while enforcing Yazelix reveal..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_helix_config_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    let result = (try {
        let helix_user_dir = ($user_config_dir | path join "helix")
        mkdir $helix_user_dir
        '[editor]
line-number = "relative"

[keys.normal]
g = "goto_file_start"
A-r = ":noop"
' | save --force --raw ($helix_user_dir | path join "config.toml")

        let merged = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            let output_path = (generate_managed_helix_config)
            {
                output_path: $output_path
                config: (open $output_path)
            }
        })

        let normal_keys = ($merged.config.keys | get normal)
        let expected_output_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "helix" "config.toml")

        if (
            ($merged.output_path == $expected_output_path)
            and (($merged.config.editor | get "line-number") == "relative")
            and (($normal_keys | get g) == "goto_file_start")
            and (($normal_keys | get "A-r") == ':sh yzx reveal "%{buffer_name}"')
        ) {
            print "  ✅ Managed Helix config preserves user overrides while forcing the Yazelix reveal binding"
            true
        } else {
            print $"  ❌ Unexpected managed Helix config: (($merged | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: launch env wraps Helix with the managed Yazelix wrapper and preserves the real binary path.
def test_get_launch_env_wraps_helix_with_managed_wrapper [] {
    print "🧪 Testing launch env wraps Helix with the Yazelix-managed wrapper and records the real binary..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_helix_launch_env_XXXXXX | str trim)
    let profile_path = ($tmp_home | path join "profile")
    let profile_bin = ($profile_path | path join "bin")
    mkdir $profile_bin
    "" | save --force --raw ($profile_bin | path join "hx")

    let result = (try {
        let launch_env = (with-env {
            HOME: $tmp_home
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            get_launch_env {} $profile_path
        })

        let expected_wrapper = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")
        let expected_binary = ($profile_bin | path join "hx")

        let retired_env_keys = [
            "YAZELIX_DEBUG_MODE"
            "YAZELIX_ENABLE_SIDEBAR"
            "YAZELIX_HELIX_MODE"
            "YAZELIX_TERMINAL_CONFIG_MODE"
            "YAZELIX_WELCOME_STYLE"
        ]
        let retired_keys_absent = (
            $retired_env_keys
            | all {|key| not ($launch_env | columns | any {|column| $column == $key }) }
        )

        if (
            ($launch_env.EDITOR == $expected_wrapper)
            and (($launch_env | get YAZELIX_MANAGED_EDITOR_KIND) == "helix")
            and (($launch_env | get YAZELIX_MANAGED_HELIX_BINARY) == $expected_binary)
            and $retired_keys_absent
        ) {
            print "  ✅ Launch env routes managed Helix sessions through the Yazelix wrapper, preserves the real Helix binary, and omits dead export-only vars"
            true
        } else {
            print $"  ❌ Unexpected managed Helix launch env: (($launch_env | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
# Defends: yzx import helix copies personal config into managed overrides and makes a backup on --force.
def test_yzx_import_helix_copies_personal_config_with_force_backups [] {
    print "🧪 Testing yzx import helix copies personal Helix config and backs up managed overrides on --force..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_import_helix_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let native_helix_dir = ($xdg_config_home | path join "helix")
    let yazelix_config_dir = ($xdg_config_home | path join "yazelix")
    let managed_helix_dir = ($yazelix_config_dir | path join "user_configs" "helix")
    mkdir $native_helix_dir
    mkdir ($yazelix_config_dir | path join "user_configs")
    mkdir $managed_helix_dir

    let result = (try {
        '[editor]
cursorline = true
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let import_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let first_import = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $yazelix_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu -c $"use \"($import_script)\" *; yzx import helix" | complete
        })

        '[editor]
cursorline = false
' | save --force --raw ($managed_helix_dir | path join "config.toml")
        '[editor]
line-number = "relative"
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let forced_import = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $yazelix_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu -c $"use \"($import_script)\" *; yzx import helix --force" | complete
        })

        let managed_config_path = ($managed_helix_dir | path join "config.toml")
        let managed_config = (open $managed_config_path)
        let backups = (ls $managed_helix_dir | where name =~ 'config\.toml\.backup-')

        if (
            ($first_import.exit_code == 0)
            and ($forced_import.exit_code == 0)
            and (($managed_config.editor | get "line-number") == "relative")
            and (($backups | length) == 1)
        ) {
            print "  ✅ yzx import helix copies personal Helix config into user_configs/helix and backs up the previous managed file on --force"
            true
        } else {
            print $"  ❌ Unexpected helix import result: first_exit=($first_import.exit_code) force_exit=($forced_import.exit_code) managed=(($managed_config | to json -r)) backups=(($backups | length)) stderr=(($forced_import.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

export def run_helix_managed_config_contract_tests [] {
    [
        (test_generate_managed_helix_config_merges_user_config_and_enforces_reveal)
        (test_get_launch_env_wraps_helix_with_managed_wrapper)
        (test_yzx_import_helix_copies_personal_config_with_force_backups)
    ]
}

export def main [] {
    let results = (run_helix_managed_config_contract_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All Helix managed config contract tests passed \(($passed)/($total)\)"
    } else {
        error make { msg: $"Helix managed config contract tests failed \(($passed)/($total)\)" }
    }
}
