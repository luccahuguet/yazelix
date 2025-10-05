#!/usr/bin/env nu
# Test integration of Nix detector with main scripts

def test_with_nix [] {
    print "🧪 Testing yzx start with Nix available..."

    # Test the ensure_nix_available function directly
    try {
        use ../utils/nix_detector.nu ensure_nix_available
        ensure_nix_available --non-interactive
        print "✅ Nix check passed - would proceed to start Yazelix"
    } catch { |err|
        print $"❌ Nix check failed: ($err.msg)"
    }
}

def test_without_nix [] {
    print "🧪 Testing yzx start without Nix in PATH..."

    # Test with modified PATH
    with-env {PATH: ($env.PATH | where $it !~ "nix")} {
        try {
            use ../utils/nix_detector.nu ensure_nix_available
            ensure_nix_available --non-interactive
            print "❌ This should not be reached!"
        } catch { |err|
            print "✅ Correctly failed with graceful error message (exit code 1 expected)"
        }
    }
}

def main [] {
    test_with_nix
    print ""
    test_without_nix
}