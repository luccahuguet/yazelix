#!/usr/bin/env nu
# Simple wrapper for open_file function
# This script is called by Yazi to open files

# Ensure we have the Yazelix environment available
# Source the nix shell environment if YAZELIX_HELIX_MODE is not set
if ($env.YAZELIX_HELIX_MODE? | is-empty) {
    print "Environment not loaded, loading from yazelix.nix configuration..."
    use ../utils/helix_mode.nu set_helix_env
    set_helix_env
}

use ./yazi.nu open_file

def main [file_path: path] {
    print $"DEBUG: Opening file ($file_path) with YAZELIX_HELIX_MODE=($env.YAZELIX_HELIX_MODE? | default 'not set')"
    open_file $file_path
}