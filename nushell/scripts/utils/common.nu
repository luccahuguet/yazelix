#!/usr/bin/env nu

# Utility functions for Yazelix

# Get the maximum number of CPU cores available on the system
export def get_max_cores [] {
    sys cpu | length
}

# Check if Helix (hx or helix) is running in a Zellij pane based on client output
export def is_hx_running [list_clients_output: string] {
    let cmd = $list_clients_output | str trim | str downcase
    
    # Split the command into parts
    let parts = $cmd | split row " "
    
    # Check if any part ends with 'hx', 'helix' or is 'hx', 'helix'
    let has_hx_paths = ($parts | any {|part| ($part | str ends-with "/hx")})
    let has_helix_paths = ($parts | any {|part| ($part | str ends-with "/helix")})
    let has_hx = $has_hx_paths or $has_helix_paths
    
    let is_hx_cmd = ($parts | any {|part| $part == "hx"})
    let is_helix_cmd = ($parts | any {|part| $part == "helix"})
    let is_hx = $is_hx_cmd or $is_helix_cmd
    
    let has_or_is_hx = $has_hx or $is_hx
    
    # Find the position of 'hx' or 'helix' in the parts
    let hx_positions = ($parts | enumerate | where {|x| 
        (($x.item == "hx") or ($x.item == "helix") or 
         ($x.item | str ends-with "/hx") or ($x.item | str ends-with "/helix"))
    } | get index)
    
    # Check if 'hx' or 'helix' is the first part or right after a path
    let is_hx_at_start = if ($hx_positions | is-empty) {
        false
    } else {
        let hx_position = $hx_positions.0
        $hx_position == 0 or ($hx_position > 0 and ($parts | get ($hx_position - 1) | str ends-with "/"))
    }
    
    let result = $has_or_is_hx and $is_hx_at_start
    
    # Debug info
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
