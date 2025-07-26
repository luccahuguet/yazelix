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

    # Extract persistent_sessions setting - properly parse the value with flexible spacing
    # Find the first non-commented line containing persistent_sessions
    let persistent_sessions_line = $config_content
        | lines
        | where ($it | str contains "persistent_sessions")
        | where not ($it | str trim | str starts-with "#")  # Skip commented lines
        | first

    let persistent_sessions = if ($persistent_sessions_line | is-empty) {
        "false"  # Default to false if not found
    } else {
        # Handle various spacing patterns: "persistent_sessions = true", "persistent_sessions=true", etc.
        let value = ($persistent_sessions_line
            | str replace "persistent_sessions = " ""
            | str replace "persistent_sessions=" ""
            | str replace ";" ""
            | str trim)
        if ($value == "true") {
            "true"
        } else {
            "false"
        }
    }

    # Extract session_name setting with flexible spacing
    # Find the first non-commented line containing session_name
    let session_name_line = $config_content
        | lines
        | where ($it | str contains "session_name")
        | where not ($it | str trim | str starts-with "#")  # Skip commented lines
        | first

    let session_name = if ($session_name_line | is-empty) {
        "yazelix"  # Default session name
    } else {
        # Handle various spacing patterns: "session_name = \"value\"", "session_name=\"value\"", etc.
        $session_name_line
            | str replace "session_name = " ""
            | str replace "session_name=" ""
            | str replace -a "\"" ""
            | str replace ";" ""
            | str trim
    }

    {
        persistent_sessions: $persistent_sessions,
        session_name: $session_name,
        config_file: $config_to_read
    }
}