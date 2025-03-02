#!/usr/bin/env nu

# Navigate Yazi to the directory of the current Helix buffer
def main [buffer_name: string] {
    # Define log file path with ~ expanded
    let log_dir = ($nu.home-path | path join ".config/yazelix/logs" | path expand)
    let log_file = ($log_dir | path join "reveal_in_yazi.log")
    
    # Ensure log directory exists
    mkdir $log_dir
    
    # Log function to append to file with timestamp
    def log [message: string] {
        let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
        $"[$timestamp] ($message)" | save -a $log_file
    }

    # Log script start
    log "Starting reveal_in_yazi.nu with buffer_name: ($buffer_name)"

    # Validate the buffer name is provided
    if ($buffer_name | is-empty) {
        log "Error: Buffer name not provided"
        print "Error: Buffer name not provided"
        return
    }
    log "Buffer name validated: ($buffer_name)"

    # Resolve the full path based on buffer_name
    # - If absolute, use it directly
    # - If relative, try initial path from open_file.nu, then fall back to PWD
    let full_path = if ($buffer_name | path type) == "absolute" {
        $buffer_name
    } else if ($env.YAZELIX_INITIAL_PATH | is-not-empty) {
        # Use the initial path’s directory as context for relative paths
        let initial_dir = ($env.YAZELIX_INITIAL_PATH | path dirname)
        log $"Resolving relative path using initial path directory: ($initial_dir)"
        ($initial_dir | path join $buffer_name | path expand)
    } else {
        # Fallback to current working directory (less reliable)
        log "Falling back to PWD for path resolution"
        ($env.PWD | path join $buffer_name | path expand)
    }
    log $"Resolved full path: ($full_path)"

    # Validate the resolved path exists
    if not ($full_path | path exists) {
        log $"Error: Resolved path ($full_path) does not exist"
        print $"Error: Resolved path ($full_path) does not exist"
        return
    }
    log "Path exists, extracted directory: ($full_path | path dirname)"

    let dir = ($full_path | path dirname)

    # Check YAZI_ID
    if ($env.YAZI_ID | is-empty) {
        log "Error: YAZI_ID not set"
        print "Error: YAZI_ID not set. Ensure Yazi is running and open_file.nu set it."
        return
    }
    log "YAZI_ID found: ($env.YAZI_ID)"

    # Navigate Yazi to the directory
    log $"Navigating Yazi to directory: ($dir)"
    ya emit-to $env.YAZI_ID cd $dir
    log "Yazi navigation completed successfully"
}

# Ensure log directory exists on script load, with ~ expanded
let log_dir = ($nu.home-path | path join ".config/yazelix/logs" | path expand)
mkdir $log_dir
