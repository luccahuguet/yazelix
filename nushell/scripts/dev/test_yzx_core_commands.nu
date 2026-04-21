#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ../core/yazelix.nu *
use ../utils/config_state.nu [compute_config_state record_materialized_state]
use ./yzx_test_helpers.nu [get_repo_config_dir repo_path resolve_test_yzx_bin resolve_test_yzx_control_bin resolve_test_yzx_core_bin setup_managed_config_fixture]

const DESKTOP_ICON_SIZES = ["48x48", "64x64", "128x128", "256x256"]

def run_yzx_command_for_fixture [fixture: record, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_STATE_DIR: ($fixture.tmp_home | path join ".local" "share" "yazelix")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    }
    let merged_env = if ($extra_env | is-empty) {
        $base_env
    } else {
        $base_env | merge $extra_env
    }

    let tokens = ($command | str trim | split row " " | where {|t| ($t | str length) > 0})
    let is_yzx_update = (
        ($tokens | length) >= 2
            and ($tokens | get 0) == "yzx"
            and ($tokens | get 1) == "update"
    )

    if $is_yzx_update {
        let yzx_cli = ($fixture.repo_root | path join "shells" "posix" "yzx_cli.sh")
        let cli_env = ($merged_env | merge {
            YAZELIX_YZX_BIN: (resolve_test_yzx_bin)
            YAZELIX_YZX_CONTROL_BIN: (resolve_test_yzx_control_bin)
            YAZELIX_TEST_PATH_PREPEND: $fixture.bin_dir
        })
        with-env $cli_env {
            ^sh $yzx_cli ...($tokens | skip 1) | complete
        }
    } else {
        with-env $merged_env {
            ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
        }
    }
}

def run_yzx_command_for_fixture_in_dir [fixture: record, working_dir: string, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_STATE_DIR: ($fixture.tmp_home | path join ".local" "share" "yazelix")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    }
    let merged_env = if ($extra_env | is-empty) {
        $base_env
    } else {
        $base_env | merge $extra_env
    }

    let tokens = ($command | str trim | split row " " | where {|t| ($t | str length) > 0})
    let is_yzx_update = (
        ($tokens | length) >= 2
            and ($tokens | get 0) == "yzx"
            and ($tokens | get 1) == "update"
    )

    if $is_yzx_update {
        let yzx_cli = ($fixture.repo_root | path join "shells" "posix" "yzx_cli.sh")
        let cli_env = ($merged_env | merge {
            YAZELIX_YZX_BIN: (resolve_test_yzx_bin)
            YAZELIX_YZX_CONTROL_BIN: (resolve_test_yzx_control_bin)
            YAZELIX_TEST_PATH_PREPEND: $fixture.bin_dir
        })
        with-env $cli_env {
            do {
                cd $working_dir
                ^sh $yzx_cli ...($tokens | skip 1) | complete
            }
        }
    } else {
        with-env $merged_env {
            do {
                cd $working_dir
                ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
            }
        }
    }
}

def run_public_yzx_command_for_fixture [fixture: record, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_STATE_DIR: ($fixture.tmp_home | path join ".local" "share" "yazelix")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
        YAZELIX_YZX_BIN: (resolve_test_yzx_bin)
        YAZELIX_YZX_CONTROL_BIN: (resolve_test_yzx_control_bin)
    }
    let merged_env = if ($extra_env | is-empty) {
        $base_env
    } else {
        $base_env | merge $extra_env
    }

    let tokens = ($command | str trim | split row " " | where {|t| ($t | str length) > 0})
    let yzx_cli = ($fixture.repo_root | path join "shells" "posix" "yzx_cli.sh")

    with-env $merged_env {
        ^sh $yzx_cli ...($tokens | skip 1) | complete
    }
}

