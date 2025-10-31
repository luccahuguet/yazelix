#!/usr/bin/env nu
# Zellij integration utilities for Yazelix

use ../utils/logging.nu *

# Get the tab name based on Git repo or working directory
export def get_tab_name [working_dir: path] {
    try {
        let git_root = (bash -c $"cd '($working_dir)' && git rev-parse --show-toplevel 2>/dev/null" | str trim)
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

# Cycle through up to max_panes, looking for a Helix pane by name or running command
export def find_and_focus_helix_pane [max_panes: int = 3, helix_pane_name: string = "editor"] {
    mut i = 0
    while ($i < $max_panes) {
        let running_command = (get_running_command)
        let pane_name = (get_focused_pane_name)
        if (is_hx_running $running_command) or ($pane_name == $helix_pane_name) {
            return true
        }
        zellij action focus-next-pane
        $i = $i + 1
    }
    return false
}

# Helper to get the name of the currently focused pane (best effort)
export def get_focused_pane_name [] {
    try {
        let output = (zellij action list-clients | lines | get 1)
        # Example output: CLIENT_ID ZELLIJ_PANE_ID RUNNING_COMMAND
        # We try to parse the pane name from the ZELLIJ_PANE_ID if possible
        $output | split row " " | get 1 | to text
    } catch {
        ""
    }
}

# Move the currently focused pane to the top of the stack by moving up 'steps' times
export def move_focused_pane_to_top [steps: int] {
    mut i = 0
    while ($i < $steps) {
        zellij action move-pane up
        $i = $i + 1
    }
}

# Generic function to open file in existing editor instance
def open_in_existing_editor [
    file_path: path
    cd_command: string
    open_command: string
    use_quotes: bool
    log_file: string
    editor_name: string
] {
    log_to_file $log_file $"Starting open_in_existing_($editor_name) with file_path: '($file_path)'"

    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    log_to_file $log_file $"Calculated working_dir: ($working_dir)"

    if not ($file_path | path exists) {
        log_to_file $log_file $"Error: File path ($file_path) does not exist"
        print $"Error: File path ($file_path) does not exist"
        return
    }

    log_to_file $log_file $"File path validated as existing"

    let tab_name = get_tab_name $working_dir
    log_to_file $log_file $"Calculated tab_name: ($tab_name)"

    try {
        zellij action write 27
        log_to_file $log_file "Sent Escape \(27\) to enter normal mode"

        let cd_cmd = if $use_quotes {
            $"($cd_command) \"($working_dir)\""
        } else {
            $"($cd_command) ($working_dir)"
        }
        zellij action write-chars $cd_cmd
        log_to_file $log_file $"Sent cd command: ($cd_cmd)"
        zellij action write 13
        log_to_file $log_file "Sent Enter \(13\) for cd command"

        let file_cmd = if $use_quotes {
            $"($open_command) \"($file_path)\""
        } else {
            $"($open_command) ($file_path)"
        }
        zellij action write-chars $file_cmd
        log_to_file $log_file $"Sent open command: ($file_cmd)"
        zellij action write 13
        log_to_file $log_file "Sent Enter \(13\) for open command"

        zellij action rename-tab $tab_name
        log_to_file $log_file $"Renamed tab to: ($tab_name)"

        log_to_file $log_file "Commands executed successfully"
    } catch {|err|
        log_to_file $log_file $"Error executing commands: ($err.msg)"
        print $"Error executing commands: ($err.msg)"
    }
}

# Open a file in an existing Helix pane and rename tab
export def open_in_existing_helix [file_path: path] {
    open_in_existing_editor $file_path ":cd" ":open" true "open_helix.log" "helix"
}

# Generic function to open a new editor pane with Yazi integration
def open_new_editor_pane [file_path: path, yazi_id: string, log_file: string] {
    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    log_to_file $log_file $"Attempting to open new pane with YAZI_ID=($yazi_id) for file=($file_path)"

    let tab_name = get_tab_name $working_dir
    log_to_file $log_file $"Calculated tab_name: ($tab_name)"

    # Use the configured editor from environment, preserving YAZI_ID
    let editor = $env.EDITOR
    let cmd = $"env YAZI_ID=($yazi_id) ($editor) '($file_path)'"

    log_to_file $log_file $"Full command to execute: ($cmd)"

    # Try to use 'editor' as the pane name, fallback to 'yazelix_editor' if needed
    let pane_name = "editor"
    try {
        log_to_file $log_file $"Preparing command: nu -c \"($cmd)\" with pane name: ($pane_name)"
        zellij run --name $pane_name --cwd $working_dir -- nu -c $cmd
        log_to_file $log_file $"Command executed successfully: nu -c \"($cmd)\" with pane name: ($pane_name)"
    } catch {|err|
        let fallback_pane_name = "yazelix_editor"
        log_to_file $log_file $"Failed to use pane name 'editor', falling back to: ($fallback_pane_name)"
        zellij run --name $fallback_pane_name --cwd $working_dir -- nu -c $cmd
        log_to_file $log_file $"Command executed successfully: nu -c \"($cmd)\" with pane name: ($fallback_pane_name)"
    }

    zellij action rename-tab $tab_name
    log_to_file $log_file $"Renamed tab to: ($tab_name)"
}

# Open a new pane and set up Helix with Yazi integration, renaming tab
export def open_new_helix_pane [file_path: path, yazi_id: string] {
    open_new_editor_pane $file_path $yazi_id "open_helix.log"
}

# Neovim integration functions

# Check if Neovim is running
export def is_nvim_running [command: string] {
    ($command | str contains "nvim") or ($command | str contains "neovim")
}

# Open a file in an existing Neovim pane and rename tab
export def open_in_existing_neovim [file_path: path] {
    open_in_existing_editor $file_path ":cd" ":edit" false "open_neovim.log" "neovim"
}

# Open a new pane and set up Neovim with Yazi integration, renaming tab
export def open_new_neovim_pane [file_path: path, yazi_id: string] {
    open_new_editor_pane $file_path $yazi_id "open_neovim.log"
}
