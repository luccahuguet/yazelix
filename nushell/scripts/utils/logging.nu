#!/usr/bin/env nu
# ~/.config/yazelix/nushell/logging.nu

# Logging utility module

export def log_to_file [log_name: string, message: string] {
    let log_dir = ($env.HOME | path join ".config/yazelix/logs" | path expand)
    let log_file = ($log_dir | path join $log_name)
    
    # Line-based trimming for different log types
    let max_lines = if $log_name == "open_helix.log" { 500 } else { 1000 }
    
    if ($log_file | path exists) {
        let current_lines = (open $log_file | lines | length)
        if $current_lines > $max_lines {
            open $log_file | lines | last $max_lines | save -f $log_file
        }
    }
    
    mkdir $log_dir
    
    let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
    $"[($timestamp)] ($message)\n" | save -a $log_file
}