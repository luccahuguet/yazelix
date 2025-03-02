#!/usr/bin/env nu

# Zellij utility functions for Yazelix

# Focus the next Zellij pane
export def focus_next_pane [] {
    zellij action focus-next-pane
}

# Get the running command from the second Zellij client
export def get_running_command [] {
    let list_clients_output = (zellij action list-clients | lines | get 1)
    $list_clients_output 
        | parse --regex '\w+\s+\w+\s+(?<rest>.*)'  # Parse to remove first two words
        | get rest 
        | to text
}

# Open a file in an existing Helix pane
export def open_in_existing_helix [file_path: path] {
    zellij action write 27  # Escape key
    zellij action write-chars $":open \"($file_path)\""  # Helix command to open file
    zellij action write 13  # Enter key
}

# Open a new pane and set up Helix with Yazi integration
export def open_new_helix_pane [file_path: path, yazi_id: string, initial_path: string] {
    zellij action new-pane
    sleep 0.5sec  # Wait for pane to initialize
    
    # Determine working directory
    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }
    
    zellij action rename-tab ($working_dir | path basename)  # Name tab after directory
    
    # Set environment variables for Yazi and initial path
    zellij action write-chars $"$env.YAZI_ID = \"($yazi_id)\"; $env.YAZELIX_INITIAL_PATH = \"($initial_path)\""
    zellij action write 13  # Enter
    sleep 0.2sec
    
    # Change to working directory
    zellij action write-chars $"cd ($working_dir)"
    zellij action write 13  # Enter
    sleep 0.2sec
    
    # Open Helix with the file
    zellij action write-chars $"hx ($file_path)"
    sleep 0.1sec
    zellij action write 13  # Enter
    sleep 0.1sec
}
