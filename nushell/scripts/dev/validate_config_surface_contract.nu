#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)

export def main [] {
    let result = (do {
        cd $REPO_ROOT
        ^nix develop -c cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- --repo-root $REPO_ROOT validate-config-surface-contract
    } | complete)

    if ($result.stdout | is-not-empty) {
        print --raw $result.stdout
    }
    if ($result.stderr | is-not-empty) {
        print --stderr --raw $result.stderr
    }

    if $result.exit_code != 0 {
        error make {
            msg: "main config surface, Home Manager desktop entry, and generated-state contract validation failed"
        }
    }
}
