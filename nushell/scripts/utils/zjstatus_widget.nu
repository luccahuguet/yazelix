#!/usr/bin/env nu

use ./config_parser.nu parse_yazelix_config
use ./detect_terminal.nu detect_terminal_name

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
    let config = if $widget == "terminal" { null } else { parse_yazelix_config }

    match $widget {
        "shell" => {
            normalize_command_label ($config.default_shell? | default "nu" | into string) "nu"
        }
        "editor" => {
            let editor_command = ($config.editor_command? | default "hx" | into string)
            normalize_command_label $editor_command "hx"
        }
        "terminal" => {
            let detected_terminal = (detect_terminal_name)
            if $detected_terminal != "unknown" {
                $detected_terminal
            } else {
                let terminal_config = (parse_yazelix_config)
                let configured_terminals = ($terminal_config.terminals? | default [])
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
