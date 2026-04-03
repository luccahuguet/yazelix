#!/usr/bin/env nu

use common.nu [get_yazelix_user_config_dir get_yazelix_state_dir]

export const USER_SHELL_HOOK_FILENAMES = {
    bash: "bash.sh"
    nushell: "nu.nu"
    fish: "fish.fish"
    zsh: "zsh.zsh"
}

def expand_shell_hook_path [value: string] {
    let trimmed = ($value | str trim)
    if ($trimmed | is-empty) {
        return $trimmed
    }

    let home_dir = ($env.HOME? | default "" | into string | str trim)
    let expanded_home = if ($home_dir | is-not-empty) and ($trimmed | str starts-with "$HOME/") {
        $trimmed | str replace "$HOME" $home_dir
    } else if ($home_dir | is-not-empty) and ($trimmed == "$HOME") {
        $home_dir
    } else {
        $trimmed
    }

    $expanded_home | path expand
}

export def get_yazelix_shell_user_hook_dir [config_root?: string] {
    if $config_root != null {
        return ((get_yazelix_user_config_dir $config_root) | path join "shells")
    }

    let configured = (
        $env.YAZELIX_USER_SHELL_HOOK_DIR?
        | default ""
        | into string
        | str trim
    )

    if ($configured | is-not-empty) {
        expand_shell_hook_path $configured
    } else {
        let user_config_dir = (get_yazelix_user_config_dir)
        ($user_config_dir | path join "shells")
    }
}

export def get_yazelix_shell_user_hook_path [shell: string, config_root?: string] {
    let filename = ($USER_SHELL_HOOK_FILENAMES | get -o $shell)
    if $filename == null {
        error make {msg: $"Unsupported managed shell hook surface: ($shell)"}
    }

    get_yazelix_shell_user_hook_dir $config_root | path join $filename
}

export def get_generated_nushell_user_hook_bridge_path [state_root?: string] {
    let state_dir = if $state_root == null {
        get_yazelix_state_dir
    } else {
        $state_root | path expand
    }
    ($state_dir | path join "initializers" "nushell" "yazelix_user_hook.nu")
}

def format_nushell_source_literal [path: string] {
    let escaped = ($path | str replace -a "\\" "\\\\" | str replace -a "\"" "\\\"")
    $"source \"($escaped)\""
}

export def sync_generated_nushell_user_hook_bridge [config_root?: string, state_root?: string] {
    let bridge_path = (get_generated_nushell_user_hook_bridge_path $state_root)
    let user_hook_path = (get_yazelix_shell_user_hook_path "nushell" $config_root)

    mkdir ($bridge_path | path dirname)

    if ($user_hook_path | path exists) and ((open --raw $user_hook_path | str trim | is-not-empty)) {
        (format_nushell_source_literal $user_hook_path) | save --force --raw $bridge_path
    } else {
        "# Yazelix managed Nushell user hook bridge (empty)" | save --force --raw $bridge_path
    }

    $bridge_path
}
