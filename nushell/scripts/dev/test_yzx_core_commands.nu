#!/usr/bin/env nu

use ../core/yazelix.nu *
use ./test_yzx_helpers.nu [get_repo_config_dir get_repo_root repo_path]
use ../utils/config_migrations.nu [
    build_config_migration_plan_from_record
    render_config_migration_plan
    validate_config_migration_rules
]
use ../utils/constants.nu [YAZELIX_VERSION]
use ../utils/upgrade_summary.nu [
    build_upgrade_summary_report
    build_current_upgrade_summary_report
    maybe_show_first_run_upgrade_summary
]
use ../utils/shell_config_generation.nu [get_yazelix_section_content]
use ../utils/config_manager.nu [check_config_versions]

def setup_relocated_runtime_fixture [] {
    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_relocated_runtime_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")

    mkdir $runtime_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir

    for entry in ["nushell", "shells", "configs", "devenv.lock", "yazelix_default.toml"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }

    cp (repo_path "yazelix_default.toml") ($config_dir | path join "yazelix.toml")

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        yzx_script: ($runtime_dir | path join "nushell" "scripts" "core" "yazelix.nu")
        startup_script: ($runtime_dir | path join "shells" "posix" "start_yazelix.sh")
    }
}

def setup_config_migrate_fixture [label: string, raw_toml: string] {
    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir

    $raw_toml | save --force --raw ($config_dir | path join "yazelix.toml")

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        config_dir: $config_dir
        config_path: ($config_dir | path join "yazelix.toml")
        yzx_script: ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
    }
}

def setup_upgrade_summary_fixture [
    label: string
    raw_toml: string
    --migration-notes
] {
    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")

    mkdir $runtime_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir

    for entry in ["nushell", "shells", "configs", "devenv.lock", "yazelix_default.toml", "CHANGELOG.md"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }

    ^cp -R (repo_path "docs") ($runtime_dir | path join "docs")
    $raw_toml | save --force --raw ($config_dir | path join "yazelix.toml")

    if $migration_notes {
        let notes_path = ($runtime_dir | path join "docs" "upgrade_notes.toml")
        let notes = (open $notes_path)
        let updated_release = (
            ($notes.releases | get $YAZELIX_VERSION)
            | upsert headline $"Config migration follow-up after the ($YAZELIX_VERSION) upgrade"
            | upsert summary [
                $"This fixture treats ($YAZELIX_VERSION) as the first release that carries config migration guidance."
                "Users with stale tray or shell toggles should migrate before relying on startup."
            ]
            | upsert upgrade_impact "migration_available"
            | upsert migration_ids [
                "remove_zellij_widget_tray_layout"
                "remove_shell_enable_atuin"
            ]
            | upsert manual_actions []
        )
        let updated_notes = ($notes | upsert releases ($notes.releases | upsert $YAZELIX_VERSION $updated_release))
        $updated_notes | to toml | save --force $notes_path
    }

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        state_dir: $state_dir
        config_path: ($config_dir | path join "yazelix.toml")
        yzx_script: ($runtime_dir | path join "nushell" "scripts" "core" "yazelix.nu")
    }
}

def run_config_migrate_command [fixture: record, args: list<string> = []] {
    let migrate_command = if ($args | is-empty) {
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
        ^nu -c $"use \"($fixture.yzx_script)\" *; ($migrate_command)" | complete
    }
}

def record_has_path [data: record, path: list<string>] {
    mut current = $data

    for segment in $path {
        if not ((($current | describe) | str contains "record")) {
            return false
        }

        let keys = ($current | columns)
        if not ($segment in $keys) {
            return false
        }

        $current = ($current | get $segment)
    }

    true
}

