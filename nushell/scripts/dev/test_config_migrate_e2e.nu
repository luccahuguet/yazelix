#!/usr/bin/env nu
# Test lane: maintainer

use ./test_yzx_helpers.nu [add_fixture_log log_block log_line setup_managed_config_fixture]

def setup_fixture [label: string, raw_toml: string] {
    add_fixture_log (setup_managed_config_fixture $label $raw_toml) "config_migrate_e2e.log"
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

    let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')
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

    let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')
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

def run_pack_split_case [] {
    let fixture = (setup_fixture
        "yazelix_migrate_e2e_pack_split"
        '[packs]
enabled = ["git", "go"]
user_packages = ["docker"]

[packs.declarations]
git = ["gh", "prek"]
go = ["gopls", "golangci-lint"]
')
    let log_file = $fixture.log_file
    let pack_path = ($fixture.user_config_dir | path join "yazelix_packs.toml")

    log_line $log_file "Case: split legacy pack surface"
    log_line $log_file $"Temp HOME: ($fixture.tmp_home)"
    log_line $log_file $"Config path: ($fixture.config_path)"
    log_line $log_file $"Pack path: ($pack_path)"
    log_line $log_file $"Log file: ($log_file)"
    log_line $log_file ""
    log_block $log_file "Input TOML" (open --raw $fixture.config_path)

    let preview = (run_migrate $fixture)
    log_block $log_file "Preview stdout" ($preview.stdout | str trim)
    log_block $log_file "Preview stderr" ($preview.stderr | str trim)

    let apply = (run_migrate $fixture ["--apply", "--yes"])
    log_block $log_file "Apply stdout" ($apply.stdout | str trim)
    log_block $log_file "Apply stderr" ($apply.stderr | str trim)
    log_block $log_file "Output main TOML" (open --raw $fixture.config_path)
    log_block $log_file "Output pack TOML" (if ($pack_path | path exists) { open --raw $pack_path } else { "<missing>" })

    let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')
    log_block $log_file "Backups" (($backups | get name | str join "\n"))

    let parsed_main = (open $fixture.config_path)
    let parsed_pack = (if ($pack_path | path exists) { open $pack_path } else { null })
    let ok = (
        ($preview.exit_code == 0)
        and ($apply.exit_code == 0)
        and (($preview.stdout | str contains "[AUTO] split_legacy_pack_config_surface"))
        and (($apply.stdout | str contains "Wrote pack config to"))
        and not ("packs" in ($parsed_main | columns))
        and ($parsed_pack.enabled == ["git", "go"])
        and ($parsed_pack.user_packages == ["docker"])
        and (($parsed_pack.declarations | get git) == ["gh", "prek"])
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

def run_ascii_mode_migration_case [
    label: string
    mode: string
    case_title: string
] {
    let fixture = (setup_fixture
        $label
        $"[ascii]\nmode = \"($mode)\"\n"
    )
    let log_file = $fixture.log_file

    log_line $log_file $case_title
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
    log_block $log_file "Output main TOML" (open --raw $fixture.config_path)

    let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')
    log_block $log_file "Backups" (($backups | get name | str join "\n"))

    let parsed_main = (open $fixture.config_path)
    let ok = (
        ($preview.exit_code == 0)
        and ($apply.exit_code == 0)
        and (($preview.stdout | str contains "[AUTO] replace_ascii_art_mode_with_welcome_style"))
        and (($apply.stdout | str contains "Applied 1 config migration"))
        and (($parsed_main.core | get welcome_style) == "random")
        and not ("ascii" in ($parsed_main | columns))
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

def run_game_of_life_style_rename_case [] {
    let fixture = (setup_fixture
        "yazelix_migrate_e2e_game_of_life"
        '[core]
welcome_style = "life"
')
    let log_file = $fixture.log_file

    log_line $log_file "Case: rename core.welcome_style = life into game_of_life"
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
    log_block $log_file "Output main TOML" (open --raw $fixture.config_path)

    let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')
    log_block $log_file "Backups" (($backups | get name | str join "\n"))

    let parsed_main = (open $fixture.config_path)
    let ok = (
        ($preview.exit_code == 0)
        and ($apply.exit_code == 0)
        and (($preview.stdout | str contains "[AUTO] rename_life_welcome_style_to_game_of_life"))
        and (($apply.stdout | str contains "Applied 1 config migration"))
        and (($parsed_main.core | get welcome_style) == "game_of_life")
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

export def main [] {
    let results = [
        (run_mixed_migration_case)
        (run_manual_conflict_case)
        (run_pack_split_case)
        (run_ascii_mode_migration_case
            "yazelix_migrate_e2e_welcome_style"
            "animated"
            "Case: migrate legacy ascii.mode into core.welcome_style"
        )
        (run_ascii_mode_migration_case
            "yazelix_migrate_e2e_welcome_style_static"
            "static"
            "Case: migrate legacy ascii.mode = static into core.welcome_style = random"
        )
        (run_game_of_life_style_rename_case)
    ]
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ Config migrate e2e checks passed \(($passed)/($total)\)"
    } else {
        print $"❌ Config migrate e2e checks failed \(($passed)/($total)\)"
        error make {msg: "config migrate e2e checks failed"}
    }
}
