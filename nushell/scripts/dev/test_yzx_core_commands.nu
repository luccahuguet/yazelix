#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ../core/yazelix.nu *
use ./yzx_test_helpers.nu [get_repo_config_dir repo_path setup_managed_config_fixture]

const DESKTOP_ICON_SIZES = ["48x48", "64x64", "128x128", "256x256"]

def run_yzx_command_for_fixture [fixture: record, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    }
    let merged_env = if ($extra_env | is-empty) {
        $base_env
    } else {
        $base_env | merge $extra_env
    }

    with-env $merged_env {
        ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
    }
}

def run_yzx_command_for_fixture_in_dir [fixture: record, working_dir: string, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    }
    let merged_env = if ($extra_env | is-empty) {
        $base_env
    } else {
        $base_env | merge $extra_env
    }

    with-env $merged_env {
        do {
            cd $working_dir
            ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
        }
    }
}

def manual_desktop_icon_records [tmp_home: string, source_root: string] {
    $DESKTOP_ICON_SIZES
    | each {|size|
        {
            size: $size
            source: ($source_root | path join "assets" "icons" $size "yazelix.png")
            path: ($tmp_home | path join ".local" "share" "icons" "hicolor" $size "apps" "yazelix.png")
        }
    }
}

def files_match [left: string, right: string] {
    let result = (^cmp -s $left $right | complete)
    $result.exit_code == 0
}

def write_test_executable [path: string, lines: list<string>] {
    ($lines | str join "\n") | save --force --raw $path
    ^chmod +x $path
}

def write_test_legacy_yzx_wrapper [path: string] {
    write_test_executable $path [
        "#!/bin/sh"
        "# Stable Yazelix CLI entrypoint for external tools and editors."
        'exec "$(dirname "$0")/../shells/posix/yzx_cli.sh" "$@"'
    ]
}

def build_test_legacy_shell_block [stale_runtime_root: string] {
    [
        "# YAZELIX START v4 - Yazelix managed configuration (do not modify this comment)"
        "# delete this whole section to re-generate the config, if needed"
        'if [ -n "$IN_YAZELIX_SHELL" ]; then'
        $"  source \"($stale_runtime_root)/shells/bash/yazelix_bash_config.sh\""
        "fi"
        "# yzx command - always available for launching/managing yazelix"
        "yzx() {"
        $"    \"($stale_runtime_root)/bin/yzx\" \"$@\""
        "}"
        "# YAZELIX END v4 - Yazelix managed configuration (do not modify this comment)"
        ""
    ] | str join "\n"
}

def setup_manual_install_takeover_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let launcher_path = ($fixture.repo_root | path join "shells" "posix" "yzx_cli.sh")
    let desktop_path = ($fixture.tmp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")
    let desktop_icons = (manual_desktop_icon_records $fixture.tmp_home $fixture.repo_root)
    let manual_wrapper = ($fixture.tmp_home | path join ".local" "bin" "yzx")
    let shell_block_path = ($fixture.tmp_home | path join ".bashrc")
    let shell_block_original_contents = "# existing bashrc\nexport TEST_BASHRC=1\n"
    let stale_runtime_root = "/nix/store/old-yazelix"
    let shell_block_contents = (build_test_legacy_shell_block $stale_runtime_root)

    mkdir ($desktop_path | path dirname)
    mkdir ($manual_wrapper | path dirname)

    for icon in $desktop_icons {
        mkdir ($icon.path | path dirname)
        ^cp $icon.source $icon.path
    }

    write_test_legacy_yzx_wrapper $manual_wrapper
    $"($shell_block_original_contents)\n($shell_block_contents)" | save --force --raw $shell_block_path

    [
        "[Desktop Entry]"
        "Type=Application"
        "Name=Yazelix"
        "Terminal=true"
        "X-Yazelix-Managed=true"
        $"Exec=\"($launcher_path)\" desktop launch"
    ] | str join "\n" | save --force --raw $desktop_path

    $fixture | merge {
        launcher_path: $launcher_path
        desktop_path: $desktop_path
        desktop_icons: $desktop_icons
        manual_wrapper: $manual_wrapper
        shell_block_path: $shell_block_path
        shell_block_original_contents: $shell_block_original_contents
        shell_block_contents: $shell_block_contents
    }
}

def setup_manual_desktop_install_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let desktop_path = ($fixture.tmp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")
    let desktop_icons = (manual_desktop_icon_records $fixture.tmp_home $fixture.repo_root)
    let launcher_path = ($fixture.repo_root | path join "shells" "posix" "yzx_cli.sh")

    $fixture | merge {
        launcher_path: $launcher_path
        desktop_path: $desktop_path
        desktop_icons: $desktop_icons
    }
}

def setup_installed_wrapper_desktop_fixture [label: string] {
    let fixture = (setup_manual_desktop_install_fixture $label)
    let installed_wrapper = ($fixture.tmp_home | path join ".local" "bin" "yzx")
    mkdir ($installed_wrapper | path dirname)
    write_test_executable $installed_wrapper [
        "#!/bin/sh"
        "exit 0"
    ]

    $fixture | merge {
        installed_wrapper: $installed_wrapper
    }
}

