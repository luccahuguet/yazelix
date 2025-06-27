#!/usr/bin/env nu
# Generate version table for Yazelix
# Usage: nu generate-version-table.nu [--save]

use utils/version-info.nu

def main [--save (-s)] {
    print "🔄 Generating Yazelix version table..."
    
    let output = (version-info)
    
    if $save {
        let table_file = "docs/table_of_versions.md"
        $output | save $table_file
        print $"✅ Version table saved to ($table_file)"
    } else {
        print $output
    }
} 