#!/usr/bin/env nu
# Upgrade-summary E2E runner

use ./yzx_test_helpers.nu [get_repo_root log_block log_line repo_path resolve_test_yzx_bin]
use ../utils/constants.nu [YAZELIX_VERSION]

def setup_fixture [] {
    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_upgrade_summary_e2e_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let config_path = ($user_config_dir | path join "yazelix.toml")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let log_file = ($tmp_home | path join "upgrade_summary_e2e.log")

    mkdir $runtime_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir

    for entry in [".taplo.toml", "nushell", "shells", "configs", "yazelix_default.toml", "CHANGELOG.md"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }

    ^cp -R (repo_path "docs") ($runtime_dir | path join "docs")
    '[core]
welcome_style = "game_of_life_gliders"
' | save --force --raw $config_path

    let notes_path = ($runtime_dir | path join "docs" "upgrade_notes.toml")
    let notes = (open $notes_path)
    let updated_release = (
        ($notes.releases | get -o $YAZELIX_VERSION)
        | upsert headline $"Config migration follow-up after the ($YAZELIX_VERSION) upgrade"
        | upsert summary [
            "This fixture simulates a historical release that mentioned config-shape changes."
            "It should render historical guidance without probing or rewriting the current config."
        ]
        | upsert upgrade_impact "migration_available"
        | upsert migration_ids [
            "remove_zellij_widget_tray_layout"
            "remove_shell_enable_atuin"
        ]
        | upsert manual_actions []
    )
    ($notes | upsert releases ($notes.releases | upsert $YAZELIX_VERSION $updated_release)) | to toml | save --force $notes_path
    "" | save --force --raw $log_file

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        user_config_dir: $user_config_dir
        config_path: $config_path
        state_dir: $state_dir
        upgrade_summary_script: ($runtime_dir | path join "nushell" "scripts" "utils" "upgrade_summary.nu")
        state_file: ($state_dir | path join "state" "upgrade_summary" "last_seen_version.txt")
        log_file: $log_file
    }
}

def run_first_run_probe [fixture: record] {
    let command = [
        $"source \"($fixture.upgrade_summary_script)\""
        "let result = (maybe_show_first_run_upgrade_summary)"
        "print '=== RESULT ==='"
        "print ($result | to json -r)"
    ] | str join "; "

    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        YAZELIX_STATE_DIR: $fixture.state_dir
    } {
        ^nu -c $command | complete
    }
}

def run_whats_new [fixture: record] {
    let yzx_bin = (resolve_test_yzx_bin)
    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        YAZELIX_STATE_DIR: $fixture.state_dir
        YAZELIX_YZX_BIN: $yzx_bin
    } {
        ^$yzx_bin whats_new | complete
    }
}

export def main [] {
    let fixture = (setup_fixture)
    let log_file = $fixture.log_file

    log_line $log_file "Case: first run, second run, and manual reopen all use the same current-version summary"
    log_line $log_file $"Temp HOME: ($fixture.tmp_home)"
    log_line $log_file $"Runtime dir: ($fixture.runtime_dir)"
    log_line $log_file $"Config path: ($fixture.config_path)"
    log_line $log_file ""
    log_block $log_file "Input TOML" (open --raw $fixture.config_path)
    log_block $log_file "Mutated upgrade_notes.toml" (open --raw ($fixture.runtime_dir | path join "docs" "upgrade_notes.toml"))

    let first = (run_first_run_probe $fixture)
    log_block $log_file "First-run stdout" ($first.stdout | str trim)
    log_block $log_file "First-run stderr" ($first.stderr | str trim)

    let state_after_first = if ($fixture.state_file | path exists) {
        open --raw $fixture.state_file | str trim
    } else {
        ""
    }
    log_block $log_file "State after first run" $state_after_first

    let second = (run_first_run_probe $fixture)
    log_block $log_file "Second-run stdout" ($second.stdout | str trim)
    log_block $log_file "Second-run stderr" ($second.stderr | str trim)

    let manual = (run_whats_new $fixture)
    log_block $log_file "yzx whats_new stdout" ($manual.stdout | str trim)
    log_block $log_file "yzx whats_new stderr" ($manual.stderr | str trim)

    let second_lines = ($second.stdout | lines)
    let second_leading_line = ($second_lines | get -o 0 | default "")

    let ok = (
        (($first.stdout | str contains $"What's New In Yazelix ($YAZELIX_VERSION)"))
        and (($first.stdout | str contains "historical release included config-shape changes"))
        and (($first.stdout | str contains "v15 no longer ships an automatic config migration engine"))
        and ($state_after_first == $YAZELIX_VERSION)
        and ($second_leading_line == "=== RESULT ===")
        and (($second.stdout | str contains '"shown":false'))
        and (($manual.stdout | str contains $"What's New In Yazelix ($YAZELIX_VERSION)"))
        and (($manual.stdout | str contains "yzx config reset"))
        and not (($manual.stdout | str contains "yzx doctor --fix"))
    )

    if $ok {
        log_line $log_file "Result: PASS"
    } else {
        log_line $log_file "Result: FAIL"
    }

    rm -rf $fixture.tmp_home

    if $ok {
        print ""
        print "✅ Upgrade summary e2e checks passed (1/1)"
    } else {
        print ""
        print "❌ Upgrade summary e2e checks failed (0/1)"
        error make {msg: "upgrade summary e2e checks failed"}
    }
}
