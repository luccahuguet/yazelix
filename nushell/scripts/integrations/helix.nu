#!/usr/bin/env nu
# Helix integration utilities for Yazelix

# Test if Helix is running and working properly
export def is_helix_running_test [] {
    print "🔍 Testing Helix integration..."

    # Test basic helix command
    try {
        let helix_version = (hx --version | lines | first)
        print $"✅ Helix found: ($helix_version)"
    } catch {
        print "❌ Helix command failed"
        return false
    }

    # Test if we're in a proper environment
    if ($env.YAZI_ID | is-empty) {
        print "⚠️  YAZI_ID not set - you might not be in Yazelix environment"
    } else {
        print $"✅ YAZI_ID found: ($env.YAZI_ID)"
    }

    print "✅ Helix integration test completed"
    return true
}

# Get the preferred Helix binary name
export def get_helix_binary [] {
    if (which helix | is-not-empty) {
        "helix"
    } else if (which hx | is-not-empty) {
        "hx"
    } else {
        error make { msg: "Neither 'helix' nor 'hx' binary found" }
    }
}