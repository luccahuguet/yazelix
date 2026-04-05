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

def setup_refresh_profile_recording_fixture [label: string] {
    let tmp_root = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_root | path join "runtime")
    let yzx_dir = ($runtime_dir | path join "nushell" "scripts" "yzx")
    let utils_dir = ($runtime_dir | path join "nushell" "scripts" "utils")
    let fake_bin = ($tmp_root | path join "bin")
    let state_dir = ($tmp_root | path join "state")
    let materialized_log = ($tmp_root | path join "materialized.json")
    let launch_log = ($tmp_root | path join "launch.json")
    let fresh_profile = ($tmp_root | path join "fresh-profile")
    let real_nu = (which nu | get -o 0.path | default "nu")

    mkdir $yzx_dir
    mkdir $utils_dir
    mkdir $fake_bin
    mkdir $state_dir
    mkdir $fresh_profile

    cp (repo_path "nushell" "scripts" "yzx" "refresh.nu") ($yzx_dir | path join "refresh.nu")

    [
        "export def ensure_nix_available [] {}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "nix_detector.nu")

    [
        "export def prepare_environment [] {"
        "    {"
        "        config: {"
        "            recommended_deps: true"
        "            yazi_extensions: true"
        "            max_jobs: \"half\""
        "            build_cores: \"2\""
        "            refresh_output: \"normal\""
        "            pack_names: []"
        "            pack_declarations: {}"
        "            user_packages: []"
        "        }"
        "        config_state: {"
        "            needs_refresh: true"
        "            combined_hash: \"new-hash\""
        "            refresh_reason: \"fixture\""
        "        }"
        "        needs_refresh: true"
        "    }"
        "}"
        "export def get_devenv_base_command ["
        "    --max-jobs: string = \"\""
        "    --build-cores: string = \"\""
        "    --quiet"
        "    --devenv-verbose"
        "    --refresh-eval-cache"
        "    --skip-shellhook-welcome"
        "] {"
        "    [\"fake-devenv\"]"
        "}"
        "export def is_unfree_enabled [] { false }"
        "export def get_refresh_output_mode [config] { \"normal\" }"
        "export def format_command_failure_summary [label, command_parts, exit_code, stderr, recovery_hint, --stderr-streamed] {"
        "    $\"($label)\""
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "environment_bootstrap.nu")

    [
        "export def compute_config_state [] {"
        "    { combined_hash: \"new-hash\", needs_refresh: false }"
        "}"
        "export def record_materialized_state [state: record] {"
        $"    $state | to json -r | save --force --raw \"($materialized_log)\""
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "config_state.nu")

    [
        "export def record_launch_profile_state [config_state: record, profile_path: string] {"
        ("    { combined_hash: $config_state.combined_hash, profile_path: $profile_path } | to json -r | save --force --raw \"" + $launch_log + "\"")
        "}"
        "export def resolve_profile_from_build_shell_output [stdout: string] {"
        "    if ($stdout | str contains '/tmp/fresh-shell') {"
        $"        '($fresh_profile)'"
        "    } else {"
        "        error make { msg: 'EXPECTED_BUILD_OUTPUT_PROFILE_SOURCE' }"
        "    }"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "launch_state.nu")

    [
        "export def describe_build_parallelism [build_cores_config?: string, max_jobs_config?: string] {"
        "    '8 jobs x 2 cores/job'"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "common.nu")

    [
        "#!/bin/sh"
        "printf '{\\n  \"shell\": \"/tmp/fresh-shell\"\\n}\\n'"
    ] | str join "\n" | save --force --raw ($fake_bin | path join "fake-devenv")
    ^chmod +x ($fake_bin | path join "fake-devenv")

    {
        tmp_root: $tmp_root
        runtime_dir: $runtime_dir
        fake_bin: $fake_bin
        real_nu: $real_nu
        state_dir: $state_dir
        fresh_profile: $fresh_profile
        materialized_log: $materialized_log
        launch_log: $launch_log
        refresh_script: ($yzx_dir | path join "refresh.nu")
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

# Regression: yzx refresh must record the built profile from `devenv build shell`, not the stale current session.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_refresh_records_profile_from_build_shell_output [] {
    print "🧪 Testing yzx refresh records the profile from build-shell output instead of stale session state..."

    let fixture = (setup_refresh_profile_recording_fixture "yazelix_refresh_profile_record")

    let result = (try {
        let command = $"use \"($fixture.refresh_script)\" *; yzx refresh --force"
        let output = (with-env {
            PATH: ([$fixture.fake_bin, "/usr/bin", "/bin"] | str join (char esep))
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_STATE_DIR: $fixture.state_dir
        } {
            ^$fixture.real_nu -c $command | complete
        })

        let launch_record = if ($fixture.launch_log | path exists) {
            open $fixture.launch_log
        } else {
            null
        }
        let materialized_record = if ($fixture.materialized_log | path exists) {
            open $fixture.materialized_log
        } else {
            null
        }

        if (
            ($output.exit_code == 0)
            and (($launch_record | default {} | get -o profile_path | default "") == $fixture.fresh_profile)
            and (($launch_record | default {} | get -o combined_hash | default "") == "new-hash")
            and (($materialized_record | default {} | get -o combined_hash | default "") == "new-hash")
            and ($output.stdout | str contains "✅ Refresh completed.")
        ) {
            print "  ✅ yzx refresh now records the freshly built profile from build-shell output"
            true
        } else {
            print $"  ❌ Unexpected refresh result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim)) launch=(($launch_record | default {} | to json -r))"
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
        (test_refresh_records_profile_from_build_shell_output)
    ]
}

export def run_refresh_tests [] {
    run_refresh_canonical_tests
}
