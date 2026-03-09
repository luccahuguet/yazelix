#!/usr/bin/env nu
# Regression tests for build parallelism propagation into devenv command construction.

use ../utils/common.nu [get_max_cores get_max_jobs describe_build_parallelism get_yazelix_nix_config]
use ../utils/environment_bootstrap.nu [get_devenv_base_command]

def assert [condition: bool, message: string] {
    if not $condition {
        error make {msg: $message}
    }
}

def test_explicit_numeric_build_cores [] {
    print "🧪 Testing explicit numeric build_cores..."

    let cmd = (get_devenv_base_command --max-jobs "2" --build-cores "1")
    let cores_index = ($cmd | enumerate | where item == "--cores" | get 0.index)
    let resolved_cores = ($cmd | get ($cores_index + 1))
    let jobs_index = ($cmd | enumerate | where item == "--max-jobs" | get 0.index)
    let resolved_jobs = ($cmd | get ($jobs_index + 1))

    assert ($resolved_cores == "1") $"Expected --cores 1, got ($resolved_cores)"
    assert ($resolved_jobs == "2") $"Expected --max-jobs 2, got ($resolved_jobs)"
    print "  ✅ Numeric build_cores is preserved in devenv command"
    true
}

def test_symbolic_build_cores [] {
    print "🧪 Testing symbolic build_cores..."

    let expected = (get_max_cores "half" | into string)
    let expected_jobs = (get_max_jobs "quarter" | into string)
    let cmd = (get_devenv_base_command --max-jobs "quarter" --build-cores "half")
    let cores_index = ($cmd | enumerate | where item == "--cores" | get 0.index)
    let resolved_cores = ($cmd | get ($cores_index + 1))
    let jobs_index = ($cmd | enumerate | where item == "--max-jobs" | get 0.index)
    let resolved_jobs = ($cmd | get ($jobs_index + 1))

    assert ($resolved_cores == $expected) $"Expected --cores ($expected), got ($resolved_cores)"
    assert ($resolved_jobs == $expected_jobs) $"Expected --max-jobs ($expected_jobs), got ($resolved_jobs)"
    print "  ✅ Symbolic build_cores resolves through shared parser"
    true
}

def test_build_parallelism_description [] {
    print "🧪 Testing build parallelism display text..."

    let description = (describe_build_parallelism "2" "half")

    assert ($description | str contains "jobs x") $"Expected jobs in description, got ($description)"
    assert ($description | str contains "build_cores=2") $"Expected description to mention build_cores=2, got ($description)"
    assert ($description | str contains "max_jobs=half") $"Expected description to mention max_jobs=half, got ($description)"
    print "  ✅ Build parallelism display text includes jobs and cores"
    true
}

def test_devenv_command_includes_numtide_cache [] {
    print "🧪 Testing numtide cache injection into devenv command..."

    let cmd = (get_devenv_base_command --max-jobs "2" --build-cores "1")
    let nix_config_arg = ($cmd | where {|item| ($item | into string) | str starts-with "NIX_CONFIG=" } | first)
    let expected_nix_config = (get_yazelix_nix_config)

    assert (($nix_config_arg | into string) | str contains "https://cache.numtide.com") $"Expected NIX_CONFIG to include cache.numtide.com, got ($nix_config_arg)"
    assert (($nix_config_arg | str replace "NIX_CONFIG=" "") == $expected_nix_config) "Expected devenv command to propagate the canonical Yazelix Nix config"
    print "  ✅ devenv command includes numtide cache settings"
    true
}

def main [] {
    print "=== Testing Build Parallelism Propagation ==="
    print ""

    let results = [
        (test_explicit_numeric_build_cores),
        (test_symbolic_build_cores),
        (test_build_parallelism_description),
        (test_devenv_command_includes_numtide_cache)
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
