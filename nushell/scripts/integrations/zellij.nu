#!/usr/bin/env nu
# Zellij integration utilities for Yazelix

use ../utils/logging.nu *

# Get the tab name based on Git repo or working directory
def get_tab_name [working_dir: path] {
    try {
        let git_root = (git rev-parse --show-toplevel | str trim)
        if ($git_root | is-not-empty) and (not ($git_root | str starts-with "fatal:")) {
            log_to_file "open_helix.log" $"Git root found: ($git_root)"
            $git_root | path basename
        } else {
            let basename = ($working_dir | str trim | path basename)
            log_to_file "open_helix.log" $"No valid Git repo, using basename of ($working_dir): ($basename)"
            if ($basename | is-empty) {
                "unnamed"
            } else {
                $basename
            }
        }
    } catch {
        $working_dir | path basename
    }
}

# Focus the helix pane
export def find_helix [] {
    zellij action move-focus right
    zellij action move-focus up
    zellij action move-focus up
    zellij action move-focus up
    zellij action move-focus up
    zellij action move-focus up
}

# Get the running command from the second Zellij client
export def get_running_command [] {
    try {
        let list_clients_output = (zellij action list-clients | lines | get 1)

        $list_clients_output
            | parse --regex '\w+\s+\w+\s+(?<rest>.*)'
            | get rest
            | to text
    } catch {
        ""
    }
}

# Check if Helix is running (simplified version for zellij integration)
export def is_hx_running [command: string] {
    ($command | str contains "hx") or ($command | str contains "helix")
}

# Open a file in an existing Helix pane and rename tab
export def open_in_existing_helix [file_path: path] {
    log_to_file "open_helix.log" $"Starting open_in_existing_helix with file_path: ($file_path)"

    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    log_to_file "open_helix.log" $"Calculated working_dir: ($working_dir)"

    if not ($file_path | path exists) {
        log_to_file "open_helix.log" $"Error: File path ($file_path) does not exist"
        print $"Error: File path ($file_path) does not exist"
        return
    }

    log_to_file "open_helix.log" $"File path validated as existing"

    let tab_name = get_tab_name $working_dir
    log_to_file "open_helix.log" $"Calculated tab_name: ($tab_name)"

    try {
        zellij action write 27
        log_to_file "open_helix.log" "Sent Escape (27) to enter command mode"

        let cd_cmd = $":cd \"($working_dir)\""
        zellij action write-chars $cd_cmd
        log_to_file "open_helix.log" $"Sent cd command: ($cd_cmd)"
        zellij action write 13
        log_to_file "open_helix.log" "Sent Enter (13) for cd command"

        let open_cmd = $":open \"($file_path)\""
        zellij action write-chars $open_cmd
        log_to_file "open_helix.log" $"Sent open command: ($open_cmd)"
        zellij action write 13
        log_to_file "open_helix.log" "Sent Enter (13) for open command"

        zellij action rename-tab $tab_name
        log_to_file "open_helix.log" $"Renamed tab to: ($tab_name)"

        log_to_file "open_helix.log" "Commands executed successfully"
    } catch {|err|
        log_to_file "open_helix.log" $"Error executing commands: ($err.msg)"
        print $"Error executing commands: ($err.msg)"
    }
}

# Open a new pane and set up Helix with Yazi integration, renaming tab
export def open_new_helix_pane [file_path: path, yazi_id: string] {
    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    log_to_file "open_helix.log" $"Attempting to open new pane with YAZI_ID=($yazi_id) for file=($file_path)"

    let tab_name = get_tab_name $working_dir
    log_to_file "open_helix.log" $"Calculated tab_name: ($tab_name)"

    # Ensure helix config directory exists
    let helix_config_dir = $"($env.HOME)/.config/helix"
    mkdir $helix_config_dir

    # Simple command - both modes use 'hx' from PATH
    let cmd = $"env YAZI_ID=($yazi_id) hx '($file_path)'"

    log_to_file "open_helix.log" $"Full command to execute: ($cmd)"

    try {
        log_to_file "open_helix.log" $"Preparing command: nu -c \"($cmd)\""
        zellij run --name "helix" --cwd $working_dir -- nu -c $cmd
        log_to_file "open_helix.log" $"Command executed successfully: nu -c \"($cmd)\""

        zellij action rename-tab $tab_name
        log_to_file "open_helix.log" $"Renamed tab to: ($tab_name)"
    } catch {|err|
        log_to_file "open_helix.log" $"Error executing command: nu -c \"($cmd)\"\nError details: ($err.msg)"
        print $"Error executing zellij command: nu -c \"($cmd)\"\nDetails: ($err.msg)"
    }
}
