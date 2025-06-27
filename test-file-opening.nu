#!/usr/bin/env nu
# Test script for file opening functionality

use nushell/scripts/integrations/yazi.nu *

print "Testing file opening functionality..."
print $"Current directory: (pwd)"
print $"README.md exists: (README.md | path exists)"

if (README.md | path exists) {
    print "Attempting to open README.md..."
    open_file README.md
} else {
    print "README.md not found, creating test file..."
    "test content" | save test.txt
    open_file test.txt
} 