def run_direct_public_yzx_command_for_fixture [fixture: record, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_STATE_DIR: ($fixture.tmp_home | path join ".local" "share" "yazelix")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
        YAZELIX_YZX_BIN: (resolve_test_yzx_bin)
        YAZELIX_YZX_CONTROL_BIN: (resolve_test_yzx_control_bin)
    }
    let merged_env = if ($extra_env | is-empty) {
        $base_env
    } else {
        $base_env | merge $extra_env
    }

    let tokens = ($command | str trim | split row " " | where {|t| ($t | str length) > 0})
    let yzx_bin = (resolve_test_yzx_bin)

    with-env $merged_env {
        ^$yzx_bin ...($tokens | skip 1) | complete
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

    mkdir ($desktop_path | path dirname)
    mkdir ($manual_wrapper | path dirname)

    for icon in $desktop_icons {
        mkdir ($icon.path | path dirname)
        ^cp $icon.source $icon.path
    }

    write_test_legacy_yzx_wrapper $manual_wrapper

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

def setup_home_manager_dangling_config_fixture [label: string] {
    let fixture = (setup_manual_install_takeover_fixture $label)
    let hm_store_config = ($fixture.tmp_home | path join "hm-store" "abc-home-manager-files" "missing-yazelix.toml")

    mkdir ($hm_store_config | path dirname)
    rm $fixture.config_path
    ^ln -s $hm_store_config $fixture.config_path

    $fixture | merge {
        hm_store_config: $hm_store_config
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

    let bin_dir = ($fixture.tmp_home | path join "bin")
    let command_log = ($fixture.tmp_home | path join "run_passthrough.json")
    mkdir $bin_dir
    let log_argv = ($bin_dir | path join "yzx_test_log_argv")
    ^cp (repo_path "nushell" "scripts" "dev" "fixtures" "yzx_run_argv_log.sh") $log_argv
    ^chmod +x $log_argv

    $fixture | merge {
        command_log: $command_log
        passthrough_bin_dir: $bin_dir
        zyx_control_bin: (resolve_test_yzx_control_bin)
    }
}

def run_stubbed_yzx_run [fixture: record, command: string] {
    let tail = ($command | str trim | str replace -r '^yzx run ' '')
    let tokens = ($tail | split row " " | where {|t| ($t | str length) > 0})
    let log_bin = ($fixture.passthrough_bin_dir | path join "yzx_test_log_argv")
    let argv = ([$log_bin] | append $tokens)
    with-env {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
        YZX_RUN_LOG: $fixture.command_log
    } {
        ^$fixture.zyx_control_bin run ...$argv | complete
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
            XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
        } {
            ^(resolve_test_yzx_core_bin) install-ownership.evaluate --from-env --runtime-dir $fixture.repo_root | complete
        })
        let resolved = if $output.exit_code == 0 {
            (($output.stdout | from json).data.stable_yzx_wrapper? | default "")
        } else {
            ""
        }

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
            XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
        } {
            ^(resolve_test_yzx_core_bin) install-ownership.evaluate --from-env --runtime-dir $fixture.repo_root | complete
        })
        let resolved = if $output.exit_code == 0 {
            (($output.stdout | from json).data.stable_yzx_wrapper? | default "")
        } else {
            ""
        }

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

