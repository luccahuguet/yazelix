#!/usr/bin/env nu
# Focus the Helix pane from Yazi

use ../utils/logging.nu log_to_file
use zellij.nu find_and_focus_helix_pane

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
        # Check up to 4 panes (same as in open_with_helix)
        let helix_found = find_and_focus_helix_pane 4 "editor"

        if $helix_found {
            log_to_file "focus_helix.log" "Successfully focused Helix pane"
        } else {
            log_to_file "focus_helix.log" "No Helix pane found to focus"
        }
    } catch {|err|
        let error_msg = $"Failed to focus Helix pane: ($err.msg)"
        log_to_file "focus_helix.log" $"ERROR: ($error_msg)"
    }
}