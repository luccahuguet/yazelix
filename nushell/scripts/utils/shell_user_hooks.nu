#!/usr/bin/env nu

use common.nu [get_yazelix_user_config_dir get_yazelix_state_dir]

export const USER_SHELL_HOOK_FILENAMES = {
    bash: "bash.sh"
    nushell: "nu.nu"
    fish: "fish.fish"
    zsh: "zsh.zsh"
}

export def get_yazelix_shell_user_hook_dir [config_root?: string] {
    let configured = (
        $env.YAZELIX_USER_SHELL_HOOK_DIR?
        | default ""
        | into string
        | str trim
    )

    if ($configured | is-not-empty) {
        $configured | path expand
    } else {
        let user_config_dir = if $config_root == null {
            get_yazelix_user_config_dir
        } else {
            get_yazelix_user_config_dir $config_root
        }
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

def get_nushell_user_hook_bridge_contents [user_hook_path: string] {
    let escaped = ($user_hook_path | str replace -a "\\" "\\\\" | str replace -a "\"" "\\\"")
    [
        $"let user_hook_path = \"($escaped)\""
        'if ($user_hook_path | path exists) and ((open --raw $user_hook_path | str trim | is-not-empty)) {'
        $"    (format_nushell_source_literal $user_hook_path)"
        "}"
    ] | str join "\n"
}

export def sync_generated_nushell_user_hook_bridge [config_root?: string, state_root?: string] {
    let bridge_path = (get_generated_nushell_user_hook_bridge_path $state_root)
    let user_hook_path = (get_yazelix_shell_user_hook_path "nushell" $config_root)

    mkdir ($bridge_path | path dirname)
    (get_nushell_user_hook_bridge_contents $user_hook_path) | save --force --raw $bridge_path

    $bridge_path
}
