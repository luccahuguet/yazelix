#!/usr/bin/env nu

export def is_hx_running [list_clients_output: string] {
    let cmd = $list_clients_output | str trim | str downcase
    
    # Split the command into parts
    let parts = $cmd | split row " "
    
    # Check if any part ends with 'hx' or is 'hx'
    let has_hx = ($parts | any {|part| $part | str ends-with "/hx"})
    let is_hx = ($parts | any {|part| $part == "hx"})
    let has_or_is_hx = $has_hx or $is_hx
    
    # Find the position of 'hx' in the parts
    let hx_positions = ($parts | enumerate | where {|x| ($x.item == "hx" or ($x.item | str ends-with "/hx"))} | get index)
    
    # Check if 'hx' is the first part or right after a path
    let is_hx_at_start = if ($hx_positions | is-empty) {
        false
    } else {
        let hx_position = $hx_positions.0
        $hx_position == 0 or ($hx_position > 0 and ($parts | get ($hx_position - 1) | str ends-with "/"))
    }
    
    let result = $has_or_is_hx and $is_hx_at_start
    
    # Debug information
    print $"input: list_clients_output = ($list_clients_output)"
    print $"treated input: cmd = ($cmd)"
    print $"  parts: ($parts)"
    print $"  has_hx: ($has_hx)"
    print $"  is_hx: ($is_hx)"
    print $"  has_or_is_hx: ($has_or_is_hx)"
    print $"  hx_positions: ($hx_positions)"
    print $"  is_hx_at_start: ($is_hx_at_start)"
    print $"  Final result: ($result)"
    print ""
    
    $result
}



def main [file_path: path] {
    # Move focus to the next pane
    zellij action focus-next-pane

    # Store the second line of the zellij clients list in a variable
    let list_clients_output = (zellij action list-clients | lines | get 1)

    # Parse the output to remove the first two words and extract the rest
    let running_command = $list_clients_output 
        | parse --regex '\w+\s+\w+\s+(?<rest>.*)'  # Use regex to match two words and capture the remaining text as 'rest'
        | get rest  # Retrieve the captured 'rest' part, which is everything after the first two words
        | to text

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
        
        zellij action rename-tab ($working_dir | path basename)

        # Change to the working directory
        zellij action write-chars $"cd ($working_dir)"
        zellij action write 13
        sleep 0.2sec
        
        # Open Helix
        zellij action write-chars $"hx ($file_path)"
        sleep 0.1sec
        zellij action write 13
        sleep 0.1sec
    }
}
