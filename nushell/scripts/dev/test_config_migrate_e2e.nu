#!/usr/bin/env nu

use ./test_yzx_helpers.nu [get_repo_config_dir]

def log_line [log_file: string, line: string] {
    print $line
    $"($line)\n" | save --append --raw $log_file
}

def log_block [log_file: string, title: string, content: string] {
    log_line $log_file $"=== ($title) ==="
    if ($content | is-empty) {
        log_line $log_file "<empty>"
    } else {
        for line in ($content | lines) {
            log_line $log_file $line
        }
    }
    log_line $log_file ""
}

def setup_fixture [label: string, raw_toml: string] {
    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let log_file = ($tmp_home | path join "config_migrate_e2e.log")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    $raw_toml | save --force --raw ($config_dir | path join "yazelix.toml")
    "" | save --force --raw $log_file

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        config_dir: $config_dir
        config_path: ($config_dir | path join "yazelix.toml")
        log_file: $log_file
        yzx_script: ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
    }
}

def run_migrate [fixture: record, args: list<string> = []] {
    let command = if ($args | is-empty) {
        "yzx config migrate"
    } else {
        let joined_args = ($args | str join " ")
        $"yzx config migrate ($joined_args)"
    }

    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } {
        ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
    }
}

def run_mixed_migration_case [] {
    let fixture = (setup_fixture
        "yazelix_migrate_e2e_mixed"
        '[terminal]
preferred_terminal = "ghostty"
extra_terminals = ["wezterm", "kitty"]
cursor_trail = "random"

[zellij]
widget_tray = ["layout", "editor", "cpu"]

[shell]
enable_atuin = true
')
    let log_file = $fixture.log_file

    log_line $log_file "Case: mixed safe and manual migrations"
    log_line $log_file $"Temp HOME: ($fixture.tmp_home)"
    log_line $log_file $"Config path: ($fixture.config_path)"
    log_line $log_file $"Log file: ($log_file)"
    log_line $log_file ""
    log_block $log_file "Input TOML" (open --raw $fixture.config_path)

    let preview = (run_migrate $fixture)
    log_block $log_file "Preview stdout" ($preview.stdout | str trim)
    log_block $log_file "Preview stderr" ($preview.stderr | str trim)

    let apply = (run_migrate $fixture ["--apply", "--yes"])
    log_block $log_file "Apply stdout" ($apply.stdout | str trim)
    log_block $log_file "Apply stderr" ($apply.stderr | str trim)
    log_block $log_file "Output TOML" (open --raw $fixture.config_path)

    let backups = (ls $fixture.config_dir | where name =~ 'yazelix\.toml\.backup-')
    log_block $log_file "Backups" (($backups | get name | str join "\n"))

    let parsed = (open $fixture.config_path)
    let ok = (
        ($preview.exit_code == 0)
        and ($apply.exit_code == 0)
        and (($preview.stdout | str contains "[AUTO] remove_zellij_widget_tray_layout"))
        and (($preview.stdout | str contains "[AUTO] unify_terminal_preference_list"))
        and (($preview.stdout | str contains "[AUTO] remove_shell_enable_atuin"))
        and (($preview.stdout | str contains "[MANUAL] review_legacy_cursor_trail_settings"))
        and (($parsed | get terminal.terminals) == ["ghostty", "wezterm", "kitty"])
        and (($parsed | get zellij.widget_tray) == ["editor", "cpu"])
        and (($parsed | get terminal.cursor_trail) == "random")
        and (($backups | length) == 1)
    )

    if $ok {
        log_line $log_file "Result: PASS"
    } else {
        log_line $log_file "Result: FAIL"
    }

    rm -rf $fixture.tmp_home
    $ok
}

def run_manual_conflict_case [] {
    let fixture = (setup_fixture
        "yazelix_migrate_e2e_manual"
        '[terminal]
preferred_terminal = "ghostty"
terminals = ["kitty"]
cursor_trail = "snow"
'
    )
    let log_file = $fixture.log_file

    log_line $log_file "Case: manual-only conflict"
    log_line $log_file $"Temp HOME: ($fixture.tmp_home)"
    log_line $log_file $"Config path: ($fixture.config_path)"
    log_line $log_file $"Log file: ($log_file)"
    log_line $log_file ""
    log_block $log_file "Input TOML" (open --raw $fixture.config_path)

    let preview = (run_migrate $fixture)
    log_block $log_file "Preview stdout" ($preview.stdout | str trim)
    log_block $log_file "Preview stderr" ($preview.stderr | str trim)

    let apply = (run_migrate $fixture ["--apply", "--yes"])
    log_block $log_file "Apply stdout" ($apply.stdout | str trim)
    log_block $log_file "Apply stderr" ($apply.stderr | str trim)
    log_block $log_file "Output TOML" (open --raw $fixture.config_path)

    let backups = (ls $fixture.config_dir | where name =~ 'yazelix\.toml\.backup-')
    log_block $log_file "Backups" (($backups | get name | str join "\n"))

    let ok = (
        ($preview.exit_code == 0)
        and ($apply.exit_code == 0)
        and (($preview.stdout | str contains "[MANUAL] unify_terminal_preference_list"))
        and (($preview.stdout | str contains "[MANUAL] review_legacy_cursor_trail_settings"))
        and (($apply.stdout | str contains "No safe config rewrites to apply."))
        and (($backups | length) == 0)
    )

    if $ok {
        log_line $log_file "Result: PASS"
    } else {
        log_line $log_file "Result: FAIL"
    }

    rm -rf $fixture.tmp_home
    $ok
}

export def main [] {
    let results = [
        (run_mixed_migration_case)
        (run_manual_conflict_case)
    ]
    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ Config migrate e2e checks passed \(($passed)/($total)\)"
    } else {
        print $"❌ Config migrate e2e checks failed \(($passed)/($total)\)"
        error make {msg: "config migrate e2e checks failed"}
    }
}
