#!/usr/bin/env nu
# Yazi integration utilities for Yazelix

use ../utils/logging.nu log_to_file
use zellij.nu [get_running_command, is_hx_running, open_in_existing_helix, open_new_helix_pane, find_and_focus_helix_pane, move_focused_pane_to_top, get_focused_pane_name]

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
        let error_msg = $"Resolved path '($full_path)' does not exist."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    if ($env.YAZI_ID | is-empty) {
        let error_msg = "YAZI_ID not set. reveal_in_yazi requires that you open helix from yazelix's yazi sidebar"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    log_to_file "reveal_in_yazi.log" $"YAZI_ID found: '($env.YAZI_ID)'"

    try {
        # Use 'reveal' command instead of 'cd' to both navigate to directory and select the file
        ya emit-to $env.YAZI_ID reveal $full_path
        log_to_file "reveal_in_yazi.log" $"Successfully sent 'reveal ($full_path)' command to yazi instance ($env.YAZI_ID)"

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

# Open file with Helix (with full Yazelix integration)
def open_with_helix [file_path: path, yazi_id: string] {
    log_to_file "open_helix.log" $"open_with_helix called with file_path: '($file_path)'"

    # Always check the topmost and next three panes below for Helix
    log_to_file "open_helix.log" "Checking up to 4 panes for Helix pane (editor)"
    let helix_pane_name = "editor"
    let max_panes = 4
    mut found_index = -1
    mut i = 0
    while ($i < $max_panes) {
        let running_command = (get_running_command)
        let pane_name = (get_focused_pane_name)
        if (is_hx_running $running_command) or ($pane_name == $helix_pane_name) {
            $found_index = $i
            break
        }
        zellij action focus-next-pane
        $i = $i + 1
    }

    if $found_index != -1 {
        log_to_file "open_helix.log" "Helix pane found and focused, moving to top and opening in existing instance"
        print "Helix pane found and focused, moving to top and opening in existing instance"
        move_focused_pane_to_top $found_index
        open_in_existing_helix $file_path
    } else {
        log_to_file "open_helix.log" "Helix pane not found in top 4, opening new pane"
        print "Helix pane not found in top 4, opening new pane"
        open_new_helix_pane $file_path $yazi_id
    }

    # Check if we should close the Yazi pane after opening file
    # Only for no-sidebar mode where Yazi starts as the main pane
    let sidebar_enabled = ($env.YAZELIX_ENABLE_SIDEBAR? | default "true") == "true"
    if (not $sidebar_enabled) {
        # In no-sidebar mode, close the Yazi pane to focus on editing
        try {
            # Since we're already focused on the Helix pane, go back to the previous pane (Yazi)
            # and close it
            zellij action focus-previous-pane
            sleep 100ms
            zellij action close-pane
            log_to_file "open_helix.log" "Closed Yazi pane in no-sidebar mode"
        } catch {
            log_to_file "open_helix.log" "Could not close Yazi pane (might not exist)"
        }
    }
    
    log_to_file "open_helix.log" "open_with_helix function completed"
}

# Open file with generic editor (basic Zellij integration)
def open_with_generic_editor [file_path: path, editor: string, yazi_id: string] {
    log_to_file "open_generic.log" $"open_with_generic_editor called with file_path: '($file_path)', editor: '($editor)'"

    # Get the directory of the file for tab naming
    let file_dir = ($file_path | path dirname)
    let tab_name = (get_tab_name $file_dir)

    try {
        # Create a new pane with the editor
        zellij action new-pane --cwd $file_dir -- $editor $file_path

        # Rename the tab
        zellij action rename-tab $tab_name

        log_to_file "open_generic.log" $"Successfully opened ($file_path) with ($editor) in new pane"
        print $"Opened ($file_path) with ($editor) in new pane"
    } catch {|err|
        let error_msg = $"Failed to open file with ($editor): ($err.msg)"
        log_to_file "open_generic.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
    }

    log_to_file "open_generic.log" "open_with_generic_editor function completed"
}


# Main file opening function - dispatches to appropriate editor handler
export def open_file_with_editor [file_path: path] {
    log_to_file "open_editor.log" $"open_file_with_editor called with file_path: '($file_path)'"
    print $"DEBUG: file_path received: ($file_path), type: ($file_path | path type)"

    if not ($file_path | path exists) {
        let error_msg = $"File path ($file_path) does not exist"
        log_to_file "open_editor.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    # Get the configured editor
    let editor = $env.EDITOR
    if ($editor | is-empty) {
        let error_msg = "EDITOR environment variable is not set"
        log_to_file "open_editor.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    log_to_file "open_editor.log" $"Using editor: ($editor)"

    # Check if sidebar is enabled
    let sidebar_enabled = ($env.YAZELIX_ENABLE_SIDEBAR? | default "true") == "true"
    log_to_file "open_editor.log" $"Sidebar enabled: ($sidebar_enabled)"

    # Capture YAZI_ID from Yazi's pane
    let yazi_id = $env.YAZI_ID
    if ($yazi_id | is-empty) {
        let warning_msg = "YAZI_ID not set in this environment. Yazi navigation may fail."
        log_to_file "open_editor.log" $"WARNING: ($warning_msg)"
        print $"Warning: ($warning_msg)"
    } else {
        log_to_file "open_editor.log" $"YAZI_ID found: '($yazi_id)'"
    }

    # For no-sidebar mode, we still use the multi-pane approach since we start with editor
    # The native Helix-Yazi integration (Ctrl+y) handles the "open in same pane" workflow

    # Sidebar mode: use the existing multi-pane logic
    if ($editor == "hx") {
        log_to_file "open_editor.log" "Detected Helix editor, using Helix-specific logic"
        open_with_helix $file_path $yazi_id
    } else {
        log_to_file "open_editor.log" $"Using generic editor approach for: ($editor)"
        open_with_generic_editor $file_path $editor $yazi_id
    }

    log_to_file "open_editor.log" "open_file_with_editor function completed"
}

