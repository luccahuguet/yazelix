#!/usr/bin/env nu

use ../core/yazelix.nu *
use ./test_yzx_helpers.nu [CLEAN_ZELLIJ_ENV_PREFIX get_repo_config_dir repo_path]
use ../utils/doctor.nu [build_zellij_plugin_health_results]

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

    let repo_root = (get_repo_config_dir)
    let tmp_home = (mktemp -d | str trim)
    let temp_yazelix_dir = ($tmp_home | path join ".config" "yazelix")

    mkdir $temp_yazelix_dir

    let result = (try {
        ^ln -s ($repo_root | path join "nushell") ($temp_yazelix_dir | path join "nushell")
        cp ($repo_root | path join "yazelix_default.toml") ($temp_yazelix_dir | path join "yazelix_default.toml")

        let stale_config = (
            open ($repo_root | path join "yazelix_default.toml")
            | upsert core.stale_field true
            | upsert packs.declarations.custom_pack ["hello"]
            | upsert packs.enabled ["custom_pack"]
        )
        $stale_config | to toml | save ($temp_yazelix_dir | path join "yazelix.toml")

        let temp_yzx_script = ($temp_yazelix_dir | path join "nushell" "scripts" "core" "yazelix.nu")
        let output = with-env { HOME: $tmp_home, YAZELIX_DIR: $temp_yazelix_dir } {
            ^nu -c $"use \"($temp_yzx_script)\" *; yzx doctor --verbose" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Stale or invalid yazelix.toml fields detected")
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

    rm -rf $tmp_home
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

export def run_doctor_tests [] {
    [
        (test_yzx_doctor_reports_zellij_plugin_context)
        (test_yzx_doctor_warns_on_stale_config_fields)
        (test_doctor_clarifies_shell_opened_editors_are_not_managed)
    ]
}
