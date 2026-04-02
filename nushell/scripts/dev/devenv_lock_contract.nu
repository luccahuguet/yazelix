#!/usr/bin/env nu

const REPO_ROOT = ((path self) | path dirname | path join ".." ".." ".." | path expand)

export const DEVENV_SKEW_WARNING = "is newer than devenv input"

export def get_locked_devenv_package_root [] {
    let result = (
        do {
            cd $REPO_ROOT
            ^nix eval --raw .#locked_devenv.outPath --extra-experimental-features "nix-command flakes" | complete
        }
    )

    if $result.exit_code != 0 {
        if ($result.stdout | is-not-empty) {
            print $result.stdout
        }
        if ($result.stderr | is-not-empty) {
            print $result.stderr
        }
        error make { msg: "Failed to resolve the lock-derived devenv package path" }
    }

    $result.stdout | str trim
}
