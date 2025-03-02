#!/usr/bin/env nu

# Open a file in Helix, integrating with Yazi and Zellij
source ~/.config/yazelix/nushell/utils.nu
source ~/.config/yazelix/nushell/zellij_utils.nu

def main [file_path: path] {
    # Capture YAZI_ID from Yazi’s pane
    let yazi_id = $env.YAZI_ID
    if ($yazi_id | is-empty) {
        print "Warning: YAZI_ID not set in this environment. Yazi navigation may fail."
    }

    # Emit toggle-pane commands
    ya emit-to $yazi_id "plugin" "toggle-pane" "reset"
    ya emit-to $yazi_id "plugin" "toggle-pane" "max-current"

    # Move focus and check Helix status
    focus_next_pane
    let running_command = (get_running_command)

    # Open file based on Helix status
    if (is_hx_running $running_command) {
        open_in_existing_helix $file_path
    } else {
        open_new_helix_pane $file_path $yazi_id
    }
}
