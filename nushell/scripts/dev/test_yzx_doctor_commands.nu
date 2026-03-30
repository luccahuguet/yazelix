#!/usr/bin/env nu
# Defends: docs/specs/test_suite_governance.md

use ../core/yazelix.nu *
use ./test_yzx_helpers.nu [get_repo_config_dir repo_path setup_managed_config_fixture]

def run_doctor_command_for_fixture [fixture: record, command: string] {
    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } {
        ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
    }
}

def test_yzx_doctor_warns_on_stale_config_fields [] {
    print "🧪 Testing yzx doctor warns about stale config fields..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_stale_fields"
        ""
    )

    let result = (try {
        let stale_config = (
            open ($fixture.repo_root | path join "yazelix_default.toml")
            | upsert core.stale_field true
            | upsert packs.declarations.custom_pack ["hello"]
            | upsert packs.enabled ["custom_pack"]
        )
        $stale_config | to toml | save --force $fixture.config_path

        let output = with-env {
            HOME: $fixture.tmp_home
            XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
        } {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx doctor --verbose" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Stale, unsupported, or migration-aware yazelix.toml entries detected")
            and ($stdout | str contains "Unknown config field: core.stale_field")
            and ($stdout | str contains "yzx config reset")
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

export def run_doctor_canonical_tests [] {
    [
        (test_yzx_doctor_warns_on_stale_config_fields)
        (test_yzx_doctor_reports_known_migration_with_fix_guidance)
        (test_yzx_doctor_fix_applies_safe_config_migrations)
        (test_yzx_doctor_fix_splits_legacy_pack_config)
    ]
}

export def run_doctor_tests [] {
    run_doctor_canonical_tests
}
