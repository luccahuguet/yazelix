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

let yazi_integration = (get_runtime_script_path "nushell/scripts/integrations/yazi.nu")
let command = ([
    $"use '($yazi_integration)' [consume_bootstrap_sidebar_cwd get_yazi_command]"
    "let bootstrap_dir = (consume_bootstrap_sidebar_cwd)"
    "let target_dir = if ($bootstrap_dir | is-not-empty) { $bootstrap_dir } else { pwd | path expand }"
    "let yazi_command = (get_yazi_command)"
    "run-external $yazi_command $target_dir"
] | str join "\n")

run-external nu "-c" $command
