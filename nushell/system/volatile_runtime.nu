#!/usr/bin/env nu

const VOLATILE_ROOT = "/run/user/1001/yazelix/volatile"
const PROFILE_RUNTIME_ROOT = "/run/user/1001/yazelix/profile-runtime"
const KACHE_ROOT = "/home/flexnetos/.cache/kache"
const DURABLE_CACHE_ROOT = "/home/flexnetos/.cache"
# Immutable, expensive-to-refetch artifacts (model weights, browser binaries)
# and the starship log dir live on durable home storage, never the tmpfs that a
# reboot wipes. Same persistence class as KACHE_ROOT.
const DURABLE_DIRS = [
    "/home/flexnetos/.cache/huggingface"
    "/home/flexnetos/.cache/torch"
    "/home/flexnetos/.cache/playwright"
    "/home/flexnetos/.cache/starship"
    "/home/flexnetos/.cache/icm/models"
]
# icm's embedding model persists primarily through HF_HOME (durable above): its
# fastembed backend downloads via the HuggingFace hub, so the ~615 MB Jina model
# lands in ~/.cache/huggingface. As defence-in-depth, also point the volatile
# icm cache dir (fastembed's *declared* cache_dir, $XDG_CACHE_HOME/icm/models per
# fastembed_embedder.rs) at a durable target, so the model survives regardless of
# which path a future fastembed version honours. (Reverse of VOLATILE_ROUTES: a
# volatile link -> durable target.) Best-effort — not a hard activation gate.
const ICM_VOLATILE_CACHE = "/run/user/1001/yazelix/volatile/cache/icm"
const ICM_DURABLE_CACHE = "/home/flexnetos/.cache/icm"
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
    "/run/user/1001/yazelix/profile-runtime"
    "/run/user/1001/yazelix/profile-runtime/yazelix"
    "/run/user/1001/yazelix/profile-runtime/codex"
    "/run/user/1001/yazelix/profile-runtime/claude"
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
    for path in [$PROFILE_RUNTIME_ROOT ($PROFILE_RUNTIME_ROOT | path join "yazelix") ($PROFILE_RUNTIME_ROOT | path join "codex") ($PROFILE_RUNTIME_ROOT | path join "claude")] {
        ^/home/flexnetos/.nix-profile/bin/chmod 0700 $path
    }
    for route in $VOLATILE_ROUTES {
        route_volatile $route
    }
    mkdir $KACHE_ROOT
    mkdir $DURABLE_CACHE_ROOT
    for path in $DURABLE_DIRS {
        mkdir $path
    }
    if (($ICM_VOLATILE_CACHE | path type) == "symlink") {
        rm --force $ICM_VOLATILE_CACHE
    } else if ($ICM_VOLATILE_CACHE | path exists) {
        rm --recursive --force $ICM_VOLATILE_CACHE
    }
    ^/home/flexnetos/.nix-profile/bin/ln --symbolic --no-target-directory $ICM_DURABLE_CACHE $ICM_VOLATILE_CACHE
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
    for path in $DURABLE_DIRS {
        if not ($path | path exists) {
            error make {msg: $"durable cache directory is missing: ($path)"}
        }
        if ($path | str starts-with $VOLATILE_ROOT) {
            error make {msg: $"durable cache must remain outside the volatile runtime root: ($path)"}
        }
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
