#!/usr/bin/env nu
# ~/.config/yazelix/nushell/zellij_utils.nu

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
export def open_new_helix_pane [file_path: path, yazi_id: string] {
    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }
    
    let log_file = ($nu.home-path | path join ".config/yazelix/logs/open_helix.log")
    mkdir ($log_file | path dirname)
    
    let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
    try {
        $"[($timestamp)] Attempting to open new pane with YAZI_ID=($yazi_id) for file=($file_path)\n" | save -a $log_file
    } catch {
        print $"Failed to write to log file: ($log_file)"
    }
    
    let cmd = $"env YAZI_ID=($yazi_id) hx '($file_path)'"
    try {
        $"[($timestamp)] Preparing command: nu -c \"($cmd)\"\n" | save -a $log_file
        zellij run --name "helix" --cwd $working_dir -- nu -c $cmd 
        sleep 0.2sec
        $"[($timestamp)] Command executed successfully: nu -c \"($cmd)\"\n" | save -a $log_file
    } catch {|err|
        $"[($timestamp)] Error executing command: nu -c \"($cmd)\"\nError details: ($err.msg)\n" | save -a $log_file
        print $"Error executing zellij command: nu -c \"($cmd)\"\nDetails: ($err.msg)"
    }
}
