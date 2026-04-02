#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const DEFAULT_SUITE_MAX_SECONDS = 60.0

def run_profiled_default_suite [] {
    let suite_script = ($REPO_ROOT | path join "nushell" "scripts" "dev" "test_yzx_commands.nu")
    let started = (date now)

    with-env { YAZELIX_DIR: $REPO_ROOT } {
        let output = (^nu $suite_script --profile | complete)
        let elapsed_seconds = (((date now) - $started) / 1sec | into float)
        $output | upsert elapsed_seconds $elapsed_seconds
    }
}

export def main [] {
    let result = (run_profiled_default_suite)

    if $result.exit_code != 0 {
        if not ($result.stdout | is-empty) {
            print $result.stdout
        }
        if not ($result.stderr | is-empty) {
            print $result.stderr
        }
        error make { msg: "Default suite failed while collecting the runtime budget profile" }
    }

    let elapsed_seconds = $result.elapsed_seconds

    if $elapsed_seconds > $DEFAULT_SUITE_MAX_SECONDS {
        print $result.stdout
        error make { msg: $"Default-suite runtime budget exceeded: ($elapsed_seconds)s > ($DEFAULT_SUITE_MAX_SECONDS)s" }
    }

    print $"✅ Default-suite runtime budget ok: ($elapsed_seconds)s <= ($DEFAULT_SUITE_MAX_SECONDS)s"
}
