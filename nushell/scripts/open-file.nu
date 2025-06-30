#!/usr/bin/env nu
# Simple wrapper for open_file function
# This script is called by Yazi to open files

# Ensure we have the Yazelix environment available
# Source the nix shell environment if YAZELIX_HELIX_MODE is not set
if ($env.YAZELIX_HELIX_MODE? | is-empty) {
    print "Environment not loaded, trying to load from nix shell..."
    # Try to source the nix environment by checking for common nix shell patterns
    let nix_shell_vars = {
        YAZELIX_HELIX_MODE: "steel",  # Default for Steel mode
        YAZELIX_PATCHY_HX: $"($env.HOME)/.config/yazelix/helix_patchy/target/release/hx"
    }
    
    # Set environment variables if they don't exist
    for var in ($nix_shell_vars | transpose key value) {
        if ($env | get --ignore-errors $var.key | is-empty) {
            load-env {($var.key): $var.value}
        }
    }
}

use integrations/yazi.nu open_file

def main [file_path: path] {
    print $"DEBUG: Opening file ($file_path) with YAZELIX_HELIX_MODE=($env.YAZELIX_HELIX_MODE? | default 'not set')"
    open_file $file_path
} 