#!/usr/bin/env nu

const VOLATILE_ROOT = "/run/user/1001/yazelix/volatile"
const KACHE_ROOT = "/home/flexnetos/.cache/kache"
const LEGACY_KACHE_ROOTS = [
    "/home/flexnetos/meta/.cache/kache"
    "/home/flexnetos/meta/var/cache/kache"
    "/home/flexnetos/meta/src/flexnetos_runner/_work/runner-home-01/.cache/kache"
    "/home/flexnetos/meta/src/flexnetos_runner/_work/runner-home-02/.cache/kache"
    "/home/flexnetos/Downloads/runner/runner-home-01/.cache/kache"
    "/home/flexnetos/Downloads/runner/runner-home-02/.cache/kache"
]
const LEGACY_KACHE_ARTIFACTS = [
    "/home/flexnetos/meta/.toolchains/kache"
    "/home/flexnetos/meta/usr/bin/kache"
    "/home/flexnetos/meta/.config/systemd/user/kache.service"
]
const VOLATILE_DIRS = [
    "/run/user/1001/yazelix/volatile/cache"
    "/run/user/1001/yazelix/volatile/tmp"
    "/run/user/1001/yazelix/volatile/cargo-home"
    "/run/user/1001/yazelix/volatile/cargo-target"
    "/run/user/1001/yazelix/volatile/rustup-home"
    "/run/user/1001/yazelix/volatile/cache/google-chrome"
]

def ensure [] {
    for path in $LEGACY_KACHE_ROOTS {
        if ($path | path exists) {
            rm --recursive --force $path
        }
    }
    for path in $LEGACY_KACHE_ARTIFACTS {
        if ($path | path exists) {
            rm --recursive --force $path
        }
    }

    let cargo_config = "/home/flexnetos/meta/.cargo/config.toml"
    if ($cargo_config | path exists) {
        let begin = "# >>> envctl kache (Epic H TASK-0055) >>>"
        let end = "# <<< envctl kache (Epic H TASK-0055) <<<"
        let filtered = (
            open --raw $cargo_config
            | lines
            | reduce --fold {dropping: false, kept: []} {|line, state|
                if $line == $begin {
                    $state | upsert dropping true
                } else if $line == $end {
                    $state | upsert dropping false
                } else if $state.dropping {
                    $state
                } else {
                    $state | upsert kept ($state.kept | append $line)
                }
            }
            | get kept
        )
        let rendered = ($filtered | str join (char newline) | str trim)
        if ($rendered | is-empty) {
            rm --force $cargo_config
        } else {
            $"($rendered)(char newline)" | save --force $cargo_config
        }
    }
    for path in $VOLATILE_DIRS {
        mkdir $path
    }
    mkdir $KACHE_ROOT
}

def check [] {
    for path in $VOLATILE_DIRS {
        if not ($path | path exists) {
            error make {msg: $"volatile runtime directory is missing: ($path)"}
        }
    }
    if not ($KACHE_ROOT | path exists) {
        error make {msg: $"Kache root is missing: ($KACHE_ROOT)"}
    }
    if ($KACHE_ROOT | str starts-with $VOLATILE_ROOT) {
        error make {msg: "Kache must remain outside the volatile runtime root"}
    }
    for path in $LEGACY_KACHE_ROOTS {
        if ($path | path exists) {
            error make {msg: $"legacy Kache root must not exist: ($path)"}
        }
    }
    for path in $LEGACY_KACHE_ARTIFACTS {
        if ($path | path exists) {
            error make {msg: $"legacy Kache delivery artifact must not exist: ($path)"}
        }
    }
    let cargo_config = "/home/flexnetos/meta/.cargo/config.toml"
    if (($cargo_config | path exists) and ((open --raw $cargo_config) | str contains "envctl kache (Epic H TASK-0055)")) {
        error make {msg: $"legacy Kache Cargo block must not exist: ($cargo_config)"}
    }
}

def main [command: string = "check"] {
    match $command {
        "ensure" => { ensure; check }
        "check" => { check }
        _ => { error make {msg: $"unknown volatile runtime command: ($command)"} }
    }
}
