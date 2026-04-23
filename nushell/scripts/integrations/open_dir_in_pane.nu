#!/usr/bin/env nu
# Open directory in new Zellij pane
# Takes the file/directory path as argument

use ../utils/logging.nu log_to_file
use ../utils/yzx_core_bridge.nu [run_zellij_pipe run_zellij_retarget]

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
        let payload = ({cwd: $target_dir} | to json -r)
        let response = (run_zellij_pipe "open_terminal_in_cwd" $payload)
        if (($response | str trim) != "ok") {
            error make {msg: $"Pane orchestrator failed to open directory pane in '($target_dir)': ($response)"}
        }

        log_to_file "open_dir_in_pane.log" $"Successfully opened new pane in directory: ($target_dir)"

        let workspace_result = (run_zellij_retarget $target_dir)
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
