#!/usr/bin/env nu
# Profile activation helpers for fast Yazelix launch/restart paths.

use ./common.nu [get_yazelix_nix_config]

def bool_to_string [value] {
    if $value { "true" } else { "false" }
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

    let env_profile = ($env.DEVENV_PROFILE? | default "")
    if ($env_profile | is-not-empty) and ($env_profile | path exists) {
        return $env_profile
    }

    ""
}

export def get_launch_profile [config_state: record, --allow-stale] {
    if (($config_state.needs_refresh? | default false) and (not $allow_stale)) {
        return null
    }

    let profile_path = resolve_built_profile
    if ($profile_path | is-empty) or (not ($profile_path | path exists)) {
        return null
    }

    let yazelix_dir = "~/.config/yazelix" | path expand
    let synced_zjstatus = ($yazelix_dir | path join "configs" "zellij" "plugins" "zjstatus.wasm")
    if not ($synced_zjstatus | path exists) {
        return null
    }

    $profile_path
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
    let yazelix_dir = ($env.HOME | path join ".config" "yazelix")
    let profile_bin = ($profile_path | path join "bin")
    let nix_config = get_yazelix_nix_config
    let enable_sidebar = ($config.enable_sidebar? | default true)
    let terminals = ($config.terminals? | default ["ghostty"])
    let preferred_terminal = if ($terminals | is-empty) { "unknown" } else { ($terminals | first | into string) }
    let editor_command = (resolve_editor_command $config $profile_path)
    let helix_runtime = (resolve_helix_runtime $config)
    mut launch_env = {
        DEVENV_PROFILE: $profile_path
        PATH: (([$profile_bin] | append $env.PATH | uniq))
        YAZELIX_DIR: $yazelix_dir
        IN_YAZELIX_SHELL: "true"
        IN_NIX_SHELL: "impure"
        NIX_CONFIG: $nix_config
        YAZELIX_DEBUG_MODE: (bool_to_string ($config.debug_mode? | default false))
        ZELLIJ_DEFAULT_LAYOUT: (if $enable_sidebar { "yzx_side" } else { "yzx_no_side" })
        YAZELIX_DEFAULT_SHELL: ($config.default_shell? | default "nu" | into string)
        YAZELIX_ENABLE_SIDEBAR: (bool_to_string $enable_sidebar)
        YAZI_CONFIG_HOME: ($env.HOME | path join ".local" "share" "yazelix" "configs" "yazi")
        YAZELIX_HELIX_MODE: ($config.helix_mode? | default "release" | into string)
        YAZELIX_PREFERRED_TERMINAL: $preferred_terminal
        YAZELIX_TERMINAL_CONFIG_MODE: ($config.terminal_config_mode? | default "yazelix" | into string)
        YAZELIX_ASCII_ART_MODE: ($config.ascii_art_mode? | default "static" | into string)
        EDITOR: $editor_command
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
