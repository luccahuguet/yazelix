#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/workspace_session_contract.md

use ../integrations/yazi.nu [resolve_reveal_target_path]
use ./yzx_test_helpers.nu [CLEAN_ZELLIJ_ENV_PREFIX get_repo_config_dir get_repo_root repo_path]

def run_nu_snippet [snippet: string, extra_env?: record] {
    if ($extra_env | is-empty) {
        ^nu -c $snippet | complete
    } else {
        with-env $extra_env {
            ^nu -c $snippet | complete
        }
    }
}

def setup_launch_path_fixture [label: string, persistent_sessions: bool, existing_session: bool] {
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let fake_bin = ($tmp_home | path join "bin")
    let zellij_log = ($tmp_home | path join "zellij.log")
    let existing_session_flag = if $existing_session { "true" } else { "false" }
    let real_nu = (which nu | get -o 0.path)

    mkdir $runtime_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir
    mkdir $fake_bin

    for entry in ["nushell", "shells", "configs", "config_metadata", "devenv.lock", "yazelix_default.toml", "docs", "CHANGELOG.md", "assets"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }

    [
        "[core]"
        "skip_welcome_screen = true"
        "recommended_deps = true"
        "yazi_extensions = true"
        "yazi_media = false"
        ""
        "[zellij]"
        $"persistent_sessions = ($persistent_sessions)"
        'session_name = "yazelix"'
        ""
        "[shell]"
        'default_shell = "nu"'
    ] | str join "\n" | save --force --raw ($user_config_dir | path join "yazelix.toml")

    [
        "#!/bin/sh"
        'log="$TMP_ZELLIJ_LOG"'
        'cmd="$1"'
        'shift'
        'case "$cmd" in'
        '  setup)'
        '    if [ "$1" = "--dump-config" ]; then'
        "      cat <<'KDL'"
        "keybinds clear-defaults=true {}"
        "themes {}"
        "KDL"
        "      exit 0"
        "    fi"
        "    ;;"
        "  list-sessions)"
        $"    if [ \"($existing_session_flag)\" = \"true\" ]; then"
        "      printf '%s\\n' 'yazelix [Created 1s ago]'"
        "    fi"
        "    exit 0"
        "    ;;"
        "  options|attach)"
        "    printf '%s\\n' \"$cmd $*\" >> \"$log\""
        "    exit 0"
        "    ;;"
        "  *)"
        "    printf '%s\\n' \"$cmd $*\" >> \"$log\""
        "    exit 0"
        "    ;;"
        "esac"
    ] | str join "\n" | save --force --raw ($fake_bin | path join "zellij")
    ^chmod +x ($fake_bin | path join "zellij")
    ^ln -s $real_nu ($fake_bin | path join "nu")

    {
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        state_dir: $state_dir
        fake_bin: $fake_bin
        zellij_log: $zellij_log
        start_inner: ($runtime_dir | path join "nushell" "scripts" "core" "start_yazelix_inner.nu")
        layout_path: ($state_dir | path join "configs" "zellij" "layouts" "yzx_side.kdl")
        env: {
            HOME: $tmp_home
            PATH: ([$fake_bin] | append $env.PATH)
            TMP_ZELLIJ_LOG: $zellij_log
            YAZELIX_RUNTIME_DIR: $runtime_dir
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
        }
    }
}

