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
const VOLATILE_DIRS = [
    "/run/user/1001/yazelix/volatile/cache"
    "/run/user/1001/yazelix/volatile/tmp"
    "/run/user/1001/yazelix/volatile/cargo-home"
    "/run/user/1001/yazelix/volatile/cargo-target"
    "/run/user/1001/yazelix/volatile/rustup-home"
]

def ensure [] {
    for path in $LEGACY_KACHE_ROOTS {
        if ($path | path exists) {
            rm --recursive --force $path
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
}

def main [command: string = "check"] {
    match $command {
        "ensure" => { ensure; check }
        "check" => { check }
        _ => { error make {msg: $"unknown volatile runtime command: ($command)"} }
    }
}
