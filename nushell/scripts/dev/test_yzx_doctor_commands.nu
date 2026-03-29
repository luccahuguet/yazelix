#!/usr/bin/env nu

use ../core/yazelix.nu *
use ./test_yzx_helpers.nu [CLEAN_ZELLIJ_ENV_PREFIX get_repo_config_dir repo_path setup_managed_config_fixture]
use ../utils/doctor.nu [build_zellij_plugin_health_results]

def run_doctor_command_for_fixture [fixture: record, command: string] {
    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } {
        ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
    }
}

def test_yzx_doctor_reports_zellij_plugin_context [] {
    print "🧪 Testing yzx doctor reports Zellij plugin context..."

    try {
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let output = (^bash -lc $"($CLEAN_ZELLIJ_ENV_PREFIX) nu -c 'use \"($yzx_script)\" *; yzx doctor --verbose'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Zellij plugin health check skipped \(not inside Zellij\)") {
            print "  ✅ yzx doctor explains when Zellij-local plugin checks are skipped"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_doctor_warns_on_stale_config_fields [] {
    print "🧪 Testing yzx doctor warns about stale config fields..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_stale_fields"
        ""
    )

    let result = (try {
        ^ln -s ($fixture.repo_root | path join "nushell") ($fixture.config_dir | path join "nushell")
        cp ($fixture.repo_root | path join "yazelix_default.toml") ($fixture.config_dir | path join "yazelix_default.toml")

        let stale_config = (
            open ($fixture.repo_root | path join "yazelix_default.toml")
            | upsert core.stale_field true
            | upsert packs.declarations.custom_pack ["hello"]
            | upsert packs.enabled ["custom_pack"]
        )
        $stale_config | to toml | save --force $fixture.config_path

        let temp_yzx_script = ($fixture.config_dir | path join "nushell" "scripts" "core" "yazelix.nu")
        let output = with-env { HOME: $fixture.tmp_home, YAZELIX_DIR: $fixture.config_dir } {
            ^nu -c $"use \"($temp_yzx_script)\" *; yzx doctor --verbose" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Stale, unsupported, or migration-aware yazelix.toml entries detected")
            and ($stdout | str contains "Unknown config field: core.stale_field")
            and ($stdout | str contains "yzx config reset --yes")
            and not ($stdout | str contains "packs.declarations.custom_pack")
        ) {
            print "  ✅ yzx doctor reports stale config fields without flagging custom pack declarations"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_doctor_reports_known_migration_with_fix_guidance [] {
    print "🧪 Testing yzx doctor reports known config migrations with fix guidance..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_migration"
        '[zellij]
widget_tray = ["layout", "editor"]
')

    let result = (try {
        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Known migration at zellij.widget_tray")
            and ($stdout | str contains "Safe preview: `yzx config migrate`")
            and ($stdout | str contains "Safe apply: `yzx config migrate --apply` or `yzx doctor --fix`")
        ) {
            print "  ✅ yzx doctor reports known migrations with the shared recovery guidance"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_doctor_fix_applies_safe_config_migrations [] {
    print "🧪 Testing yzx doctor --fix applies safe config migrations..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_fix"
        '[zellij]
widget_tray = ["layout", "editor"]

[shell]
enable_atuin = true
')

    let result = (try {
        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --fix")
        let stdout = ($output.stdout | str trim)
        let rewritten = (open $fixture.config_path)
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Applied 2 config migration fix")
            and (($rewritten | get zellij.widget_tray) == ["editor"])
            and not (($rewritten.shell? | default {}) | columns | any {|column| $column == "enable_atuin" })
            and (($backups | length) == 1)
        ) {
            print "  ✅ yzx doctor --fix applies safe config migrations with backup"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) rewritten=($rewritten | to json -r) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_doctor_fix_splits_legacy_pack_config [] {
    print "🧪 Testing yzx doctor --fix relocates legacy pack config into user_configs/yazelix_packs.toml..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_fix_packs"
        '[packs]
enabled = ["git"]
user_packages = ["docker"]

[packs.declarations]
git = ["gh", "prek"]
'
        --legacy-root
    )

    let result = (try {
        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --fix")
        let stdout = ($output.stdout | str trim)
        let rewritten = (open ($fixture.user_config_dir | path join "yazelix.toml"))
        let pack_path = ($fixture.user_config_dir | path join "yazelix_packs.toml")
        let pack_rewritten = (if ($pack_path | path exists) { open $pack_path } else { null })
        let pack_rendered = (if $pack_rewritten == null { "<missing>" } else { $pack_rewritten | to json -r })
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Applied 1 config migration fix")
            and ($stdout | str contains "Wrote pack config to")
            and not ("packs" in ($rewritten | columns))
            and ($pack_rewritten.enabled == ["git"])
            and ($pack_rewritten.user_packages == ["docker"])
            and (($pack_rewritten.declarations | get git) == ["gh", "prek"])
            and (($backups | length) == 1)
            and not ($fixture.config_path | path exists)
        ) {
            print "  ✅ yzx doctor --fix relocates legacy pack ownership into user_configs"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) main=($rewritten | to json -r) pack=($pack_rendered) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_doctor_clarifies_shell_opened_editors_are_not_managed [] {
    print "🧪 Testing doctor clarifies that shell-opened editors are not managed..."

    try {
        let results = (build_zellij_plugin_health_results {
            permissions_granted: true
            active_tab_position: 0
            editor_pane_id: null
            sidebar_pane_id: null
            active_swap_layout_name: "single_open"
        } true)

        let editor_result = (
            $results
            | where message == "Managed editor pane not detected in the current tab"
            | get 0
        )

        if ($editor_result.details | str contains "An editor started manually from an ordinary shell pane does not count as the managed editor pane.") {
            print "  ✅ doctor explains that shell-opened editors do not count as managed panes"
            true
        } else {
            print $"  ❌ Unexpected details: ($editor_result.details)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_doctor_canonical_tests [] {
    [
        (test_yzx_doctor_warns_on_stale_config_fields)
        (test_yzx_doctor_reports_known_migration_with_fix_guidance)
        (test_yzx_doctor_fix_applies_safe_config_migrations)
        (test_yzx_doctor_fix_splits_legacy_pack_config)
    ]
}

export def run_doctor_noncanonical_tests [] {
    []
}

export def run_doctor_tests [] {
    [
        (run_doctor_canonical_tests)
        (run_doctor_noncanonical_tests)
    ] | flatten
}
