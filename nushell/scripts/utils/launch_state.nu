#!/usr/bin/env nu
# Persisted launch state for fast Yazelix restarts outside devenv shell.

def get_state_dir [] {
    "~/.local/share/yazelix/state" | path expand
}

export def get_launch_state_path [] {
    (get_state_dir | path join "launch_state.json")
}

def ensure_state_dir [] {
    let state_dir = get_state_dir
    if not ($state_dir | path exists) {
        mkdir $state_dir
    }
}

def resolve_profile_candidate [candidate: string] {
    if ($candidate | is-empty) or (not ($candidate | path exists)) {
        return ""
    }

    try {
        let resolved = (^readlink -f $candidate | str trim)
        if ($resolved | is-not-empty) and ($resolved | path exists) {
            $resolved
        } else {
            ""
        }
    } catch {
        ""
    }
}

export def resolve_built_profile [] {
    let env_profile = ($env.DEVENV_PROFILE? | default "")
    if ($env_profile | is-not-empty) and ($env_profile | path exists) {
        return $env_profile
    }

    let yazelix_dir = "~/.config/yazelix" | path expand
    let candidates = [
        ($yazelix_dir | path join ".devenv/profile")
        ($yazelix_dir | path join ".devenv/gc/shell")
    ]

    for candidate in $candidates {
        let resolved = (resolve_profile_candidate $candidate)
        if ($resolved | is-not-empty) {
            return $resolved
        }
    }

    ""
}

def collect_runtime_env [] {
    let env_keys = [
        "DEVENV_PROFILE"
        "YAZELIX_DIR"
        "IN_YAZELIX_SHELL"
        "IN_NIX_SHELL"
        "NIX_CONFIG"
        "YAZELIX_DEBUG_MODE"
        "YAZELIX_BUILD_CORES"
        "ZELLIJ_DEFAULT_LAYOUT"
        "YAZELIX_DEFAULT_SHELL"
        "YAZELIX_ENABLE_SIDEBAR"
        "YAZI_CONFIG_HOME"
        "YAZELIX_HELIX_MODE"
        "YAZELIX_PREFERRED_TERMINAL"
        "YAZELIX_TERMINAL_CONFIG_MODE"
        "YAZELIX_ASCII_ART_MODE"
        "YAZELIX_ZJSTATUS_WASM"
        "EDITOR"
        "HELIX_RUNTIME"
    ]

    mut runtime_env = {}
    for key in $env_keys {
        let value = ($env | get -o $key)
        if $value != null {
            $runtime_env = ($runtime_env | upsert $key ($value | into string))
        }
    }

    $runtime_env
}

export def read_launch_state [] {
    let state_path = get_launch_state_path
    if not ($state_path | path exists) {
        return null
    }

    try {
        open --raw $state_path | from json
    } catch {
        null
    }
}

export def get_matching_launch_state [
    config_state: record
    profile_override?: string
] {
    let state = read_launch_state
    if $state == null {
        return null
    }

    if not (($state | describe) | str starts-with "record") {
        return null
    }

    let combined_hash = ($state | get -o combined_hash | default "")
    if $combined_hash != ($config_state.combined_hash? | default "") {
        return null
    }

    let profile_path = ($state | get -o profile_path | default "")
    if ($profile_path | is-empty) or (not ($profile_path | path exists)) {
        return null
    }

    if ($profile_override | is-not-empty) and ($profile_override != $profile_path) {
        return null
    }

    let runtime_env = ($state | get -o runtime_env | default null)
    if not (($runtime_env | describe) | str starts-with "record") {
        return null
    }

    {
        combined_hash: $combined_hash
        profile_path: $profile_path
        runtime_env: $runtime_env
    }
}

export def write_launch_state_from_env [config_state: record] {
    let profile_path = resolve_built_profile
    if ($profile_path | is-empty) or (not ($profile_path | path exists)) {
        return
    }

    let runtime_env = (
        collect_runtime_env
        | upsert DEVENV_PROFILE $profile_path
        | upsert IN_YAZELIX_SHELL "true"
        | upsert IN_NIX_SHELL (($env.IN_NIX_SHELL? | default "impure"))
    )

    ensure_state_dir

    let state = {
        combined_hash: ($config_state.combined_hash? | default "")
        profile_path: $profile_path
        runtime_env: $runtime_env
    }

    let state_path = get_launch_state_path
    let temp_path = $"($state_path).tmp"
    $state | to json | save --force $temp_path
    mv --force $temp_path $state_path
}

export def --env activate_launch_state [state: record] {
    let runtime_env = ($state.runtime_env? | default {})
    load-env $runtime_env

    let profile_path = ($state.profile_path? | default "")
    if ($profile_path | is-not-empty) and ($profile_path | path exists) {
        $env.DEVENV_PROFILE = $profile_path
        let profile_bin = ($profile_path | path join "bin")
        if ($profile_bin | path exists) {
            $env.PATH = ([$profile_bin] | append $env.PATH | uniq)
        }
    }

    $env.IN_YAZELIX_SHELL = "true"
    if ($env.IN_NIX_SHELL? | is-empty) {
        $env.IN_NIX_SHELL = "impure"
    }
}
