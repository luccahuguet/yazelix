#!/usr/bin/env nu
# Defends: docs/specs/test_suite_governance.md

use ./test_yzx_helpers.nu [get_repo_config_dir get_repo_root repo_path]
use ../setup/yazi_config_merger.nu [generate_merged_yazi_config]
use ../setup/helix_config_merger.nu [generate_managed_helix_config]
use ../utils/launch_state.nu [get_launch_env]

def test_render_welcome_style_interruptibly_repaints_logo_after_game_of_life_skip [] {
    print "🧪 Testing skipping game_of_life repaints the resting logo frame..."

    try {
        let art_script = (repo_path "nushell" "scripts" "utils" "ascii_art.nu")
        let output = (^nu -c $"use \"($art_script)\" [render_welcome_style_interruptibly]; render_welcome_style_interruptibly game_of_life 0.5 60 {|timeout| true } | ignore" | complete)
        let clean_stdout = (
            $output.stdout
            | str replace -ar '\u001b\[[0-9;?]*[A-Za-z]' ''
            | str replace -a "\r" ""
        )

        if (
            ($output.exit_code == 0)
            and ($clean_stdout | str contains "YAZELIX")
            and ($clean_stdout | str contains "your reproducible terminal IDE")
            and ($clean_stdout | str contains "welcome to yazelix")
        ) {
            print "  ✅ Welcome skip repaints the resting logo frame instead of leaving animated output behind"
            true
        } else {
            print $"  ❌ Unexpected skip repaint result: exit=($output.exit_code) stdout=($clean_stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_generate_merged_yazi_keymap_uses_zoxide_editor_plugin [] {
    print "🧪 Testing merged Yazi keymap uses the bundled zoxide editor plugin..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_yazi_zoxide_plugin_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")

    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir
    mkdir ($temp_config_dir | path join "user_configs")

    let result = (try {
        let merged_keymap = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            generate_merged_yazi_config $repo_root --quiet | ignore
            open --raw ($tmp_home | path join ".local" "share" "yazelix" "configs" "yazi" "keymap.toml")
        })
        let plugin_main = ($tmp_home | path join ".local" "share" "yazelix" "configs" "yazi" "plugins" "zoxide-editor.yazi" "main.lua")

        if (
            ($merged_keymap | str contains 'run = "plugin zoxide-editor"')
            and not ($merged_keymap | str contains "zoxide_open_in_editor.nu")
            and ($plugin_main | path exists)
        ) {
            print "  ✅ Merged Yazi config binds Alt+z to the bundled zoxide editor plugin and ships the plugin files"
            true
        } else {
            print $"  ❌ Unexpected merged zoxide keymap/plugin state: keymap=($merged_keymap) plugin_exists=(($plugin_main | path exists))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

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

        if (
            ($launch_env.EDITOR == $expected_wrapper)
            and (($launch_env | get YAZELIX_MANAGED_EDITOR_KIND) == "helix")
            and (($launch_env | get YAZELIX_MANAGED_HELIX_BINARY) == $expected_binary)
        ) {
            print "  ✅ Launch env now routes managed Helix sessions through the Yazelix wrapper while preserving the real Helix binary"
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

export def run_generated_config_extended_tests [] {
    [
        (test_render_welcome_style_interruptibly_repaints_logo_after_game_of_life_skip)
        (test_generate_merged_yazi_keymap_uses_zoxide_editor_plugin)
        (test_generate_managed_helix_config_merges_user_config_and_enforces_reveal)
        (test_get_launch_env_wraps_helix_with_managed_wrapper)
    ]
}

export def main [] {
    let results = (run_generated_config_extended_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All extended generated-config tests passed \(($passed)/($total)\)"
    } else {
        error make { msg: $"Extended generated-config tests failed \(($passed)/($total)\)" }
    }
}
