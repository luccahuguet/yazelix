#!/usr/bin/env nu
# Helix integration utilities for Yazelix

# Test if Helix is running and working properly
export def is_helix_running_test [] {
    print "🔍 Testing Helix integration..."
    
    # Test basic hx command
    try {
        let helix_version = (hx --version | lines | first)
        print $"✅ Helix found: ($helix_version)"
    } catch {
        print "❌ Helix command failed"
        return
    }
    
    # Test Zellij integration
    try {
        let zellij_clients = (zellij list-clients)
        print $"✅ Zellij clients: ($zellij_clients | length) active"
    } catch {
        print "⚠️  Zellij not running or accessible"
    }
    
    print "✅ Helix integration test completed"
}

# Get the Helix binary name (always hx since Nix provides it)
export def get_helix_binary [] {
    "hx"
}