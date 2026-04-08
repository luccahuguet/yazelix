#!/usr/bin/env nu
# Dynamic file opener that respects the configured editor
# This script is called by Yazi to open files

use ./managed_editor.nu open_file_with_editor

def main [file_path: path] {
    open_file_with_editor $file_path
}
