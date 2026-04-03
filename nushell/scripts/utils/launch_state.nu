#!/usr/bin/env nu
# Profile activation helpers for fast Yazelix launch/restart paths.

use ./common.nu [ensure_yazelix_runtime_project_dir get_yazelix_nix_config get_yazelix_dir get_yazelix_state_dir resolve_yazelix_nu_bin]

def normalize_path_entries [value: any] {
    let described = ($value | describe)

    if ($described | str starts-with "list") {
        $value | each {|entry| $entry | into string }
    } else {
        let text = ($value | into string | str trim)
        if ($text | is-empty) {
            []
        } else {
            $text | split row (char esep)
        }
    }
}

def resolve_profile_candidate [candidate: string] {
    if ($candidate | is-empty) or (not ($candidate | path exists)) {
        return ""
    }

    try {
        let result = (^readlink -f $candidate | complete)
        if $result.exit_code != 0 {
            return ""
        }
        let resolved = ($result.stdout | str trim)
        if ($resolved | is-not-empty) and ($resolved | path exists) {
            $resolved
        } else {
            ""
        }
    } catch {
        ""
    }
}

def get_launch_state_path [] {
    (get_yazelix_state_dir | path join "state" "launch_state.json")
}

def load_launch_state [] {
    let state_path = (get_launch_state_path)
    if not ($state_path | path exists) {
        return null
    }

    try {
        open $state_path
    } catch {
        null
    }
}

