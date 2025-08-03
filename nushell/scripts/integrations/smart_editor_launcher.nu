#!/usr/bin/env nu
# Smart editor launcher with directory selection and tab renaming

use smart_directory_start.nu get_smart_directory

# Get tab name from directory (same logic as yazi.nu)
def get_tab_name [working_dir: path] {
    try {
        let git_root = (git rev-parse --show-toplevel | str trim)
        if ($git_root | is-not-empty) and (not ($git_root | str starts-with "fatal:")) {
            $git_root | path basename
        } else {
            let basename = ($working_dir | str trim | path basename)
            if ($basename | is-empty) {
                "unnamed"
            } else {
                $basename
            }
        }
    } catch {
        $working_dir | path basename
    }
}

def main [] {
    # Check if smart directory start is enabled
    let smart_start = ($env.YAZELIX_SMART_DIRECTORY_START? | default "true") == "true"
    
    # Get target directory
    let target_dir = if $smart_start {
        get_smart_directory
    } else {
        pwd
    }
    
    # Change to target directory
    cd $target_dir
    
    # Get and set tab name
    let tab_name = (get_tab_name $target_dir)
    zellij action rename-tab $tab_name
    
    # Show info message if using smart start
    if $smart_start {
        print $"🎯 Opening editor in smart directory: ($target_dir)"
    }
    
    # Start the editor
    exec $env.EDITOR
}