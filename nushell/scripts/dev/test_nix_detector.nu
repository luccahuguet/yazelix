#!/usr/bin/env nu
# Test script for the Nix detector

use ../utils/nix_detector.nu *

def main [] {
    print "🧪 Testing Nix detector..."
    print ""
    
    let nix_status = check_nix_installation
    
    print "Nix detection results:"
    print $"  Installed: ($nix_status.installed)"
    print $"  Error: ($nix_status.error)"
    print $"  Message: ($nix_status.message)"
    print ""
    
    if $nix_status.installed and ($nix_status.error | is-empty) {
        print "✅ Nix is properly configured!"
    } else {
        print "❌ Nix issues detected. Here's what the user would see:"
        print ""
        show_nix_installation_help $nix_status.error
    }
}