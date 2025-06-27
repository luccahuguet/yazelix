#!/usr/bin/env nu
# Yazi integration utilities for Yazelix

# Navigate Yazi to the directory of the current Helix buffer
export def reveal_in_yazi [buffer_name: string] {
    print $"Starting reveal_in_yazi with buffer_name: ($buffer_name)"

    if ($buffer_name | is-empty) {
        print "Error: Buffer name not provided"
        return
    }

    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }

    let full_path = ($env.PWD | path join $normalized_buffer_name | path expand)

    if not ($full_path | path exists) {
        print $"Error: Resolved path '($full_path)' does not exist"
        return
    }

    let dir = ($full_path | path dirname)

    if ($env.YAZI_ID | is-empty) {
        print "Error: YAZI_ID not set. reveal-in-yazi requires that you open helix from yazelix's yazi."
        return
    }

    ya emit-to $env.YAZI_ID cd $dir
    zellij action move-focus left
}

# Simple check if Helix is running
def is_hx_running [command: string] {
    ($command | str contains "hx") or ($command | str contains "helix")
}

# Get running command from zellij
def get_running_command [] {
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

# Focus the helix pane
def find_helix [] {
    zellij action move-focus right
    zellij action move-focus up
    zellij action move-focus up
    zellij action move-focus up
    zellij action move-focus up
    zellij action move-focus up
}

# Get tab name from directory
def get_tab_name [working_dir: path] {
    try {
        let git_root = (git rev-parse --show-toplevel | str trim)
        if ($git_root | is-not-empty) and (not ($git_root | str starts-with "fatal:")) {
            $git_root | path basename
        } else {
            let basename = ($working_dir | str trim | path basename)
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

# Open file in existing Helix instance
def open_in_existing_helix [file_path: path] {
    print $"Opening ($file_path) in existing Helix instance"

    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    if not ($file_path | path exists) {
        print $"Error: File path ($file_path) does not exist"
        return
    }

    let tab_name = get_tab_name $working_dir

    try {
        zellij action write 27  # Escape

        let cd_cmd = $":cd \"($working_dir)\""
        zellij action write-chars $cd_cmd
        zellij action write 13  # Enter

        let open_cmd = $":open \"($file_path)\""
        zellij action write-chars $open_cmd
        zellij action write 13  # Enter

        zellij action rename-tab $tab_name
        print $"✅ Opened ($file_path) in existing Helix"
    } catch {|err|
        print $"❌ Error executing commands: ($err.msg)"
    }
}

# Open new Helix pane
def open_new_helix_pane [file_path: path, yazi_id: string] {
    print $"Opening new Helix pane for ($file_path)"

    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    let tab_name = get_tab_name $working_dir

    # Detect available Helix binary
    let editor_command = if (which helix | is-not-empty) { "helix" } else { "hx" }
    let cmd = $"env YAZI_ID=($yazi_id) ($editor_command) '($file_path)'"

    try {
        zellij run --name "helix" --cwd $working_dir -- nu -c $cmd
        zellij action rename-tab $tab_name
        print $"✅ Opened new Helix pane for ($file_path)"
    } catch {|err|
        print $"❌ Error opening new pane: ($err.msg)"
    }
}

# Open a file in Helix, integrating with Yazi and Zellij
export def open_file [file_path: path] {
    print $"DEBUG: file_path received: ($file_path), type: ($file_path | path type)"
    if not ($file_path | path exists) {
        print $"Error: File path ($file_path) does not exist"
        return
    }

    # Capture YAZI_ID from Yazi's pane
    let yazi_id = $env.YAZI_ID
    if ($yazi_id | is-empty) {
        print "Warning: YAZI_ID not set in this environment. Yazi navigation may fail."
    }

    # Move focus and check Helix status
    find_helix
    let running_command = (get_running_command)

    print $"DEBUG: Running command detected: ($running_command)"

    # Open file based on whether Helix is already running
    if (is_hx_running $running_command) {
        print "Helix is running, opening in existing instance"
        open_in_existing_helix $file_path
    } else {
        print "Helix not running, opening new pane"
        open_new_helix_pane $file_path $yazi_id
    }
}