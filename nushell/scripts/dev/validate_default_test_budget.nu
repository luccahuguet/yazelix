#!/usr/bin/env nu

use ./test_yzx_commands.nu [run_default_canonical_suite]
use ./yzx_test_helpers.nu [format_test_profile_report]

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
# Raised after the runtime-state, desktop-launch, flake-interface, Home Manager,
# and launch-profile/runtime-repair regressions were promoted into the default suite.
# Keep the cap explicit so future growth still has to justify itself.
const DEFAULT_SUITE_MAX_SECONDS = 105.0

export def profile_suite_runner [runner: closure] {
    let started = (date now)
    let result = (do $runner)
    let elapsed_seconds = (((date now) - $started) / 1sec | into float)

    {
        result: $result
        elapsed_seconds: $elapsed_seconds
    }
}

def run_profiled_default_suite [] {
    do {
        cd $REPO_ROOT
        with-env { YAZELIX_DIR: $REPO_ROOT } {
            profile_suite_runner { run_default_canonical_suite --profile }
        }
    }
}

export def main [] {
    let result = (run_profiled_default_suite)
    let suite_run = $result.result

    let elapsed_seconds = $result.elapsed_seconds

    if $suite_run.passed != $suite_run.total {
        error make { msg: "Default suite failed while collecting the runtime budget profile" }
    }

    if $elapsed_seconds > $DEFAULT_SUITE_MAX_SECONDS {
        print (format_test_profile_report $suite_run.suite_results "=== test_yzx_commands.nu profile ===")
        error make { msg: $"Default-suite runtime budget exceeded: ($elapsed_seconds)s > ($DEFAULT_SUITE_MAX_SECONDS)s" }
    }

    print $"✅ Default-suite runtime budget ok: ($elapsed_seconds)s <= ($DEFAULT_SUITE_MAX_SECONDS)s"
}
