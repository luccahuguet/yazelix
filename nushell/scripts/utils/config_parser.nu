#!/usr/bin/env nu
# Configuration parser for yazelix.nix files

# Parse yazelix configuration file and extract persistent session settings
export def parse_yazelix_config [] {
    # Read configuration directly from yazelix.nix file
    let yazelix_dir = "~/.config/yazelix" | path expand
    let config_file = ($yazelix_dir | path join "yazelix.nix")
    let default_config_file = ($yazelix_dir | path join "yazelix_default.nix")
    
    # Determine which config file to use
    let config_to_read = if ($config_file | path exists) {
        $config_file
    } else if ($default_config_file | path exists) {
        $default_config_file
    } else {
        error make {msg: "No yazelix configuration file found"}
    }
    
    # Parse the configuration file to extract persistent session settings
    let config_content = (open $config_to_read | str trim)
    
    # Extract persistent_sessions setting
    let persistent_sessions = if ($config_content | str contains "persistent_sessions = true") {
        "true"
    } else {
        "false"
    }
    
    # Extract session_name setting
    let session_name = if ($config_content | str contains "session_name =") {
        $config_content 
        | lines 
        | where ($it | str contains "session_name =") 
        | first 
        | str replace "session_name = " "" 
        | str replace -a "\"" "" 
        | str replace ";" ""
        | str trim
    } else {
        "yazelix"
    }
    
    {
        persistent_sessions: $persistent_sessions,
        session_name: $session_name,
        config_file: $config_to_read
    }
} 