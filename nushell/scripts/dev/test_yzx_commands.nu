#!/usr/bin/env nu
# Core regression runner for high-signal yzx CLI contracts
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/specs/floating_tui_panes.md
# Defends: docs/workspace_session_contract.md

use ./yzx_test_helpers.nu [format_test_profile_report setup_test_home test_profiling_enabled]
use ./test_yzx_core_commands.nu [run_core_canonical_tests]
use ./test_yzx_doctor_commands.nu [run_doctor_canonical_tests]
use ./test_yzx_generated_configs.nu [run_generated_config_canonical_tests]
use ./test_yzx_popup_commands.nu [run_popup_canonical_tests]
use ./test_yzx_refresh_commands.nu [run_refresh_canonical_tests]
use ./test_yzx_screen_commands.nu [run_screen_canonical_tests]
use ./test_yzx_workspace_commands.nu [run_workspace_canonical_tests]
use ./test_yzx_yazi_commands.nu [run_yazi_canonical_tests]

def build_profiled_suite_result [name: string, runner: closure] {
    let started = (date now)
    let results = (do $runner)
    let elapsed = ((date now) - $started)

    {
        name: $name
        elapsed_ms: ($elapsed / 1ms)
        results: $results
    }
}

export def run_default_canonical_suite [--profile] {
    let fixture = (setup_test_home)
    let profiling = if $profile {
        test_profiling_enabled --profile
    } else {
        test_profiling_enabled
    }
    let suite_results = (with-env {
        HOME: $fixture.tmp_home
        YAZELIX_RUNTIME_DIR: null
        YAZELIX_DIR: null
    } {
        [
            (build_profiled_suite_result "core" { run_core_canonical_tests })
            (build_profiled_suite_result "doctor" { run_doctor_canonical_tests })
            (build_profiled_suite_result "generated" { run_generated_config_canonical_tests })
            (build_profiled_suite_result "popup" { run_popup_canonical_tests })
            (build_profiled_suite_result "refresh" { run_refresh_canonical_tests })
            (build_profiled_suite_result "screen" { run_screen_canonical_tests })
            (build_profiled_suite_result "workspace" { run_workspace_canonical_tests })
            (build_profiled_suite_result "yazi" { run_yazi_canonical_tests })
        ]
    })
    rm -rf $fixture.tmp_home

    let results = ($suite_results | each {|suite| $suite.results } | flatten)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    {
        profiling: $profiling
        suite_results: $suite_results
        results: $results
        passed: $passed
        total: $total
    }
}

def main [--profile] {
    print "=== Testing core yzx contracts ==="
    print ""

    let suite_run = if $profile {
        run_default_canonical_suite --profile
    } else {
        run_default_canonical_suite
    }

    print ""
    if $suite_run.profiling {
        print (format_test_profile_report $suite_run.suite_results "=== test_yzx_commands.nu profile ===")
        print ""
    }

    if $suite_run.passed == $suite_run.total {
        print $"✅ All core yzx tests passed \(($suite_run.passed)/($suite_run.total)\)"
    } else {
        print $"❌ Some core yzx tests failed \(($suite_run.passed)/($suite_run.total)\)"
        error make { msg: "core yzx tests failed" }
    }
}
