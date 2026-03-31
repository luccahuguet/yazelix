#!/usr/bin/env nu
# Shared resolution logic for the preferred standalone devenv CLI.

use common.nu get_yazelix_dir

def get_external_command_path [command_name: string] {
    let matches = (which $command_name | where type == "external")
    if ($matches | is-empty) {
        null
    } else {
        $matches | get path | first
    }
}

def get_nix_profile_entry [entry_name: string] {
    let profile = try {
        let result = (^nix profile list --json | complete)
        if $result.exit_code != 0 {
            return null
        }
        $result.stdout | from json
    } catch {
        null
    }

    if $profile == null {
        return null
    }

    let elements = ($profile | get -o elements)
    if $elements == null {
        return null
    }

    $elements | get -o $entry_name
}

export def resolve_preferred_devenv_path [] {
    let profile_entry = (get_nix_profile_entry "devenv")
    if $profile_entry != null {
        let store_path = ($profile_entry | get -o storePaths.0 | default "")
        if ($store_path | is-not-empty) {
            let candidate = ($store_path | path join "bin" "devenv")
            if ($candidate | path exists) {
                return $candidate
            }
        }
    }

    let path_match = (get_external_command_path "devenv")
    if $path_match != null {
        return $path_match
    }

    error make {msg: "devenv command not found in the active Nix profile or PATH"}
}

export def is_preferred_devenv_available [] {
    try {
        resolve_preferred_devenv_path | ignore
        true
    } catch {
        false
    }
}

export def get_preferred_devenv_version_line [] {
    let devenv_path = (resolve_preferred_devenv_path)
    try {
        ^$devenv_path --version | lines | first
    } catch {
        error make {msg: $"Failed to read version from preferred devenv CLI: ($devenv_path)"}
    }
}

def load_yazelix_lockfile [] {
    let lock_path = ((get_yazelix_dir) | path join "devenv.lock")
    if not ($lock_path | path exists) {
        error make {msg: $"Yazelix lockfile not found: ($lock_path)"}
    }

    try {
        open --raw $lock_path | from json
    } catch {|err|
        error make {msg: $"Failed to read Yazelix lockfile: ($err.msg)"}
    }
}

export def get_pinned_devenv_installable [] {
    let lockfile = (load_yazelix_lockfile)
    let node = ($lockfile | get -o nodes | get -o devenv)
    if $node == null {
        error make {msg: "Yazelix lockfile does not contain a devenv node"}
    }

    let locked = ($node | get -o locked)
    if $locked == null {
        error make {msg: "Yazelix lockfile is missing locked metadata for the devenv node"}
    }

    let owner = ($locked | get -o owner | default "")
    let repo = ($locked | get -o repo | default "")
    let rev = ($locked | get -o rev | default "")

    if ($owner | is-empty) or ($repo | is-empty) or ($rev | is-empty) {
        error make {msg: "Yazelix lockfile is missing owner/repo/rev metadata for the devenv node"}
    }

    $"github:($owner)/($repo)/($rev)#devenv"
}
