#!/usr/bin/env nu

use common.nu get_yazelix_runtime_dir
use install_ownership.nu [
    get_manual_desktop_entry_path
    get_manual_yzx_wrapper_path
    has_home_manager_managed_install
    is_legacy_manual_yzx_wrapper_path
]
use launcher_resolution.nu [
    get_existing_home_manager_yzx_profile_path
    get_home_manager_yzx_profile_paths
    resolve_desktop_launcher_path
]

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

def get_shell_resolved_yzx_path [] {
    let invoked = (
        $env.YAZELIX_INVOKED_YZX_PATH?
        | default ""
        | into string
        | str trim
    )

    if ($invoked | is-not-empty) {
        return ($invoked | path expand --no-symlink)
    }

    let resolved = (
        which yzx
        | where type == "external"
        | get -o 0.path
        | default null
    )

    if $resolved == null {
        null
    } else {
        $resolved | path expand --no-symlink
    }
}

def get_stale_store_shadow_context [] {
    let redirected_from = (
        $env.YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH?
        | default ""
        | into string
        | str trim
    )
    let profile_wrapper = (get_existing_home_manager_yzx_profile_path)

    if ($redirected_from | is-empty) or ($profile_wrapper == null) {
        return null
    }

    {
        redirected_from: ($redirected_from | path expand --no-symlink)
        profile_wrapper: ($profile_wrapper | path expand --no-symlink)
    }
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
            [ (resolve_desktop_launcher_path $runtime_dir) ]
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

export def check_shell_yzx_wrapper_shadowing [] {
    let stale_store_shadow = (get_stale_store_shadow_context)
    if $stale_store_shadow != null {
        let details_lines = [
            $"Stale host-shell invocation: ($stale_store_shadow.redirected_from)"
            $"Current profile-owned yzx: ($stale_store_shadow.profile_wrapper)"
            "Yazelix redirected this invocation to the current profile command so the requested action could still run"
            "A stale host-shell function or alias is still shadowing `yzx` in at least one shell startup file"
            "Open a fresh shell after removing the old Yazelix-managed shell block, or bypass host-shell functions with `command yzx` until cleanup is complete"
        ]

        return [{
            status: "warning"
            message: "A stale host-shell yzx function or alias is shadowing the current profile command"
            details: ($details_lines | str join "\n")
            fix_available: false
        }]
    }

    let manual_wrapper = (get_manual_yzx_wrapper_path)
    if not (is_legacy_manual_yzx_wrapper_path $manual_wrapper) {
        return []
    }

    let profile_wrapper = (get_existing_home_manager_yzx_profile_path)
    if $profile_wrapper == null {
        return []
    }

    let shell_resolved = (get_shell_resolved_yzx_path)
    if $shell_resolved == null {
        return []
    }

    let expanded_manual_wrapper = ($manual_wrapper | path expand --no-symlink)
    let expanded_profile_wrapper = ($profile_wrapper | path expand --no-symlink)

    if $shell_resolved != $expanded_manual_wrapper {
        return []
    }

    let details_lines = [
        $"Shell-resolved yzx: ($shell_resolved)"
        $"Legacy local wrapper: ($expanded_manual_wrapper)"
        $"Profile-owned yzx: ($expanded_profile_wrapper)"
        "Choose one clear owner for this install"
        "If you are migrating to Home Manager, run `yzx home_manager prepare --apply`, then rerun `home-manager switch`"
        "If a profile install owns this runtime, remove the stale `~/.local/bin/yzx` wrapper and keep the profile-owned `yzx` command"
    ]

    [{
        status: "warning"
        message: "A stale user-local yzx wrapper shadows the profile-owned Yazelix command"
        details: ($details_lines | str join "\n")
        fix_available: false
    }]
}
