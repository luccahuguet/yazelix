#!/usr/bin/env nu

use ~/.config/yazelix/nushell/scripts/integrations/zellij.nu [get_current_tab_workspace_root run_pane_orchestrator_command_raw]

let workspace_root = (get_current_tab_workspace_root)
let target_dir = if ($workspace_root | is-not-empty) {
    $workspace_root
} else {
    error make {msg: "Could not resolve a target directory for Alt+m. Yazelix has no current tab workspace root."}
}

let payload = ({cwd: $target_dir} | to json -r)
let response = (run_pane_orchestrator_command_raw "open_terminal_in_cwd" $payload)

if (($response | str trim) != "ok") {
    error make {msg: $"Pane orchestrator failed to open terminal in cwd '($target_dir)': ($response)"}
}
