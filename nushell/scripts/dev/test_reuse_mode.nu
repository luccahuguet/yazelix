#!/usr/bin/env nu
# Regression tests for stale profile reuse in --reuse command paths.
# Regression-only default-suite entrypoint justified in
# nushell/scripts/dev/validate_default_test_traceability.nu.

use ../utils/launch_state.nu [get_launch_profile require_reused_launch_profile resolve_built_profile]

def assert [condition: bool, message: string] {
    if not $condition {
        error make {msg: $message}
    }
}

def test_reuse_mode_uses_last_built_profile [] {
    print "🧪 Testing --reuse stale profile reuse..."

    let expected_profile = (resolve_built_profile)
    let stale_state = {needs_refresh: true}
    let fresh_state = {needs_refresh: false}
    let default_stale_profile = (get_launch_profile $stale_state)
    let fast_profile = (get_launch_profile $stale_state --allow-stale)
    let fresh_profile = (get_launch_profile $fresh_state)
    let required_profile = (require_reused_launch_profile $stale_state "yzx launch --reuse")

    assert (($expected_profile | is-not-empty) and ($expected_profile | path exists)) "Expected an existing built profile for reuse mode test"
    assert ($default_stale_profile == null) "Expected stale profile lookup to be disabled by default"
    assert ($fast_profile == $expected_profile) $"Expected --allow-stale to reuse ($expected_profile), got ($fast_profile)"
    assert ($fresh_profile == $expected_profile) $"Expected fresh profile lookup to reuse ($expected_profile), got ($fresh_profile)"
    assert ($required_profile == $expected_profile) $"Expected fast profile requirement to reuse ($expected_profile), got ($required_profile)"

    print "  ✅ --reuse reuses the last built profile when hashes are stale"
    true
}

def main [] {
    print "=== Testing Reuse Mode Profile Reuse ==="
    print ""

    let results = [
        (test_reuse_mode_uses_last_built_profile)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All reuse mode tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some reuse mode tests failed \(($passed)/($total)\)"
        exit 1
    }
}
