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

# Get the Helix binary path (patchy version if available, otherwise hx)
export def get_helix_binary [] {
    # Check if patchy is currently enabled
    let use_patchy = ($env.YAZELIX_USE_PATCHY_HELIX? | default "false") == "true"
    
    if $use_patchy and ($env.YAZELIX_PATCHY_HX? | is-not-empty) and ($env.YAZELIX_PATCHY_HX | path exists) {
        # Set runtime for patchy when using the binary
        let patchy_runtime = $"($env.HOME)/.config/yazelix/helix_patchy/runtime"
        $env.HELIX_RUNTIME = $patchy_runtime
        $env.YAZELIX_PATCHY_HX
    } else {
        "hx"
    }
}