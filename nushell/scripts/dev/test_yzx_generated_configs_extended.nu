#!/usr/bin/env nu
# Defends: docs/specs/test_suite_governance.md

use ./test_yzx_helpers.nu [get_repo_config_dir repo_path]
use ../setup/yazi_config_merger.nu [generate_merged_yazi_config]

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

export def run_generated_config_extended_tests [] {
    [
        (test_render_welcome_style_interruptibly_repaints_logo_after_game_of_life_skip)
        (test_generate_merged_yazi_keymap_uses_zoxide_editor_plugin)
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
