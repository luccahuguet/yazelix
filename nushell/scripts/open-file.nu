#!/usr/bin/env nu
# Simple wrapper for open_file function

use integrations/yazi.nu open_file

def main [file_path: path] {
    open_file $file_path
} 