def fail [message: string] {
    print --stderr $"profile environment frontdoor test: ($message)"
    exit 1
}

def expect [condition: bool, message: string] {
    if not $condition { fail $message }
}

def render [source: path, destination: path, values: record] {
    mut rendered = (open --raw $source)
    for key in ($values | columns) {
        $rendered = ($rendered | str replace --all $"@($key)@" ($values | get $key))
    }
    $rendered | save --raw --force $destination
}

def executable-script [path: path, nu_bin: path, body: string, chmod_bin: path] {
    $"#!($nu_bin)(char newline)($body)" | save --raw --force $path
    ^$chmod_bin 0755 $path
}

def mode [path: path] {
    ls -la ($path | path dirname)
    | where {|entry| $entry.name == ($path | into string) }
    | get 0.mode
}

def main [
    root: path
    source: path
    nu_bin: path
    chmod_bin: path
] {
    let real_home = ($root | path join "home")
    let data_home = ($root | path join "meta" "var" "lib")
    let state_home = ($root | path join "meta" "var" "state")
    let cache_home = ($root | path join "run" "cache")
    let runtime_dir = ($root | path join "run")
    let yazelix_state = ($root | path join "run" "profile-runtime" "yazelix")
    let payload = ($root | path join "payload")
    let wrapper = ($root | path join "frontdoor")
    mkdir $real_home

    executable-script $payload $nu_bin 'def --wrapped main [...args] {
  {
    xdg_data_home: ($env.XDG_DATA_HOME? | default "")
    xdg_state_home: ($env.XDG_STATE_HOME? | default "")
    xdg_cache_home: ($env.XDG_CACHE_HOME? | default "")
    xdg_runtime_dir: ($env.XDG_RUNTIME_DIR? | default "")
    yazelix_state_dir: ($env.YAZELIX_STATE_DIR? | default "")
    shell: ($env.SHELL? | default "")
    args: $args
  } | to json
}
' $chmod_bin

    render $source $wrapper {
        tool: "fixture"
        payload: ($payload | into string)
        realHome: ($real_home | into string)
        dataHome: ($data_home | into string)
        stateHome: ($state_home | into string)
        cacheHome: ($cache_home | into string)
        runtimeDir: ($runtime_dir | into string)
        yazelixStateDir: ($yazelix_state | into string)
        profileNu: ($nu_bin | into string)
        chmod: ($chmod_bin | into string)
    }

    let activated = (with-env {
        HOME: ($real_home | into string)
        XDG_DATA_HOME: ($root | path join "competing-data" | into string)
        XDG_STATE_HOME: ($root | path join "competing-state" | into string)
        XDG_CACHE_HOME: ($root | path join "competing-cache" | into string)
        XDG_RUNTIME_DIR: ($root | path join "competing-runtime" | into string)
        YAZELIX_STATE_DIR: ($root | path join "competing-yazelix" | into string)
        SHELL: "/competing/shell"
    } {
        do { ^$nu_bin $wrapper "proof" "argument" } | complete
    })
    expect ($activated.exit_code == 0) $"profile environment activation failed: ($activated.stderr)"
    let report = ($activated.stdout | from json)
    expect ($report.xdg_data_home == ($data_home | into string)) "XDG data escaped the profile-owned Meta root"
    expect ($report.xdg_state_home == ($state_home | into string)) "XDG state escaped the profile-owned Meta root"
    expect ($report.xdg_cache_home == ($cache_home | into string)) "XDG cache escaped the volatile runtime"
    expect ($report.xdg_runtime_dir == ($runtime_dir | into string)) "XDG runtime directory drifted"
    expect ($report.yazelix_state_dir == ($yazelix_state | into string)) "Yazelix state escaped the volatile runtime"
    expect ($report.shell == ($nu_bin | into string)) "profile Nushell was not exported as SHELL"
    expect ($report.args == ["proof" "argument"]) "frontdoor did not preserve arguments"
    expect (($data_home | path type) == "dir") "profile data root was not created"
    expect (($state_home | path type) == "dir") "profile state root was not created"
    expect ((mode $cache_home) == "rwx------") "profile cache root is not mode 0700"
    expect ((mode $yazelix_state) == "rwx------") "Yazelix runtime root is not mode 0700"

    let fixture_home = ($root | path join "fixture-home")
    mkdir $fixture_home
    let preserved = (with-env {
        HOME: ($fixture_home | into string)
        XDG_DATA_HOME: "fixture-data"
        XDG_STATE_HOME: "fixture-state"
        XDG_CACHE_HOME: "fixture-cache"
        XDG_RUNTIME_DIR: "fixture-runtime"
        YAZELIX_STATE_DIR: "fixture-yazelix"
        SHELL: "fixture-shell"
    } {
        do { ^$nu_bin $wrapper } | complete
    })
    expect ($preserved.exit_code == 0) $"fixture environment execution failed: ($preserved.stderr)"
    let fixture = ($preserved.stdout | from json)
    expect ($fixture.xdg_data_home == "fixture-data") "non-product fixture XDG data was overwritten"
    expect ($fixture.xdg_state_home == "fixture-state") "non-product fixture XDG state was overwritten"
    expect ($fixture.xdg_cache_home == "fixture-cache") "non-product fixture cache was overwritten"
    expect ($fixture.yazelix_state_dir == "fixture-yazelix") "non-product fixture Yazelix state was overwritten"
    expect ($fixture.shell == "fixture-shell") "non-product fixture shell was overwritten"

    print "ok profile environment frontdoor: Meta data/state, volatile runtime, fixture isolation"
}
