#!/usr/bin/env nu

use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu [get_active_sidebar_cwd]
use ~/.config/yazelix/nushell/scripts/integrations/zellij.nu [get_current_tab_workspace_root run_pane_orchestrator_command_raw]

let sidebar_cwd = (get_active_sidebar_cwd)
let workspace_root = if ($sidebar_cwd | is-not-empty) {
    null
} else {
    get_current_tab_workspace_root
}
let target_dir = if ($sidebar_cwd | is-not-empty) {
    $sidebar_cwd
} else {
    if ($workspace_root | is-not-empty) {
        $workspace_root
    } else {
        error make {msg: "Could not resolve a target directory for Alt+m. The sidebar cwd is unavailable and Yazelix has no current tab workspace root."}
    }
}

let payload = ({cwd: $target_dir} | to json -r)
let response = (run_pane_orchestrator_command_raw "open_terminal_in_cwd" $payload)

if (($response | str trim) != "ok") {
    error make {msg: $"Pane orchestrator failed to open terminal in cwd '($target_dir)': ($response)"}
}