def test_yzx_status [] {
    print "🧪 Testing yzx status..."

    try {
        yzx status | ignore
        print "  ✅ yzx status runs successfully"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_status_versions_uses_invoking_path_for_versions [] {
    print "🧪 Testing yzx status --versions resolves tool versions from the invoking PATH..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)

        '#!/bin/sh
echo "zellij 9.9.9"
' | save --force --raw ($fake_bin | path join "zellij")
        ^chmod +x ($fake_bin | path join "zellij")
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        let env_overlay = {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            PATH: ([$fake_bin] | append $env.PATH)
        }

        let output = with-env $env_overlay {
            do {
                cd $fixture.runtime_dir
                ^nu -c $"use \"($fixture.yzx_script)\" *; yzx status --versions" | complete
            }
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Yazelix Tool Versions") and ($stdout | str contains "9.9.9") {
            print "  ✅ yzx status --versions uses the invoking PATH for version resolution"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_desktop_install_writes_valid_absolute_launcher [] {
    print "🧪 Testing yzx desktop install writes a valid absolute launcher entry..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        } {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx desktop install --print-path" | complete
        })
        let desktop_path = ($output.stdout | str trim)
        let desktop_file = if ($desktop_path | is-not-empty) and ($desktop_path | path exists) {
            open --raw $desktop_path
        } else {
            ""
        }
        let expected_exec = $"Exec=\"($fixture.runtime_dir | path join "shells" "posix" "desktop_launcher.sh")\""

        if ($output.exit_code == 0) and ($desktop_path == ($fixture.tmp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")) and ($desktop_file | str contains $expected_exec) {
            print "  ✅ yzx desktop install writes the desktop entry with a direct absolute launcher path"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) path=($desktop_path) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_desktop_uninstall_removes_generated_entry [] {
    print "🧪 Testing yzx desktop uninstall removes the generated entry..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let env_overlay = {
            HOME: $fixture.tmp_home
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        }
        let install_output = (with-env $env_overlay {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx desktop install --print-path" | complete
        })
        let desktop_path = ($install_output.stdout | str trim)
        let uninstall_output = (with-env $env_overlay {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx desktop uninstall --print-path" | complete
        })
        let removed_path = ($uninstall_output.stdout | str trim)

        if ($install_output.exit_code == 0) and ($uninstall_output.exit_code == 0) and ($removed_path == $desktop_path) and (not ($desktop_path | path exists)) {
            print "  ✅ yzx desktop uninstall removes the generated desktop entry"
            true
        } else {
            print $"  ❌ Unexpected result: install_exit=($install_output.exit_code) uninstall_exit=($uninstall_output.exit_code) path=($removed_path) stderr=($uninstall_output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_tutor_prints_guided_overview [] {
    print "🧪 Testing yzx tutor prints the Yazelix guided overview..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let real_nu = (which nu | get 0.path)
        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        } {
            ^$real_nu -c $"use \"($fixture.yzx_script)\" *; yzx tutor" | complete
        })
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Yazelix tutor") and ($stdout | str contains "yzx keys") and ($stdout | str contains "yzx tutor hx") and ($stdout | str contains "yzx help") {
            print "  ✅ yzx tutor prints the guided Yazelix overview"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($output.stderr | str trim)"
            print $"     stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_tutor_help_surface_stays_small [] {
    print "🧪 Testing yzx tutor help surface stays small..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let real_nu = (which nu | get 0.path)
        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        } {
            ^$real_nu -c $"use \"($fixture.yzx_script)\" *; help commands | where name =~ '^yzx tutor' | get name | to json -r" | complete
        })
        let names = (if ($output.stdout | str trim | is-empty) { [] } else { $output.stdout | from json })

        if ($output.exit_code == 0) and ($names == ["yzx tutor", "yzx tutor helix", "yzx tutor hx", "yzx tutor nu", "yzx tutor nushell"]) {
            print "  ✅ yzx tutor exposes only the intended command surface"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($output.stderr | str trim) names=($names | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_tutor_hx_delegates_to_helix_tutor [] {
    print "🧪 Testing yzx tutor hx delegates to hx --tutor..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        let args_file = ($fixture.tmp_home | path join "hx_args.txt")
        let real_nu = (which nu | get 0.path)
        mkdir $fake_bin

        [
            "#!/bin/sh"
            $"printf '%s\\n' \"$@\" > \"($args_file)\""
            "echo \"fake hx tutor\""
        ] | str join "\n" | save --force --raw ($fake_bin | path join "hx")
        ^chmod +x ($fake_bin | path join "hx")

        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            PATH: ([$fake_bin] | append $env.PATH)
        } {
            ^$real_nu -c $"use \"($fixture.yzx_script)\" *; yzx tutor hx" | complete
        })
        let stdout = ($output.stdout | str trim)
        let recorded_args = if ($args_file | path exists) {
            open --raw $args_file | str trim
        } else {
            ""
        }

        if ($output.exit_code == 0) and ($stdout == "fake hx tutor") and ($recorded_args == "--tutor") {
            print "  ✅ yzx tutor hx delegates to Helix's built-in tutor"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim) args=($recorded_args)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_tutor_nu_delegates_to_nushell_tutor [] {
    print "🧪 Testing yzx tutor nu delegates to nu -c tutor..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        let args_file = ($fixture.tmp_home | path join "nu_args.txt")
        let real_nu = (which nu | get 0.path)
        mkdir $fake_bin

        [
            "#!/bin/sh"
            $"printf '%s\\n' \"$@\" > \"($args_file)\""
            "echo \"fake nu tutor\""
        ] | str join "\n" | save --force --raw ($fake_bin | path join "nu")
        ^chmod +x ($fake_bin | path join "nu")

        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            PATH: ([$fake_bin] | append $env.PATH)
        } {
            ^$real_nu -c $"use \"($fixture.yzx_script)\" *; yzx tutor nu" | complete
        })
        let stdout = ($output.stdout | str trim)
        let recorded_args = if ($args_file | path exists) {
            open --raw $args_file | lines
        } else {
            []
        }

        if ($output.exit_code == 0) and ($stdout == "fake nu tutor") and ($recorded_args == ["-c", "tutor"]) {
            print "  ✅ yzx tutor nu delegates to Nushell's built-in tutor"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim) args=($recorded_args | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_config_migration_rule_metadata_is_complete [] {
    print "🧪 Testing config migration rule metadata completeness..."

    try {
        let errors = (validate_config_migration_rules)

        if ($errors | is-empty) {
            print "  ✅ Config migration rules expose complete metadata"
            true
        } else {
            print $"  ❌ Unexpected metadata errors: ($errors | str join ' | ')"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_config_migration_plan_orders_safe_rewrites [] {
    print "🧪 Testing config migration plan ordering and deterministic rewrites..."

    try {
        let plan = (build_config_migration_plan_from_record {
            zellij: { widget_tray: ["layout", "editor", "cpu"] }
            terminal: {
                preferred_terminal: "ghostty"
                extra_terminals: ["wezterm", "kitty", "wezterm"]
            }
            shell: { enable_atuin: true }
        })
        let ids = ($plan.results | get id)
        let migrated = $plan.migrated_config

        if (
            ($ids == [
                "remove_zellij_widget_tray_layout",
                "unify_terminal_preference_list",
                "remove_shell_enable_atuin"
            ])
            and ($plan.auto_count == 3)
            and ($plan.manual_count == 0)
            and (($migrated | get zellij.widget_tray) == ["editor", "cpu"])
            and (($migrated | get terminal.terminals) == ["ghostty", "wezterm", "kitty"])
            and not (record_has_path $migrated ["terminal", "preferred_terminal"])
            and not (record_has_path $migrated ["terminal", "extra_terminals"])
            and not (record_has_path $migrated ["shell", "enable_atuin"])
        ) {
            print "  ✅ Migration plan preserves rule order and applies deterministic rewrites"
            true
        } else {
            print $"  ❌ Unexpected plan result: ids=($ids | to json -r) migrated=($migrated | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_config_migration_plan_marks_ambiguous_cases_manual [] {
    print "🧪 Testing config migration plan leaves ambiguous cases manual-only..."

    try {
        let plan = (build_config_migration_plan_from_record {
            terminal: {
                preferred_terminal: "ghostty"
                terminals: ["kitty"]
                cursor_trail: "random"
            }
        })
        let manual_ids = ($plan.manual_results | get id)

        if (
            ($plan.auto_count == 0)
            and ($plan.manual_count == 2)
            and ($manual_ids == [
                "unify_terminal_preference_list",
                "review_legacy_cursor_trail_settings"
            ])
        ) {
            print "  ✅ Conflicting or lossy migrations stay manual-only"
            true
        } else {
            print $"  ❌ Unexpected manual plan: ($plan | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_config_migration_preview_rendering_is_high_signal [] {
    print "🧪 Testing config migration preview rendering..."

    try {
        let plan = (build_config_migration_plan_from_record {
            zellij: { widget_tray: ["layout", "editor"] }
            terminal: { cursor_trail: "snow" }
        } "/tmp/yazelix.toml")
        let rendered = (render_config_migration_plan $plan)

        if (
            ($rendered | str contains "[AUTO] remove_zellij_widget_tray_layout")
            and ($rendered | str contains '[MANUAL] review_legacy_cursor_trail_settings')
            and ($rendered | str contains 'Preview only. Re-run with `yzx config migrate --apply`')
            and ($rendered | str contains "Manual-only items will stay untouched on apply.")
        ) {
            print "  ✅ Migration preview clearly distinguishes safe and manual cases"
            true
        } else {
            print $"  ❌ Unexpected preview output: ($rendered)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_migrate_preview_reports_known_migrations [] {
    print "🧪 Testing yzx config migrate preview reports safe and manual migrations..."

    let fixture = (setup_config_migrate_fixture
        "yazelix_migrate_preview"
        '[terminal]
preferred_terminal = "ghostty"
extra_terminals = ["wezterm"]
cursor_trail = "random"

[zellij]
widget_tray = ["layout", "editor"]

[shell]
enable_atuin = true
')

    let result = (try {
        let output = (run_config_migrate_command $fixture)
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "[AUTO] remove_zellij_widget_tray_layout")
            and ($stdout | str contains "[AUTO] unify_terminal_preference_list")
            and ($stdout | str contains "[AUTO] remove_shell_enable_atuin")
            and ($stdout | str contains "[MANUAL] review_legacy_cursor_trail_settings")
        ) {
            print "  ✅ yzx config migrate preview reports real migration matches"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_config_migrate_apply_rewrites_config_with_backup [] {
    print "🧪 Testing yzx config migrate apply rewrites config with backup..."

    let fixture = (setup_config_migrate_fixture
        "yazelix_migrate_apply"
        '[terminal]
preferred_terminal = "ghostty"
extra_terminals = ["wezterm", "kitty", "wezterm"]

[zellij]
widget_tray = ["layout", "editor"]

[shell]
enable_atuin = true
')

    let result = (try {
        let output = (run_config_migrate_command $fixture ["--apply", "--yes"])
        let stdout = ($output.stdout | str trim)
        let updated = (open $fixture.config_path)
        let backups = (
            ls $fixture.config_dir
            | where name =~ 'yazelix\.toml\.backup-'
        )

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Backed up previous config")
            and ($stdout | str contains "Applied 3 config migration")
            and (($updated | get terminal.terminals) == ["ghostty", "wezterm", "kitty"])
            and (($updated | get zellij.widget_tray) == ["editor"])
            and not (record_has_path $updated ["terminal", "preferred_terminal"])
            and not (record_has_path $updated ["terminal", "extra_terminals"])
            and not (record_has_path $updated ["shell", "enable_atuin"])
            and (($backups | length) == 1)
        ) {
            print "  ✅ yzx config migrate applies deterministic rewrites and keeps a backup"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) updated=($updated | to json -r) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_config_migrate_apply_noops_on_current_config [] {
    print "🧪 Testing yzx config migrate apply noops on a current config..."

    let fixture = (setup_config_migrate_fixture
        "yazelix_migrate_clean"
        (open --raw (repo_path "yazelix_default.toml"))
    )

    let result = (try {
        let output = (run_config_migrate_command $fixture ["--apply", "--yes"])
        let stdout = ($output.stdout | str trim)
        let backups = (
            ls $fixture.config_dir
            | where name =~ 'yazelix\.toml\.backup-'
        )

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "No known config migrations detected.")
            and ($stdout | str contains "No safe config rewrites to apply.")
            and (($backups | length) == 0)
        ) {
            print "  ✅ yzx config migrate leaves current configs untouched"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_upgrade_summary_first_run_marks_seen_and_second_run_stays_quiet [] {
    print "🧪 Testing first-run upgrade summary marks the version as seen..."

    let fixture = (setup_upgrade_summary_fixture
        "yazelix_upgrade_summary_seen"
        (open --raw (repo_path "yazelix_default.toml"))
    )

    let result = (try {
        let first = with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_STATE_DIR: $fixture.state_dir
        } {
            maybe_show_first_run_upgrade_summary
        }
        let second = with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_STATE_DIR: $fixture.state_dir
        } {
            maybe_show_first_run_upgrade_summary
        }
        let stored_version = (open --raw ($fixture.state_dir | path join "state" "upgrade_summary" "last_seen_version.txt") | str trim)

        if (
            ($first.shown == true)
            and ($first.output | str contains $"What's New In Yazelix ($YAZELIX_VERSION)")
            and ($second.shown == false)
            and ($second.reason == "already_seen")
            and ($stored_version == $YAZELIX_VERSION)
        ) {
            print "  ✅ First-run upgrade summary persists the seen version and suppresses the repeat"
            true
        } else {
            print $"  ❌ Unexpected result: first=($first | to json -r) second=($second | to json -r) stored=($stored_version)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_upgrade_summary_report_detects_matching_migrations [] {
    print "🧪 Testing upgrade summary report detects matching migration candidates..."

    let fixture = (setup_upgrade_summary_fixture
        "yazelix_upgrade_summary_migrations"
        '[zellij]
widget_tray = ["layout", "editor"]

[shell]
enable_atuin = true
'
        --migration-notes
    )

    let result = (try {
        let report = with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_STATE_DIR: $fixture.state_dir
        } {
            build_current_upgrade_summary_report
        }

        if (
            ($report.found == true)
            and (($report.matching_migration_ids | length) == 2)
            and ("remove_zellij_widget_tray_layout" in $report.matching_migration_ids)
            and ("remove_shell_enable_atuin" in $report.matching_migration_ids)
            and ($report.output | str contains "Detected matching migration candidates in your current config")
            and ($report.output | str contains "yzx config migrate --apply")
        ) {
            print "  ✅ Upgrade summary report points current stale config at the migration flow"
            true
        } else {
            print $"  ❌ Unexpected report: ($report | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_whats_new_reopens_current_summary_even_after_seen [] {
    print "🧪 Testing yzx whats_new reopens the current summary even after the version was seen..."

    let fixture = (setup_upgrade_summary_fixture
        "yazelix_upgrade_summary_reopen"
        '[zellij]
widget_tray = ["layout", "editor"]
'
        --migration-notes
    )

    let result = (try {
        mkdir ($fixture.state_dir | path join "state" "upgrade_summary")
        $YAZELIX_VERSION | save --force --raw ($fixture.state_dir | path join "state" "upgrade_summary" "last_seen_version.txt")
        let output = with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_STATE_DIR: $fixture.state_dir
        } {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx whats_new" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains $"What's New In Yazelix ($YAZELIX_VERSION)")
            and ($stdout | str contains "Detected matching migration candidates in your current config")
            and ($stdout | str contains "Reopen later: `yzx whats_new`")
        ) {
            print "  ✅ yzx whats_new reopens the current release summary regardless of seen state"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_historical_upgrade_notes_cover_v12_v13_tag_floor [] {
    print "🧪 Testing historical upgrade notes cover the supported v12/v13 tag floor..."

    let repo_root = (get_repo_config_dir)
    let repo_git_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_historical_notes_XXXXXX | str trim)
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir

    let result = (try {
        let tags = (
            ^git -C $repo_git_root tag --sort=creatordate
            | lines
            | where {|tag| ($tag | str starts-with "v12") or ($tag | str starts-with "v13") }
        )
        let notes = (open ($repo_root | path join "docs" "upgrade_notes.toml"))
        let release_keys = ($notes.releases | columns)
        let missing = ($tags | where {|tag| not ($tag in $release_keys) })
        let sample_versions = ["v12", "v12.10", "v13.2", "v13.7"]
        let reports = (
            $sample_versions
            | each {|version|
                with-env {
                    HOME: $tmp_home
                    YAZELIX_RUNTIME_DIR: $repo_root
                    YAZELIX_CONFIG_OVERRIDE: ($repo_root | path join "yazelix_default.toml")
                    YAZELIX_STATE_DIR: $state_dir
                } {
                    build_upgrade_summary_report $version
                }
            }
        )

        if (
            ($missing | is-empty)
            and ($reports | all {|report| $report.found == true })
            and (($reports | where version == "v12.10" | first).entry.upgrade_impact == "migration_available")
            and (($reports | where version == "v13.2" | first).entry.upgrade_impact == "manual_action_required")
        ) {
            print "  ✅ Historical notes cover the supported v12/v13 tag floor and load through exact-version reports"
            true
        } else {
            print $"  ❌ Missing tags: ($missing | to json -r)"
            print $"  ❌ Reports: ($reports | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_yzx_config_view [] {
    print "🧪 Testing yzx config..."

    try {
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let output = (
            ^nu -c $"use \"($yzx_script)\" *; yzx config | columns | str join ','" | complete
        ).stdout | str trim

        if ($output | str contains "core") and ($output | str contains "terminal") and not ($output | str contains "packs") {
            print "  ✅ yzx config hides packs by default"
            true
        } else {
            print $"  ❌ Unexpected output: ($output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_sections [] {
    print "🧪 Testing yzx config section views..."

    try {
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let yazi_merger = (repo_path "nushell" "scripts" "setup" "yazi_config_merger.nu")
        let zellij_merger = (repo_path "nushell" "scripts" "setup" "zellij_config_merger.nu")
        let root = (get_repo_config_dir)
        ^nu -c $"use \"($yazi_merger)\" *; generate_merged_yazi_config \"($root)\" --quiet | ignore" | complete | ignore
        let hx_output = (^nu -c $"use \"($yzx_script)\" *; yzx config hx | columns | str join ','" | complete).stdout | str trim
        let yazi_output = (^nu -c $"use \"($yzx_script)\" *; yzx config yazi | columns | str join ','" | complete).stdout | str trim
        if (which zellij | is-empty) {
            if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") {
                print "  ℹ️  Skipping zellij config section check because zellij is not available"
                print "  ✅ yzx config section commands return focused sections"
                return true
            }
        }

        ^nu -c $"use \"($zellij_merger)\" *; generate_merged_zellij_config \"($root)\" | ignore" | complete | ignore
        let zellij_output = (^nu -c $"use \"($yzx_script)\" *; yzx config zellij" | complete).stdout | str trim

        if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") and ($zellij_output | str contains "default_layout") {
            print "  ✅ yzx config section commands return focused sections"
            true
        } else {
            print $"  ❌ Unexpected section output: hx=($hx_output) yazi=($yazi_output) zellij=($zellij_output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_reset_replaces_with_backup [] {
    print "🧪 Testing yzx config reset replaces the config with backup..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_config_reset_XXXXXX | str trim)
    let temp_yazelix_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir $temp_yazelix_dir

    let result = (try {
        ^ln -s ($repo_root | path join "nushell") ($temp_yazelix_dir | path join "nushell")
        cp ($repo_root | path join "yazelix_default.toml") ($temp_yazelix_dir | path join "yazelix_default.toml")
        '[shell]
default_shell = "bash"
' | save --force --raw ($temp_yazelix_dir | path join "yazelix.toml")

        let temp_yzx_script = ($temp_yazelix_dir | path join "nushell" "scripts" "core" "yazelix.nu")
        let output = with-env { HOME: $tmp_home, YAZELIX_DIR: $temp_yazelix_dir } {
            ^nu -c $"use \"($temp_yzx_script)\" *; yzx config reset --yes" | complete
        }
        let stdout = ($output.stdout | str trim)
        let new_config = (open --raw ($temp_yazelix_dir | path join "yazelix.toml"))
        let default_config = (open --raw ($temp_yazelix_dir | path join "yazelix_default.toml"))
        let backups = (
            ls $temp_yazelix_dir
            | where name =~ 'yazelix\.toml\.backup-'
        )
        let backup_content = if ($backups | is-empty) { "" } else { open --raw (($backups | first).name) }

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Backed up previous config")
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template")
            and ($new_config == $default_config)
            and ($backup_content | str contains 'default_shell = "bash"')
        ) {
            print "  ✅ yzx config reset backs up the current config and restores the template"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) backups=(($backups | length))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_invalid_config_is_classified_as_config_problem [] {
    print "🧪 Testing invalid config values are classified as config problems..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_invalid_config_XXXXXX | str trim)
    let temp_yazelix_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir $temp_yazelix_dir

    let result = (try {
        ^ln -s ($repo_root | path join "nushell") ($temp_yazelix_dir | path join "nushell")
        cp ($repo_root | path join "yazelix_default.toml") ($temp_yazelix_dir | path join "yazelix_default.toml")

        let invalid_config = (
            open ($repo_root | path join "yazelix_default.toml")
            | upsert core.refresh_output "loud"
        )
        $invalid_config | to toml | save ($temp_yazelix_dir | path join "yazelix.toml")

        let parser_script = ($temp_yazelix_dir | path join "nushell" "scripts" "utils" "config_parser.nu")
        let output = with-env { HOME: $tmp_home, YAZELIX_DIR: $temp_yazelix_dir } {
            ^nu -c $"source \"($parser_script)\"; try { parse_yazelix_config | ignore } catch {|err| print $err.msg }" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Unsupported config value at core.refresh_output")
            and ($stdout | str contains "Invalid value for core.refresh_output: loud")
            and ($stdout | str contains "Failure class: config problem.")
            and ($stdout | str contains "yzx config reset --yes")
        ) {
            print "  ✅ Invalid config values are classified as config problems"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_startup_reports_known_config_migration_before_generic_wrappers [] {
    print "🧪 Testing startup reports known config migrations before generic wrappers..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_startup_migration_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        '[zellij]
widget_tray = ["layout", "editor"]
' | save --force --raw ($temp_config_dir | path join "yazelix.toml")

        let inner_script = ($repo_root | path join "nushell" "scripts" "core" "start_yazelix_inner.nu")
        let output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"source \"($inner_script)\"; try { main \"($tmp_home)\" \"($tmp_home | path join "unused.kdl")\" } catch {|err| print $err.msg }" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Known migration at zellij.widget_tray")
            and ($stdout | str contains "after v13.7 on 2026-03-27")
            and ($stdout | str contains "yzx config migrate --apply")
            and ($stdout | str contains "yzx doctor --fix")
            and not ($stdout | str contains "Failed to generate Zellij configuration")
        ) {
            print "  ✅ Startup surfaces migration-aware config failures before generic wrappers"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_config_state_supports_split_config_and_runtime_dirs [] {
    print "🧪 Testing config state supports split config and runtime directories..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_split_roots_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        cp ($repo_root | path join "yazelix_default.toml") ($temp_config_dir | path join "yazelix.toml")
        let state_script = ($repo_root | path join "nushell" "scripts" "utils" "config_state.nu")
        let snippet = ([
            $"source \"($state_script)\""
            'let state = (compute_config_state)'
            'print ({'
            '    config_file: $state.config_file'
            '    lock_hash_empty: (($state.lock_hash | default "") | is-empty)'
            '    runtime_lock_path: ($env.YAZELIX_RUNTIME_DIR | path join "devenv.lock")'
            '} | to json -r)'
        ] | str join "\n")
        let output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $snippet | complete
        }
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)

        if (
            ($output.exit_code == 0)
            and ($resolved.config_file == ($temp_config_dir | path join "yazelix.toml"))
            and ($resolved.lock_hash_empty == false)
            and (($resolved.runtime_lock_path | path exists))
        ) {
            print "  ✅ Config state reads config from the config dir and hashes inputs from the runtime dir"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_config_reset_supports_split_config_and_runtime_dirs [] {
    print "🧪 Testing yzx config reset supports split config and runtime directories..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_split_reset_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        '[shell]
default_shell = "bash"
' | save --force --raw ($temp_config_dir | path join "yazelix.toml")

        let yzx_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx config reset --yes" | complete
        }
        let stdout = ($output.stdout | str trim)
        let new_config = (open --raw ($temp_config_dir | path join "yazelix.toml"))
        let default_config = (open --raw ($repo_root | path join "yazelix_default.toml"))

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template")
            and ($new_config == $default_config)
        ) {
            print "  ✅ yzx config reset reads the template from the runtime root and writes to the config root"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_shell_hook_generation_uses_runtime_root [] {
    print "🧪 Testing shell hook generation uses the configured runtime root..."

    let runtime_dir = "/tmp/yazelix-runtime"

    let result = (try {
        let bash_section = (get_yazelix_section_content "bash" $runtime_dir)
        let nu_section = (get_yazelix_section_content "nushell" $runtime_dir)

        if (
            ($bash_section | str contains $"source \"($runtime_dir)/shells/bash/yazelix_bash_config.sh\"")
            and ($bash_section | str contains $"use ($runtime_dir)/nushell/scripts/core/yazelix.nu *; yzx $*")
            and ($nu_section | str contains $"source \"($runtime_dir)/nushell/config/config.nu\"")
            and ($nu_section | str contains $"use ($runtime_dir)/nushell/scripts/core/yazelix.nu *")
            and not ($bash_section | str contains "~/.config/yazelix")
            and not ($nu_section | str contains "~/.config/yazelix")
        ) {
            print "  ✅ Shell hook generation resolves sourced Yazelix files from the runtime root"
            true
        } else {
            print "  ❌ Shell hook generation still contains stale repo-shaped runtime paths"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    $result
}

def test_shell_hook_version_check_accepts_runtime_root_hooks [] {
    print "🧪 Testing shell hook version checks accept relocated runtime paths..."

    let tmp_home = (^mktemp -d /tmp/yazelix_shell_hooks_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    mkdir $runtime_dir

    let result = (try {
        let bashrc = ($tmp_home | path join ".bashrc")
        let config_nu = ($tmp_home | path join ".config" "nushell" "config.nu")
        mkdir ($tmp_home | path join ".config")
        mkdir ($tmp_home | path join ".config" "nushell")

        (get_yazelix_section_content "bash" $runtime_dir) | save --force --raw $bashrc
        (get_yazelix_section_content "nushell" $runtime_dir) | save --force --raw $config_nu

        let statuses = (with-env { HOME: $tmp_home } { check_config_versions $runtime_dir })
        let bash_status = ($statuses | where shell == "bash" | first)
        let nu_status = ($statuses | where shell == "nushell" | first)

        if ($bash_status.status == "current") and ($nu_status.status == "current") {
            print "  ✅ Shell hook version checks treat relocated runtime hooks as current"
            true
        } else {
            print $"  ❌ Unexpected statuses: bash=($bash_status.status) nushell=($nu_status.status)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_bash_runtime_config_uses_its_own_runtime_root [] {
    print "🧪 Testing bash runtime config derives its own Yazelix runtime root..."

    let tmp_home = (^mktemp -d /tmp/yazelix_bash_runtime_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let bash_runtime_dir = ($runtime_dir | path join "shells" "bash")
    mkdir $bash_runtime_dir

    let result = (try {
        cp (repo_path "shells" "bash" "yazelix_bash_config.sh") ($bash_runtime_dir | path join "yazelix_bash_config.sh")
        mkdir ($tmp_home | path join ".local" "share" "yazelix" "initializers" "bash")

        let output = with-env { HOME: $tmp_home, YAZELIX_HELIX_MODE: "release" } {
            ^bash -lc $"source \"($bash_runtime_dir | path join "yazelix_bash_config.sh")\"; alias yazelix" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains $"alias yazelix='nu ($runtime_dir)/nushell/scripts/core/launch_yazelix.nu'") {
            print "  ✅ Bash runtime config routes the yazelix alias through its own runtime root"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_runtime_shell_assets_avoid_repo_shaped_runtime_paths [] {
    print "🧪 Testing runtime shell assets avoid repo-shaped internal script paths..."

    try {
        let bash_config = (open --raw (repo_path "shells" "bash" "yazelix_bash_config.sh"))
        let fish_config = (open --raw (repo_path "shells" "fish" "yazelix_fish_config.fish"))
        let zsh_config = (open --raw (repo_path "shells" "zsh" "yazelix_zsh_config.zsh"))

        if (
            not ($bash_config | str contains "~/.config/yazelix/nushell/scripts")
            and not ($fish_config | str contains "~/.config/yazelix/nushell/scripts")
            and not ($zsh_config | str contains "~/.config/yazelix/nushell/scripts")
            and ($bash_config | str contains 'YAZELIX_RUNTIME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"')
            and ($fish_config | str contains 'set -l YAZELIX_RUNTIME_DIR')
            and ($zsh_config | str contains 'YAZELIX_RUNTIME_DIR="$(cd "$(dirname "${(%):-%N}")/../.." && pwd)"')
        ) {
            print "  ✅ Runtime shell assets derive internal script paths from their own runtime root"
            true
        } else {
            print "  ❌ One or more runtime shell assets still hardcode repo-shaped internal script paths"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_pane_orchestrator_tracked_path_defaults_to_runtime_root [] {
    print "🧪 Testing pane orchestrator tracked path defaults to the runtime root..."

    let runtime_dir = "/tmp/yazelix-runtime"

    try {
        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let output = with-env { YAZELIX_RUNTIME_DIR: $runtime_dir } {
            ^nu -c $"source \"($helper_script)\"; get_tracked_pane_orchestrator_wasm_path" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == ($runtime_dir | path join "configs" "zellij" "plugins" "yazelix_pane_orchestrator.wasm")) {
            print "  ✅ Pane orchestrator helpers default to the configured runtime root"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_pane_orchestrator_permission_cache_is_preserved_for_stable_runtime_path [] {
    print "🧪 Testing pane orchestrator sync preserves granted permissions for the stable runtime path..."

    let tmp_home = (^mktemp -d /tmp/yazelix_plugin_permissions_XXXXXX | str trim)
    let tracked_path = ($tmp_home | path join ".config" "yazelix" "configs" "zellij" "plugins" "yazelix_pane_orchestrator.wasm")
    let runtime_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_pane_orchestrator.wasm")
    let permissions_dir = ($tmp_home | path join ".cache" "zellij")
    let permissions_path = ($permissions_dir | path join "permissions.kdl")

    let result = (try {
        mkdir ($tracked_path | path dirname)
        mkdir ($runtime_path | path dirname)
        mkdir $permissions_dir

        let existing_block = [
            $"\"($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_pane_orchestrator_deadbeef1234.wasm")\" {"
            "    ReadApplicationState"
            "    OpenTerminalsOrPlugins"
            "    ChangeApplicationState"
            "    WriteToStdin"
            "    ReadCliPipes"
            "}"
        ] | str join "\n"
        $existing_block | save --force --raw $permissions_path

        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let snippet = ([
            $"source '($helper_script)'"
            ("let result = (preserve_pane_orchestrator_permissions '"
                + $tracked_path
                + "' '"
                + $runtime_path
                + "')")
            "print ($result | to json -r)"
        ] | str join "\n")
        let output = with-env { HOME: $tmp_home } {
            ^nu -c $snippet | complete
        }
        let stdout = ($output.stdout | str trim)
        let cache_contents = (open --raw $permissions_path)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains '"status":"updated"')
            and ($cache_contents | str contains $tracked_path)
            and ($cache_contents | str contains $runtime_path)
            and (($cache_contents | str contains "OpenTerminalsOrPlugins"))
            and (($cache_contents | str contains "WriteToStdin"))
        ) {
            print "  ✅ Granted pane-orchestrator permissions are preserved onto the stable runtime path"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim) cache=($cache_contents)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_popup_runner_tracked_path_defaults_to_runtime_root [] {
    print "🧪 Testing popup runner tracked path defaults to the runtime root..."

    let tmp_home = (^mktemp -d /tmp/yazelix_popup_runner_paths_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    mkdir ($runtime_dir | path join "configs" "zellij" "plugins")

    let result = (try {
        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let output = with-env { YAZELIX_RUNTIME_DIR: $runtime_dir } {
            ^nu -c $"source \"($helper_script)\"; get_tracked_popup_runner_wasm_path" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == ($runtime_dir | path join "configs" "zellij" "plugins" "yazelix_popup_runner.wasm")) {
            print "  ✅ Popup runner helpers default to the configured runtime root"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_popup_runner_permission_cache_is_preserved_for_stable_runtime_path [] {
    print "🧪 Testing popup runner sync preserves granted permissions for the stable runtime path..."

    let tmp_home = (^mktemp -d /tmp/yazelix_popup_permissions_XXXXXX | str trim)
    let tracked_path = ($tmp_home | path join ".config" "yazelix" "configs" "zellij" "plugins" "yazelix_popup_runner.wasm")
    let runtime_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_popup_runner.wasm")
    let permissions_dir = ($tmp_home | path join ".cache" "zellij")
    let permissions_path = ($permissions_dir | path join "permissions.kdl")

    let result = (try {
        mkdir ($tracked_path | path dirname)
        mkdir ($runtime_path | path dirname)
        mkdir $permissions_dir

        let existing_block = [
            $"\"($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_popup_runner_deadbeef1234.wasm")\" {"
            "    ReadApplicationState"
            "    ChangeApplicationState"
            "    ReadCliPipes"
            "}"
        ] | str join "\n"
        $existing_block | save --force --raw $permissions_path

        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let snippet = ([
            $"source '($helper_script)'"
            ("let result = (sync_popup_runner_runtime_wasm '" + ($tmp_home | path join ".config" "yazelix") + "')")
            "print $result"
        ] | str join "\n")

        # materialize tracked file so sync can copy it
        "popup" | save --force --raw $tracked_path

        let output = with-env { HOME: $tmp_home } {
            ^nu -c $snippet | complete
        }
        let stdout = ($output.stdout | str trim)
        let cache_contents = (open --raw $permissions_path)

        if (
            ($output.exit_code == 0)
            and ($stdout == $runtime_path)
            and ($cache_contents | str contains $tracked_path)
            and ($cache_contents | str contains $runtime_path)
            and ($cache_contents | str contains "ReadCliPipes")
            and not ($cache_contents | str contains "OpenTerminalsOrPlugins")
            and not ($cache_contents | str contains "RunCommands")
        ) {
            print "  ✅ Granted popup-runner permissions are preserved onto the stable runtime path"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim) cache=($cache_contents)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_seed_yazelix_plugin_permissions_materializes_required_grants [] {
    print "🧪 Testing zellij permission repair seeds Yazelix plugin grants..."

    let tmp_home = (^mktemp -d /tmp/yazelix_seed_permissions_XXXXXX | str trim)
    let yazelix_dir = ($tmp_home | path join ".config" "yazelix")
    let tracked_plugins_dir = ($yazelix_dir | path join "configs" "zellij" "plugins")
    let permissions_path = ($tmp_home | path join ".cache" "zellij" "permissions.kdl")
    let runtime_pane_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_pane_orchestrator.wasm")
    let runtime_popup_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_popup_runner.wasm")

    let result = (try {
        mkdir $tracked_plugins_dir
        "pane" | save --force --raw ($tracked_plugins_dir | path join "yazelix_pane_orchestrator.wasm")
        "popup" | save --force --raw ($tracked_plugins_dir | path join "yazelix_popup_runner.wasm")

        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let snippet = ([
            $"source '($helper_script)'"
            ("let result = (seed_yazelix_plugin_permissions '" + $yazelix_dir + "')")
            "print ($result | to json -r)"
        ] | str join "\n")
        let output = with-env { HOME: $tmp_home } {
            ^nu -c $snippet | complete
        }
        let stdout = ($output.stdout | str trim)
        let cache_contents = (open --raw $permissions_path)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains $permissions_path)
            and ($runtime_pane_path | path exists)
            and ($runtime_popup_path | path exists)
            and ($cache_contents | str contains ($tracked_plugins_dir | path join "yazelix_pane_orchestrator.wasm"))
            and ($cache_contents | str contains $runtime_pane_path)
            and ($cache_contents | str contains "OpenTerminalsOrPlugins")
            and ($cache_contents | str contains "WriteToStdin")
            and ($cache_contents | str contains ($tracked_plugins_dir | path join "yazelix_popup_runner.wasm"))
            and ($cache_contents | str contains $runtime_popup_path)
            and ($cache_contents | str contains "ReadCliPipes")
        ) {
            print "  ✅ Yazelix permission repair seeds both plugin grants and runtime wasm paths"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim) cache=($cache_contents)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_packs_helper_uses_runtime_root_for_devenv_links [] {
    print "🧪 Testing yzx packs helper reads .devenv links from the runtime root..."

    let tmp_home = (^mktemp -d /tmp/yazelix_packs_runtime_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let fake_store_target = ($tmp_home | path join "fake-shell")
    mkdir ($runtime_dir | path join ".devenv" "gc")
    mkdir $fake_store_target

    let result = (try {
        ^ln -s $fake_store_target ($runtime_dir | path join ".devenv" "gc" "shell")
        let packs_script = (repo_path "nushell" "scripts" "yzx" "packs.nu")
        let output = with-env { YAZELIX_RUNTIME_DIR: $runtime_dir } {
            ^nu -c $"source \"($packs_script)\"; get_devenv_shell" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == $fake_store_target) {
            print "  ✅ yzx packs resolves .devenv links from the runtime root"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_relocated_runtime_smoke_supports_status_and_terminal_config_rendering [] {
    print "🧪 Testing relocated runtime smoke path supports status and terminal-config rendering..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let env_overlay = {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        }

        let status_output = with-env $env_overlay {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx status" | complete
        }
        let gen_output = with-env $env_overlay { ^nu -c $"use \"($fixture.runtime_dir | path join "nushell" "scripts" "yzx" "gen_config.nu")\" [render_terminal_config]; render_terminal_config ghostty" | complete }

        let status_stdout = ($status_output.stdout | str trim)
        let gen_stdout = ($gen_output.stdout | str trim)

        if (
            ($status_output.exit_code == 0)
            and ($gen_output.exit_code == 0)
            and ($status_stdout | str contains $"Config File: ($fixture.config_dir | path join "yazelix.toml")")
            and ($status_stdout | str contains $"Directory: ($fixture.runtime_dir)")
            and ($status_stdout | str contains $"Logs: ($fixture.runtime_dir | path join "logs")")
            and ($gen_stdout | str contains $"exec ($fixture.startup_script)")
            and not ($gen_stdout | str contains $fixture.repo_root)
        ) {
            print "  ✅ Relocated runtime smoke path resolves config, runtime, and internal terminal launchers from split roots"
            true
        } else {
            print $"  ❌ Unexpected result: status_exit=($status_output.exit_code) gen_exit=($gen_output.exit_code)"
            print $"     status=($status_stdout)"
            print $"     gen=($gen_stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

export def run_core_canonical_tests [] {
    [
        (test_yzx_status)
        (test_yzx_status_versions_uses_invoking_path_for_versions)
        (test_yzx_desktop_install_writes_valid_absolute_launcher)
        (test_yzx_desktop_uninstall_removes_generated_entry)
        (test_yzx_tutor_prints_guided_overview)
        (test_yzx_tutor_help_surface_stays_small)
        (test_yzx_tutor_hx_delegates_to_helix_tutor)
        (test_yzx_tutor_nu_delegates_to_nushell_tutor)
        (test_config_migration_rule_metadata_is_complete)
        (test_config_migration_plan_orders_safe_rewrites)
        (test_config_migration_plan_marks_ambiguous_cases_manual)
        (test_config_migration_preview_rendering_is_high_signal)
        (test_yzx_config_migrate_preview_reports_known_migrations)
        (test_yzx_config_migrate_apply_rewrites_config_with_backup)
        (test_yzx_config_migrate_apply_noops_on_current_config)
        (test_upgrade_summary_first_run_marks_seen_and_second_run_stays_quiet)
        (test_upgrade_summary_report_detects_matching_migrations)
        (test_yzx_whats_new_reopens_current_summary_even_after_seen)
        (test_historical_upgrade_notes_cover_v12_v13_tag_floor)
        (test_invalid_config_is_classified_as_config_problem)
        (test_startup_reports_known_config_migration_before_generic_wrappers)
        (test_config_state_supports_split_config_and_runtime_dirs)
        (test_relocated_runtime_smoke_supports_status_and_terminal_config_rendering)
    ]
}

export def run_core_noncanonical_tests [] {
    [
        (test_seed_yazelix_plugin_permissions_materializes_required_grants)
        (test_yzx_config_reset_replaces_with_backup)
    ]
}

export def run_core_tests [] {
    [
        (run_core_canonical_tests)
        (run_core_noncanonical_tests)
    ] | flatten
}