def setup_home_manager_desktop_fixture [label: string] {
    let fixture = (setup_manual_install_takeover_fixture $label)
    let hm_store_config = ($fixture.tmp_home | path join "hm-store" "abc-home-manager-files" "yazelix.toml")
    let hm_profile_yzx = ($fixture.tmp_home | path join ".nix-profile" "bin" "yzx")

    mkdir ($hm_store_config | path dirname)
    mkdir ($hm_profile_yzx | path dirname)
    '[core]
welcome_style = "random"
' | save --force --raw $hm_store_config
    rm $fixture.config_path
    ^ln -s $hm_store_config $fixture.config_path

    write_test_executable $hm_profile_yzx [
        "#!/bin/sh"
        "exit 0"
    ]

    $fixture | merge {
        hm_profile_yzx: $hm_profile_yzx
    }
}

def setup_home_manager_broken_profile_wrapper_fixture [label: string] {
    let fixture = (setup_manual_install_takeover_fixture $label)
    let hm_store_config = ($fixture.tmp_home | path join "hm-store" "abc-home-manager-files" "yazelix.toml")
    let hm_profile_yzx = ($fixture.tmp_home | path join ".nix-profile" "bin" "yzx")
    let missing_wrapper_target = ($fixture.tmp_home | path join "missing_store" "bin" "yzx")

    mkdir ($hm_store_config | path dirname)
    mkdir ($hm_profile_yzx | path dirname)
    '[core]
welcome_style = "random"
' | save --force --raw $hm_store_config
    rm $fixture.config_path
    ^ln -s $hm_store_config $fixture.config_path
    ^ln -s $missing_wrapper_target $hm_profile_yzx

    $fixture | merge {
        hm_profile_yzx: $hm_profile_yzx
        missing_wrapper_target: $missing_wrapper_target
    }
}

def setup_update_wrapper_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let bin_dir = ($fixture.tmp_home | path join "bin")
    let command_log = ($fixture.tmp_home | path join "update_wrapper_commands.log")
    let flake_dir = ($fixture.tmp_home | path join "home_manager_flake")

    mkdir $bin_dir
    mkdir $flake_dir
    "" | save --force --raw $command_log
    "{ description = \"test flake\"; outputs = { self }: {}; }\n" | save --force --raw ($flake_dir | path join "flake.nix")

    write_test_executable ($bin_dir | path join "nix") [
        "#!/bin/sh"
        "printf 'nix:%s\\n' \"$*\" >> \"$YZX_TEST_LOG\""
        "if [ \"$1\" = \"profile\" ] && [ \"$2\" = \"list\" ] && [ \"$3\" = \"--json\" ]; then"
        "  printf '%s\\n' \"$YZX_TEST_PROFILE_LIST_JSON\""
        "fi"
    ]
    write_test_executable ($bin_dir | path join "home-manager") [
        "#!/bin/sh"
        "printf 'home-manager:%s\\n' \"$*\" >> \"$YZX_TEST_LOG\""
    ]

    $fixture | merge {
        bin_dir: $bin_dir
        command_log: $command_log
        flake_dir: $flake_dir
    }
}

def setup_profile_wrapper_redirect_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let old_runtime = ($fixture.tmp_home | path join "old_runtime")
    let profile_yzx = ($fixture.tmp_home | path join ".nix-profile" "bin" "yzx")
    let redirect_log = ($fixture.tmp_home | path join "redirected_args.log")

    mkdir $old_runtime
    mkdir ($profile_yzx | path dirname)
    "" | save --force --raw $redirect_log

    ^ln -s (repo_path "shells") ($old_runtime | path join "shells")

    write_test_executable $profile_yzx [
        "#!/bin/sh"
        'printf "%s\n" "$*" > "$YZX_REDIRECT_LOG"'
    ]

    {
        fixture: $fixture
        old_runtime: $old_runtime
        profile_yzx: $profile_yzx
        redirect_log: $redirect_log
    }
}

def setup_run_passthrough_fixture [label: string] {
    let fixture = (setup_managed_config_fixture
        $label
        '[core]
welcome_style = "random"
'
    )

    let stub_root = ($fixture.tmp_home | path join "run_passthrough_stub")
    let yzx_dir = ($stub_root | path join "yzx")
    let utils_dir = ($stub_root | path join "utils")
    let command_log = ($fixture.tmp_home | path join "run_passthrough.json")
    let stub_run_script = ($yzx_dir | path join "run.nu")

    mkdir $yzx_dir
    mkdir $utils_dir
    cp (repo_path "nushell" "scripts" "yzx" "run.nu") $stub_run_script

    [
        "#!/usr/bin/env nu"
        "export def prepare_environment [--verbose] {"
        "    {"
        "        config: {}"
        "        config_state: {}"
        "        needs_refresh: false"
        "    }"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "environment_bootstrap.nu")

    [
        "#!/usr/bin/env nu"
        "export def run_runtime_argv ["
        "    argv: list<string>"
        "    --cwd: string = \"\""
        "    --config: record"
        "] {"
        "    let command = ($argv | first)"
        "    let args = ($argv | skip 1)"
        "    {"
        "        command: $command"
        "        args: $args"
        "        cwd: $cwd"
        "        config_present: (($config | describe) | str starts-with \"record\")"
        "    } | to json -r | save --force --raw $env.YZX_RUN_LOG"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "runtime_env.nu")

    $fixture | merge {
        command_log: $command_log
        stub_run_script: $stub_run_script
    }
}

