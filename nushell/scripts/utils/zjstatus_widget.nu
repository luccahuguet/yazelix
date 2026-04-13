#!/usr/bin/env nu

use ./common.nu get_yazelix_runtime_dir
use ./config_surfaces.nu get_main_user_config_path
use ./constants.nu DEFAULT_SHELL

def detect_terminal_name [] {
    if ($env.YAZELIX_TERMINAL? | is-not-empty) {
        return $env.YAZELIX_TERMINAL
    }

    if ($env.TERM_PROGRAM? | is-not-empty) {
        return ($env.TERM_PROGRAM | str downcase)
    }

    if ($env.KITTY_WINDOW_ID? | is-not-empty) {
        return "kitty"
    }

    if ($env.WEZTERM_EXECUTABLE? | is-not-empty) {
        return "wezterm"
    }

    if ($env.ALACRITTY_SOCKET? | is-not-empty) {
        return "alacritty"
    }

    if ($env.GHOSTTY_BIN_DIR? | is-not-empty) {
        return "ghostty"
    }

    if ($env.TERM? | is-not-empty) and ($env.TERM | str starts-with "foot") {
        return "foot"
    }

    if ($env.XDG_CURRENT_DESKTOP? | is-not-empty) and ($env.XDG_CURRENT_DESKTOP | str contains -i "cosmic") {
        return "cosmic-term"
    }

    "unknown"
}

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
            normalize_command_label (($shell_config | get -o default_shell | default $DEFAULT_SHELL) | into string) $DEFAULT_SHELL
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