# Regression: a dangling Home Manager config symlink still means desktop integration is Home Manager-owned.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_install_refuses_dangling_home_manager_config [] {
    print "🧪 Testing yzx desktop install refuses a Home Manager-owned install even when the config symlink target is missing..."

    let fixture = (setup_home_manager_dangling_config_fixture "yazelix_desktop_install_home_manager_dangling_config")

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
            print "  ✅ yzx desktop install still refuses Home Manager-owned desktop integration when the tracked config symlink is dangling"
            true
        } else {
            print $"  ❌ Unexpected dangling Home Manager desktop install result: exit=($output.exit_code) stderr=($stderr) desktop_exists=($desktop_entry_still_exists)"
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

# Defends: the public yzx home_manager root still advertises the takeover helper entrypoints after the Rust owner cut.
# Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=2 total=7/10
def test_public_yzx_home_manager_lists_takeover_helpers [] {
    print "🧪 Testing public yzx home_manager still lists takeover helpers..."

    let fixture = (setup_managed_config_fixture
        "yazelix_home_manager_root"
        '[core]
welcome_style = "random"
'
    )

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx home_manager")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix Home Manager helpers")
            and ($stdout | str contains "yzx home_manager prepare")
            and ($stdout | str contains "yzx update home_manager")
        ) {
            print "  ✅ public yzx home_manager still lists the takeover helper entrypoints"
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

# Defends: the public Rust-owned `yzx why` route must keep the existing elevator-pitch copy after deleting the Nushell support owner.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_public_yzx_why_prints_elevator_pitch [] {
    print "🧪 Testing public yzx why still prints the Yazelix elevator pitch..."

    let fixture = (setup_managed_config_fixture
        "yazelix_why_pitch"
        '[core]
welcome_style = "random"
'
    )

    let result = (try {
        let output = (run_direct_public_yzx_command_for_fixture $fixture "yzx why")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix is a reproducible terminal IDE")
            and ($stdout | str contains "Zero")
            and ($stdout | str contains "Install once, get the same environment everywhere.")
        ) {
            print "  ✅ public yzx why keeps the existing pitch copy through the Rust owner"
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

# Defends: the public Rust-owned `yzx sponsor` route must still fall back to printing the sponsor URL when no opener is available.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_sponsor_falls_back_to_printed_url_without_openers [] {
    print "🧪 Testing public yzx sponsor falls back to printing the URL when no opener is available..."

    let fixture = (setup_managed_config_fixture
        "yazelix_sponsor_fallback"
        '[core]
welcome_style = "random"
'
    )
    let bin_dir = ($fixture.tmp_home | path join "bin")
    mkdir $bin_dir

    let result = (try {
        let output = (run_direct_public_yzx_command_for_fixture $fixture "yzx sponsor" {
            PATH: $bin_dir
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Support Yazelix:")
            and ($stdout | str contains "https://github.com/sponsors/luccahuguet")
            and not ($stdout | str contains "Opened sponsor page.")
        ) {
            print "  ✅ public yzx sponsor still prints the sponsor URL when no opener is available"
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

# Defends: the public Rust-owned `yzx keys` root keeps the sectioned keybinding discoverability surface instead of collapsing into a flat text dump.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_keys_root_preserves_discoverability_sections [] {
    print "🧪 Testing public yzx keys keeps the sectioned discoverability surface..."

    let fixture = (setup_managed_config_fixture
        "yazelix_keys_root"
        '[core]
welcome_style = "random"
'
    )

    let result = (try {
        let output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix keybindings")
            and ($stdout | str contains "Workspace actions")
            and ($stdout | str contains "Command access")
            and ($stdout | str contains "Tab and pane movement")
            and ($stdout | str contains "Alt+Shift+M")
            and ($stdout | str contains "yzx keys yazi")
            and ($stdout | str contains "yzx keys hx")
            and ($stdout | str contains "yzx keys nu")
        ) {
            print "  ✅ public yzx keys keeps the sectioned keybinding discoverability surface through the Rust owner"
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

# Defends: the Rust keys owner cut must preserve the public alias family exactly so users can keep using yzx/yazi/hx/helix/nu/nushell views.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_keys_aliases_preserve_views [] {
    print "🧪 Testing public yzx keys aliases preserve the expected view outputs..."

    let fixture = (setup_managed_config_fixture
        "yazelix_keys_aliases"
        '[core]
welcome_style = "random"
'
    )

    let result = (try {
        let root_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys")
        let yzx_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys yzx")
        let hx_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys hx")
        let helix_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys helix")
        let nu_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys nu")
        let nushell_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys nushell")

        if (
            ($root_output.exit_code == 0)
            and ($yzx_output.exit_code == 0)
            and ($hx_output.exit_code == 0)
            and ($helix_output.exit_code == 0)
            and ($nu_output.exit_code == 0)
            and ($nushell_output.exit_code == 0)
            and ($root_output.stdout == $yzx_output.stdout)
            and ($hx_output.stdout == $helix_output.stdout)
            and ($nu_output.stdout == $nushell_output.stdout)
        ) {
            print "  ✅ public yzx keys aliases still resolve to the same keybinding views after the Rust owner cut"
            true
        } else {
            print $"  ❌ Unexpected alias result: root_exit=($root_output.exit_code) yzx_exit=($yzx_output.exit_code) hx_exit=($hx_output.exit_code) helix_exit=($helix_output.exit_code) nu_exit=($nu_output.exit_code) nushell_exit=($nushell_output.exit_code)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: the Rust-owned yzx keys leaf views must still carry the tool-specific guidance instead of regressing to the root summary for every subcommand.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_keys_tool_specific_views_keep_guidance [] {
    print "🧪 Testing public yzx keys tool-specific views keep their tool guidance..."

    let fixture = (setup_managed_config_fixture
        "yazelix_keys_leaf_views"
        '[core]
welcome_style = "random"
'
    )

    let result = (try {
        let yazi_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys yazi")
        let helix_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys hx")
        let nu_output = (run_direct_public_yzx_command_for_fixture $fixture "yzx keys nu")

        if (
            ($yazi_output.exit_code == 0)
            and ($helix_output.exit_code == 0)
            and ($nu_output.exit_code == 0)
            and (($yazi_output.stdout | str contains "Yazi keybindings"))
            and (($yazi_output.stdout | str contains "Focus the Yazi pane and press `~`"))
            and (($helix_output.stdout | str contains "Helix keybindings"))
            and (($helix_output.stdout | str contains "https://docs.helix-editor.com/master/keymap.html"))
            and (($nu_output.stdout | str contains "Nushell keybindings"))
            and (($nu_output.stdout | str contains "https://www.nushell.sh/book/line_editor.html"))
        ) {
            print "  ✅ public yzx keys leaf views keep their tool-specific discoverability guidance through the Rust owner"
            true
        } else {
            print $"  ❌ Unexpected result: yazi_exit=($yazi_output.exit_code) helix_exit=($helix_output.exit_code) nu_exit=($nu_output.exit_code)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: Home Manager takeover preview surfaces both blocking and cleanup-only manual artifacts through the public yzx route.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_home_manager_prepare_preview_reports_manual_takeover_artifacts [] {
    print "🧪 Testing yzx home_manager prepare preview reports takeover blockers and cleanup-only manual artifacts..."

    let fixture = (setup_manual_install_takeover_fixture "yazelix_home_manager_prepare_preview")

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx home_manager prepare")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Blocking manual-install artifacts:")
            and ($stdout | str contains $fixture.config_path)
            and ($stdout | str contains "Cleanup-only manual-install artifacts:")
            and ($stdout | str contains $fixture.desktop_path)
            and ($stdout | str contains (($fixture.desktop_icons | first).path))
            and ($stdout | str contains $fixture.manual_wrapper)
            and ($fixture.config_path | path exists)
            and ($fixture.desktop_path | path exists)
            and ($fixture.manual_wrapper | path exists)
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

# Defends: Home Manager takeover apply archives the blocking manual paths and points users at home-manager switch through the public yzx route.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_home_manager_prepare_apply_archives_manual_takeover_artifacts [] {
    print "🧪 Testing yzx home_manager prepare --apply archives manual-install takeover artifacts..."

    let fixture = (setup_manual_install_takeover_fixture "yazelix_home_manager_prepare_apply")

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx home_manager prepare --apply --yes")
        let stdout = ($output.stdout | str trim)
        let main_backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.home-manager-prepare-backup-')
        let desktop_backups = (ls ($fixture.desktop_path | path dirname) | where name =~ 'com\.yazelix\.Yazelix\.desktop\.home-manager-prepare-backup-')
        let wrapper_backups = (ls ($fixture.manual_wrapper | path dirname) | where name =~ 'yzx\.home-manager-prepare-backup-')
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
            and (($main_backups | length) == 1)
            and (($desktop_backups | length) == 1)
            and (($wrapper_backups | length) == 1)
            and ($icon_backup_count == ($fixture.desktop_icons | length))
        ) {
            print "  ✅ yzx home_manager prepare --apply archives the real takeover blockers and cleanup-only manual artifacts, then points users at home-manager switch"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) main_backups=(($main_backups | length)) desktop_backups=(($desktop_backups | length)) wrapper_backups=(($wrapper_backups | length)) icon_backups=($icon_backup_count)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: the public Rust-owned `yzx config --path` route returns the resolved managed config path instead of depending on the deleted Nu owner.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_public_yzx_config_prints_resolved_path [] {
    print "🧪 Testing public yzx config --path returns the resolved managed config path..."

    let fixture = (setup_managed_config_fixture
        "yazelix_config_print_path"
        '[core]
welcome_style = "random"
'
    )

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx config --path")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout == $fixture.config_path)
        ) {
            print "  ✅ public yzx config --path now returns the managed config path through the Rust owner"
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

# Defends: the public Rust-owned `yzx config` route still bootstraps a missing managed config from the shipped default and prints the resulting TOML.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_config_bootstraps_missing_user_config [] {
    print "🧪 Testing public yzx config bootstraps a missing managed config from the shipped default..."

    let fixture = (setup_managed_config_fixture
        "yazelix_config_bootstrap"
        '[core]
welcome_style = "random"
'
    )
    rm $fixture.config_path

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx config")
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)
        let written_config = (open --raw $fixture.config_path)
        let default_config = (open --raw ($fixture.repo_root | path join "yazelix_default.toml"))

        if (
            ($output.exit_code == 0)
            and ($fixture.config_path | path exists)
            and ($stderr | str contains "Creating yazelix.toml from yazelix_default.toml")
            and ($stderr | str contains "yazelix.toml created")
            and (($written_config | str trim) == ($default_config | str trim))
            and (($stdout | str trim) == ($default_config | str trim))
        ) {
            print "  ✅ public yzx config still bootstraps the managed config from the shipped default through the Rust owner"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=(($stdout | str substring 0..200)) stderr=($stderr)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: `yzx config reset --yes` must replace the managed config with the shipped default and keep a readable backup after the Rust owner cut.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_config_reset_writes_backup_and_restores_default [] {
    print "🧪 Testing public yzx config reset --yes writes a backup and restores the shipped default..."

    let custom_config = '[core]
welcome_style = "minimal"
'
    let fixture = (setup_managed_config_fixture
        "yazelix_config_reset_backup"
        $custom_config
    )

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx config reset --yes")
        let stdout = ($output.stdout | str trim)
        let restored_config = (open --raw $fixture.config_path)
        let default_config = (open --raw ($fixture.repo_root | path join "yazelix_default.toml"))
        let backups = (
            ls $fixture.user_config_dir
            | where name =~ 'yazelix\.toml\.backup-\d{8}_\d{6}$'
        )
        let backup_content = if (($backups | length) == 1) {
            open --raw (($backups | get 0.name) | into string)
        } else {
            ""
        }

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Backed up previous config to:")
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template:")
            and (($backups | length) == 1)
            and (($backup_content | str trim) == ($custom_config | str trim))
            and (($restored_config | str trim) == ($default_config | str trim))
        ) {
            print "  ✅ public yzx config reset --yes now preserves a readable backup and restores the shipped default through the Rust owner"
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

# Defends: `yzx config reset --yes --no-backup` must replace the managed config without leaving backup files behind after the owner cut.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_config_reset_without_backup_replaces_config [] {
    print "🧪 Testing public yzx config reset --yes --no-backup replaces the managed config without backups..."

    let fixture = (setup_managed_config_fixture
        "yazelix_config_reset_no_backup"
        '[core]
welcome_style = "minimal"
'
    )

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx config reset --yes --no-backup")
        let stdout = ($output.stdout | str trim)
        let restored_config = (open --raw $fixture.config_path)
        let default_config = (open --raw ($fixture.repo_root | path join "yazelix_default.toml"))
        let backups = (
            ls $fixture.user_config_dir
            | where name =~ 'yazelix\.toml\.backup-\d{8}_\d{6}$'
        )

        if (
            ($output.exit_code == 0)
            and not ($stdout | str contains "Backed up previous config to:")
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template:")
            and ($stdout | str contains "Previous config surface was removed without backup.")
            and (($backups | length) == 0)
            and (($restored_config | str trim) == ($default_config | str trim))
        ) {
            print "  ✅ public yzx config reset --yes --no-backup now replaces the managed config without preserving a backup"
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

# Regression: the public Rust yzx root must route env/run/status/update/doctor through Rust even when the remaining direct Nu route modules are unavailable.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_public_yzx_root_routes_rust_control_family_without_direct_nu_route_modules [] {
    print "🧪 Testing the public Rust yzx root keeps env/run/cwd/reveal/status/update/doctor off the old Nu root registry..."

    let fixture = (setup_managed_config_fixture
        "yazelix_public_root_control_family"
        '[core]
welcome_style = "random"
'
    )
    let missing_route_root = ($fixture.tmp_home | path join "missing_internal_nu_route_root")

    let result = (try {
        let update_output = (run_public_yzx_command_for_fixture $fixture "yzx update" {
            YAZELIX_YZX_NU_ROUTE_ROOT: $missing_route_root
        })
        let cwd_help = (run_public_yzx_command_for_fixture $fixture "yzx cwd --help" {
            YAZELIX_YZX_NU_ROUTE_ROOT: $missing_route_root
        })
        let env_help = (run_public_yzx_command_for_fixture $fixture "yzx env --help" {
            YAZELIX_YZX_NU_ROUTE_ROOT: $missing_route_root
        })
        let reveal_help = (run_public_yzx_command_for_fixture $fixture "yzx reveal --help" {
            YAZELIX_YZX_NU_ROUTE_ROOT: $missing_route_root
        })
        let run_help = (run_public_yzx_command_for_fixture $fixture "yzx run --help" {
            YAZELIX_YZX_NU_ROUTE_ROOT: $missing_route_root
        })
        let status_output = (run_public_yzx_command_for_fixture $fixture "yzx status --help" {
            YAZELIX_YZX_NU_ROUTE_ROOT: $missing_route_root
        })
        let doctor_output = (run_public_yzx_command_for_fixture $fixture "yzx doctor --json" {
            YAZELIX_YZX_NU_ROUTE_ROOT: $missing_route_root
        })
        let update_stdout = ($update_output.stdout | str trim)
        let cwd_stdout = ($cwd_help.stdout | str trim)
        let env_stdout = ($env_help.stdout | str trim)
        let reveal_stdout = ($reveal_help.stdout | str trim)
        let run_stdout = ($run_help.stdout | str trim)
        let status_stdout = ($status_output.stdout | str trim)
        let doctor_report = ($doctor_output.stdout | from json)

        if (
            ($update_output.exit_code == 0)
            and ($cwd_help.exit_code == 0)
            and ($env_help.exit_code == 0)
            and ($reveal_help.exit_code == 0)
            and ($run_help.exit_code == 0)
            and ($status_output.exit_code == 0)
            and ($doctor_output.exit_code == 0)
            and ($update_stdout | str contains "Available update commands:")
            and ($update_stdout | str contains "yzx update upstream")
            and ($cwd_stdout | str contains "Usage:")
            and ($cwd_stdout | str contains "yzx cwd [target]")
            and ($env_stdout | str contains "Usage:")
            and ($env_stdout | str contains "yzx env [--no-shell]")
            and ($reveal_stdout | str contains "Usage:")
            and ($reveal_stdout | str contains "yzx reveal <target>")
            and ($run_stdout | str contains "Usage:")
            and ($run_stdout | str contains "yzx run <command> [args...]")
            and ($status_stdout | str contains "Usage:")
            and ($status_stdout | str contains "yzx status [--versions] [--json]")
            and ($status_stdout | str contains "--versions")
            and (($doctor_report.title? | default "") == "Yazelix Health Checks")
            and (($doctor_report.results? | default [] | length) >= 1)
        ) {
            print "  ✅ the public Rust yzx root now owns env/run/cwd/reveal/status/update/doctor routing without depending on the old Nu root registry"
            true
        } else {
            print $"  ❌ Unexpected public-root routing result: update_exit=($update_output.exit_code) cwd_exit=($cwd_help.exit_code) env_exit=($env_help.exit_code) reveal_exit=($reveal_help.exit_code) run_exit=($run_help.exit_code) status_exit=($status_output.exit_code) doctor_exit=($doctor_output.exit_code) update_stdout=($update_stdout) cwd_stdout=($cwd_stdout) env_stdout=($env_stdout) reveal_stdout=($reveal_stdout) run_stdout=($run_stdout) status_stdout=($status_stdout) doctor_stdout=(($doctor_output.stdout | str trim)) update_stderr=(($update_output.stderr | str trim)) cwd_stderr=(($cwd_help.stderr | str trim)) env_stderr=(($env_help.stderr | str trim)) reveal_stderr=(($reveal_help.stderr | str trim)) run_stderr=(($run_help.stderr | str trim)) status_stderr=(($status_output.stderr | str trim)) doctor_stderr=(($doctor_output.stderr | str trim))"
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

# Regression: yzx status must reach the Rust status helper and render the live summary.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_status_reports_basic_runtime_summary [] {
    print "🧪 Testing yzx status reports the basic runtime summary from yzx_core..."

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
        let output = (run_public_yzx_command_for_fixture $fixture "yzx status")
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
            print "  ✅ yzx status now reports the live config summary via the Rust status helper"
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

# Regression: yzx status should expose the same runtime summary as machine-readable structured data.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_status_json_reports_typed_summary [] {
    print "🧪 Testing yzx status --json reports a typed runtime summary..."

    let fixture = (setup_managed_config_fixture
        "yazelix_status_summary_json"
        '[shell]
default_shell = "nu"

[terminal]
terminals = ["ghostty"]
'
    )

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx status --json")
        let report = ($output.stdout | from json)
        let summary = ($report.summary? | default {})

        if (
            ($output.exit_code == 0)
            and (($report.title? | default "") == "Yazelix status")
            and (($summary.config_file? | default "") | str ends-with "yazelix.toml")
            and (($summary.default_shell? | default "") == "nu")
            and (($summary.terminals? | default []) == ["ghostty"])
            and (($summary.generated_state_repair_needed? | default null) != null)
            and (($summary.generated_state_materialization_status? | default "") | is-not-empty)
            and (($summary.generated_state_materialization_reason? | default "") | describe) == "string"
            and (($summary.persistent_sessions? | default null) == false)
            and (($summary.session_name? | default null) == null)
        ) {
            print "  ✅ yzx status --json now exposes the structured runtime summary behind the human table rendering"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx status --versions must keep the public Rust owner while still exposing the tool version matrix.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_status_versions_prints_tool_version_matrix [] {
    print "🧪 Testing yzx status --versions prints the Rust-owned tool version matrix..."

    let fixture = (setup_managed_config_fixture
        "yazelix_status_versions"
        '[terminal]
terminals = ["ghostty"]
'
    )

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx status --versions")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix status")
            and ($stdout | str contains "Yazelix Tool Versions")
            and ($stdout | str contains "nix")
            and ($stdout | str contains "nushell")
        ) {
            print "  ✅ yzx status --versions now stays on the Rust-owned path and prints the tool matrix"
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

# Regression: yzx status --json --versions must attach the optional versions report promised by the machine-readable contract.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_status_json_with_versions_reports_tool_matrix [] {
    print "🧪 Testing yzx status --json --versions includes the optional tool version matrix..."

    let fixture = (setup_managed_config_fixture
        "yazelix_status_json_versions"
        '[terminal]
terminals = ["ghostty"]
'
    )

    let result = (try {
        let output = (run_public_yzx_command_for_fixture $fixture "yzx status --json --versions")
        let report = ($output.stdout | from json)
        let versions = ($report.versions? | default null)
        let tools = ($versions.tools? | default [])

        if (
            ($output.exit_code == 0)
            and (($report.title? | default "") == "Yazelix status")
            and (($versions.title? | default "") == "Yazelix Tool Versions")
            and (($tools | where tool == "nix") | length) == 1
            and ((($tools | where tool == "nix" | get -o 0.runtime | default "") | into string | str trim) | is-not-empty)
        ) {
            print "  ✅ yzx status --json --versions now includes the optional versions payload"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx status must surface Rust-owned materialization classification, not only config-state.needs_refresh.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_status_json_reports_materialization_repair_when_artifacts_missing [] {
    print "🧪 Testing yzx status --json reports Rust materialization repair when managed artifacts are missing..."

    let fixture = (setup_managed_config_fixture
        "yazelix_status_missing_managed_artifacts"
        '[shell]
default_shell = "nu"

[terminal]
terminals = ["ghostty"]
'
    )

    let result = (try {
        let base_env = {
            HOME: $fixture.tmp_home
            XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
            XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
            YAZELIX_STATE_DIR: ($fixture.tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
        }
        with-env $base_env {
            let st = (compute_config_state)
            record_materialized_state $st
        }

        let output = (run_public_yzx_command_for_fixture $fixture "yzx status --json")
        let report = ($output.stdout | from json)
        let summary = ($report.summary? | default {})

        if (
            ($output.exit_code == 0)
            and ($summary.generated_state_repair_needed == true)
            and (($summary.generated_state_materialization_status? | default "") == "repair_missing_artifacts")
            and (($summary.generated_state_materialization_reason? | default "") | str contains "generated runtime artifacts missing")
        ) {
            print "  ✅ yzx status now reflects Rust runtime-materialization.plan when hashes are current but generated files are absent"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) summary=(($summary | to json -r)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx menu should derive its catalog from Rust-owned public command metadata instead of a handwritten list or Nushell scope probe.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_menu_catalog_tracks_live_exported_command_surface [] {
    print "🧪 Testing yzx menu catalog tracks Rust-owned command metadata..."

    let repo_root = (repo_path)
    let menu_script = (repo_path "nushell" "scripts" "yzx" "menu.nu")

    let result = (try {
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_YZX_CORE_BIN: (resolve_test_yzx_core_bin)
        } {
            ^nu --config /dev/null --env-config /dev/null -c $"source \"($menu_script)\"; { entries: \(get_palette_command_entries | select id category\), post_prompt: \(popup_post_action_prompt\), esc_decision: \(popup_post_action_key_decision esc\), enter_decision: \(popup_post_action_key_decision enter\), backspace_decision: \(popup_post_action_key_decision backspace\) } | to json -r" | complete
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
            and ("yzx update" in $ids)
            and ("yzx update upstream" in $ids)
            and ("yzx update home_manager" in $ids)
            and ("yzx update nix" in $ids)
            and not ("yzx env" in $ids)
            and not ("yzx run" in $ids)
            and not ("yzx cwd" in $ids)
            and not ("yzx dev sync_issues" in $ids)
            and (($entries | where id == "yzx" | get -o 0.category | default "") == "help")
            and (($entries | where id == "yzx launch" | get -o 0.category | default "") == "session")
            and (($entries | where id == "yzx screen" | get -o 0.category | default "") == "workspace")
            and (($entries | where id == "yzx status" | get -o 0.category | default "") == "system")
            and (($entries | where id == "yzx update" | get -o 0.category | default "") == "system")
            and ($contract.post_prompt == "Backspace: return to menu | Enter: close")
            and ($contract.esc_decision == "continue")
            and ($contract.enter_decision == "close")
            and ($contract.backspace_decision == "menu")
        ) {
            print "  ✅ yzx menu now derives its catalog from Rust-owned command metadata, applies explicit exclusions, and leaves Escape out of its own close path"
            true
        } else {
            print $"  ❌ Unexpected menu catalog or key contract result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    $result
}

# Regression: yzx menu actions must dispatch through the public launcher so Rust-owned leaves like `yzx update` stay executable.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_menu_dispatches_catalog_actions_through_launcher [] {
    print "🧪 Testing yzx menu dispatches catalog actions through the public launcher..."

    let repo_root = (repo_path)
    let menu_script = (repo_path "nushell" "scripts" "yzx" "menu.nu")

    let result = (try {
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_YZX_CORE_BIN: (resolve_test_yzx_core_bin)
            YAZELIX_YZX_BIN: (resolve_test_yzx_bin)
            YAZELIX_YZX_CONTROL_BIN: (resolve_test_yzx_control_bin)
        } {
            ^nu --config /dev/null --env-config /dev/null -c $"source \"($menu_script)\"; run_menu_action \"yzx update\"" | complete
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Available update commands:")
            and ($stdout | str contains "yzx update upstream")
        ) {
            print "  ✅ yzx menu now dispatches Rust-owned catalog actions through the public launcher"
            true
        } else {
            print $"  ❌ Unexpected menu action dispatch result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

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
        (test_yzx_desktop_install_refuses_dangling_home_manager_config)
        (test_yzx_desktop_uninstall_preserves_home_manager_cleanup_path)
        (test_yzx_desktop_uninstall_removes_manual_entry_and_icons)
        (test_public_yzx_home_manager_lists_takeover_helpers)
        (test_public_yzx_why_prints_elevator_pitch)
        (test_public_yzx_sponsor_falls_back_to_printed_url_without_openers)
        (test_public_yzx_keys_root_preserves_discoverability_sections)
        (test_public_yzx_keys_aliases_preserve_views)
        (test_public_yzx_keys_tool_specific_views_keep_guidance)
        (test_yzx_home_manager_prepare_preview_reports_manual_takeover_artifacts)
        (test_yzx_home_manager_prepare_apply_archives_manual_takeover_artifacts)
        (test_public_yzx_config_prints_resolved_path)
        (test_public_yzx_config_bootstraps_missing_user_config)
        (test_public_yzx_config_reset_writes_backup_and_restores_default)
        (test_public_yzx_config_reset_without_backup_replaces_config)
        (test_yzx_update_upstream_upgrades_matching_profile_entry)
        (test_yzx_update_upstream_fails_early_for_home_manager_owned_install)
        (test_yzx_update_upstream_fails_without_matching_profile_entry)
        (test_yzx_update_home_manager_updates_input_and_prints_manual_switch_step)
        (test_stale_store_pinned_yzx_invocation_redirects_to_profile_wrapper)
        (test_public_yzx_root_routes_rust_control_family_without_direct_nu_route_modules)
        (test_yzx_run_passes_dash_prefixed_args_through_unchanged)
        (test_yzx_run_treats_child_verbose_flag_as_child_argv)
        (test_yzx_edit_targets_print_paths)
        (test_invalid_config_is_classified_as_config_problem)
        (test_yzx_status_reports_basic_runtime_summary)
        (test_yzx_status_json_reports_typed_summary)
        (test_yzx_status_json_reports_materialization_repair_when_artifacts_missing)
        (test_yzx_menu_catalog_tracks_live_exported_command_surface)
        (test_yzx_menu_dispatches_catalog_actions_through_launcher)
        (test_yzx_exported_commands_have_help_descriptions)
    ]
}
