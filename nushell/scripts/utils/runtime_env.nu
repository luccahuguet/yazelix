#!/usr/bin/env nu
# Runtime environment helpers for the trimmed Yazelix entry surface.

use ./common.nu get_yazelix_runtime_dir
use ./config_parser.nu parse_yazelix_config

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

def runtime_owned_path_entries [runtime_dir: string] {
    [
        ($runtime_dir | path join "toolbin")
        ($runtime_dir | path join "bin")
        ($runtime_dir | path join "libexec")
    ]
}

def strip_runtime_owned_path_entries [entries: list<string>, runtime_dir: string] {
    let runtime_owned = (runtime_owned_path_entries $runtime_dir)
    $entries | where {|entry| not ($runtime_owned | any {|owned| $entry == $owned }) }
}

def is_helix_editor_command [editor: string] {
    let normalized = ($editor | str trim)
    ($normalized | str ends-with "/hx") or ($normalized == "hx") or ($normalized | str ends-with "/helix") or ($normalized == "helix")
}

def is_neovim_editor_command [editor: string] {
    let normalized = ($editor | str trim)
    ($normalized | str ends-with "/nvim") or ($normalized == "nvim") or ($normalized | str ends-with "/neovim") or ($normalized == "neovim")
}

def resolve_editor_command [config: record] {
    let configured_editor = ($config.editor_command? | default null)
    if $configured_editor != null {
        let editor_text = ($configured_editor | into string | str trim)
        if ($editor_text | is-not-empty) {
            return $editor_text
        }
    }

    "hx"
}

def resolve_helix_runtime [config: record] {
    let configured_runtime = ($config.helix_runtime_path? | default null)
    if $configured_runtime != null {
        let runtime_text = ($configured_runtime | into string | str trim)
        if ($runtime_text | is-not-empty) {
            return $runtime_text
        }
    }

    ""
}

export def get_runtime_env [config?: record] {
    let resolved_config = if $config == null {
        parse_yazelix_config
    } else {
        $config
    }
    let runtime_dir = (get_yazelix_runtime_dir)
    let runtime_toolbin = ($runtime_dir | path join "toolbin")
    let runtime_bin = ($runtime_dir | path join "bin")
    let normalized_path_entries = (normalize_path_entries ($env.PATH? | default []))
    let current_path_entries = (strip_runtime_owned_path_entries $normalized_path_entries $runtime_dir)
    let runtime_path_entries = (
        [$runtime_toolbin, $runtime_bin]
        | where {|entry| $entry | path exists }
    )
    let path_entries = if ($runtime_path_entries | is-not-empty) {
        $runtime_path_entries | append $current_path_entries | uniq
    } else {
        $current_path_entries
    }
    let enable_sidebar = ($resolved_config.enable_sidebar? | default true)
    let resolved_editor_command = (resolve_editor_command $resolved_config)
    let editor_kind = if (is_helix_editor_command $resolved_editor_command) {
        "helix"
    } else if (is_neovim_editor_command $resolved_editor_command) {
        "neovim"
    } else {
        ""
    }
    let editor_command = if $editor_kind == "helix" {
        ($runtime_dir | path join "shells" "posix" "yazelix_hx.sh")
    } else {
        $resolved_editor_command
    }
    let helix_runtime = (resolve_helix_runtime $resolved_config)
    mut runtime_env = {
        PATH: $path_entries
        YAZELIX_RUNTIME_DIR: $runtime_dir
        IN_YAZELIX_SHELL: "true"
        ZELLIJ_DEFAULT_LAYOUT: (if $enable_sidebar { "yzx_side" } else { "yzx_no_side" })
        YAZI_CONFIG_HOME: ($env.HOME | path join ".local" "share" "yazelix" "configs" "yazi")
        EDITOR: $editor_command
        VISUAL: $editor_command
    }

    if $editor_kind == "helix" {
        $runtime_env = ($runtime_env | upsert YAZELIX_MANAGED_HELIX_BINARY $resolved_editor_command)
    }

    if ($helix_runtime | is-not-empty) {
        $runtime_env = ($runtime_env | upsert HELIX_RUNTIME $helix_runtime)
    }

    $runtime_env
}

export def --env activate_runtime_env [config?: record] {
    load-env (get_runtime_env $config)
}

export def run_runtime_argv [
    argv: list<string>
    --cwd: string = ""
    --config: record
] {
    if ($argv | is-empty) {
        error make {msg: "No command provided"}
    }

    let command = ($argv | first)
    let args = ($argv | skip 1)
    let requested_cwd = $cwd
    let runtime_env = if $config == null {
        get_runtime_env
    } else {
        get_runtime_env $config
    }

    with-env $runtime_env {
        if ($requested_cwd | is-not-empty) {
            cd ($requested_cwd | path expand)
        }
        ^$command ...$args
    }
}