def run_stubbed_yzx_run [fixture: record, command: string] {
    with-env {
        YZX_RUN_LOG: $fixture.command_log
    } {
        ^nu -c $"use \"($fixture.stub_run_script)\" *; ($command)" | complete
    }
}

def setup_palette_catalog_runtime_fixture [label: string] {
    let tmp_root = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_root = ($tmp_root | path join "runtime")
    let core_dir = ($runtime_root | path join "nushell" "scripts" "core")
    let core_script = ($core_dir | path join "yazelix.nu")

    mkdir $core_dir
    "" | save --force --raw ($runtime_root | path join "yazelix_default.toml")

    [
        "#!/usr/bin/env nu"
        "export def yzx [] {}"
        "export def \"yzx launch\" [] {}"
        "export def \"yzx status\" [] {}"
        "export def \"yzx screen\" [] {}"
        "export def \"yzx why\" [] {}"
        "export def \"yzx env\" [] {}"
        "export def \"yzx run\" [...argv: string] { $argv | ignore }"
        "export def \"yzx cwd\" [target?: string] { $target | ignore }"
        "export def \"yzx dev sync_issues\" [] {}"
        "export def \"yzx test_dynamic\" [] {}"
    ] | str join "\n" | save --force --raw $core_script

    {
        tmp_root: $tmp_root
        runtime_root: $runtime_root
        core_script: $core_script
    }
}

