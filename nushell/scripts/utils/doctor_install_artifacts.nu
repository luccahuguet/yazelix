#!/usr/bin/env nu

use common.nu get_yazelix_runtime_dir
use install_ownership.nu [
    get_manual_desktop_entry_path
    has_home_manager_managed_install
]
use launcher_resolution.nu [
    get_existing_home_manager_yzx_profile_path
    get_home_manager_yzx_profile_paths
]
use shell_config_generation.nu get_yzx_cli_path

def resolve_realpath_or_null [target: string] {
    let result = (^readlink -f $target | complete)
    if $result.exit_code == 0 {
        let resolved = ($result.stdout | str trim)
        if ($resolved | is-empty) { null } else { $resolved }
    } else {
        null
    }
}

def detect_install_owner [] {
    if (has_home_manager_managed_install) {
        "home-manager"
    } else if (
        ((get_existing_home_manager_yzx_profile_path) != null)
        or ((get_home_manager_profile_desktop_entry_path | path exists))
    ) {
        "home-manager"
    } else {
        "manual"
    }
}

def get_home_manager_profile_desktop_entry_path [] {
    ($env.HOME | path join ".nix-profile" "share" "applications" "yazelix.desktop")
}

def get_desktop_entry_exec [desktop_path: string] {
    if not ($desktop_path | path exists) {
        return null
    }

    let entry = (open $desktop_path --raw)
    let marker = (
        $entry
        | lines
        | where {|line| $line | str starts-with "Exec="}
        | get -o 0
    )

    if $marker == null {
        null
    } else {
        (
            $marker
            | str replace 'Exec=' ""
            | str trim
        )
    }
}

def desktop_entry_exec_matches_expected [desktop_exec, expected_execs: list<string>] {
    if $desktop_exec == null {
        false
    } else {
        $expected_execs | any {|expected_exec| $desktop_exec == $expected_exec }
    }
}

def get_expected_desktop_entry_execs [install_owner: string] {
    let launcher_paths = if $install_owner == "home-manager" {
        get_home_manager_yzx_profile_paths
    } else {
        let runtime_dir = (get_yazelix_runtime_dir)
        if $runtime_dir == null {
            []
        } else {
            [ (get_yzx_cli_path $runtime_dir) ]
        }
    }

    (
        $launcher_paths
        | each {|launcher_path|
            [
                $"\"($launcher_path)\" desktop launch"
                $"($launcher_path) desktop launch"
            ]
        }
        | flatten
        | uniq
    )
}

export def check_desktop_entry_freshness [] {
    let install_owner = (detect_install_owner)
    let local_desktop_path = (get_manual_desktop_entry_path)
    let profile_desktop_path = (get_home_manager_profile_desktop_entry_path)
    let local_desktop_exists = ($local_desktop_path | path exists)
    let profile_desktop_exists = ($profile_desktop_path | path exists)
    let desktop_path = if $local_desktop_exists {
        $local_desktop_path
    } else if $profile_desktop_exists {
        $profile_desktop_path
    } else {
        null
    }
    let expected_execs = (get_expected_desktop_entry_execs $install_owner)

    if $desktop_path == null {
        let details = if $install_owner == "home-manager" {
            $"Home Manager-managed desktop entries usually resolve through ($profile_desktop_path). Reapply your Home Manager configuration if it is missing."
        } else {
            "Run `yzx desktop install` if you want application-launcher integration."
        }
        return {
            status: "info"
            message: "Yazelix desktop entry not installed"
            details: $details
            fix_available: false
        }
    }

    let local_desktop_exec = if $local_desktop_exists {
        get_desktop_entry_exec $local_desktop_path
    } else {
        null
    }
    let profile_desktop_exec = if $profile_desktop_exists {
        get_desktop_entry_exec $profile_desktop_path
    } else {
        null
    }

    if (
        $install_owner == "home-manager"
        and $local_desktop_exists
        and $profile_desktop_exists
        and (not (desktop_entry_exec_matches_expected $local_desktop_exec $expected_execs))
        and (desktop_entry_exec_matches_expected $profile_desktop_exec $expected_execs)
    ) {
        return {
            status: "warning"
            message: "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry"
            details: $"Shadowing local entry: ($local_desktop_path)\nLocal Exec: ($local_desktop_exec | default '<missing>')\nHome Manager entry: ($profile_desktop_path)\nProfile Exec: ($profile_desktop_exec)\nRemove the shadowing local entry with `yzx desktop uninstall`, then reapply your Home Manager configuration if the profile desktop entry is missing or stale."
            fix_available: false
        }
    }

    let desktop_exec = if $desktop_path == $local_desktop_path {
        $local_desktop_exec
    } else {
        $profile_desktop_exec
    }
    let repair_hint = if $desktop_path == $profile_desktop_path {
        "Repair by reapplying your Home Manager configuration."
    } else {
        "Repair with `yzx desktop install`."
    }

    if $desktop_exec == null {
        return {
            status: "warning"
            message: "Yazelix desktop entry is invalid"
            details: $"The installed desktop entry has no Exec line. ($repair_hint)"
            fix_available: false
        }
    }

    if not (desktop_entry_exec_matches_expected $desktop_exec $expected_execs) {
        return {
            status: "warning"
            message: "Yazelix desktop entry does not use the expected launcher path"
            details: $"Desktop entry Exec: ($desktop_exec)\nExpected one of: ($expected_execs | str join ', ')\n($repair_hint)"
            fix_available: false
        }
    }

    {
        status: "ok"
        message: "Yazelix desktop entry uses the expected launcher path"
        details: $desktop_path
        fix_available: false
    }
}
