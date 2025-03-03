#!/usr/bin/env nu
# ~/.config/yazelix/nushell/reveal_in_yazi.nu

# Navigate Yazi to the directory of the current Helix buffer
def main [buffer_name: string] {
    # Define log file path with ~ expanded
    let log_dir = ($nu.home-path | path join ".config/yazelix/logs" | path expand)
    let log_file = ($log_dir | path join "reveal_in_yazi.log")
    
    # Trim log file to last 1000 lines if it exceeds 0.5 MB
    if ($log_file | path exists) and ((ls $log_file).size.0 > 0.5mb) {
        open $log_file | lines | last 1000 | save -f $log_file
    }
    
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

    # Normalize buffer_name by expanding ~ if present
    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }

    # Resolve the full path using PWD (Helix’s working directory)
    log $"Trying to resolve path using PWD: ($env.PWD)"
    let full_path = ($env.PWD | path join $normalized_buffer_name | path expand)
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
