#!/usr/bin/env nu

def main [file_path: path] {
    # Move focus to the next pane
    zellij action focus-next-pane

    # Get the running command in the current pane
    let running_command = (zellij action list-clients | detect columns | get "RUNNING_COMMAND" | to text)

    # Check if the command running in the current pane is helix (hx)
    if ($running_command | str ends-with "/hx") {
        # The current pane is running helix, use zellij actions to open the file
        zellij action write 27
        zellij action write-chars $":open \"($file_path)\""
        zellij action write 13
    } else {
        # The current pane is not running helix, so open helix in a new pane
        zellij action new-pane
        sleep 0.5sec
        # Get the working directory
        let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
            $file_path
        } else {
            $file_path | path dirname
        }
        zellij action write-chars $"hx ($file_path) -w ($working_dir)"
        zellij action write 13
    }
}
