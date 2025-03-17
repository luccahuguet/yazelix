#!/usr/bin/env nu
# ~/.config/yazelix/nushell/reveal_in_yazi.nu

source ~/.config/yazelix/nushell/logging.nu  # Import logging module

# Navigate Yazi to the directory of the current Helix buffer
def main [buffer_name: string] {
    log_to_file "reveal_in_yazi.log" $"Starting reveal_in_yazi.nu with buffer_name: ($buffer_name)"
    
    if ($buffer_name | is-empty) {
        log_to_file "reveal_in_yazi.log" "Error: Buffer name not provided"
        print "Error: Buffer name not provided"
        return
    }
    
    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }
    
    log_to_file "reveal_in_yazi.log" $"Trying to resolve path using PWD: ($env.PWD)"
    let full_path = ($env.PWD | path join $normalized_buffer_name | path expand)
    log_to_file "reveal_in_yazi.log" $"Resolved full path: ($full_path)"
    
    if not ($full_path | path exists) {
        log_to_file "reveal_in_yazi.log" $"Error: Resolved path ($full_path) does not exist"
        print $"Error: Resolved path '($full_path)' does not exist"
        return
    }
    
    log_to_file "reveal_in_yazi.log" $"Path exists, extracted directory: '($full_path | path dirname)'"
    let dir = ($full_path | path dirname)
    
    if ($env.YAZI_ID | is-empty) {
        log_to_file "reveal_in_yazi.log" "Error: YAZI_ID not set"
        print "Error: YAZI_ID not set. reveal-in-yazi requires that you open helix from yazelix's yazi."
        return
    }
    
    log_to_file "reveal_in_yazi.log" $"YAZI_ID found: ($env.YAZI_ID)"
    log_to_file "reveal_in_yazi.log" $"Navigating Yazi to directory: ($dir)"
    
    ya emit-to $env.YAZI_ID cd $dir
    zellij action move-focus left
    
    log_to_file "reveal_in_yazi.log" "Yazi navigation completed successfully"
}
