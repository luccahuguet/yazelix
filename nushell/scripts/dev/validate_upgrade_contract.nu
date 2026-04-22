#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)

export def main [
    --ci
    --diff-base: string = ""
] {
    mut validator_args = [
        "--repo-root"
        $REPO_ROOT
        "validate-upgrade-contract"
    ]

    if $ci {
        $validator_args = ($validator_args | append "--ci")
    }
    if (($diff_base | str trim) | is-not-empty) {
        $validator_args = ($validator_args | append ["--diff-base" ($diff_base | str trim)])
    }
    let resolved_validator_args = $validator_args

    let result = (do {
        cd $REPO_ROOT
        ^nix develop -c cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- ...$resolved_validator_args
    } | complete)

    if ($result.stdout | is-not-empty) {
        print --raw $result.stdout
    }
    if ($result.stderr | is-not-empty) {
        print --stderr --raw $result.stderr
    }
    if $result.exit_code != 0 {
        error make { msg: "Upgrade contract validation failed" }
    }
}