# Regression: `yzx desktop install` must install the icon assets needed by the manual desktop entry.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_install_writes_entry_and_icon_assets [] {
    print "🧪 Testing yzx desktop install writes the desktop entry and icon assets..."

    let fixture = (setup_manual_desktop_install_fixture "yazelix_desktop_install_icons")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx desktop install")
        let stdout = ($output.stdout | str trim)
        let desktop_entry = (open --raw $fixture.desktop_path)
        let icons_ok = (
            $fixture.desktop_icons
            | all {|icon| ($icon.path | path exists) and (files_match $icon.source $icon.path) }
        )

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Installed Yazelix desktop entry")
            and ($stdout | str contains $fixture.desktop_path)
            and ($desktop_entry | str contains 'Icon=yazelix')
            and ($desktop_entry | str contains 'X-Yazelix-Managed=true')
            and ($desktop_entry | str contains 'Terminal=true')
            and ($desktop_entry | str contains $fixture.launcher_path)
            and $icons_ok
        ) {
            print "  ✅ yzx desktop install now writes the manual desktop entry and its icon assets together"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) desktop_exists=(($fixture.desktop_path | path exists)) icons_ok=($icons_ok)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: desktop install must prefer the stable installed wrapper when it exists, not a runtime-pinned store path.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_install_prefers_installed_wrapper [] {
    print "🧪 Testing yzx desktop install prefers the stable installed wrapper..."

    let fixture = (setup_installed_wrapper_desktop_fixture "yazelix_desktop_install_prefers_installed_wrapper")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx desktop install")
        let desktop_entry = (open --raw $fixture.desktop_path)

        if (
            ($output.exit_code == 0)
            and ($desktop_entry | str contains $fixture.installed_wrapper)
            and not ($desktop_entry | str contains $fixture.launcher_path)
        ) {
            print "  ✅ yzx desktop install now anchors the desktop entry to the stable installed wrapper"
            true
        } else {
            print $"  ❌ Unexpected desktop entry contents: exit=($output.exit_code) entry=($desktop_entry)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: Home Manager mode must resolve through the profile-owned yzx wrapper even if an old manual wrapper still exists.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_stable_yzx_wrapper_prefers_home_manager_profile_owner [] {
    print "🧪 Testing stable yzx wrapper resolution prefers the Home Manager profile owner..."

    let fixture = (setup_home_manager_desktop_fixture "yazelix_stable_wrapper_home_manager")

    let result = (try {
        let output = (with-env {
            HOME: $fixture.tmp_home
            XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
        } {
            ^nu -c 'use nushell/scripts/utils/launcher_resolution.nu resolve_stable_yzx_wrapper_path; resolve_stable_yzx_wrapper_path' | complete
        })
        let resolved = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($resolved == $fixture.hm_profile_yzx)
            and ($resolved != $fixture.manual_wrapper)
        ) {
            print "  ✅ stable wrapper resolution now follows the Home Manager profile owner"
            true
        } else {
            print $"  ❌ Unexpected stable wrapper: exit=($output.exit_code) resolved=($resolved) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: Home Manager wrapper resolution must keep the profile-owned symlink path even when the current target is broken.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_stable_yzx_wrapper_keeps_home_manager_broken_profile_symlink [] {
    print "🧪 Testing stable yzx wrapper resolution keeps a Home Manager profile symlink even when its target is broken..."

    let fixture = (setup_home_manager_broken_profile_wrapper_fixture "yazelix_stable_wrapper_home_manager_broken_symlink")

    let result = (try {
        let output = (with-env {
            HOME: $fixture.tmp_home
            XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
        } {
            ^nu -c 'use nushell/scripts/utils/launcher_resolution.nu resolve_stable_yzx_wrapper_path; resolve_stable_yzx_wrapper_path' | complete
        })
        let resolved = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($resolved == $fixture.hm_profile_yzx)
            and ($resolved != $fixture.manual_wrapper)
            and not ($fixture.hm_profile_yzx | path exists)
        ) {
            print "  ✅ stable wrapper resolution now preserves the Home Manager-owned profile symlink path even when the current target is broken"
            true
        } else {
            print $"  ❌ Unexpected stable wrapper for broken profile symlink: exit=($output.exit_code) resolved=($resolved) profile_exists=(($fixture.hm_profile_yzx | path exists)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: Home Manager owns desktop integration in Home Manager mode; manual install must refuse without blocking cleanup.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_install_refuses_home_manager_owned_install [] {
    print "🧪 Testing yzx desktop install refuses Home Manager-owned installs..."

    let fixture = (setup_home_manager_desktop_fixture "yazelix_desktop_install_home_manager_refusal")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx desktop install")
        let stderr = ($output.stderr | str trim)
        let desktop_entry_still_exists = ($fixture.desktop_path | path exists)

        if (
            ($output.exit_code != 0)
            and ($stderr | str contains "Home Manager owns Yazelix desktop integration")
            and ($stderr | str contains "yzx desktop uninstall")
            and $desktop_entry_still_exists
        ) {
            print "  ✅ yzx desktop install now refuses Home Manager-owned installs without deleting stale cleanup targets"
            true
        } else {
            print $"  ❌ Unexpected Home Manager desktop install result: exit=($output.exit_code) stderr=($stderr) desktop_exists=($desktop_entry_still_exists)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: Home Manager mode must still allow yzx desktop uninstall to remove a stale user-local desktop entry.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_uninstall_preserves_home_manager_cleanup_path [] {
    print "🧪 Testing yzx desktop uninstall still cleans stale manual entries in Home Manager mode..."

    let fixture = (setup_home_manager_desktop_fixture "yazelix_desktop_uninstall_home_manager_cleanup")

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx desktop uninstall")
        let stdout = ($output.stdout | str trim)
        let icons_removed = ($fixture.desktop_icons | all {|icon| not ($icon.path | path exists) })

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Removed Yazelix desktop entry")
            and not ($fixture.desktop_path | path exists)
            and $icons_removed
        ) {
            print "  ✅ yzx desktop uninstall remains the cleanup path for stale Home Manager-shadowing manual entries"
            true
        } else {
            print $"  ❌ Unexpected Home Manager desktop uninstall result: exit=($output.exit_code) stdout=($stdout) desktop_exists=(($fixture.desktop_path | path exists)) icons_removed=($icons_removed)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: desktop uninstall removes the explicit user-local integration artifacts created by yzx desktop install.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_uninstall_removes_manual_entry_and_icons [] {
    print "🧪 Testing yzx desktop uninstall removes the manual desktop entry and icon assets..."

    let fixture = (setup_manual_desktop_install_fixture "yazelix_desktop_uninstall")

    let result = (try {
        let install_output = (run_yzx_command_for_fixture $fixture "yzx desktop install")
        let uninstall_output = (run_yzx_command_for_fixture $fixture "yzx desktop uninstall")
        let stdout = ($uninstall_output.stdout | str trim)
        let icons_removed = ($fixture.desktop_icons | all {|icon| not ($icon.path | path exists) })

        if (
            ($install_output.exit_code == 0)
            and ($uninstall_output.exit_code == 0)
            and ($stdout | str contains "Removed Yazelix desktop entry")
            and ($stdout | str contains $fixture.desktop_path)
            and not ($fixture.desktop_path | path exists)
            and $icons_removed
        ) {
            print "  ✅ yzx desktop uninstall now removes the explicit manual desktop integration artifacts"
            true
        } else {
            print $"  ❌ Unexpected result: install_exit=($install_output.exit_code) uninstall_exit=($uninstall_output.exit_code) stdout=($stdout) desktop_exists=(($fixture.desktop_path | path exists)) icons_removed=($icons_removed)"
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
            and ($stdout | str contains $fixture.config_path)
            and ($stdout | str contains "Cleanup-only manual-install artifacts:")
            and ($stdout | str contains $fixture.desktop_path)
            and ($stdout | str contains (($fixture.desktop_icons | first).path))
            and ($stdout | str contains $fixture.manual_wrapper)
            and ($stdout | str contains $fixture.shell_block_path)
            and ($fixture.config_path | path exists)
            and ($fixture.desktop_path | path exists)
            and ($fixture.manual_wrapper | path exists)
            and ($fixture.shell_block_path | path exists)
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
        let desktop_backups = (ls ($fixture.desktop_path | path dirname) | where name =~ 'com\.yazelix\.Yazelix\.desktop\.home-manager-prepare-backup-')
        let wrapper_backups = (ls ($fixture.manual_wrapper | path dirname) | where name =~ 'yzx\.home-manager-prepare-backup-')
        let shell_block_backups = (ls -a ($fixture.shell_block_path | path dirname) | where name =~ '\.bashrc\.home-manager-prepare-backup-')
        let shell_block_current = (open --raw $fixture.shell_block_path)
        let shell_block_backup = if (($shell_block_backups | length) == 1) {
            open --raw ($shell_block_backups | get 0.name)
        } else {
            ""
        }
        let icon_backup_count = (
            $fixture.desktop_icons
            | each {|icon|
                ls ($icon.path | path dirname)
                | where name =~ 'yazelix\.png\.home-manager-prepare-backup-'
                | length
            }
            | math sum
        )

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Archived manual-install artifacts for Home Manager takeover")
            and ($stdout | str contains "home-manager switch")
            and not ($fixture.config_path | path exists)
            and not ($fixture.desktop_path | path exists)
            and not ($fixture.manual_wrapper | path exists)
            and ($fixture.shell_block_path | path exists)
            and (($main_backups | length) == 1)
            and (($desktop_backups | length) == 1)
            and (($wrapper_backups | length) == 1)
            and (($shell_block_backups | length) == 1)
            and ($icon_backup_count == ($fixture.desktop_icons | length))
            and ($shell_block_current == $fixture.shell_block_original_contents)
            and ($shell_block_backup == $fixture.shell_block_contents)
        ) {
            print "  ✅ yzx home_manager prepare --apply archives the real takeover blockers and cleanup-only manual artifacts, then points users at home-manager switch"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) main_backups=(($main_backups | length)) desktop_backups=(($desktop_backups | length)) wrapper_backups=(($wrapper_backups | length)) shell_block_backups=(($shell_block_backups | length)) icon_backups=($icon_backup_count) shell_block_current=($shell_block_current) shell_block_backup=($shell_block_backup)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: yzx update upstream must resolve the active profile-owned Yazelix package and upgrade that exact entry.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_update_upstream_upgrades_matching_profile_entry [] {
    print "🧪 Testing yzx update upstream upgrades the exact profile entry that owns the active runtime..."

    let fixture = (setup_update_wrapper_fixture "yazelix_update_upstream_wrapper")
    let result = (try {
        let profile_list_json = ({
            elements: {
                yazelix: {
                    active: true
                    originalUrl: "github:luccahuguet/yazelix"
                    attrPath: "packages.x86_64-linux.yazelix"
                    storePaths: [$fixture.repo_root]
                }
            }
            version: 3
        } | to json -r)
        let output = (run_yzx_command_for_fixture $fixture "yzx update upstream" {
            PATH: ($env.PATH | prepend $fixture.bin_dir)
            YZX_TEST_LOG: $fixture.command_log
            YZX_TEST_PROFILE_LIST_JSON: $profile_list_json
        })
        let stdout = ($output.stdout | str trim)
        let log_text = (open --raw $fixture.command_log | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Requested update path: default Nix profile.")
            and ($stdout | str contains "Use this only when a Nix profile package owns the active Yazelix runtime.")
            and not ($stdout | str contains "Using the default-profile update path for this install.")
            and ($stdout | str contains "Running:")
            and ($stdout | str contains "nix profile upgrade --refresh yazelix")
            and ($log_text | str contains "nix:profile list --json")
            and ($log_text | str contains "nix:profile upgrade --refresh yazelix")
            and not ($log_text | str contains "home-manager:")
        ) {
            print "  ✅ yzx update upstream now upgrades the exact profile-owned Yazelix entry that matches the active runtime"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) log=($log_text) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: Home Manager-owned installs should not see upstream update wording that implies profile ownership.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_update_upstream_fails_early_for_home_manager_owned_install [] {
    print "🧪 Testing yzx update upstream fails early and clearly for a Home Manager-owned install..."

    let fixture = (setup_update_wrapper_fixture "yazelix_update_upstream_home_manager_owned")
    let result = (try {
        let hm_store_config = ($fixture.tmp_home | path join "hm-store" "abc-home-manager-files" "yazelix.toml")
        mkdir ($hm_store_config | path dirname)
        '[core]
welcome_style = "random"
' | save --force --raw $hm_store_config
        rm $fixture.config_path
        ^ln -s $hm_store_config $fixture.config_path

        let output = (run_yzx_command_for_fixture $fixture "yzx update upstream" {
            PATH: ($env.PATH | prepend $fixture.bin_dir)
            YZX_TEST_LOG: $fixture.command_log
        })
        let stdout = ($output.stdout | str trim)
        let log_text = (open --raw $fixture.command_log | str trim)

        if (
            ($output.exit_code != 0)
            and ($stdout | str contains "this Yazelix runtime appears to be Home Manager-owned")
            and ($stdout | str contains "Run `yzx update home_manager` from the Home Manager flake that owns this install")
            and ($stdout | str contains "home-manager switch")
            and not ($stdout | str contains "Using the default-profile update path for this install.")
            and not ($stdout | str contains "Requested update path: default Nix profile.")
            and ($log_text | is-empty)
        ) {
            print "  ✅ yzx update upstream now rejects Home Manager-owned runtimes before profile-update wording or profile probing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) log=($log_text) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx update upstream must fail fast when the active runtime is not owned by the default Nix profile.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_update_upstream_fails_without_matching_profile_entry [] {
    print "🧪 Testing yzx update upstream fails clearly when no default-profile entry owns the active runtime..."

    let fixture = (setup_update_wrapper_fixture "yazelix_update_upstream_missing_profile")
    let result = (try {
        let profile_list_json = ({
            elements: {
                unrelated: {
                    active: true
                    originalUrl: "flake:nixpkgs"
                    attrPath: "legacyPackages.x86_64-linux.hello"
                    storePaths: ["/nix/store/fake-unrelated"]
                }
            }
            version: 3
        } | to json -r)
        let output = (run_yzx_command_for_fixture $fixture "yzx update upstream" {
            PATH: ($env.PATH | prepend $fixture.bin_dir)
            YZX_TEST_LOG: $fixture.command_log
            YZX_TEST_PROFILE_LIST_JSON: $profile_list_json
        })
        let stdout = ($output.stdout | str trim)
        let log_text = (open --raw $fixture.command_log | str trim)

        if (
            ($output.exit_code != 0)
            and ($stdout | str contains "could not find the active Yazelix runtime in the default Nix profile")
            and ($stdout | str contains "profile-installed Yazelix packages")
            and ($stdout | str contains "nix profile add github:luccahuguet/yazelix#yazelix")
            and ($log_text | str contains "nix:profile list --json")
            and not ($log_text | str contains "nix:profile upgrade")
        ) {
            print "  ✅ yzx update upstream now fails clearly instead of guessing when the current runtime is not profile-owned"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) log=($log_text) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: yzx update home_manager must update only the current flake input and print the manual switch step.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_update_home_manager_updates_input_and_prints_manual_switch_step [] {
    print "🧪 Testing yzx update home_manager updates only the current flake input and prints the manual switch step..."

    let fixture = (setup_update_wrapper_fixture "yazelix_update_home_manager_wrapper")
    let result = (try {
        let output = (run_yzx_command_for_fixture_in_dir $fixture $fixture.flake_dir "yzx update home_manager" {
            PATH: ($env.PATH | prepend $fixture.bin_dir)
            YZX_TEST_LOG: $fixture.command_log
        })
        let stdout = ($output.stdout | str trim)
        let log_text = (open --raw $fixture.command_log | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Requested update path: Home Manager flake input.")
            and ($stdout | str contains "Use this only when Home Manager owns the active Yazelix runtime.")
            and not ($stdout | str contains "Using the Home Manager update path for this install.")
            and ($stdout | str contains "Running:")
            and ($stdout | str contains "nix flake update yazelix")
            and ($stdout | str contains "Next step:")
            and ($stdout | str contains "home-manager switch")
            and ($log_text | str contains "nix:flake update yazelix")
            and not ($log_text | str contains "home-manager:")
        ) {
            print "  ✅ yzx update home_manager now refreshes only the current flake input and leaves the switch step to the user"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) log=($log_text) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: future profile updates must not stay trapped on an older store-pinned yzx invocation from a stale host-shell function.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_stale_store_pinned_yzx_invocation_redirects_to_profile_wrapper [] {
    print "🧪 Testing stale store-pinned yzx invocation redirects to the current profile wrapper..."

    let fixture = (setup_profile_wrapper_redirect_fixture "yazelix_store_pinned_yzx_redirect")
    let stale_store_yzx = "/nix/store/old-yazelix/bin/yzx"
    let old_cli = ($fixture.old_runtime | path join "shells" "posix" "yzx_cli.sh")

    let result = (try {
        let output = (with-env {
            HOME: $fixture.fixture.tmp_home
            USER: ($env.USER? | default "test-user")
            YZX_REDIRECT_LOG: $fixture.redirect_log
            YAZELIX_INVOKED_YZX_PATH: $stale_store_yzx
        } {
            ^sh $old_cli update upstream --yes | complete
        })
        let redirected_args = (open --raw $fixture.redirect_log | str trim)

        if (
            ($output.exit_code == 0)
            and ($redirected_args == "update upstream --yes")
        ) {
            print "  ✅ Stale store-pinned yzx invocations now hand off to the current profile wrapper instead of staying on the old runtime"
            true
        } else {
            print $"  ❌ Unexpected redirect result: exit=($output.exit_code) redirected_args=($redirected_args) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.fixture.tmp_home
    $result
}

# Defends: yzx run must forward dash-prefixed child args without forcing quoting or wrapper-side parsing.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_run_passes_dash_prefixed_args_through_unchanged [] {
    print "🧪 Testing yzx run forwards dash-prefixed child args unchanged..."

    let fixture = (setup_run_passthrough_fixture "yazelix_run_passes_dash_prefixed_args")
    let result = (try {
        let output = (run_stubbed_yzx_run $fixture "yzx run rg --files --hidden")
        let logged = (open $fixture.command_log)

        if (
            ($output.exit_code == 0)
            and ($logged.command == "rg")
            and ($logged.args == ["--files", "--hidden"])
            and ($logged.cwd == (pwd))
            and $logged.config_present
        ) {
            print "  ✅ yzx run now treats dash-prefixed child args as child argv instead of wrapper flags"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) logged=($logged | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx run must not consume child --verbose flags as Yazelix wrapper flags.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_run_treats_child_verbose_flag_as_child_argv [] {
    print "🧪 Testing yzx run leaves child --verbose flags inside child argv..."

    let fixture = (setup_run_passthrough_fixture "yazelix_run_child_verbose_passthrough")
    let result = (try {
        let output = (run_stubbed_yzx_run $fixture "yzx run cargo --verbose check")
        let logged = (open $fixture.command_log)

        if (
            ($output.exit_code == 0)
            and ($logged.command == "cargo")
            and ($logged.args == ["--verbose", "check"])
            and ($logged.cwd == (pwd))
            and $logged.config_present
        ) {
            print "  ✅ yzx run no longer steals child --verbose flags for wrapper parsing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) logged=($logged | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
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
        let yazi_keymap_stdout = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx edit keymap --print
        }
        let yazi_init_stdout = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            yzx edit init --print
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
        let expected_helix = ($temp_config_dir | path join "user_configs" "helix" "config.toml")
        let expected_zellij = ($temp_config_dir | path join "user_configs" "zellij" "config.kdl")
        let expected_yazi = ($temp_config_dir | path join "user_configs" "yazi" "yazi.toml")
        let expected_yazi_keymap = ($temp_config_dir | path join "user_configs" "yazi" "keymap.toml")
        let expected_yazi_init = ($temp_config_dir | path join "user_configs" "yazi" "init.lua")
        let missing_subcommand_stderr = ($missing_subcommand_output.stderr | str trim)
        let invalid_stderr = ($invalid_output.stderr | str trim)

        if (
            ($missing_subcommand_output.exit_code != 0)
            and ($invalid_output.exit_code != 0)
            and ($main_stdout == $expected_main)
            and ($helix_stdout == $expected_helix)
            and ($zellij_stdout == $expected_zellij)
            and ($yazi_stdout == $expected_yazi)
            and ($yazi_keymap_stdout == $expected_yazi_keymap)
            and ($yazi_init_stdout == $expected_yazi_init)
            and ($missing_subcommand_stderr | str contains "requires a target query")
            and ($invalid_stderr | str contains "No managed Yazelix config surface matched")
        ) {
            print "  ✅ yzx edit resolves canonical managed surfaces through permissive target queries and rejects unsupported noninteractive cases"
            true
        } else {
            print $"  ❌ Unexpected result: main=($main_stdout) helix=($helix_stdout) zellij=($zellij_stdout) yazi=($yazi_stdout) yazi_keymap=($yazi_keymap_stdout) yazi_init=($yazi_init_stdout) missing_exit=($missing_subcommand_output.exit_code) missing_stderr=($missing_subcommand_stderr) invalid_exit=($invalid_output.exit_code) invalid_stderr=($invalid_stderr)"
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
            and ($stdout | str contains "Unknown config field at core.refresh_output")
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

# Regression: yzx status must import and use the shared environment bootstrap successfully.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_status_reports_basic_runtime_summary [] {
    print "🧪 Testing yzx status reports the basic runtime summary through the shared environment bootstrap..."

    let fixture = (setup_managed_config_fixture
        "yazelix_status_summary"
        '[core]
welcome_style = "random"

[shell]
default_shell = "nu"

[terminal]
terminals = ["ghostty"]
'
    )

    let result = (try {
        let output = (run_yzx_command_for_fixture $fixture "yzx status")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix status")
            and ($stdout | str contains $"config_file")
            and ($stdout | str contains "yazelix.toml")
            and ($stdout | str contains "default_shell")
            and ($stdout | str contains "nu")
            and ($stdout | str contains "terminals")
            and ($stdout | str contains "ghostty")
        ) {
            print "  ✅ yzx status now reaches the shared environment bootstrap and reports the live config summary"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx menu should derive its catalog from the live exported command tree instead of a handwritten list.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_menu_catalog_tracks_live_exported_command_surface [] {
    print "🧪 Testing yzx menu catalog tracks the live exported command surface..."

    let fixture = (setup_palette_catalog_runtime_fixture "yazelix_menu_catalog_runtime")
    let menu_script = (repo_path "nushell" "scripts" "yzx" "menu.nu")

    let result = (try {
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $fixture.runtime_root
        } {
            ^nu -c $"source \"($menu_script)\"; { entries: \(get_palette_command_entries | select id category\), prompt: \(menu_prompt\), post_prompt: \(popup_post_action_prompt\), esc_decision: \(popup_post_action_key_decision esc\), enter_decision: \(popup_post_action_key_decision enter\), backspace_decision: \(popup_post_action_key_decision backspace\) } | to json -r" | complete
        })
        let contract = ($output.stdout | from json)
        let entries = ($contract.entries | default [])
        let ids = ($entries | get id)

        if (
            ($output.exit_code == 0)
            and ("yzx" in $ids)
            and ("yzx launch" in $ids)
            and ("yzx status" in $ids)
            and ("yzx screen" in $ids)
            and ("yzx test_dynamic" in $ids)
            and not ("yzx env" in $ids)
            and not ("yzx run" in $ids)
            and not ("yzx cwd" in $ids)
            and not ("yzx dev sync_issues" in $ids)
            and (($entries | where id == "yzx" | get -o 0.category | default "") == "help")
            and (($entries | where id == "yzx launch" | get -o 0.category | default "") == "session")
            and (($entries | where id == "yzx screen" | get -o 0.category | default "") == "workspace")
            and (($entries | where id == "yzx status" | get -o 0.category | default "") == "system")
            and (($entries | where id == "yzx test_dynamic" | get -o 0.category | default "") == "system")
            and ($contract.prompt == "yzx menu> ")
            and ($contract.post_prompt == "Backspace: return to menu | Enter: close")
            and ($contract.esc_decision == "continue")
            and ($contract.enter_decision == "close")
            and ($contract.backspace_decision == "menu")
        ) {
            print "  ✅ yzx menu now derives its catalog from the live exported command surface, applies explicit exclusions, and leaves Escape out of its own close path"
            true
        } else {
            print $"  ❌ Unexpected menu catalog or key contract result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_root
    $result
}

# Defends: exported yzx commands carry concise help descriptions without maintaining a second command tree.
# Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=2 total=8/10
def test_yzx_exported_commands_have_help_descriptions [] {
    print "🧪 Testing exported yzx commands have help descriptions..."

    let blank_descriptions = (
        help commands
        | where name =~ "^yzx( |$)"
        | where {|command| (($command.description? | default "" | into string | str trim) | is-empty)}
        | get name
    )

    if ($blank_descriptions | is-empty) {
        print "  ✅ Every exported yzx command now carries a nonblank Nushell help description"
        true
    } else {
        print $"  ❌ Exported yzx commands with blank descriptions: ($blank_descriptions | str join ', ')"
        false
    }
}

export def run_core_canonical_tests [] {
    [
        (test_yzx_desktop_install_writes_entry_and_icon_assets)
        (test_yzx_desktop_install_prefers_installed_wrapper)
        (test_stable_yzx_wrapper_prefers_home_manager_profile_owner)
        (test_stable_yzx_wrapper_keeps_home_manager_broken_profile_symlink)
        (test_yzx_desktop_install_refuses_home_manager_owned_install)
        (test_yzx_desktop_uninstall_preserves_home_manager_cleanup_path)
        (test_yzx_desktop_uninstall_removes_manual_entry_and_icons)
        (test_yzx_home_manager_prepare_preview_reports_manual_takeover_artifacts)
        (test_yzx_home_manager_prepare_apply_archives_manual_takeover_artifacts)
        (test_yzx_update_upstream_upgrades_matching_profile_entry)
        (test_yzx_update_upstream_fails_early_for_home_manager_owned_install)
        (test_yzx_update_upstream_fails_without_matching_profile_entry)
        (test_yzx_update_home_manager_updates_input_and_prints_manual_switch_step)
        (test_stale_store_pinned_yzx_invocation_redirects_to_profile_wrapper)
        (test_yzx_run_passes_dash_prefixed_args_through_unchanged)
        (test_yzx_run_treats_child_verbose_flag_as_child_argv)
        (test_yzx_edit_targets_print_paths)
        (test_invalid_config_is_classified_as_config_problem)
        (test_yzx_status_reports_basic_runtime_summary)
        (test_yzx_menu_catalog_tracks_live_exported_command_surface)
        (test_yzx_exported_commands_have_help_descriptions)
    ]
}
