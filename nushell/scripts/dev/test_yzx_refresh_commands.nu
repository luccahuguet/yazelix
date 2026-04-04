#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [repo_path]

def setup_shellhook_quiet_fixture [label: string] {
    let tmp_root = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let tmp_home = ($tmp_root | path join "home")
    let runtime_dir = ($tmp_root | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let log_dir = ($state_dir | path join "logs")
    let runtime_bin_dir = ($runtime_dir | path join "bin")

    mkdir $runtime_dir
    mkdir $runtime_bin_dir
    mkdir $user_config_dir
    mkdir ($tmp_home | path join ".config" "nushell")
    mkdir $log_dir
    mkdir ($state_dir | path join "state")

    for entry in [".taplo.toml", "nushell", "shells", "configs", "config_metadata", "assets", "yazelix_default.toml", "yazelix_packs_default.toml"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }
    ^ln -s (repo_path "shells" "posix" "yzx_cli.sh") ($runtime_bin_dir | path join "yzx")

    cp (repo_path "yazelix_default.toml") ($user_config_dir | path join "yazelix.toml")
    cp (repo_path "yazelix_packs_default.toml") ($user_config_dir | path join "yazelix_packs.toml")
    "" | save --force --raw ($tmp_home | path join ".bashrc")
    "" | save --force --raw ($tmp_home | path join ".config" "nushell" "config.nu")

    {
        tmp_root: $tmp_root
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        state_dir: $state_dir
        log_dir: $log_dir
        environment_script: ($runtime_dir | path join "nushell" "scripts" "setup" "environment.nu")
    }
}

# Defends: refresh failures include command tail and recovery guidance.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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

# Regression: noninteractive shellHook setup must stay quiet when welcome is skipped.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
def test_skip_welcome_shellhook_setup_stays_quiet [] {
    print "🧪 Testing skip-welcome shellHook setup stays quiet..."

    let fixture = (setup_shellhook_quiet_fixture "yazelix_shellhook_quiet")

    let result = (try {
        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_STATE_DIR: $fixture.state_dir
            YAZELIX_LOGS_DIR: $fixture.log_dir
        } {
            ^nu $fixture.environment_script --skip-welcome | complete
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and (not ($stdout | str contains "📝 Logging to:"))
            and (not ($stdout | str contains "Generated "))
            and (not ($stdout | str contains "config already sourced"))
            and (not ($stdout | str contains "Tools not found:"))
            and (not ($stdout | str contains "Yazelix environment setup complete!"))
        ) {
            print "  ✅ Skip-welcome shellHook entry no longer replays routine setup chatter"
            true
        } else {
            print $"  ❌ Unexpected output: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_root
    $result
}

export def run_refresh_canonical_tests [] {
    [
        (test_command_failure_summary_includes_command_tail_and_recovery)
        (test_skip_welcome_shellhook_setup_stays_quiet)
    ]
}

export def run_refresh_tests [] {
    run_refresh_canonical_tests
}
