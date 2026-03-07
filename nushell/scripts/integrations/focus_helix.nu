#!/usr/bin/env nu
# Focus the Helix pane from Yazi

use ../utils/logging.nu log_to_file
use ../utils/config_parser.nu parse_yazelix_config
use zellij.nu [focus_managed_pane]

export def main [] {
    log_to_file "focus_helix.log" "focus_helix called from Yazi"

    # Check if sidebar mode is enabled
    let config = parse_yazelix_config
    let sidebar_enabled = ($config.enable_sidebar? | default true)
    if (not $sidebar_enabled) {
        log_to_file "focus_helix.log" "Sidebar mode disabled - focus_helix not available"
        return
    }

    try {
        let managed_focus_result = (focus_managed_pane "editor" "focus_helix.log")
        if $managed_focus_result.status == "ok" {
            log_to_file "focus_helix.log" "Focused managed Helix pane through pane orchestrator"
        } else {
            let error_msg = $"Managed Helix focus failed \(status=($managed_focus_result.status)\). Ensure the Yazelix pane orchestrator plugin is loaded and the editor pane title is 'editor'."
            log_to_file "focus_helix.log" $"ERROR: ($error_msg)"
        }
    } catch {|err|
        let error_msg = $"Failed to focus Helix pane: ($err.msg)"
        log_to_file "focus_helix.log" $"ERROR: ($error_msg)"
    }
}
