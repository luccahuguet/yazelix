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

# Get the Helix binary path (both modes use hx from PATH)
export def get_helix_binary [] {
    # Only return EDITOR if it's actually Helix, fallback to 'hx' for safety
    let editor = $env.EDITOR
    let is_helix = ($editor | str ends-with "/hx") or ($editor == "hx") or ($editor | str ends-with "/helix") or ($editor == "helix")
    if $is_helix {
        $editor
    } else {
        "hx"  # Fallback for non-Helix editors
    }
}