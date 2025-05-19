#!/usr/bin/env nu
# ~/.config/yazelix/nushell/logging.nu

# Logging utility module

export def log_to_file [log_name: string, message: string] {
    let log_dir = ($nu.home-path | path join ".config/yazelix/logs" | path expand)
    let log_file = ($log_dir | path join $log_name)
    
    if ($log_file | path exists) and ((ls $log_file).size.0 > 0.5mb) {
        open $log_file | lines | last 1000 | save -f $log_file
    }
    
    mkdir $log_dir
    
    let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
    $"[($timestamp)] ($message)\n" | save -a $log_file
}

let log_dir = ($nu.home-path | path join ".config/yazelix/logs" | path expand)
mkdir $log_dir
