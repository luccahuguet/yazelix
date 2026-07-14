const PROFILE_ROOT = "/home/flexnetos/.nix-profile"

def fail [message: string] {
    error make {msg: $"flexnetos runner policy: ($message)"}
}

def require_exact_env [name: string, expected: string] {
    let actual = $env | get -o $name | default ""
    if $actual != $expected {
        fail $"($name) must be ($expected), got ($actual | to nuon)"
    }
}

def runtime_check [instance: string] {
    if $instance !~ '^[A-Za-z0-9_]+$' {
        fail $"invalid runner instance ($instance | to nuon)"
    }
    let runtime_root = $"(($env.XDG_RUNTIME_DIR? | default "/run/user/1001"))/yazelix/runners/($instance)"
    require_exact_env "SHELL" $"($PROFILE_ROOT)/toolbin/nu"
    require_exact_env "RUSTC_WRAPPER" $"($PROFILE_ROOT)/bin/kache-rustc-wrapper"
    require_exact_env "CARGO_BUILD_RUSTC_WRAPPER" $"($PROFILE_ROOT)/bin/kache-rustc-wrapper"
    require_exact_env "KACHE_BIN" $"($PROFILE_ROOT)/bin/kache"
    require_exact_env "KACHE_CACHE_DIR" $"/home/flexnetos/.cache/kache/runners/($instance)"
    require_exact_env "HOME" $"($runtime_root)/home"
    require_exact_env "XDG_CACHE_HOME" $"($runtime_root)/xdg_cache"
    require_exact_env "CARGO_HOME" $"($runtime_root)/cargo_home"
    require_exact_env "CARGO_TARGET_DIR" $"($runtime_root)/cargo_target"
    for banned in ["SCCACHE_DIR" "SCCACHE_ENDPOINT" "RUSTC_CACHE"] {
        if (($env | get -o $banned | default "") | is-not-empty) {
            fail $"($banned) enables a non-Kache cache"
        }
    }
    print $"ok runner policy: instance=($instance) cache=kache shell=nu volatile_root=($runtime_root)"
}

def main [mode: string = "runtime", instance: string = "01"] {
    if $mode != "runtime" {
        fail $"unsupported mode ($mode | to nuon)"
    }
    runtime_check $instance
}
