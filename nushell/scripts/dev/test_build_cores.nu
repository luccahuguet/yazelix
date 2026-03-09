#!/usr/bin/env nu
# Regression tests for build_cores propagation into devenv command construction.

use ../utils/common.nu [get_max_cores]
use ../utils/environment_bootstrap.nu [get_devenv_base_command]

def assert [condition: bool, message: string] {
    if not $condition {
        error make {msg: $message}
    }
}

def test_explicit_numeric_build_cores [] {
    print "🧪 Testing explicit numeric build_cores..."

    let cmd = (get_devenv_base_command --build-cores "1")
    let cores_index = ($cmd | enumerate | where item == "--cores" | get 0.index)
    let resolved_cores = ($cmd | get ($cores_index + 1))

    assert ($resolved_cores == "1") $"Expected --cores 1, got ($resolved_cores)"
    print "  ✅ Numeric build_cores is preserved in devenv command"
    true
}

def test_symbolic_build_cores [] {
    print "🧪 Testing symbolic build_cores..."

    let expected = (get_max_cores "half" | into string)
    let cmd = (get_devenv_base_command --build-cores "half")
    let cores_index = ($cmd | enumerate | where item == "--cores" | get 0.index)
    let resolved_cores = ($cmd | get ($cores_index + 1))

    assert ($resolved_cores == $expected) $"Expected --cores ($expected), got ($resolved_cores)"
    print "  ✅ Symbolic build_cores resolves through shared parser"
    true
}

def main [] {
    print "=== Testing Build Cores Propagation ==="
    print ""

    let results = [
        (test_explicit_numeric_build_cores),
        (test_symbolic_build_cores)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All build_cores tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some build_cores tests failed \(($passed)/($total)\)"
        exit 1
    }
}
