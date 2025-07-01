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

# Get the Helix binary path (custom version if available, otherwise hx)
export def get_helix_binary [] {
    # Check if custom helix is currently enabled
    let use_custom_helix = ($env.YAZELIX_HELIX_MODE? | default "default") in ["steel", "source"]
    
    if $use_custom_helix and ($env.YAZELIX_CUSTOM_HELIX? | is-not-empty) and ($env.YAZELIX_CUSTOM_HELIX | path exists) {
        # Set runtime for custom build when using the binary
        let custom_runtime = $"($env.HOME)/.config/yazelix/helix_custom/runtime"
        $env.HELIX_RUNTIME = $custom_runtime
        $env.YAZELIX_CUSTOM_HELIX
    } else {
        "hx"
    }
}