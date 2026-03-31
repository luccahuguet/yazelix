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
    $"use '($zellij_integration)' [get_current_tab_workspace_root_including_bootstrap run_pane_orchestrator_command_raw]"
    "let workspace_root = (get_current_tab_workspace_root_including_bootstrap)"
    "let target_dir = if ($workspace_root | is-not-empty) { $workspace_root } else { error make {msg: 'Could not resolve a target directory for Alt+m. Yazelix has no current tab workspace root.'} }"
    "let payload = ({cwd: $target_dir} | to json -r)"
    "let response = (run_pane_orchestrator_command_raw 'open_terminal_in_cwd' $payload)"
    "if (($response | str trim) != 'ok') {"
    "    error make {msg: $'Pane orchestrator failed to open terminal in cwd ''($target_dir)'': ($response)'}"
    "}"
] | str join "\n")

run-external nu "-c" $command
