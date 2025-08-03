#!/usr/bin/env nu

# Helix-Yazi file picker integration
# Usage: Call this from Helix with the current buffer path as argument
# This script writes the selected file to /tmp/yazi-helix-chooser for Helix to read

def main [
    current_file?: string  # Current buffer file path (optional)
] {
    # Basic file picker functionality
    
    # Use fixed temp file that Helix expects
    let chooser_file = "/tmp/yazi-helix-chooser"
    
    # Determine starting directory
    let start_path = if ($current_file | is-empty) {
        pwd
    } else if ($current_file | path exists) {
        if ($current_file | path type) == "file" {
            $current_file | path dirname
        } else {
            $current_file
        }
    } else {
        # If file doesn't exist yet, use its parent directory
        let parent = $current_file | path dirname
        if ($parent | path exists) {
            $parent
        } else {
            pwd
        }
    }
    
    # Debug: print what we're about to run
    # print $"Running: yazi ($start_path) --chooser-file ($chooser_file)"
    
    # Clean up any existing chooser file
    if ($chooser_file | path exists) {
        rm $chooser_file
    }
    
    # Launch Yazi with proper argument order
    if ($start_path | path exists) {
        run-external "yazi" $start_path "--chooser-file" $chooser_file
    } else {
        # Fallback to current directory if path doesn't exist
        run-external "yazi" "." "--chooser-file" $chooser_file
    }
    
    # Basic file picker functionality only
}