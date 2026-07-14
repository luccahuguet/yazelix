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
const VOLATILE_ROUTES = [
    { link: "/home/flexnetos/.config/google-chrome/Default/Service Worker", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/service-worker" }
    { link: "/home/flexnetos/.config/google-chrome/Default/Shared Dictionary", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/shared-dictionary" }
    { link: "/home/flexnetos/.config/google-chrome/component_crx_cache", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/component-crx" }
    { link: "/home/flexnetos/.config/google-chrome/extensions_crx_cache", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/extensions-crx" }
    { link: "/home/flexnetos/.config/google-chrome/GrShaderCache", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/gr-shader" }
    { link: "/home/flexnetos/.config/google-chrome/GPUPersistentCache", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/gpu-persistent" }
    { link: "/home/flexnetos/.config/google-chrome/ShaderCache", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/shader" }
    { link: "/home/flexnetos/.config/google-chrome/Default/GPUCache", target: "/run/user/1001/yazelix/volatile/cache/google-chrome/default-gpu" }
    { link: "/home/flexnetos/.config/google-chrome/Crash Reports", target: "/run/user/1001/yazelix/volatile/tmp/google-chrome-crash-reports" }
    { link: "/home/flexnetos/.config/google-chrome/BrowserMetrics", target: "/run/user/1001/yazelix/volatile/tmp/google-chrome-browser-metrics" }
    { link: "/home/flexnetos/.config/google-chrome/DeferredBrowserMetrics", target: "/run/user/1001/yazelix/volatile/tmp/google-chrome-deferred-metrics" }
    { link: "/home/flexnetos/.config/Code/Cache", target: "/run/user/1001/yazelix/volatile/cache/code/cache" }
    { link: "/home/flexnetos/.config/Code/CachedData", target: "/run/user/1001/yazelix/volatile/cache/code/cached-data" }
    { link: "/home/flexnetos/.config/Code/CachedConfigurations", target: "/run/user/1001/yazelix/volatile/cache/code/cached-configurations" }
    { link: "/home/flexnetos/.config/Code/CachedProfilesData", target: "/run/user/1001/yazelix/volatile/cache/code/cached-profiles" }
    { link: "/home/flexnetos/.config/Code/Code Cache", target: "/run/user/1001/yazelix/volatile/cache/code/code-cache" }
    { link: "/home/flexnetos/.config/Code/GPUCache", target: "/run/user/1001/yazelix/volatile/cache/code/gpu" }
    { link: "/home/flexnetos/.config/Code/DawnGraphiteCache", target: "/run/user/1001/yazelix/volatile/cache/code/dawn-graphite" }
    { link: "/home/flexnetos/.config/Code/DawnWebGPUCache", target: "/run/user/1001/yazelix/volatile/cache/code/dawn-webgpu" }
    { link: "/home/flexnetos/.config/Code/Shared Dictionary", target: "/run/user/1001/yazelix/volatile/cache/code/shared-dictionary" }
    { link: "/home/flexnetos/.config/Code/logs", target: "/run/user/1001/yazelix/volatile/tmp/code-logs" }
    { link: "/home/flexnetos/.codex/cache", target: "/run/user/1001/yazelix/volatile/cache/codex" }
    { link: "/home/flexnetos/.codex/tmp", target: "/run/user/1001/yazelix/volatile/tmp/codex" }
    { link: "/home/flexnetos/.local/share/yazelix/logs", target: "/run/user/1001/yazelix/volatile/tmp/yazelix-logs" }
    { link: "/home/flexnetos/.local/share/rtk/tee", target: "/run/user/1001/yazelix/volatile/tmp/rtk-tee" }
    { link: "/home/flexnetos/.local/share/ai.lifeos.desktop/WebKitCache", target: "/run/user/1001/yazelix/volatile/cache/lifeos-webkit" }
    { link: "/home/flexnetos/.local/share/ai.lifeos.desktop/CacheStorage", target: "/run/user/1001/yazelix/volatile/cache/lifeos-cache-storage" }
]

def route_volatile [route: record] {
    let link_type = ($route.link | path type)
    if ($link_type != "") {
        rm --recursive --force $route.link
    }
    if ($route.target | path exists) {
        rm --recursive --force $route.target
    }
    mkdir ($route.link | path dirname)
    mkdir $route.target
    ^/home/flexnetos/.nix-profile/bin/ln --symbolic --no-target-directory $route.target $route.link
}

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
    for route in $VOLATILE_ROUTES {
        route_volatile $route
    }
    mkdir $KACHE_ROOT
}

def check [] {
    for path in $VOLATILE_DIRS {
        if not ($path | path exists) {
            error make {msg: $"volatile runtime directory is missing: ($path)"}
        }
    }
    for route in $VOLATILE_ROUTES {
        if (($route.link | path type) != "symlink") {
            error make {msg: $"persistent application cache/log path is not a volatile link: ($route.link)"}
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
