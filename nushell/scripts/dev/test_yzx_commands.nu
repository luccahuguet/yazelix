#!/usr/bin/env nu
# Test runner for yzx CLI commands

use ./test_yzx_helpers.nu [setup_test_home]
use ./test_yzx_core_commands.nu [run_core_tests]
use ./test_yzx_dev_commands.nu [run_dev_tests]
use ./test_yzx_doctor_commands.nu [run_doctor_tests]
use ./test_yzx_generated_configs.nu [run_generated_config_tests]
use ./test_yzx_workspace_commands.nu [run_workspace_tests]
use ./test_yzx_yazi_commands.nu [run_yazi_tests]

def main [] {
    print "=== Testing yzx Commands ==="
    print ""

    let fixture = (setup_test_home)
    let results = (with-env { HOME: $fixture.tmp_home, YAZELIX_DIR: $fixture.config_dir } {
        [
            (run_core_tests)
            (run_dev_tests)
            (run_doctor_tests)
            (run_generated_config_tests)
            (run_workspace_tests)
            (run_yazi_tests)
        ] | flatten
    })
    rm -rf $fixture.tmp_home

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx command tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some tests failed \(($passed)/($total)\)"
        error make { msg: "yzx command tests failed" }
    }
}
