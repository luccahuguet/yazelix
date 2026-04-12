#!/usr/bin/env nu

use install_ownership.nu has_home_manager_managed_install
use shell_config_generation.nu get_yzx_cli_path

export def get_manual_yzx_wrapper_path [] {
    ($env.HOME | path join ".local" "bin" "yzx")
}

export def get_home_manager_yzx_profile_paths [] {
    mut candidates = [
        ($env.HOME | path join ".nix-profile" "bin" "yzx")
    ]

    if ("USER" in $env) and (($env.USER | default "" | into string | str trim) | is-not-empty) {
        $candidates = ($candidates | append ("/etc/profiles/per-user" | path join $env.USER "bin" "yzx"))
    }

    $candidates | uniq
}

def path_is_symlink [target: string] {
    let result = (^bash -lc $"test -L ($target | into string | to json -r)" | complete)
    $result.exit_code == 0
}

export def get_existing_home_manager_yzx_profile_path [] {
    (
        get_home_manager_yzx_profile_paths
        | where {|path| ($path | path exists) or (path_is_symlink $path) }
        | get -o 0
    )
}

export def resolve_stable_yzx_wrapper_path [] {
    let manual_wrapper = (get_manual_yzx_wrapper_path)
    let home_manager_wrapper = (get_existing_home_manager_yzx_profile_path)

    if (has_home_manager_managed_install) and ($home_manager_wrapper != null) {
        return $home_manager_wrapper
    }

    if ($manual_wrapper | path exists) {
        return $manual_wrapper
    }

    $home_manager_wrapper
}

export def resolve_desktop_launcher_path [runtime_dir: string] {
    let stable_wrapper = (resolve_stable_yzx_wrapper_path)
    if $stable_wrapper != null {
        $stable_wrapper
    } else {
        get_yzx_cli_path $runtime_dir
    }
}
