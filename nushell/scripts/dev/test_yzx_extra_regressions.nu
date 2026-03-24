#!/usr/bin/env nu
# Small extra-regression runner for cheap non-core yzx checks

use ./test_yzx_helpers.nu [setup_test_home]
use ./test_yzx_core_commands.nu [run_core_noncanonical_tests]
use ./test_yzx_dev_commands.nu [run_dev_noncanonical_tests]
use ./test_yzx_doctor_commands.nu [run_doctor_noncanonical_tests]
use ./test_yzx_generated_configs.nu [run_generated_config_noncanonical_tests]
use ./test_yzx_popup_commands.nu [run_popup_noncanonical_tests]
use ./test_yzx_refresh_commands.nu [run_refresh_noncanonical_tests]
use ./test_yzx_workspace_commands.nu [run_workspace_noncanonical_tests]
use ./test_yzx_yazi_commands.nu [run_yazi_noncanonical_tests]

def main [] {
    print "=== Testing yzx extra regressions ==="
    print ""

    let fixture = (setup_test_home)
    let results = (with-env { HOME: $fixture.tmp_home, YAZELIX_DIR: $fixture.config_dir } {
        [
            (run_core_noncanonical_tests)
            (run_dev_noncanonical_tests)
            (run_doctor_noncanonical_tests)
            (run_generated_config_noncanonical_tests)
            (run_popup_noncanonical_tests)
            (run_refresh_noncanonical_tests)
            (run_workspace_noncanonical_tests)
            (run_yazi_noncanonical_tests)
        ] | flatten
    })
    rm -rf $fixture.tmp_home

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx extra regressions passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some yzx extra regressions failed \(($passed)/($total)\)"
        error make { msg: "yzx extra regressions failed" }
    }
}
