#!/usr/bin/env nu
# ~/.config/yazelix/nushell/logging.nu

# Logging utility module

export def log_to_file [log_name: string, message: string] {
    use ./constants.nu YAZELIX_LOGS_DIR
    let log_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    let log_file = ($log_dir | path join $log_name)
    
    if ($log_file | path exists) and ((ls $log_file).size.0 > 0.5mb) {
        open $log_file | lines | last 1000 | save -f $log_file
    }
    
    mkdir $log_dir
    
    let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
    $"[($timestamp)] ($message)\n" | save -a $log_file
}