export def resolve_built_profile [] {
    let env_profile = ($env.DEVENV_PROFILE? | default "")
    let resolved_env_profile = (resolve_profile_candidate $env_profile)
    if ($resolved_env_profile | is-not-empty) {
        return $resolved_env_profile
    }

    let yazelix_dir = (ensure_yazelix_runtime_project_dir)
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

def resolve_recorded_launch_profile [config_state: record, --allow-stale] {
    let launch_state = (load_launch_state)
    if $launch_state == null {
        return null
    }

    let recorded_profile = (
        $launch_state
        | get -o profile_path
        | default ""
        | into string
    )
    let resolved_profile = (resolve_profile_candidate $recorded_profile)
    if ($resolved_profile | is-empty) {
        return null
    }

    let recorded_hash = (
        $launch_state
        | get -o combined_hash
        | default ""
        | into string
    )
    let current_hash = (
        $config_state
        | get -o combined_hash
        | default ""
        | into string
    )

    if (not $allow_stale) and ($recorded_hash != $current_hash) {
        return null
    }

    $resolved_profile
}

export def has_matching_launch_state [config_state: record, --allow-stale] {
    (resolve_recorded_launch_profile $config_state --allow-stale=$allow_stale) != null
}

export def get_launch_profile [config_state: record, --allow-stale] {
    if (($config_state.needs_refresh? | default false) and (not $allow_stale)) {
        return null
    }

    let profile_path = (resolve_recorded_launch_profile $config_state --allow-stale=$allow_stale)
    if ($profile_path | is-empty) or (not ($profile_path | path exists)) {
        return null
    }

    let yazelix_dir = get_yazelix_dir
    let synced_zjstatus = ($yazelix_dir | path join "configs" "zellij" "plugins" "zjstatus.wasm")
    if not ($synced_zjstatus | path exists) {
        return null
    }

    $profile_path
}

export def record_launch_state [config_state: record, profile_path?: string] {
    let preferred_profile = if $profile_path == null {
        ($env.DEVENV_PROFILE? | default "")
    } else {
        $profile_path
    }
    let resolved_profile = (resolve_profile_candidate $preferred_profile)
    if ($resolved_profile | is-empty) {
        return
    }

    let state_path = (get_launch_state_path)
    let state_dir = ($state_path | path dirname)
    if not ($state_dir | path exists) {
        mkdir $state_dir
    }

    {
        combined_hash: (
            $config_state
            | get -o combined_hash
            | default ""
            | into string
        )
        profile_path: $resolved_profile
    } | to json | save --force $state_path
}

export def require_reused_launch_profile [config_state: record, command_name: string] {
    let profile_path = (get_launch_profile $config_state --allow-stale)
    if $profile_path == null {
        error make {msg: $"No cached Yazelix profile is available for '($command_name)'. Run 'yzx refresh' or rerun without --reuse."}
    }

    $profile_path
}

def resolve_editor_command [config: record, profile_path: string] {
    let configured_editor = ($config.editor_command? | default null)
    if $configured_editor != null {
        let editor_text = ($configured_editor | into string)
        if ($editor_text | is-not-empty) {
            if ($editor_text == "nvim") or ($editor_text == "neovim") {
                let profile_nvim = ($profile_path | path join "bin" "nvim")
                if ($profile_nvim | path exists) {
                    return $profile_nvim
                }
            }
            return $editor_text
        }
    }

    let profile_hx = ($profile_path | path join "bin" "hx")
    if ($profile_hx | path exists) {
        return $profile_hx
    } else {
        return "hx"
    }
}

def is_helix_editor_command [editor: string] {
    let normalized = ($editor | str trim)
    ($normalized | str ends-with "/hx") or ($normalized == "hx") or ($normalized | str ends-with "/helix") or ($normalized == "helix")
}

def is_neovim_editor_command [editor: string] {
    let normalized = ($editor | str trim)
    ($normalized | str ends-with "/nvim") or ($normalized == "nvim") or ($normalized | str ends-with "/neovim") or ($normalized == "neovim")
}

def resolve_helix_runtime [config: record] {
    let configured_runtime = ($config.helix_runtime_path? | default null)
    if $configured_runtime != null {
        let runtime_text = ($configured_runtime | into string)
        if ($runtime_text | is-not-empty) {
            return $runtime_text
        }
    }

    ""
}

export def get_launch_env [config: record, profile_path: string] {
    let yazelix_dir = get_yazelix_dir
    let profile_bin = ($profile_path | path join "bin")
    let current_path_entries = (normalize_path_entries ($env.PATH? | default []))
    let nix_config = get_yazelix_nix_config
    let enable_sidebar = ($config.enable_sidebar? | default true)
    let resolved_editor_command = (resolve_editor_command $config $profile_path)
    let editor_kind = if (is_helix_editor_command $resolved_editor_command) {
        "helix"
    } else if (is_neovim_editor_command $resolved_editor_command) {
        "neovim"
    } else {
        ""
    }
    let editor_command = if $editor_kind == "helix" {
        ($yazelix_dir | path join "shells" "posix" "yazelix_hx.sh")
    } else {
        $resolved_editor_command
    }
    let helix_runtime = (resolve_helix_runtime $config)
    mut launch_env = {
        DEVENV_PROFILE: $profile_path
        PATH: (([$profile_bin] | append $current_path_entries | uniq))
        YAZELIX_RUNTIME_DIR: $yazelix_dir
        YAZELIX_DIR: $yazelix_dir
        YAZELIX_NU_BIN: (resolve_yazelix_nu_bin)
        IN_YAZELIX_SHELL: "true"
        IN_NIX_SHELL: "impure"
        NIX_CONFIG: $nix_config
        ZELLIJ_DEFAULT_LAYOUT: (if $enable_sidebar { "yzx_side" } else { "yzx_no_side" })
        YAZI_CONFIG_HOME: ($env.HOME | path join ".local" "share" "yazelix" "configs" "yazi")
        YAZELIX_TERMINAL_CONFIG_MODE: ($config.terminal_config_mode? | default "yazelix" | into string)
        EDITOR: $editor_command
    }

    if ($editor_kind | is-not-empty) {
        $launch_env = ($launch_env | upsert YAZELIX_MANAGED_EDITOR_KIND $editor_kind)
    }

    if $editor_kind == "helix" {
        $launch_env = ($launch_env | upsert YAZELIX_MANAGED_HELIX_BINARY $resolved_editor_command)
    }

    if ($helix_runtime | is-not-empty) {
        $launch_env = ($launch_env | upsert HELIX_RUNTIME $helix_runtime)
    }

    $launch_env
}

export def --env activate_launch_profile [config: record, profile_path: string] {
    mut launch_env = (get_launch_env $config $profile_path)

    if ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
        $launch_env = ($launch_env | upsert ZELLIJ_DEFAULT_LAYOUT $env.ZELLIJ_DEFAULT_LAYOUT)
    }

    load-env $launch_env
}