# Defends: startup rejects a missing working directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_startup_rejects_missing_working_dir [] {
    print "🧪 Testing startup rejects missing working directories..."

    try {
        let start_script = (repo_path "nushell" "scripts" "core" "start_yazelix.nu")
        let snippet = ([
            $"source \"($start_script)\""
            'try {'
            '    validate_startup_working_dir "/tmp/yazelix_missing_start_dir" | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Startup directory does not exist") {
            print "  ✅ Startup path validation fails early for missing directories"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Defends: launch rejects a file path as the working directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_rejects_file_working_dir [] {
    print "🧪 Testing launch rejects file paths as working directories..."

    let tmpdir = (^mktemp -d /tmp/yazelix_launch_path_test_XXXXXX | str trim)

    let result = (try {
        let file_path = ($tmpdir | path join "not_a_dir.txt")
        "hello" | save --force --raw $file_path
        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            'try {'
            '    validate_launch_working_dir $env.YAZELIX_TEST_FILE_PATH | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet {YAZELIX_TEST_FILE_PATH: $file_path})
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Launch path is not a directory") {
            print "  ✅ Launch path validation rejects files before terminal startup"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: desktop launch ignores hostile inherited shell env.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_desktop_launch_ignores_hostile_shell_env [] {
    print "🧪 Testing yzx CLI desktop launch ignores hostile shell env..."

    let tmpdir = (^mktemp -d /tmp/yazelix_posix_desktop_env_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let fake_profile_bin = ($fake_home | path join ".local" "state" "nix" "profile" "bin")
        let nu_log = ($tmpdir | path join "nu_invocation.txt")
        let env_file = ($tmpdir | path join "env.sh")
        mkdir $fake_profile_bin

        [
            "#!/bin/sh"
            $"printf '%s\\n' \"$*\" > '($nu_log)'"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_profile_bin | path join "nu")
        ^chmod +x ($fake_profile_bin | path join "nu")

        [
            "echo SHOULD_NOT_SOURCE_ENV >&2"
            "exit 94"
        ] | str join "\n" | save --force --raw $env_file

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {HOME: $fake_home, BASH_ENV: $env_file, ENV: $env_file} {
            ^$launcher_script desktop launch | complete
        })
        let stderr = ($output.stderr | str trim)
        let nu_invocation = if ($nu_log | path exists) {
            open --raw $nu_log | str trim
        } else {
            ""
        }

        if ($output.exit_code == 0) and ($stderr == "") and ($nu_invocation | str contains "yzx desktop launch") {
            print "  ✅ yzx CLI reaches desktop launch without sourcing hostile shell env files"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) nu=($nu_invocation)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: nonpersistent launch --here uses the requested directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions [] {
    print "🧪 Testing non-persistent startup keeps the requested directory for both launch and restart..."

    let fixture = (setup_launch_path_fixture "yazelix_launch_here_path_nonpersistent" false false)

    let result = (try {
        let target_dir = ($fixture.tmp_home | path join "project")
        mkdir $target_dir
        let launch_output = (with-env $fixture.env {
            ^nu $fixture.start_inner $target_dir $fixture.layout_path | complete
        })
        let launch_stdout = ($launch_output.stdout | str trim)
        let launch_stderr = ($launch_output.stderr | str trim)
        let launch_zellij_log = if ($fixture.zellij_log | path exists) { open --raw $fixture.zellij_log | str trim } else { "" }

        let restart_state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix" "state" "restart")
        mkdir $restart_state_dir
        let restart_bootstrap_file = ($restart_state_dir | path join "sidebar_cwd_restart.txt")
        $target_dir | save --force --raw $restart_bootstrap_file
        "" | save --force --raw $fixture.zellij_log
        let restart_output = (with-env ($fixture.env | merge {
            YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: $restart_bootstrap_file
        }) {
            ^nu $fixture.start_inner $target_dir $fixture.layout_path | complete
        })
        let restart_stdout = ($restart_output.stdout | str trim)
        let restart_stderr = ($restart_output.stderr | str trim)
        let restart_zellij_log = if ($fixture.zellij_log | path exists) { open --raw $fixture.zellij_log | str trim } else { "" }

        let expected_shell = ($fixture.runtime_dir | path join "shells" "posix" "yazelix_nu.sh")
        let launch_ok = ($launch_output.exit_code == 0) and ($launch_zellij_log | str contains $"options --default-cwd ($target_dir)") and ($launch_zellij_log | str contains $"--default-shell ($expected_shell)") and (not ($launch_stdout | str contains "--path ignored"))
        let restart_ok = ($restart_output.exit_code == 0) and ($restart_zellij_log | str contains $"options --default-cwd ($target_dir)") and ($restart_zellij_log | str contains $"--default-shell ($expected_shell)") and (not ($restart_stdout | str contains "--path ignored"))

        if $launch_ok and $restart_ok {
            print "  ✅ Non-persistent sessions keep the requested directory as Zellij's cwd, including restart bootstrap flows"
            true
        } else {
            print $"  ❌ Unexpected launch result: exit=($launch_output.exit_code) stdout=($launch_stdout) stderr=($launch_stderr) zellij=($launch_zellij_log)"
            print $"  ❌ Unexpected restart result: exit=($restart_output.exit_code) stdout=($restart_stdout) stderr=($restart_stderr) zellij=($restart_zellij_log)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: persistent session reuse warns when it ignores the requested directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_here_path_warns_when_existing_persistent_session_ignores_it [] {
    print "🧪 Testing yzx launch --here --path warns when an existing persistent session ignores the requested directory..."

    let fixture = (setup_launch_path_fixture "yazelix_launch_here_path_persistent" true true)

    let result = (try {
        let target_dir = ($fixture.tmp_home | path join "project")
        mkdir $target_dir
        let output = (with-env $fixture.env {
            ^nu $fixture.start_inner $target_dir $fixture.layout_path | complete
        })
        let stdout = ($output.stdout | str trim)
        let zellij_log = if ($fixture.zellij_log | path exists) {
            open --raw $fixture.zellij_log | str trim
        } else {
            ""
        }

        if ($output.exit_code == 0) and ($stdout | str contains "Session 'yazelix' already exists - --path ignored.") and ($stdout | str contains "zellij kill-session yazelix") and ($zellij_log | str contains "attach yazelix") and (not ($zellij_log | str contains "--default-cwd")) {
            print "  ✅ Existing persistent sessions warn clearly and reattach without pretending --path will take effect"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) zellij=($zellij_log)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: launch falls through to the next managed terminal and ignores bare host terminal binaries.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_launch_falls_through_after_immediate_terminal_failure [] {
    print "🧪 Testing managed terminal launch skips bare host binaries and falls through after immediate failure..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_fallback_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        let fake_runtime = ($tmpdir | path join "runtime")
        let fake_shells = ($fake_runtime | path join "shells" "posix")
        mkdir $fake_bin
        mkdir $fake_runtime
        mkdir ($fake_runtime | path join "shells")
        mkdir $fake_shells
        "" | save --force --raw ($fake_runtime | path join "yazelix_default.toml")

        [
            "#!/bin/sh"
            "echo wezterm-boom >&2"
            "exit 27"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "yazelix-wezterm")
        ^chmod +x ($fake_bin | path join "yazelix-wezterm")

        [
            "#!/bin/sh"
            "sleep 2"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "yazelix-alacritty")
        ^chmod +x ($fake_bin | path join "yazelix-alacritty")

        let fake_wezterm = ($fake_bin | path join "yazelix-wezterm")
        let fake_alacritty = ($fake_bin | path join "yazelix-alacritty")
        [
            "#!/bin/sh"
            "echo raw-kitty-should-not-run >&2"
            "exit 88"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "kitty")
        ^chmod +x ($fake_bin | path join "kitty")
        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            "let candidates = (resolve_terminal_candidates '' ['wezterm', 'kitty', 'alacritty'] true)"
            "if (($candidates | length) != 2) { error make { msg: ($candidates | to json -r) } }"
            "if (($candidates | get 0.command) != $env.FAKE_WEZTERM) { error make { msg: 'wezterm wrapper not selected first' } }"
            "if (($candidates | get 1.command) != $env.FAKE_ALACRITTY) { error make { msg: 'bare kitty binary should not be treated as a managed candidate' } }"
            "let launched = (launch_terminal_candidates $candidates 'yazelix' $env.PWD false $env.YAZELIX_RUNTIME_DIR false '')"
            "print ($launched.terminal)"
        ] | str join "\n")
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $fake_runtime
            DEVENV_PROFILE: $tmpdir
            YAZELIX_STATE_DIR: ($tmpdir | path join "state")
            PATH: ([$fake_bin] | append $env.PATH)
            FAKE_WEZTERM: $fake_wezterm
            FAKE_ALACRITTY: $fake_alacritty
        } {
            run_nu_snippet $snippet
        })
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "failed to start; trying Yazelix - Alacritty") and ($stdout | str ends-with "alacritty") and ($stderr == "") {
            print "  ✅ Managed launch ignores bare host binaries and falls through to the next Yazelix wrapper"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: startup preflight requires the generated layout path before deeper launch work.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_startup_requires_generated_layout_path [] {
    print "🧪 Testing startup preflight requires an existing Zellij layout..."

    try {
        let start_script = (repo_path "nushell" "scripts" "core" "start_yazelix.nu")
        let snippet = ([
            $"source \"($start_script)\""
            'try {'
            '    require_generated_layout "/tmp/yazelix_missing_layout.kdl" | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Missing Yazelix generated Zellij layout") and ($stdout | str contains "yzx refresh") and ($stdout | str contains "Failure class: generated-state problem.") {
            print "  ✅ Startup preflight fails clearly when the generated layout is missing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Defends: yzx cwd fails clearly outside Zellij.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
def test_yzx_cwd_requires_zellij [] {
    print "🧪 Testing yzx cwd outside Zellij..."

    try {
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let output = (^bash -lc $"($CLEAN_ZELLIJ_ENV_PREFIX) nu -c 'use \"($yzx_script)\" *; yzx cwd .'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 1) and ($stdout | str contains "only works inside Zellij") {
            print "  ✅ yzx cwd fails clearly outside Zellij"
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

export def run_workspace_canonical_tests [] {
    [
        (test_yzx_cli_desktop_launch_ignores_hostile_shell_env)
        (test_launch_falls_through_after_immediate_terminal_failure)
        (test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions)
        (test_launch_here_path_warns_when_existing_persistent_session_ignores_it)
        (test_startup_rejects_missing_working_dir)
        (test_launch_rejects_file_working_dir)
        (test_startup_requires_generated_layout_path)
        (test_yzx_cwd_requires_zellij)
    ]
}

export def run_workspace_tests [] {
    run_workspace_canonical_tests
}
