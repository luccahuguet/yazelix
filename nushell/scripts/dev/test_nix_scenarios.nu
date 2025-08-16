#!/usr/bin/env nu
# Test script for different Nix installation scenarios

use ../utils/nix_detector.nu *

def test_scenario [name: string, path_modification: closure] {
    print $"🧪 Testing scenario: ($name)"
    print "=" * 50
    
    # Modify environment and test
    with-env (do $path_modification) {
        let nix_status = check_nix_installation
        
        print $"  Installed: ($nix_status.installed)"
        print $"  Error: ($nix_status.error)"
        print $"  Message: ($nix_status.message)"
        print ""
        
        if not $nix_status.installed or ($nix_status.error | is-not-empty) {
            print "Error message that would be shown to user:"
            print "-" * 30
            show_nix_installation_help $nix_status.error
        } else {
            print "✅ Nix detected successfully!"
        }
    }
    
    print ""
    print ""
}

def main [] {
    print "🧪 Testing Nix detector with different scenarios..."
    print ""
    
    # Test 1: Normal case (Nix installed)
    test_scenario "Normal - Nix installed" {|| {}}
    
    # Test 2: Nix not in PATH
    test_scenario "Nix not in PATH" {|| {
        PATH: ($env.PATH | where $it !~ "nix")
    }}
    
    # Test 3: Empty PATH (extreme case)
    test_scenario "Empty PATH" {|| {
        PATH: []
    }}
}