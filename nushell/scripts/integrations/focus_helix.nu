#!/usr/bin/env nu
# Focus the Helix pane from Yazi

use ../utils/logging.nu log_to_file
use zellij.nu [get_running_command, is_hx_running, get_focused_pane_name, move_focused_pane_to_top]

export def main [] {
    log_to_file "focus_helix.log" "focus_helix called from Yazi"

    # Check if sidebar mode is enabled
    let sidebar_enabled = ($env.YAZELIX_ENABLE_SIDEBAR? | default "true") == "true"
    if (not $sidebar_enabled) {
        log_to_file "focus_helix.log" "Sidebar mode disabled - focus_helix not available"
        return
    }

    try {
        # Look for the Helix pane (named "editor" or running "hx") 
        # Use the same logic as open_with_helix for consistency
        log_to_file "focus_helix.log" "Checking up to 4 panes for Helix pane (editor)"
        let helix_pane_name = "editor"
        let max_panes = 4
        mut found_index = -1
        mut i = 0
        while ($i < $max_panes) {
            let running_command = (get_running_command)
            let pane_name = (get_focused_pane_name)
            if (is_hx_running $running_command) or ($pane_name == $helix_pane_name) {
                $found_index = $i
                break
            }
            zellij action focus-next-pane
            $i = $i + 1
        }

        if $found_index != -1 {
            log_to_file "focus_helix.log" "Helix pane found and focused, moving to top"
            move_focused_pane_to_top $found_index
            log_to_file "focus_helix.log" "Successfully focused and moved Helix pane to top"
        } else {
            log_to_file "focus_helix.log" "No Helix pane found to focus"
        }
    } catch {|err|
        let error_msg = $"Failed to focus Helix pane: ($err.msg)"
        log_to_file "focus_helix.log" $"ERROR: ($error_msg)"
    }
}