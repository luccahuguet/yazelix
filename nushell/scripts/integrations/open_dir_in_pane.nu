#!/usr/bin/env nu
# Open directory in new Zellij pane
# Takes the file/directory path as argument

use ../utils/logging.nu log_to_file
use ./zellij.nu [get_workspace_root, set_workspace_for_path]

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

    let workspace_root = (get_workspace_root $target_dir)
    log_to_file "open_dir_in_pane.log" $"Target directory: ($target_dir)"
    log_to_file "open_dir_in_pane.log" $"Workspace root: ($workspace_root)"

    try {
        # Open the pane in the selected directory, but record the stable workspace root for the tab
        log_to_file "open_dir_in_pane.log" $"About to run: zellij action new-pane --cwd ($target_dir)"
        zellij action new-pane --cwd $target_dir
        log_to_file "open_dir_in_pane.log" $"Successfully opened new pane in directory: ($target_dir)"

        let workspace_result = (set_workspace_for_path $target_dir "open_dir_in_pane.log")
        if $workspace_result.status == "ok" {
            log_to_file "open_dir_in_pane.log" $"Updated workspace root to: ($workspace_result.workspace_root)"
        } else {
            log_to_file "open_dir_in_pane.log" $"WARNING: Failed to update workspace root \(status=($workspace_result.status)\)"
        }
    } catch {|err|
        let error_msg = $"Failed to open new pane: ($err.msg)"
        log_to_file "open_dir_in_pane.log" $"ERROR: ($error_msg)"
    }
}
