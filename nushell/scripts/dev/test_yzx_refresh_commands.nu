#!/usr/bin/env nu

use ./test_yzx_helpers.nu [repo_path]

def test_command_failure_summary_includes_command_tail_and_recovery [] {
    print "🧪 Testing refresh/rebuild failure summaries include command, stderr tail, and recovery..."

    try {
        let bootstrap_script = (repo_path "nushell" "scripts" "utils" "environment_bootstrap.nu")
        let snippet = ([
            $"source \"($bootstrap_script)\""
            'print (format_command_failure_summary "Refresh failed" ["env", "-C", "/tmp/yazelix repo", "devenv", "build", "shell"] 17 "line1\nline2\nline3\nline4\nline5\nline6" "Run `yzx doctor`.")'
        ] | str join "\n")
        let output = (^nu -c $snippet | complete)
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Refresh failed")
            and ($stdout | str contains 'Command: env -C "/tmp/yazelix repo" devenv build shell')
            and ($stdout | str contains "line2")
            and ($stdout | str contains "line6")
            and (not ($stdout | str contains "line1"))
            and ($stdout | str contains "Recovery: Run `yzx doctor`.")
        ) {
            print "  ✅ Failure summaries preserve the command, stderr tail, and recovery hint"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_command_failure_summary_marks_streamed_output [] {
    print "🧪 Testing refresh/rebuild failure summaries explain streamed stderr..."

    try {
        let bootstrap_script = (repo_path "nushell" "scripts" "utils" "environment_bootstrap.nu")
        let snippet = ([
            $"source \"($bootstrap_script)\""
            'print (format_command_failure_summary "Environment rebuild failed" ["env", "-C", "/tmp/yazelix", "devenv", "--quiet", "build", "shell"] 23 "" "Retry after fixing the build." --stderr-streamed)'
        ] | str join "\n")
        let output = (^nu -c $snippet | complete)
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Environment rebuild failed")
            and ($stdout | str contains "stderr tail: output was streamed directly above.")
            and ($stdout | str contains "Recovery: Retry after fixing the build.")
        ) {
            print "  ✅ Failure summaries explain when stderr was already streamed"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_refresh_canonical_tests [] {
    [
        (test_command_failure_summary_includes_command_tail_and_recovery)
    ]
}

export def run_refresh_tests [] {
    run_refresh_canonical_tests
}
