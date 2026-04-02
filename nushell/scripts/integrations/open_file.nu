#!/usr/bin/env nu
# Dynamic file opener that respects the configured editor
# This script is called by Yazi to open files

use ./yazi.nu open_file_with_editor

def main [file_path: path] {
    print $"DEBUG: Opening file ($file_path) with EDITOR=($env.EDITOR? | default 'not set')"
    open_file_with_editor $file_path
}
