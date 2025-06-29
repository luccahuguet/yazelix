#!/usr/bin/env nu
# Yazi integration utilities for Yazelix

use ../utils/logging.nu log_to_file
use zellij.nu [find_helix, get_running_command, is_hx_running, open_in_existing_helix, open_new_helix_pane]

# Navigate Yazi to the directory of the current Helix buffer
export def reveal_in_yazi [buffer_name: string] {
    log_to_file "reveal_in_yazi.log" $"reveal_in_yazi called with buffer_name: '($buffer_name)'"

    if ($buffer_name | is-empty) {
        let error_msg = "Buffer name not provided"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }

    log_to_file "reveal_in_yazi.log" $"Normalized buffer name: '($normalized_buffer_name)'"

    let full_path = ($env.PWD | path join $normalized_buffer_name | path expand)
    log_to_file "reveal_in_yazi.log" $"Resolved full path: '($full_path)'"

    if not ($full_path | path exists) {
        let error_msg = $"Resolved path '($full_path)' does not exist"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    let dir = ($full_path | path dirname)
    log_to_file "reveal_in_yazi.log" $"Target directory: '($dir)'"

    if ($env.YAZI_ID | is-empty) {
        let error_msg = "YAZI_ID not set. reveal-in-yazi requires that you open helix from yazelix's yazi and that you build helix from source since the feature it uses is not yet in the latest release."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    log_to_file "reveal_in_yazi.log" $"YAZI_ID found: '($env.YAZI_ID)'"

    try {
        ya emit-to $env.YAZI_ID cd $dir
        log_to_file "reveal_in_yazi.log" $"Successfully sent 'cd ($dir)' command to yazi instance ($env.YAZI_ID)"

        zellij action move-focus left
        log_to_file "reveal_in_yazi.log" "Successfully moved focus left to yazi pane"
    } catch {|err|
        let error_msg = $"Failed to execute yazi/zellij commands: ($err.msg)"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
    }
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

# Open a file in Helix, integrating with Yazi and Zellij
export def open_file [file_path: path] {
    log_to_file "open_helix.log" $"open_file called with file_path: '($file_path)'"
    print $"DEBUG: file_path received: ($file_path), type: ($file_path | path type)"
    if not ($file_path | path exists) {
        let error_msg = $"File path ($file_path) does not exist"
        log_to_file "open_helix.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    # Capture YAZI_ID from Yazi's pane
    let yazi_id = $env.YAZI_ID
    if ($yazi_id | is-empty) {
        let warning_msg = "YAZI_ID not set in this environment. Yazi navigation may fail."
        log_to_file "open_helix.log" $"WARNING: ($warning_msg)"
        print $"Warning: ($warning_msg)"
    } else {
        log_to_file "open_helix.log" $"YAZI_ID found: '($yazi_id)'"
    }

    # Move focus and check Helix status
    log_to_file "open_helix.log" "Moving focus to find helix"
    find_helix
    let running_command = (get_running_command)

    log_to_file "open_helix.log" $"Running command detected: '($running_command)'"
    print $"DEBUG: Running command detected: ($running_command)"

    # Open file based on whether Helix is already running
    if (is_hx_running $running_command) {
        log_to_file "open_helix.log" "Helix is running, opening in existing instance"
        print "Helix is running, opening in existing instance"
        open_in_existing_helix $file_path
    } else {
        log_to_file "open_helix.log" "Helix not running, opening new pane"
        print "Helix not running, opening new pane"
        open_new_helix_pane $file_path $yazi_id
    }

    log_to_file "open_helix.log" "open_file function completed"
}