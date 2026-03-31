#!/usr/bin/env nu

def get_runtime_script_path [relative_path: string] {
    let runtime_dir = ($env.YAZELIX_RUNTIME_DIR? | default "" | str trim)
    if ($runtime_dir | is-empty) {
        error make {msg: "Missing YAZELIX_RUNTIME_DIR for Yazelix Zellij helper script."}
    }

    let expanded_runtime_dir = ($runtime_dir | path expand)
    if not ($expanded_runtime_dir | path exists) {
        error make {msg: $"Configured YAZELIX_RUNTIME_DIR does not exist: ($expanded_runtime_dir)"}
    }

    ($expanded_runtime_dir | path join $relative_path)
}

let zellij_integration = (get_runtime_script_path "nushell/scripts/integrations/zellij.nu")
let command = ([
    $"use '($zellij_integration)' *"
    "let result = (previous_layout_family)"
    "if $result.status != 'ok' {"
    "    print $'Error: previous layout family failed \(status=($result.status)\)'"
    "    exit 1"
    "}"
] | str join "\n")

run-external nu "-c" $command
