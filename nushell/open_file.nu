#!/usr/bin/env nu

# Open a file in Helix, integrating with Yazi and Zellij
source ~/.config/yazelix/nushell/utils.nu  # Import utilities

def main [file_path: path] {
    # Capture YAZI_ID from Yazi’s pane
    let yazi_id = $env.YAZI_ID
    if ($yazi_id | is-empty) {
        print "Warning: YAZI_ID not set in this environment. Yazi navigation may fail."
    }

    # Emit toggle-pane commands
    ya emit-to $yazi_id "plugin" "toggle-pane" "reset"
    ya emit-to $yazi_id "plugin" "toggle-pane" "max-current"

    # Move focus to the next pane
    zellij action focus-next-pane

    # Store the second line of the zellij clients list
    let list_clients_output = (zellij action list-clients | lines | get 1)

    # Parse the output to extract the running command
    let running_command = $list_clients_output 
        | parse --regex '\w+\s+\w+\s+(?<rest>.*)' 
        | get rest 
        | to text

    # Check if Helix is running
    if (is_hx_running $running_command) {
        # Open file in existing Helix
        zellij action write 27
        zellij action write-chars $":open \"($file_path)\""
        zellij action write 13
    } else {
        # Open new pane for Helix
        zellij action new-pane
        sleep 0.5sec
        
        # Determine working directory
        let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
            $file_path
        } else {
            $file_path | path dirname
        }
        
        zellij action rename-tab ($working_dir | path basename)

        # Set YAZI_ID in Nushell syntax
        zellij action write-chars $"$env.YAZI_ID = \"($yazi_id)\""
        zellij action write 13
        sleep 0.2sec
        
        # Change to working directory
        zellij action write-chars $"cd ($working_dir)"
        zellij action write 13
        sleep 0.2sec
        
        # Open Helix
        zellij action write-chars $"hx ($file_path)"
        sleep 0.1sec
        zellij action write 13
        sleep 0.1sec
    }
}
