#!/usr/bin/env nu

def main [] {
    # Check if we're in a Yazi subshell with YAZI_ID
    let yazi_id = ($env.YAZI_ID? | default "")
    # print $"YAZI_ID: ($yazi_id)"  # Debug output
    
    # Get the zoxide path interactively
    let path = (zoxide query -i --exclude (pwd) | str trim)
    
    # Check if a path was selected (not null or empty)
    if not ($path | is-empty) and ($path | str length) > 0 {
        # Launch the new Yazi instance in the background
        nu -c $"yazi ($path)" &!
        # Small delay to ensure the new instance starts
        sleep 0ms
        # If YAZI_ID exists, tell the current instance to quit
        if not ($yazi_id | is-empty) {
            # print $"Quitting Yazi instance: ($yazi_id)"  # Debug output
            ya emit quit
        } else {
            # print "No YAZI_ID found, old instance may persist"
        }
    }
}
