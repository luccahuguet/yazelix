#!/usr/bin/env nu

use ../utils/runtime_paths.nu get_yazelix_runtime_dir

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

def is_valid_repo_root [candidate?: string] {
    if $candidate == null {
        return false
    }

    let candidate_path = (resolve_existing_path $candidate)
    if $candidate_path == null {
        return false
    }

    let git_marker = ($candidate_path | path join ".git")
    let flake_nix = ($candidate_path | path join "flake.nix")
    let default_config = ($candidate_path | path join "yazelix_default.toml")

    ($git_marker | path exists) and ($flake_nix | path exists) and ($default_config | path exists)
}

def resolve_git_repo_root_from_pwd [] {
    let pwd = ($env.PWD? | default "" | into string | str trim)
    if ($pwd | is-empty) {
        return null
    }

    try {
        let result = (^git -C $pwd rev-parse --show-toplevel | complete)
        if $result.exit_code != 0 {
            return null
        }

        let candidate = ($result.stdout | str trim)
        if (is_valid_repo_root $candidate) {
            resolve_existing_path $candidate
        } else {
            null
        }
    } catch {
        null
    }
}

def get_yazelix_repo_root [] {
    let pwd_repo_root = (resolve_git_repo_root_from_pwd)
    if $pwd_repo_root != null {
        return $pwd_repo_root
    }

    let inferred_runtime = (get_yazelix_runtime_dir)
    if (is_valid_repo_root $inferred_runtime) {
        return $inferred_runtime
    }

    null
}

export def require_yazelix_repo_root [] {
    let repo_root = (get_yazelix_repo_root)
    if $repo_root == null {
        error make {msg: "This maintainer command requires a writable Yazelix repo checkout. Run it from the repo root or another directory inside the same checkout."}
    }

    $repo_root
}
