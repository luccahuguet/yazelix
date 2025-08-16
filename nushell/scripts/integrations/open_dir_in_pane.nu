#!/usr/bin/env nu
# Open directory in new Zellij pane
# Takes the file/directory path as argument

use ../utils/logging.nu log_to_file

export def main [file_path: string] {
    log_to_file "open_dir_in_pane.log" $"open_dir_in_pane called with file_path: '($file_path)'"

    if ($file_path | is-empty) {
        log_to_file "open_dir_in_pane.log" "ERROR: No file path provided"
        return
    }

    # Check if the path exists
    if not ($file_path | path exists) {
        log_to_file "open_dir_in_pane.log" $"ERROR: Path does not exist: ($file_path)"
        return
    }

    # Determine the target directory
    let target_dir = if ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    log_to_file "open_dir_in_pane.log" $"Target directory: ($target_dir)"

    try {
        # Create new pane and explicitly change directory, using nu as the shell
        log_to_file "open_dir_in_pane.log" $"About to run: zellij action new-pane -- nu -c 'cd \"($target_dir)\"; nu'"
        zellij action new-pane -- nu -c $"cd \"($target_dir)\"; nu"
        log_to_file "open_dir_in_pane.log" $"Successfully opened new pane in directory: ($target_dir)"
    } catch {|err|
        let error_msg = $"Failed to open new pane: ($err.msg)"
        log_to_file "open_dir_in_pane.log" $"ERROR: ($error_msg)"
    }
}