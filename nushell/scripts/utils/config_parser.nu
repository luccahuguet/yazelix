#!/usr/bin/env nu
# Configuration parser for yazelix.nix files

# Extract a simple string value using line parsing (last assignment wins)
def extract_config_value [key: string, default: string, config_content: string] {
    # Find non-commented lines containing the key assignment
    let matching_lines = ($config_content 
        | lines 
        | where not ($it | str trim | str starts-with "#")  # Exclude comments first
        | where ($it | str contains $key)                   # Then check if line contains the key
        | where ($it | str contains "="))                   # And has an assignment
    
    if ($matching_lines | is-empty) {
        $default
    } else {
        # Take the last match (in case there are multiple, use the final one)
        let line = ($matching_lines | last)
        let value_part = ($line | split row '=' | get 1 | str trim)
        
        # Clean up the value (remove quotes, semicolons, etc.)
        $value_part 
        | str replace -a '"' '' 
        | str replace -a "'" ''
        | str replace ';' ''
        | str trim
    }
}

# Extract the first uncommented item from a list assignment like:
# key = [
#   "option1"
#   # "option2"
# ]
def extract_first_list_value [key: string, default: string, config_content: string] {
    let lines = ($config_content | lines)
    # Find start of list for the key
    let start_idx = ($lines | enumerate | where {|it| ($it.item | str contains $key) and ($it.item | str contains "=") and ($it.item | str contains "[") } | get index? | default -1)
    if $start_idx == -1 {
        $default
    } else {
        # From start line onward, gather until closing bracket
        let tail = ($lines | skip ($start_idx))
        mut items = []
        for l in $tail {
            let t = ($l | str trim)
            if ($t | str contains "]") {
                break
            }
            if ($t | str starts-with "#") {
                continue
            }
            # Capture first quoted token on the line
            let captured = ($t | parse '"{val}"' | get val? | default [])
            if (not ($captured | is-empty)) {
                $items = ($items | append ($captured | first))
            }
        }
        if ($items | is-empty) { $default } else { ($items | first) }
    }
}

# Parse yazelix configuration file and extract settings
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

    # Parse the configuration file
    let config_content = (open $config_to_read)

    # Extract all configuration values using helpers
    # For cursor_trail: support either a string assignment or a list; list takes precedence
    let cursor_trail_list_first = (extract_first_list_value "cursor_trail" "" $config_content)
    let cursor_trail_value = if $cursor_trail_list_first == "" {
        extract_config_value "cursor_trail" "blaze" $config_content
    } else {
        $cursor_trail_list_first
    }

    {
        persistent_sessions: (extract_config_value "persistent_sessions" "false" $config_content),
        session_name: (extract_config_value "session_name" "yazelix" $config_content),
        preferred_terminal: (extract_config_value "preferred_terminal" "ghostty" $config_content),
        extra_terminals: (extract_config_value "extra_terminals" "[]" $config_content),
        cursor_trail: $cursor_trail_value,
        transparency: (extract_config_value "transparency" "low" $config_content),
        default_shell: (extract_config_value "default_shell" "nu" $config_content),
        helix_mode: (extract_config_value "helix_mode" "release" $config_content),
        config_file: $config_to_read
    }
}
