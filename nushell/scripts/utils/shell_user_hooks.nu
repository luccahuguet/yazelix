#!/usr/bin/env nu

use runtime_paths.nu [get_yazelix_user_config_dir get_yazelix_state_dir]
use atomic_writes.nu write_text_atomic

export const USER_SHELL_HOOK_FILENAMES = {
    bash: "bash.sh"
    nushell: "nu.nu"
    fish: "fish.fish"
    zsh: "zsh.zsh"
}

export def get_yazelix_shell_user_hook_path [shell: string, config_root?: string] {
    let filename = ($USER_SHELL_HOOK_FILENAMES | get -o $shell)
    if $filename == null {
        error make {msg: $"Unsupported managed shell hook surface: ($shell)"}
    }

    let user_config_dir = (get_yazelix_user_config_dir $config_root)
    ($user_config_dir | path join "shells" $filename)
}

def format_nushell_source_literal [path: string] {
    let escaped = ($path | str replace -a "\\" "\\\\" | str replace -a "\"" "\\\"")
    $"source \"($escaped)\""
}

export def sync_generated_nushell_user_hook_bridge [config_root?: string, state_root?: string] {
    let state_dir = if $state_root == null {
        get_yazelix_state_dir
    } else {
        $state_root | path expand
    }
    let bridge_path = ($state_dir | path join "initializers" "nushell" "yazelix_user_hook.nu")
    let user_hook_path = (get_yazelix_shell_user_hook_path "nushell" $config_root)

    if ($user_hook_path | path exists) and ((open --raw $user_hook_path | str trim | is-not-empty)) {
        write_text_atomic $bridge_path (format_nushell_source_literal $user_hook_path) --raw | ignore
    } else {
        write_text_atomic $bridge_path "# Yazelix managed Nushell user hook bridge (empty)" --raw | ignore
    }

    $bridge_path
}
