#!/usr/bin/env nu

use common.nu [get_yazelix_state_dir get_yazelix_runtime_dir require_yazelix_runtime_dir]

const RUNTIME_PROJECT_ENTRIES = [
    ".taplo.toml"
    "assets"
    "config_metadata"
    "configs"
    "docs"
    "nushell"
    "rust_plugins"
    "shells"
    "CHANGELOG.md"
    "devenv.lock"
    "devenv.nix"
    "devenv.yaml"
    "yazelix_default.toml"
    "yazelix_packs_default.toml"
]

def resolve_existing_path [candidate?: string] {
    if $candidate == null {
        return null
    }

    let expanded = ($candidate | path expand)
    if not ($expanded | path exists) {
        return null
    }

    try {
        let result = (^readlink -f $expanded | complete)
        if $result.exit_code == 0 {
            let resolved = ($result.stdout | str trim)
            if ($resolved | is-not-empty) and ($resolved | path exists) {
                return $resolved
            }
        }
    } catch {}

    $expanded
}

def get_yazelix_runtime_project_dir [] {
    (get_yazelix_state_dir | path join "runtime" "project")
}

def runtime_project_matches_runtime_root [project_root: string, runtime_root: string] {
    let project_sentinel = ($project_root | path join "devenv.nix")
    let runtime_sentinel = ($runtime_root | path join "devenv.nix")

    if not ($project_sentinel | path exists) or not ($runtime_sentinel | path exists) {
        return false
    }

    let resolved_project_sentinel = (resolve_existing_path $project_sentinel)
    let resolved_runtime_sentinel = (resolve_existing_path $runtime_sentinel)

    if ($resolved_project_sentinel == null) or ($resolved_runtime_sentinel == null) {
        return false
    }

    $resolved_project_sentinel == $resolved_runtime_sentinel
}

export def get_existing_yazelix_runtime_project_dir [] {
    let runtime_root = (get_yazelix_runtime_dir)
    if $runtime_root == null {
        return null
    }

    let project_root = (get_yazelix_runtime_project_dir)
    if (
        ($project_root | path exists)
        and (($project_root | path type) == "dir")
        and (runtime_project_matches_runtime_root $project_root $runtime_root)
    ) {
        $project_root
    } else {
        null
    }
}

export def materialize_yazelix_runtime_project_dir [] {
    let runtime_root = (require_yazelix_runtime_dir)
    let project_root = (get_yazelix_runtime_project_dir)

    mkdir $project_root

    for entry in $RUNTIME_PROJECT_ENTRIES {
        let source = ($runtime_root | path join $entry)
        let target = ($project_root | path join $entry)
        if ($target | path exists) {
            rm -rf $target
        }
        if not ($source | path exists) {
            continue
        }
        ^ln -s $source $target
    }

    $project_root
}
