#!/usr/bin/env nu

use ./common.nu get_yazelix_runtime_dir
use ./config_surfaces.nu get_main_user_config_path
use ./detect_terminal.nu detect_terminal_name

def load_widget_main_config [] {
    let configured_override = (
        $env.YAZELIX_CONFIG_OVERRIDE?
        | default ""
        | into string
        | str trim
    )
    let config_path = if ($configured_override | is-not-empty) and ($configured_override | path exists) {
        $configured_override | path expand
    } else {
        let user_config = (get_main_user_config_path)
        if ($user_config | path exists) {
            $user_config
        } else {
            (get_yazelix_runtime_dir | path join "yazelix_default.toml")
        }
    }

    if not ($config_path | path exists) {
        return {}
    }

    try {
        open $config_path
    } catch {
        {}
    }
}

def normalize_command_label [value: string, fallback: string] {
    let trimmed = ($value | str trim)
    if ($trimmed | is-empty) {
        return $fallback
    }

    let executable = (
        $trimmed
        | split row " "
        | where {|part| $part != ""}
        | first
    )

    if ($executable | path basename | is-not-empty) {
        $executable | path basename
    } else {
        $fallback
    }
}

export def main [widget: string] {
    match $widget {
        "shell" => {
            let config = (load_widget_main_config)
            let shell_config = ($config.shell? | default {})
            normalize_command_label (($shell_config | get -o default_shell | default "nu") | into string) "nu"
        }
        "editor" => {
            let config = (load_widget_main_config)
            let editor_config = ($config.editor? | default {})
            let editor_command = (($editor_config | get -o command | default "hx") | into string)
            normalize_command_label $editor_command "hx"
        }
        "terminal" => {
            let detected_terminal = (detect_terminal_name)
            if $detected_terminal != "unknown" {
                $detected_terminal
            } else {
                let config = (load_widget_main_config)
                let terminal_config = ($config.terminal? | default {})
                let configured_terminals = ($terminal_config | get -o terminals | default [])
                if ($configured_terminals | is-empty) {
                    "unknown"
                } else {
                    $configured_terminals | first | into string
                }
            }
        }
        _ => {
            error make {msg: $"Unknown zjstatus widget '($widget)'. Expected one of: shell, editor, terminal."}
        }
    }
}
