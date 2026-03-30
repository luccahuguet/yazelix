#!/usr/bin/env nu

use ../core/yazelix.nu *
use ./test_yzx_helpers.nu [get_repo_config_dir repo_path setup_managed_config_fixture]
use ../utils/shell_config_generation.nu [get_yazelix_section_content]
use ../utils/config_manager.nu [check_config_versions]

def setup_config_migrate_fixture [label: string, raw_toml: string] {
    setup_managed_config_fixture $label $raw_toml
}

def setup_legacy_root_config_migrate_fixture [label: string, raw_toml: string] {
    setup_managed_config_fixture $label $raw_toml --legacy-root
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

        $current = ($current | get $segment)
    }

    true
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

def test_yzx_config_migrate_apply_relocates_legacy_root_config_into_user_configs [] {
    print "🧪 Testing yzx config migrate apply relocates legacy root-level config into user_configs..."

    let fixture = (setup_legacy_root_config_migrate_fixture
        "yazelix_migrate_root_relocate"
        '[shell]
default_shell = "bash"
')

    let result = (try {
        let output = (run_config_migrate_command $fixture ["--apply", "--yes"])
        let stdout = ($output.stdout | str trim)
        let relocated_main = ($fixture.user_config_dir | path join "yazelix.toml")
        let updated = (open $relocated_main)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "relocate_root_config_surfaces_into_user_configs")
            and ($stdout | str contains "Relocated managed config into")
            and ($stdout | str contains "No additional TOML rewrites were needed")
            and (($updated.shell.default_shell? | default "") == "bash")
            and ($relocated_main | path exists)
            and not (($fixture.config_dir | path join "yazelix.toml") | path exists)
        ) {
            print "  ✅ yzx config migrate --apply now owns the legacy root-to-user_configs path relocation"
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

def test_yzx_config_view [] {
    print "🧪 Testing yzx config..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_config_view_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($temp_config_dir | path join "user_configs")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir
    mkdir $user_config_dir

    let result = (try {
        let yzx_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        '[core]
debug_mode = false
' | save --force --raw ($user_config_dir | path join "yazelix.toml")
        'enabled = ["git"]

[declarations]
git = ["gh"]
' | save --force --raw ($user_config_dir | path join "yazelix_packs.toml")

        let command_output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx config | columns | str join ','" | complete
        }
        let output = ($command_output.stdout | str trim)

        if (
            ($command_output.exit_code == 0)
            and ($output | str contains "core")
            and not ($output | str contains "packs")
        ) {
            print "  ✅ yzx config hides packs by default"
            true
        } else {
            print $"  ❌ Unexpected output: exit=($command_output.exit_code) stdout=($output) stderr=($command_output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

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
        let yzx_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")

        '[core]
debug_mode = false
' | save --force --raw ($user_config_dir | path join "yazelix.toml")

        'enabled = ["git"]

[declarations]
git = ["gh", "prek"]
' | save --force --raw ($user_config_dir | path join "yazelix_packs.toml")

        let output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx config --full | to json -r" | complete
        }
        let stdout = ($output.stdout | str trim)
        let rendered = ($stdout | from json)

        if (
            ($output.exit_code == 0)
            and (($rendered.packs.enabled? | default []) == ["git"])
            and ((($rendered.packs.declarations? | default {}) | get git) == ["gh", "prek"])
        ) {
            print "  ✅ yzx config --full renders the merged pack sidecar view"
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

def test_yzx_config_sections [] {
    print "🧪 Testing yzx open downstream config views..."

    try {
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let yazi_merger = (repo_path "nushell" "scripts" "setup" "yazi_config_merger.nu")
        let zellij_merger = (repo_path "nushell" "scripts" "setup" "zellij_config_merger.nu")
        let root = (get_repo_config_dir)
        ^nu -c $"use \"($yazi_merger)\" *; generate_merged_yazi_config \"($root)\" --quiet | ignore" | complete | ignore
        let hx_output = (^nu -c $"use \"($yzx_script)\" *; yzx open hx | columns | str join ','" | complete).stdout | str trim
        let yazi_output = (^nu -c $"use \"($yzx_script)\" *; yzx open yazi | columns | str join ','" | complete).stdout | str trim
        if (which zellij | is-empty) {
            if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") {
                print "  ℹ️  Skipping zellij config section check because zellij is not available"
                print "  ✅ yzx open section commands return focused sections"
                return true
            }
        }

        ^nu -c $"use \"($zellij_merger)\" *; generate_merged_zellij_config \"($root)\" | ignore" | complete | ignore
        let zellij_output = (^nu -c $"use \"($yzx_script)\" *; yzx open zellij" | complete).stdout | str trim

        if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") and ($zellij_output | str contains "default_layout") {
            print "  ✅ yzx open section commands return focused sections"
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

def test_yzx_edit_targets_print_paths [] {
    print "🧪 Testing yzx edit config and yzx edit packs print the managed config paths..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_config_open_targets_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        let yzx_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let main_output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx edit config --print" | complete
        }
        let packs_output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx edit packs --print" | complete
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
        let main_stdout = ($main_output.stdout | str trim)
        let packs_stdout = ($packs_output.stdout | str trim)
        let missing_subcommand_stderr = ($missing_subcommand_output.stderr | str trim)
        let invalid_stderr = ($invalid_output.stderr | str trim)

        if (
            ($main_output.exit_code == 0)
            and ($packs_output.exit_code == 0)
            and ($missing_subcommand_output.exit_code != 0)
            and ($invalid_output.exit_code != 0)
            and ($main_stdout == $expected_main)
            and ($packs_stdout == $expected_packs)
            and ($missing_subcommand_stderr | str contains "edit")
            and ($invalid_stderr | str contains "edit")
        ) {
            print "  ✅ yzx edit config and yzx edit packs resolve the managed config paths and reject unknown leaf commands"
            true
        } else {
            print $"  ❌ Unexpected result: main_exit=($main_output.exit_code) main=($main_stdout) packs_exit=($packs_output.exit_code) packs=($packs_stdout) missing_exit=($missing_subcommand_output.exit_code) missing_stderr=($missing_subcommand_stderr) invalid_exit=($invalid_output.exit_code) invalid_stderr=($invalid_stderr)"
            false
        }
    } catch {|err|
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
        'enabled = ["git"]
' | save --force --raw ($temp_config_dir | path join "yazelix_packs.toml")

        let yzx_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx config reset --yes" | complete
        }
        let stdout = ($output.stdout | str trim)
        let user_config_dir = ($temp_config_dir | path join "user_configs")
        let new_config = (open --raw ($user_config_dir | path join "yazelix.toml"))
        let default_config = (open --raw ($repo_root | path join "yazelix_default.toml"))
        let new_pack_config = (open --raw ($user_config_dir | path join "yazelix_packs.toml"))
        let default_pack_config = (open --raw ($repo_root | path join "yazelix_packs_default.toml"))

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template")
            and ($stdout | str contains "Replaced yazelix_packs.toml with a fresh template")
            and ($new_config == $default_config)
            and ($new_pack_config == $default_pack_config)
        ) {
            print "  ✅ yzx config reset reads both split templates from the runtime root and writes them to the config root"
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

export def run_core_canonical_tests [] {
    [
        (test_yzx_config_migrate_preview_reports_known_migrations)
        (test_yzx_config_migrate_apply_rewrites_config_with_backup)
        (test_yzx_config_migrate_apply_splits_legacy_packs_into_sidecar)
        (test_yzx_config_migrate_apply_relocates_legacy_root_config_into_user_configs)
        (test_yzx_config_migrate_apply_noops_on_current_config)
        (test_entrypoint_preflight_auto_applies_safe_migrations)
        (test_entrypoint_preflight_applies_auto_changes_then_blocks_on_manual_followup)
        (test_entrypoint_preflight_relocates_legacy_root_config_surfaces)
        (test_yzx_config_view)
        (test_yzx_config_full_merges_pack_sidecar)
        (test_yzx_edit_targets_print_paths)
        (test_invalid_config_is_classified_as_config_problem)
        (test_startup_reports_known_config_migration_before_generic_wrappers)
    ]
}

export def run_core_tests [] {
    run_core_canonical_tests
}
