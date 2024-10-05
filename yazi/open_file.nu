#!/usr/bin/env nu

def is_hx_running [command: string] {
    let cmd = $command | str trim | str downcase
    
    # Match if hx appears at the end of a path, like '/hx' or '/some/path/hx'
    let hx_in_path = ($cmd =~ '(.*/)?hx(\s.*)?$')
    
    # Match if hx is followed by flags or arguments, like 'hx --flag' or '/hx --flag'
    let hx_with_flag_or_path = ($cmd =~ '(^|\s|/)(hx)(\s+|/|-|--)[a-z-]*')
    
    $hx_in_path or $hx_with_flag_or_path
}



def main [file_path: path] {
    # Move focus to the next pane
    zellij action focus-next-pane

    # Get the running command in the current pane
    let running_command = (zellij action list-clients | detect columns | get "RUNNING_COMMAND" | to text)

    # Check if the command running in the current pane is hx
    if (is_hx_running $running_command) {
        # The current pane is running hx, use zellij actions to open the file
        zellij action write 27
        zellij action write-chars $":open \"($file_path)\""
        zellij action write 13
    } else {
        # The current pane is not running hx, so open hx in a new pane
        zellij action new-pane
        sleep 0.5sec
        
        # Determine the working directory
        let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
            $file_path
        } else {
            $file_path | path dirname
        }
        
        # Change to the working directory
        zellij action write-chars $"cd ($working_dir)"
        zellij action write 13
        sleep 0.2sec
        
        # Open Helix
        zellij action write-chars $"hx ($file_path)"
        zellij action write 13
    }
}
