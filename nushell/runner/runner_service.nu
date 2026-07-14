const PROFILE_ROOT = "/home/flexnetos/.nix-profile"

def main [instance: string = "01"] {
    if $instance !~ '^[A-Za-z0-9_]+$' {
        error make {msg: $"invalid runner instance ($instance | to nuon)"}
    }
    let runtime_root = $"(($env.XDG_RUNTIME_DIR? | default "/run/user/1001"))/yazelix/runners/($instance)"
    let kache_dir = $"/home/flexnetos/.cache/kache/runners/($instance)"
    if ($runtime_root | path exists) {
        rm --recursive --force $runtime_root
    }
    mkdir $runtime_root
    mkdir $kache_dir
    let runner_env = {
        SHELL: $"($PROFILE_ROOT)/toolbin/nu"
        HOME: $"($runtime_root)/home"
        XDG_CACHE_HOME: $"($runtime_root)/xdg_cache"
        CARGO_HOME: $"($runtime_root)/cargo_home"
        CARGO_TARGET_DIR: $"($runtime_root)/cargo_target"
        RUNNER_HOME: $"($runtime_root)/runner"
        RUNNER_WORK_DIR: $"($runtime_root)/work"
        RUNNER_SERVICE_HOME: $"($runtime_root)/home"
        KACHE_BIN: $"($PROFILE_ROOT)/bin/kache"
        KACHE_CACHE_DIR: $kache_dir
        RUSTC_WRAPPER: $"($PROFILE_ROOT)/bin/kache-rustc-wrapper"
        CARGO_BUILD_RUSTC_WRAPPER: $"($PROFILE_ROOT)/bin/kache-rustc-wrapper"
    }
    let result = with-env $runner_env {
        ^$"($PROFILE_ROOT)/bin/flexnetos_runner_policy" runtime $instance
        let install = (^$"($PROFILE_ROOT)/bin/fxrun-actions" --confirm true --dry-run false install | complete)
        if $install.exit_code != 0 {
            print --stderr $install.stderr
            $install
        } else {
            ^$"($PROFILE_ROOT)/bin/fxrun-actions" --confirm true --dry-run false run-once | complete
        }
    }
    if ($runtime_root | path exists) {
        rm --recursive --force $runtime_root
    }
    if $result.exit_code != 0 {
        print --stderr $result.stderr
    }
    exit $result.exit_code
}
