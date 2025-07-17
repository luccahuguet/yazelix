#!/usr/bin/env nu
# Dynamic file opener that respects the configured editor
# This script is called by Yazi to open files

# Ensure we have the Yazelix environment available
# Source the nix shell environment if YAZELIX_HELIX_MODE is not set
if ($env.YAZELIX_HELIX_MODE? | is-empty) {
    print "Environment not loaded, loading from yazelix.nix configuration..."
    use ../utils/helix_mode.nu set_helix_env
    set_helix_env
}

use ./yazi.nu open_file_with_editor

def main [file_path: path] {
    print $"DEBUG: Opening file ($file_path) with EDITOR=($env.EDITOR? | default 'not set')"
    open_file_with_editor $file_path
}