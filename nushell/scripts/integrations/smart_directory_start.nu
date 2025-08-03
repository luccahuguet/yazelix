#!/usr/bin/env nu
# Smart directory selection using zoxide database for Yazelix

# Get the most frequently accessed directory from zoxide, excluding config directories
export def get_smart_directory [] {
    try {
        # Get zoxide database entries, excluding yazelix config and other config dirs
        let zoxide_entries = (zoxide query --list | lines | where $it !~ "config" | where $it !~ "cache")
        
        # If we have entries, return the most accessed one
        if ($zoxide_entries | length) > 0 {
            let top_dir = ($zoxide_entries | first)
            if ($top_dir | path exists) {
                return $top_dir
            }
        }
        
        # Fallback to home directory if no suitable entries found
        $env.HOME
    } catch {
        # If zoxide fails, fallback to home directory
        $env.HOME
    }
}

# Get the smart directory and output it for use in shell commands
def main [] {
    get_smart_directory
}