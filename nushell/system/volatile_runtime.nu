#!/usr/bin/env nu

const VOLATILE_ROOT = "/run/user/1001/yazelix/volatile"
const KACHE_ROOT = "/home/flexnetos/.cache/kache"
const VOLATILE_DIRS = [
    "/run/user/1001/yazelix/volatile/cache"
    "/run/user/1001/yazelix/volatile/tmp"
    "/run/user/1001/yazelix/volatile/cargo-home"
    "/run/user/1001/yazelix/volatile/cargo-target"
    "/run/user/1001/yazelix/volatile/rustup-home"
]

def ensure [] {
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
}

def main [command: string = "check"] {
    match $command {
        "ensure" => { ensure; check }
        "check" => { check }
        _ => { error make {msg: $"unknown volatile runtime command: ($command)"} }
    }
}
