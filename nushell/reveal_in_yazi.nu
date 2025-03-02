#!/usr/bin/env nu

# Navigate Yazi to the directory of the current Helix buffer
def main [buffer_name: string] {
    # Define log file path with ~ expanded
    let log_dir = ($nu.home-path | path join ".config/yazelix/logs" | path expand)
    let log_file = ($log_dir | path join "reveal_in_yazi.log")
    
    # Ensure log directory exists
    mkdir $log_dir
    
    # Log function to append to file with timestamp and newline
    def log [message: string] {
        let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
        # Use \n to ensure each log entry is on a new line
        $"[($timestamp)] ($message)\n" | save -a $log_file
    }

    # Log script start
    log $"Starting reveal_in_yazi.nu with buffer_name: ($buffer_name)"

    # Validate the buffer name is provided
    if ($buffer_name | is-empty) {
        log "Error: Buffer name not provided"
        print "Error: Buffer name not provided"
        return
    }
    log $"Buffer name validated: ($buffer_name)"

    # Normalize buffer_name by expanding ~ if present
    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }
    log $"Normalized buffer_name: ($normalized_buffer_name)"

    # Resolve the full path based on normalized_buffer_name
    # - If absolute, use it directly
    # - If relative, try to resolve using PWD first, then fall back to YAZELIX_INITIAL_PATH if needed
    let full_path = if ($normalized_buffer_name | path type) == "absolute" {
        $normalized_buffer_name
    } else {
        # Try resolving relative to PWD first (more likely to reflect current buffer context)
        let pwd_path = ($env.PWD | path join $normalized_buffer_name | path expand)
        log $"Trying to resolve relative path using PWD: ($env.PWD)"
        if ($pwd_path | path exists) {
            $pwd_path
        } else if ($env.YAZELIX_INITIAL_PATH | is-not-empty) {
            # Fall back to YAZELIX_INITIAL_PATH if PWD doesn’t work
            let initial_dir = ($env.YAZELIX_INITIAL_PATH | path dirname)
            log $"Initial path resolution failed, falling back to initial path directory: ($initial_dir)"
            ($initial_dir | path join $normalized_buffer_name | path expand)
        } else {
            # Fallback if neither PWD nor YAZELIX_INITIAL_PATH works
            log "Falling back to default resolution (no PWD or YAZELIX_INITIAL_PATH)"
            ($nu.home-path | path join $normalized_buffer_name | path expand)
        }
    }
    log $"Resolved full path: ($full_path)"

    # Validate the resolved path exists
    if not ($full_path | path exists) {
        log $"Error: Resolved path ($full_path) does not exist"
        print $"Error: Resolved path '($full_path)' does not exist"
        return
    }
    log $"Path exists, extracted directory: '($full_path | path dirname)'"

    let dir = ($full_path | path dirname)

    # Check YAZI_ID
    if ($env.YAZI_ID | is-empty) {
        log "Error: YAZI_ID not set"
        print "Error: YAZI_ID not set. Ensure Yazi is running and open_file.nu set it."
        return
    }
    log $"YAZI_ID found: ($env.YAZI_ID)"

    # Navigate Yazi to the directory
    log $"Navigating Yazi to directory: ($dir)"
    ya emit-to $env.YAZI_ID cd $dir
    log "Yazi navigation completed successfully"
}

# Ensure log directory exists on script load, with ~ expanded
let log_dir = ($nu.home-path | path join ".config/yazelix/logs" | path expand)
mkdir $log_dir
