#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const DEFAULT_SUITE_MAX_SECONDS = 45.0

def run_profiled_default_suite [] {
    let dev_script = ($REPO_ROOT | path join "nushell" "scripts" "yzx" "dev.nu")
    let snippet = $"source \"($dev_script)\"; yzx dev test --profile"

    with-env { YAZELIX_DIR: $REPO_ROOT } {
        ^nu -c $snippet | complete
    }
}

def extract_default_suite_seconds [stdout: string] {
    let profile_line = (
        $stdout
        | lines
        | where { |line| $line | str contains "- test_yzx_commands:" }
        | last
        | default ""
    )

    if ($profile_line | is-empty) {
        error make { msg: "Could not find the default-suite profile line in `yzx dev test --profile` output" }
    }

    let parsed = (
        $profile_line
        | str replace -r '.*test_yzx_commands:\s+' ''
        | str replace -r 's.*' ''
        | str trim
        | default ""
    )

    if ($parsed | is-empty) {
        error make { msg: $"Could not parse default-suite seconds from profile line: ($profile_line)" }
    }

    $parsed | into float
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

    let elapsed_seconds = (extract_default_suite_seconds $result.stdout)

    if $elapsed_seconds > $DEFAULT_SUITE_MAX_SECONDS {
        print $result.stdout
        error make { msg: $"Default-suite runtime budget exceeded: ($elapsed_seconds)s > ($DEFAULT_SUITE_MAX_SECONDS)s" }
    }

    print $"✅ Default-suite runtime budget ok: ($elapsed_seconds)s <= ($DEFAULT_SUITE_MAX_SECONDS)s"
}
