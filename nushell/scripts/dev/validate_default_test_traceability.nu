#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)

def run_repo_validator [command: string] {
    let result = (do {
        cd $REPO_ROOT
        ^nix develop -c cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- $command
    } | complete)

    if ($result.stdout | is-not-empty) {
        print --raw $result.stdout
    }
    if ($result.stderr | is-not-empty) {
        print --stderr --raw $result.stderr
    }
    if $result.exit_code != 0 {
        error make { msg: "Governed test traceability validation failed" }
    }
}

export def main [] {
    run_repo_validator "validate-default-test-traceability"
}
