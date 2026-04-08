#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ../core/yazelix.nu *
use ../utils/config_migration_transactions.nu [get_managed_config_transaction_dir]
use ./yzx_test_helpers.nu [get_repo_config_dir repo_path setup_managed_config_fixture]

def setup_config_migrate_fixture [label: string, raw_toml: string] {
    setup_managed_config_fixture $label $raw_toml
}

def setup_legacy_root_config_migrate_fixture [label: string, raw_toml: string] {
    setup_managed_config_fixture $label $raw_toml --legacy-root
}

def run_yzx_command_for_fixture [fixture: record, command: string] {
    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } {
        ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
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

def setup_manual_install_takeover_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let runtime_target = ($fixture.tmp_home | path join "manual_runtime")
    let runtime_bin = ($runtime_target | path join "bin")
    let runtime_reference = ($fixture.tmp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let local_yzx = ($fixture.tmp_home | path join ".local" "bin" "yzx")
    let desktop_path = ($fixture.tmp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")
    let taplo_path = ($fixture.config_dir | path join ".taplo.toml")
    let generated_yazi = ($fixture.tmp_home | path join ".local" "share" "yazelix" "configs" "yazi")
    let generated_zellij = ($fixture.tmp_home | path join ".local" "share" "yazelix" "configs" "zellij")
    let pack_config_path = ($fixture.user_config_dir | path join "yazelix_packs.toml")

    mkdir $runtime_bin
    mkdir ($runtime_reference | path dirname)
    mkdir ($local_yzx | path dirname)
    mkdir ($desktop_path | path dirname)
    mkdir $generated_yazi
    mkdir $generated_zellij

    [
        "#!/bin/sh"
        "exit 0"
    ] | str join "\n" | save --force --raw ($runtime_bin | path join "yzx")
    ^chmod +x ($runtime_bin | path join "yzx")

    ^ln -s $runtime_target $runtime_reference
    ^ln -s ($runtime_reference | path join "bin" "yzx") $local_yzx

    [
        "[Desktop Entry]"
        "Type=Application"
        "Name=Yazelix"
        $"Exec=\"($local_yzx)\" desktop launch"
    ] | str join "\n" | save --force --raw $desktop_path

    'enabled = ["git"]

[declarations]
git = ["gh"]
' | save --force --raw $pack_config_path
    "[global]\n" | save --force --raw $taplo_path
    "# yazi\n" | save --force --raw ($generated_yazi | path join "yazi.toml")
    "// zellij\n" | save --force --raw ($generated_zellij | path join "config.kdl")

    $fixture | merge {
        runtime_target: $runtime_target
        runtime_reference: $runtime_reference
        local_yzx: $local_yzx
        desktop_path: $desktop_path
        taplo_path: $taplo_path
        generated_yazi: $generated_yazi
        generated_zellij: $generated_zellij
        pack_config_path: $pack_config_path
    }
}

def setup_home_manager_owned_install_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let hm_store = ($fixture.tmp_home | path join "fake-home-manager-files")
    let hm_runtime = ($hm_store | path join ".local" "share" "yazelix" "runtime" "current")
    let runtime_reference = ($fixture.tmp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let hm_main = ($hm_store | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let hm_pack = ($hm_store | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let pack_config_path = ($fixture.user_config_dir | path join "yazelix_packs.toml")

    rm -f $fixture.config_path

    mkdir ($hm_runtime | path dirname)
    mkdir ($hm_main | path dirname)
    mkdir ($runtime_reference | path dirname)

    cp ($fixture.repo_root | path join "yazelix_default.toml") $hm_main
    'enabled = ["git"]

[declarations]
git = ["gh"]
' | save --force --raw $hm_pack
    mkdir ($hm_runtime | path join "bin")
    [
        "#!/bin/sh"
        "exit 0"
    ] | str join "\n" | save --force --raw ($hm_runtime | path join "bin" "yzx")
    ^chmod +x ($hm_runtime | path join "bin" "yzx")

    ^ln -s $hm_runtime $runtime_reference
    ^ln -s $hm_main $fixture.config_path
    ^ln -s $hm_pack $pack_config_path

    $fixture | merge {
        runtime_reference: $runtime_reference
        pack_config_path: $pack_config_path
    }
}

def setup_noninstaller_runtime_reference_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let runtime_reference = ($fixture.tmp_home | path join ".local" "share" "yazelix" "runtime" "current")
    mkdir $runtime_reference
    "# not an installer symlink\n" | save --force --raw ($runtime_reference | path join "README")

    $fixture | merge {
        runtime_reference: $runtime_reference
    }
}

def run_entrypoint_preflight_command [fixture: record, entrypoint_label: string, --allow-noninteractive] {
    let helper_script = (repo_path "nushell" "scripts" "utils" "entrypoint_config_migrations.nu")
    let allow_suffix = if $allow_noninteractive { " --allow-noninteractive" } else { "" }

    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } {
        ^nu -c $"use \"($helper_script)\" [run_entrypoint_config_migration_preflight]; run_entrypoint_config_migration_preflight \"($entrypoint_label)\"($allow_suffix)" | complete
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

        $current = ($current | get -o $segment)
    }

    true
}

# Defends: config migration preview surfaces known safe changes before writes.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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

# Defends: applying config migrations rewrites user config with a backup.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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
            ls $fixture.user_config_dir
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

# Defends: legacy inline pack ownership is split into the supported sidecar surface.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yzx_config_migrate_apply_splits_legacy_packs_into_sidecar [] {
    print "🧪 Testing yzx config migrate apply moves legacy [packs] into yazelix_packs.toml..."

    let fixture = (setup_config_migrate_fixture
        "yazelix_migrate_pack_split"
        '[packs]
enabled = ["git", "go"]
user_packages = ["docker"]

[packs.declarations]
git = ["gh", "prek"]
go = ["gopls", "golangci-lint"]
')

    let result = (try {
        let output = (run_config_migrate_command $fixture ["--apply", "--yes"])
        let stdout = ($output.stdout | str trim)
        let updated_main = (open $fixture.config_path)
        let pack_path = ($fixture.user_config_dir | path join "yazelix_packs.toml")
        let updated_pack = (if ($pack_path | path exists) { open $pack_path } else { null })
        let updated_pack_rendered = (if $updated_pack == null { "<missing>" } else { $updated_pack | to json -r })
        let backups = (
            ls $fixture.user_config_dir
            | where name =~ 'yazelix\.toml\.backup-'
        )

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "[AUTO] split_legacy_pack_config_surface")
            and ($stdout | str contains "Wrote pack config to")
            and not (record_has_path $updated_main ["packs"])
            and ($updated_pack.enabled == ["git", "go"])
            and ($updated_pack.user_packages == ["docker"])
            and (($updated_pack.declarations | get git) == ["gh", "prek"])
            and (($updated_pack.declarations | get go) == ["gopls", "golangci-lint"])
            and (($backups | length) == 1)
        ) {
            print "  ✅ yzx config migrate now moves legacy pack ownership into yazelix_packs.toml"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) main=($updated_main | to json -r) pack=($updated_pack_rendered) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Invariant: current config shapes do not churn under migrate apply.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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
            ls $fixture.user_config_dir
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

# Defends: Home Manager takeover preview surfaces both blocking and cleanup-only manual artifacts.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_home_manager_prepare_preview_reports_manual_takeover_artifacts [] {
    print "🧪 Testing yzx home_manager prepare preview reports takeover blockers and cleanup-only manual artifacts..."

    let fixture = (setup_manual_install_takeover_fixture "yazelix_home_manager_prepare_preview")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx home_manager prepare")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Blocking manual-install artifacts:")
            and ($stdout | str contains $fixture.runtime_reference)
            and ($stdout | str contains $fixture.config_path)
            and ($stdout | str contains $fixture.pack_config_path)
            and ($stdout | str contains "Cleanup-only manual-install artifacts:")
            and ($stdout | str contains $fixture.local_yzx)
            and ($stdout | str contains $fixture.desktop_path)
            and ($fixture.runtime_reference | path exists)
            and ($fixture.config_path | path exists)
            and ($fixture.pack_config_path | path exists)
            and ($fixture.local_yzx | path exists)
            and ($fixture.desktop_path | path exists)
        ) {
            print "  ✅ yzx home_manager prepare preview shows the real takeover blockers and cleanup-only manual artifacts without mutating them"
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

# Defends: Home Manager takeover apply archives the blocking manual paths and points users at home-manager switch.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_home_manager_prepare_apply_archives_manual_takeover_artifacts [] {
    print "🧪 Testing yzx home_manager prepare --apply archives manual-install takeover artifacts..."

    let fixture = (setup_manual_install_takeover_fixture "yazelix_home_manager_prepare_apply")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx home_manager prepare --apply --yes")
        let stdout = ($output.stdout | str trim)
        let main_backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.home-manager-prepare-backup-')
        let pack_backups = (ls $fixture.user_config_dir | where name =~ 'yazelix_packs\.toml\.home-manager-prepare-backup-')
        let launcher_backups = (ls ($fixture.local_yzx | path dirname) | where name =~ 'yzx\.home-manager-prepare-backup-')
        let desktop_backups = (ls ($fixture.desktop_path | path dirname) | where name =~ 'com\.yazelix\.Yazelix\.desktop\.home-manager-prepare-backup-')
        let runtime_backups = (ls ($fixture.runtime_reference | path dirname) | where name =~ 'current\.home-manager-prepare-backup-')

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Archived manual-install artifacts for Home Manager takeover")
            and ($stdout | str contains "home-manager switch")
            and not ($fixture.runtime_reference | path exists)
            and not ($fixture.config_path | path exists)
            and not ($fixture.pack_config_path | path exists)
            and not ($fixture.local_yzx | path exists)
            and not ($fixture.desktop_path | path exists)
            and (($main_backups | length) == 1)
            and (($pack_backups | length) == 1)
            and (($launcher_backups | length) == 1)
            and (($desktop_backups | length) == 1)
            and (($runtime_backups | length) == 1)
        ) {
            print "  ✅ yzx home_manager prepare --apply archives the real takeover blockers and cleanup-only manual artifacts, then points users at home-manager switch"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) main_backups=(($main_backups | length)) pack_backups=(($pack_backups | length)) launcher_backups=(($launcher_backups | length)) desktop_backups=(($desktop_backups | length)) runtime_backups=(($runtime_backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: uninstall removes manual installer-owned artifacts but preserves managed config by default.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_uninstall_apply_removes_manual_artifacts_but_preserves_config [] {
    print "🧪 Testing yzx uninstall --apply removes manual installer artifacts but preserves managed config..."

    let fixture = (setup_manual_install_takeover_fixture "yazelix_uninstall_apply")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx uninstall --apply --yes")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Removed installer-owned Yazelix artifacts:")
            and ($stdout | str contains "Preserved by default:")
            and not ($fixture.runtime_reference | path exists)
            and not ($fixture.local_yzx | path exists)
            and not ($fixture.desktop_path | path exists)
            and not ($fixture.taplo_path | path exists)
            and not ($fixture.generated_yazi | path exists)
            and not ($fixture.generated_zellij | path exists)
            and ($fixture.config_path | path exists)
            and ($fixture.pack_config_path | path exists)
        ) {
            print "  ✅ yzx uninstall removes manual installer-owned artifacts while preserving the managed Yazelix config surfaces"
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

# Regression: uninstall must not claim Home Manager-owned surfaces as manual uninstall targets.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_uninstall_reports_home_manager_managed_install [] {
    print "🧪 Testing yzx uninstall reports Home Manager-managed install surfaces instead of removing them..."

    let fixture = (setup_home_manager_owned_install_fixture "yazelix_uninstall_home_manager_owned")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx uninstall")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "No manual installer-owned Yazelix artifacts were found.")
            and ($stdout | str contains "Home Manager-managed Yazelix surfaces are present.")
            and ($fixture.runtime_reference | path exists)
            and ($fixture.config_path | path exists)
            and ($fixture.pack_config_path | path exists)
        ) {
            print "  ✅ yzx uninstall leaves Home Manager-owned surfaces alone and points users at home-manager switch"
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

# Regression: uninstall must not claim a plain runtime/current directory as installer-owned.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_uninstall_ignores_noninstaller_runtime_current_directory [] {
    print "🧪 Testing yzx uninstall ignores a non-installer runtime/current directory..."

    let fixture = (setup_noninstaller_runtime_reference_fixture "yazelix_uninstall_noninstaller_runtime_current")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx uninstall")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "No manual installer-owned Yazelix artifacts were found.")
            and not ($stdout | str contains $fixture.runtime_reference)
            and ($fixture.runtime_reference | path exists)
        ) {
            print "  ✅ yzx uninstall now ignores runtime/current paths that do not match the installer-owned symlink shape"
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

# Defends: startup preflight auto-applies safe migrations before launch.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_entrypoint_preflight_auto_applies_safe_migrations [] {
    print "🧪 Testing entrypoint migration preflight auto-applies deterministic rewrites..."

    let fixture = (setup_config_migrate_fixture
        "yazelix_entrypoint_preflight_auto"
        '[zellij]
widget_tray = ["layout", "editor"]

[shell]
enable_atuin = true
')

    let result = (try {
        let output = (run_entrypoint_preflight_command $fixture "yzx launch" --allow-noninteractive)
        let stdout = ($output.stdout | str trim)
        let updated = (open $fixture.config_path)
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix auto-applied 2 safe config migration")
            and (($updated | get zellij.widget_tray) == ["editor"])
            and not (($updated.shell? | default {}) | columns | any {|column| $column == "enable_atuin" })
            and (($backups | length) == 1)
        ) {
            print "  ✅ Entry-point preflight auto-applies deterministic config rewrites with backup"
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

# Regression: legacy helix command strings are migrated during preflight.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_entrypoint_preflight_migrates_legacy_helix_command [] {
    print "🧪 Testing entrypoint migration preflight rewrites legacy helix.command into editor.command..."

    let fixture = (setup_config_migrate_fixture
        "yazelix_entrypoint_preflight_legacy_helix_command"
        '[helix]
command = "/tmp/custom-hx"
runtime_path = "/tmp/custom-runtime"
'
    )

    let result = (try {
        let output = (run_entrypoint_preflight_command $fixture "yzx launch" --allow-noninteractive)
        let stdout = ($output.stdout | str trim)
        let updated = (open $fixture.config_path)
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix auto-applied 1 safe config migration")
            and (($updated.editor.command? | default "") == "/tmp/custom-hx")
            and (($updated.helix.runtime_path? | default "") == "/tmp/custom-runtime")
            and not (($updated.helix? | default {}) | columns | any {|column| $column == "command" })
            and (($backups | length) == 1)
        ) {
            print "  ✅ Entry-point preflight preserves custom Helix runtime settings while migrating the legacy command field"
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

# Defends: startup blocks on remaining manual config work after safe rewrites.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_entrypoint_preflight_applies_auto_changes_then_blocks_on_manual_followup [] {
    print "🧪 Testing entrypoint migration preflight applies safe rewrites before blocking on manual follow-up..."

    let fixture = (setup_config_migrate_fixture
        "yazelix_entrypoint_preflight_mixed"
        '[zellij]
widget_tray = ["layout", "editor"]

[terminal]
config_mode = "auto"
')

    let result = (try {
        let output = (run_entrypoint_preflight_command $fixture "yzx launch" --allow-noninteractive)
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)
        let updated = (open $fixture.config_path)
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($output.exit_code != 0)
            and ($stdout | str contains "Yazelix auto-applied 1 safe config migration")
            and ($stderr | str contains "[MANUAL] review_terminal_config_mode_auto")
            and (($updated | get zellij.widget_tray) == ["editor"])
            and (($updated | get terminal.config_mode) == "auto")
            and (($backups | length) == 1)
        ) {
            print "  ✅ Entry-point preflight fixes the deterministic subset and then blocks on manual-only config follow-up"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr) updated=($updated | to json -r) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: legacy root config surfaces are detected and relocated only through the managed path.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_entrypoint_preflight_relocates_legacy_root_config_surfaces [] {
    print "🧪 Testing entrypoint migration preflight relocates legacy root config into user_configs..."

    let fixture = (setup_legacy_root_config_migrate_fixture
        "yazelix_entrypoint_preflight_root_relocate"
        '[shell]
default_shell = "bash"
'
    )

    let result = (try {
        let output = (run_entrypoint_preflight_command $fixture "yzx launch" --allow-noninteractive)
        let stdout = ($output.stdout | str trim)
        let relocated_main = ($fixture.user_config_dir | path join "yazelix.toml")
        let updated = (open $relocated_main)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "relocated the managed config into user_configs")
            and ($relocated_main | path exists)
            and not ($fixture.config_path | path exists)
            and (($updated.shell.default_shell? | default "") == "bash")
        ) {
            print "  ✅ Entry-point preflight relocates deterministic legacy root config ownership before continuing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) relocated_exists=(($relocated_main | path exists)) updated=($updated | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: pack-only legacy relocation should still be surfaced to the user.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_entrypoint_preflight_reports_pack_only_legacy_root_relocation [] {
    print "🧪 Testing entrypoint migration preflight reports pack-only legacy-root relocation..."

    let fixture = (setup_legacy_root_config_migrate_fixture
        "yazelix_entrypoint_preflight_pack_only_relocate"
        "[core]\nwelcome_style = \"random\"\n"
    )

    let result = (try {
        rm $fixture.config_path
        let legacy_pack = ($fixture.config_dir | path join "yazelix_packs.toml")
        'enabled = ["git"]
' | save --force --raw $legacy_pack

        let output = (run_entrypoint_preflight_command $fixture "yzx launch" --allow-noninteractive)
        let stdout = ($output.stdout | str trim)
        let relocated_pack = ($fixture.user_config_dir | path join "yazelix_packs.toml")

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "relocated the managed pack config into user_configs")
            and ($relocated_pack | path exists)
            and not ($legacy_pack | path exists)
        ) {
            print "  ✅ Entry-point preflight reports pack-only legacy relocation instead of moving it silently"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) relocated_pack_exists=(($relocated_pack | path exists)) legacy_pack_exists=(($legacy_pack | path exists))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: entrypoint preflight relocates legacy-root config and applies the deterministic subset before blocking.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_entrypoint_preflight_relocates_legacy_root_and_applies_safe_subset_before_manual_block [] {
    print "🧪 Testing entrypoint migration preflight relocates legacy-root config and applies safe rewrites before blocking..."

    let fixture = (setup_legacy_root_config_migrate_fixture
        "yazelix_entrypoint_preflight_root_relocate_mixed"
        '[zellij]
widget_tray = ["layout", "editor"]

[terminal]
config_mode = "auto"
'
    )

    let result = (try {
        let output = (run_entrypoint_preflight_command $fixture "yzx launch" --allow-noninteractive)
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)
        let relocated_main = ($fixture.user_config_dir | path join "yazelix.toml")
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')
        let updated = (open $relocated_main)

        if (
            ($output.exit_code != 0)
            and ($stdout | str contains "relocated the managed config into user_configs")
            and ($stdout | str contains "Yazelix auto-applied 1 safe config migration")
            and ($stderr | str contains "[MANUAL] review_terminal_config_mode_auto")
            and ($relocated_main | path exists)
            and not ($fixture.config_path | path exists)
            and (($updated | get zellij.widget_tray) == ["editor"])
            and (($updated | get terminal.config_mode) == "auto")
            and (($backups | length) == 0)
        ) {
            print "  ✅ Entry-point preflight now relocates legacy-root config and applies the deterministic subset inside the same managed transition"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr) relocated_exists=(($relocated_main | path exists)) updated=($updated | to json -r) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: interrupted relocation recovery must run before duplicate-surface validation.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_entrypoint_preflight_recovers_stale_relocation_before_duplicate_surface_error [] {
    print "🧪 Testing entrypoint migration preflight recovers stale relocation state before duplicate-surface validation..."

    let fixture = (setup_legacy_root_config_migrate_fixture
        "yazelix_entrypoint_preflight_recover_stale_relocation"
        '[shell]
default_shell = "bash"
'
    )

    let result = (try {
        let relocated_main = ($fixture.user_config_dir | path join "yazelix.toml")
        let transaction_root = (get_managed_config_transaction_dir $relocated_main)
        let transaction_id = "txn_stale_entrypoint_relocation"
        let work_dir = ($transaction_root | path join $transaction_id)
        let manifest_path = ($work_dir | path join "manifest.json")
        let staged_main = ($work_dir | path join "yazelix.toml")

        mkdir $work_dir
        '[core]
welcome_style = "random"
' | save --force --raw $relocated_main
        '# stale staged main
' | save --force --raw $staged_main
        {
            schema_version: 1
            transaction_id: $transaction_id
            caller: "entrypoint_preflight"
            phase: "validated"
            targets: [
                {
                    role: "main"
                    target_path: $relocated_main
                    staged_path: $staged_main
                    backup_path: null
                    existed_before: false
                }
            ]
            cleanup_sources: []
        } | to json | save --force --raw $manifest_path

        let output = (run_entrypoint_preflight_command $fixture "yzx launch" --allow-noninteractive)
        let stdout = ($output.stdout | str trim)
        let updated = (open $relocated_main)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Recovered 1 interrupted managed-config transaction")
            and ($stdout | str contains "relocated the managed config into user_configs")
            and ($relocated_main | path exists)
            and not ($fixture.config_path | path exists)
            and (($updated.shell.default_shell? | default "") == "bash")
            and not ($manifest_path | path exists)
        ) {
            print "  ✅ Entry-point preflight recovers stale relocation state before validating duplicate config surfaces"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) relocated_exists=(($relocated_main | path exists)) updated=($updated | to json -r) manifest_exists=(($manifest_path | path exists))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Invariant: pack sidecar config is merged into the full config view.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
def test_yzx_config_full_merges_pack_sidecar [] {
    print "🧪 Testing yzx config --full merges the dedicated pack sidecar..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_config_full_sidecar_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($temp_config_dir | path join "user_configs")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir
    mkdir $user_config_dir

    let result = (try {
        '[core]
debug_mode = false
' | save --force --raw ($user_config_dir | path join "yazelix.toml")

        'enabled = ["git"]

[declarations]
git = ["gh", "prek"]
' | save --force --raw ($user_config_dir | path join "yazelix_packs.toml")

        let rendered = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx config --full
        }

        if (
            (($rendered.packs.enabled? | default []) == ["git"])
            and ((($rendered.packs.declarations? | default {}) | get git) == ["gh", "prek"])
        ) {
            print "  ✅ yzx config --full renders the merged pack sidecar view"
            true
        } else {
            print $"  ❌ Unexpected result: ($rendered | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Defends: yzx edit fuzzy-style target queries resolve to canonical managed config surfaces and reject ambiguous noninteractive use.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
def test_yzx_edit_targets_print_paths [] {
    print "🧪 Testing yzx edit resolves the supported managed config targets and rejects noninteractive ambiguity..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_config_open_targets_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        let yzx_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let main_stdout = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx edit config --print
        }
        let packs_stdout = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx edit packs --print
        }
        let helix_stdout = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx edit hel --print
        }
        let zellij_stdout = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx edit zell --print
        }
        let yazi_stdout = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx edit yazi --print
        }
        let missing_subcommand_output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx edit --print" | complete
        }
        let invalid_output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx edit weird --print" | complete
        }

        let expected_main = ($temp_config_dir | path join "user_configs" "yazelix.toml")
        let expected_packs = ($temp_config_dir | path join "user_configs" "yazelix_packs.toml")
        let expected_helix = ($temp_config_dir | path join "user_configs" "helix" "config.toml")
        let expected_zellij = ($temp_config_dir | path join "user_configs" "zellij" "config.kdl")
        let expected_yazi = ($temp_config_dir | path join "user_configs" "yazi" "yazi.toml")
        let missing_subcommand_stderr = ($missing_subcommand_output.stderr | str trim)
        let invalid_stderr = ($invalid_output.stderr | str trim)

        if (
            ($missing_subcommand_output.exit_code != 0)
            and ($invalid_output.exit_code != 0)
            and ($main_stdout == $expected_main)
            and ($packs_stdout == $expected_packs)
            and ($helix_stdout == $expected_helix)
            and ($zellij_stdout == $expected_zellij)
            and ($yazi_stdout == $expected_yazi)
            and ($missing_subcommand_stderr | str contains "requires a target query")
            and ($invalid_stderr | str contains "No managed Yazelix config surface matched")
        ) {
            print "  ✅ yzx edit resolves canonical managed surfaces through permissive target queries and rejects unsupported noninteractive cases"
            true
        } else {
            print $"  ❌ Unexpected result: main=($main_stdout) packs=($packs_stdout) helix=($helix_stdout) zellij=($zellij_stdout) yazi=($yazi_stdout) missing_exit=($missing_subcommand_output.exit_code) missing_stderr=($missing_subcommand_stderr) invalid_exit=($invalid_output.exit_code) invalid_stderr=($invalid_stderr)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Defends: invalid config is surfaced as a config problem, not a generic wrapper failure.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_invalid_config_is_classified_as_config_problem [] {
    print "🧪 Testing invalid config values are classified as config problems..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_invalid_config_XXXXXX | str trim)
    let temp_yazelix_dir = ($tmp_home | path join ".config" "yazelix")
    let xdg_config_home = ($tmp_home | path join ".config")
    mkdir $temp_yazelix_dir

    let result = (try {
        ^ln -s ($repo_root | path join "nushell") ($temp_yazelix_dir | path join "nushell")
        cp ($repo_root | path join "yazelix_default.toml") ($temp_yazelix_dir | path join "yazelix_default.toml")
        let user_config_dir = ($temp_yazelix_dir | path join "user_configs")
        mkdir $user_config_dir

        let invalid_config = (
            open ($repo_root | path join "yazelix_default.toml")
            | upsert core.refresh_output "loud"
        )
        $invalid_config | to toml | save ($user_config_dir | path join "yazelix.toml")

        let parser_script = ($temp_yazelix_dir | path join "nushell" "scripts" "utils" "config_parser.nu")
        let output = with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $temp_yazelix_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"source \"($parser_script)\"; try { parse_yazelix_config | ignore } catch {|err| print $err.msg }" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Unsupported config value at core.refresh_output")
            and ($stdout | str contains "Invalid value for core.refresh_output: loud")
            and ($stdout | str contains "Failure class: config problem.")
            and ($stdout | str contains "yzx config reset")
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

# Regression: startup reports known migration needs before generic wrapper noise.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_startup_reports_known_config_migration_before_generic_wrappers [] {
    print "🧪 Testing startup reports known config migrations before generic wrappers..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_startup_migration_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($temp_config_dir | path join "user_configs")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir
    mkdir $user_config_dir

    let result = (try {
        '[zellij]
widget_tray = ["layout", "editor"]
' | save --force --raw ($user_config_dir | path join "yazelix.toml")

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

export def run_core_canonical_tests [] {
    [
        (test_yzx_config_migrate_preview_reports_known_migrations)
        (test_yzx_config_migrate_apply_rewrites_config_with_backup)
        (test_yzx_config_migrate_apply_splits_legacy_packs_into_sidecar)
        (test_yzx_config_migrate_apply_noops_on_current_config)
        (test_entrypoint_preflight_auto_applies_safe_migrations)
        (test_entrypoint_preflight_migrates_legacy_helix_command)
        (test_entrypoint_preflight_applies_auto_changes_then_blocks_on_manual_followup)
        (test_entrypoint_preflight_relocates_legacy_root_config_surfaces)
        (test_entrypoint_preflight_reports_pack_only_legacy_root_relocation)
        (test_entrypoint_preflight_relocates_legacy_root_and_applies_safe_subset_before_manual_block)
        (test_entrypoint_preflight_recovers_stale_relocation_before_duplicate_surface_error)
        (test_yzx_home_manager_prepare_preview_reports_manual_takeover_artifacts)
        (test_yzx_home_manager_prepare_apply_archives_manual_takeover_artifacts)
        (test_yzx_uninstall_apply_removes_manual_artifacts_but_preserves_config)
        (test_yzx_uninstall_reports_home_manager_managed_install)
        (test_yzx_uninstall_ignores_noninstaller_runtime_current_directory)
        (test_yzx_config_full_merges_pack_sidecar)
        (test_yzx_edit_targets_print_paths)
        (test_invalid_config_is_classified_as_config_problem)
        (test_startup_reports_known_config_migration_before_generic_wrappers)
    ]
}

export def run_core_tests [] {
    run_core_canonical_tests
}
