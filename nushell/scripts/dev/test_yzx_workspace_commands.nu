#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/workspace_session_contract.md

use ./yzx_test_helpers.nu [CLEAN_ZELLIJ_ENV_PREFIX get_repo_config_dir get_repo_root repo_path setup_managed_config_fixture]
use ../integrations/zellij.nu [retarget_workspace_for_path]

def run_nu_snippet [snippet: string, extra_env?: record] {
    if ($extra_env | is-empty) {
        ^nu -c $snippet | complete
    } else {
        with-env $extra_env {
            ^nu -c $snippet | complete
        }
    }
}

def setup_cli_probe_fixture [label: string] {
    let tmpdir = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let fake_home = ($tmpdir | path join "home")
    let fake_profile_bin = ($fake_home | path join ".local" "state" "nix" "profile" "bin")
    let nu_log = ($tmpdir | path join "nu_invocation.txt")

    mkdir $fake_profile_bin

    {
        tmpdir: $tmpdir
        fake_home: $fake_home
        fake_profile_bin: $fake_profile_bin
        nu_log: $nu_log
    }
}

def write_probe_nu [probe_path: string, script_lines: list<string>] {
    $script_lines | str join "\n" | save --force --raw $probe_path
    ^chmod +x $probe_path
}

def read_probe_lines [log_path: string] {
    if ($log_path | path exists) {
        open --raw $log_path | lines
    } else {
        []
    }
}

def read_probe_string [log_path: string] {
    if ($log_path | path exists) {
        open --raw $log_path | str trim
    } else {
        ""
    }
}

def install_argument_logging_probe [fixture: record] {
    write_probe_nu ($fixture.fake_profile_bin | path join "nu") [
        "#!/bin/sh"
        ": > \"$NU_LOG\""
        "for arg in \"$@\"; do"
        "  printf '%s\n' \"$arg\" >> \"$NU_LOG\""
        "done"
        "exit 0"
    ]
}

def setup_desktop_runtime_probe_fixture [label: string, --with_hidden_launch_module] {
    let cli_fixture = (setup_cli_probe_fixture $label)
    let runtime_store = ($cli_fixture.tmpdir | path join "runtime_store")
    let runtime_reference_root = ($cli_fixture.fake_home | path join ".local" "share" "yazelix" "runtime")
    let runtime_dir = ($runtime_store | path expand)
    let runtime_scripts_dir = ($runtime_dir | path join "nushell" "scripts")

    mkdir ($runtime_scripts_dir | path join "core")
    mkdir ($runtime_scripts_dir | path join "yzx")
    mkdir $runtime_reference_root

    ^ln -s $runtime_dir ($runtime_reference_root | path join "current")
    ^cp --recursive (repo_path "nushell" "scripts" "utils") $runtime_scripts_dir
    ^cp (repo_path "nushell" "scripts" "core" "launch_yazelix.nu") ($runtime_scripts_dir | path join "core" "launch_yazelix.nu")
    ^cp (repo_path "nushell" "scripts" "yzx" "desktop.nu") ($runtime_scripts_dir | path join "yzx" "desktop.nu")
    if $with_hidden_launch_module {
        ^cp (repo_path "nushell" "scripts" "yzx" "launch.nu") ($runtime_scripts_dir | path join "yzx" "launch.nu")
    }
    ^ln -s (repo_path ".taplo.toml") ($runtime_dir | path join ".taplo.toml")
    ^ln -s (repo_path "yazelix_default.toml") ($runtime_dir | path join "yazelix_default.toml")

    $cli_fixture | merge {
        runtime_store: $runtime_store
        runtime_reference_root: $runtime_reference_root
        runtime_dir: $runtime_dir
    }
}

def setup_startup_bootstrap_probe_fixture [label: string] {
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let probe_log = ($tmp_home | path join "bootstrap_probe.log")
    let start_script = ($runtime_dir | path join "shells" "posix" "start_yazelix.sh")
    let runtime_env_script = ($runtime_dir | path join "shells" "posix" "runtime_env.sh")
    let startup_nu = ($runtime_dir | path join "nushell" "scripts" "core" "start_yazelix.nu")
    let fake_nu = ($runtime_dir | path join "bin" "nu")

    mkdir ($runtime_dir | path join "shells" "posix")
    mkdir ($runtime_dir | path join "nushell" "scripts" "core")
    mkdir ($runtime_dir | path join "bin")

    ^ln -s (repo_path "shells" "posix" "start_yazelix.sh") $start_script
    ^ln -s (repo_path "shells" "posix" "runtime_env.sh") $runtime_env_script
    "" | save --force --raw $startup_nu

    write_probe_nu $fake_nu [
        "#!/bin/sh"
        ": > \"$YAZELIX_BOOTSTRAP_PROBE_LOG\""
        "printf 'YAZELIX_RUNTIME_DIR=%s\\n' \"${YAZELIX_RUNTIME_DIR-unset}\" >> \"$YAZELIX_BOOTSTRAP_PROBE_LOG\""
        "printf 'YAZELIX_CONFIG_DIR=%s\\n' \"${YAZELIX_CONFIG_DIR-unset}\" >> \"$YAZELIX_BOOTSTRAP_PROBE_LOG\""
        "printf 'YAZELIX_STATE_DIR=%s\\n' \"${YAZELIX_STATE_DIR-unset}\" >> \"$YAZELIX_BOOTSTRAP_PROBE_LOG\""
        "printf 'YAZELIX_LOGS_DIR=%s\\n' \"${YAZELIX_LOGS_DIR-unset}\" >> \"$YAZELIX_BOOTSTRAP_PROBE_LOG\""
        "printf 'ARG1=%s\\n' \"$1\" >> \"$YAZELIX_BOOTSTRAP_PROBE_LOG\""
        "exit 0"
    ]

    {
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        probe_log: $probe_log
        start_script: $start_script
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

    for entry in [".taplo.toml", "nushell", "shells", "configs", "config_metadata", "yazelix_default.toml", "docs", "CHANGELOG.md", "assets"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }

    [
        "[core]"
        "skip_welcome_screen = true"
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

def setup_enter_forwarding_fixture [label: string] {
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let scripts_dir = ($runtime_dir | path join "nushell" "scripts")
    let yzx_dir = ($scripts_dir | path join "yzx")
    let utils_dir = ($scripts_dir | path join "utils")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let call_log = ($tmp_home | path join "start_yazelix_session.json")

    mkdir $runtime_dir
    mkdir ($runtime_dir | path join "nushell")
    mkdir $scripts_dir
    mkdir $yzx_dir
    mkdir $utils_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir

    ^ln -s (repo_path "nushell" "scripts" "yzx" "launch.nu") ($yzx_dir | path join "launch.nu")

    let bootstrap_stub = ([
        "export def prepare_environment [--verbose] {"
            "    error make {msg: \"PREPARE_ENVIRONMENT_SHOULD_NOT_RUN\"}"
        "}"
        "export def ensure_environment_available [] {"
        "    error make {msg: \"ENSURE_ENVIRONMENT_AVAILABLE_SHOULD_NOT_RUN\"}"
        "}"
    ] | str join "\n")
    $bootstrap_stub | save --force --raw ($utils_dir | path join "environment_bootstrap.nu")

    [
        "export def require_yazelix_runtime_dir [] {"
        "    $env.YAZELIX_RUNTIME_DIR"
        "}"
        "export def resolve_yazelix_nu_bin [] {"
        "    which nu | get -o 0.path | default \"nu\""
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "common.nu")

    [
        "export def get_runtime_env [config?: record] {"
        "    {}"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "runtime_env.nu")

    [
        "export def check_runtime_script [script_path: string, field: string, label: string, context: string] {"
        "    {path: $script_path}"
        "}"
        "export def require_runtime_check [check: record] {"
        "    $check"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "runtime_contract_checker.nu")

    {
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        call_log: $call_log
        launch_script: ($yzx_dir | path join "launch.nu")
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

# Regression: startup bootstrap must export writable Yazelix state and logs dirs before entering Nushell.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_startup_bootstrap_runtime_env_exports_state_and_logs_dirs [] {
    print "🧪 Testing startup bootstrap runtime env exports Yazelix state and logs dirs..."

    let fixture = (setup_startup_bootstrap_probe_fixture "yazelix_startup_bootstrap_env")

    let result = (try {
        let xdg_config_home = ($fixture.tmp_home | path join "xdg_config")
        let xdg_data_home = ($fixture.tmp_home | path join "xdg_data")
        mkdir $xdg_config_home
        mkdir $xdg_data_home

        let output = (with-env {
            HOME: $fixture.tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            XDG_DATA_HOME: $xdg_data_home
            PATH: "/usr/bin:/bin"
            YAZELIX_CONFIG_DIR: null
            YAZELIX_STATE_DIR: null
            YAZELIX_LOGS_DIR: null
            YAZELIX_BOOTSTRAP_PROBE_LOG: $fixture.probe_log
        } {
            ^$fixture.start_script | complete
        })

        let lines = (read_probe_lines $fixture.probe_log)
        let expected_runtime = $"YAZELIX_RUNTIME_DIR=($fixture.runtime_dir)"
        let expected_config = $"YAZELIX_CONFIG_DIR=($xdg_config_home | path join "yazelix")"
        let expected_state = $"YAZELIX_STATE_DIR=($xdg_data_home | path join "yazelix")"
        let expected_logs = $"YAZELIX_LOGS_DIR=($xdg_data_home | path join "yazelix" "logs")"
        let expected_arg = $"ARG1=($fixture.runtime_dir | path join "nushell" "scripts" "core" "start_yazelix.nu")"

        if (
            ($output.exit_code == 0)
            and ($lines | any {|line| $line == $expected_runtime })
            and ($lines | any {|line| $line == $expected_config })
            and ($lines | any {|line| $line == $expected_state })
            and ($lines | any {|line| $line == $expected_logs })
            and ($lines | any {|line| $line == $expected_arg })
        ) {
            print "  ✅ Startup bootstrap now exports Yazelix config, state, and logs dirs before entering Nushell"
            true
        } else {
            print $"  ❌ Unexpected bootstrap result: exit=($output.exit_code) lines=($lines | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
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

    let fixture = (setup_cli_probe_fixture "yazelix_posix_desktop_env")

    let result = (try {
        let env_file = ($fixture.tmpdir | path join "env.sh")

        write_probe_nu ($fixture.fake_profile_bin | path join "nu") [
            "#!/bin/sh"
            $"printf '%s\\n' \"$*\" > '($fixture.nu_log)'"
            "exit 0"
        ]

        [
            "echo SHOULD_NOT_SOURCE_ENV >&2"
            "exit 94"
        ] | str join "\n" | save --force --raw $env_file

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {HOME: $fixture.fake_home, BASH_ENV: $env_file, ENV: $env_file} {
            ^$launcher_script desktop launch | complete
        })
        let stderr = ($output.stderr | str trim)
        let nu_invocation = (read_probe_string $fixture.nu_log)

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

    rm -rf $fixture.tmpdir
    $result
}

# Regression: desktop launch must use the installed runtime fast path and clear Yazelix-owned desktop launch state before invoking it.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_launch_uses_leaf_launch_module_with_clean_env [] {
    print "🧪 Testing yzx desktop launch uses the installed runtime fast path and clears Yazelix-owned launch state..."

    let fixture = (setup_desktop_runtime_probe_fixture "yazelix_desktop_leaf_runtime")

    let result = (try {
        let hostile_nu = ($fixture.fake_profile_bin | path join "nu")
        let runtime_nu = ($fixture.runtime_dir | path join "bin" "nu")
        mkdir ($runtime_nu | path dirname)

        write_probe_nu $hostile_nu [
            "#!/bin/sh"
            $"printf 'hostile-nu\\n' > '($fixture.nu_log)'"
            "exit 9"
        ]

        write_probe_nu $runtime_nu [
            "#!/bin/sh"
            $"printf '%s\\n' \"$1\" > '($fixture.nu_log)'"
            "shift"
            "for arg in \"$@\"; do"
            $"  printf '%s\\n' \"$arg\" >> '($fixture.nu_log)'"
            "done"
            $"printf 'YAZELIX_RUNTIME_DIR=%s\\n' \"${YAZELIX_RUNTIME_DIR-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_DIR=%s\\n' \"${YAZELIX_DIR-unset}\" >> '($fixture.nu_log)'"
            $"printf 'IN_YAZELIX_SHELL=%s\\n' \"${IN_YAZELIX_SHELL-unset}\" >> '($fixture.nu_log)'"
            $"printf 'IN_NIX_SHELL=%s\\n' \"${IN_NIX_SHELL-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_TERMINAL=%s\\n' \"${YAZELIX_TERMINAL-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_MENU_POPUP=%s\\n' \"${YAZELIX_MENU_POPUP-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_POPUP_PANE=%s\\n' \"${YAZELIX_POPUP_PANE-unset}\" >> '($fixture.nu_log)'"
            $"printf 'ZELLIJ_SESSION_NAME=%s\\n' \"${ZELLIJ_SESSION_NAME-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZI_ID=%s\\n' \"${YAZI_ID-unset}\" >> '($fixture.nu_log)'"
            "exit 0"
        ]

        let desktop_script = ($fixture.runtime_dir | path join "nushell" "scripts" "yzx" "desktop.nu")
        let output = (with-env {
            HOME: $fixture.fake_home
            YAZELIX_NU_BIN: $hostile_nu
            YAZELIX_RUNTIME_DIR: ($fixture.tmpdir | path join "hostile_runtime")
            YAZELIX_DIR: "/hostile/legacy_runtime"
            IN_YAZELIX_SHELL: "true"
            IN_NIX_SHELL: "impure"
            YAZELIX_TERMINAL: "ghostty"
            YAZELIX_MENU_POPUP: "true"
            YAZELIX_POPUP_PANE: "true"
            ZELLIJ_SESSION_NAME: "yazelix"
            YAZI_ID: "1234"
        } {
            ^nu -c $"use \"($desktop_script)\" *; yzx desktop launch" | complete
        })
        let stderr = ($output.stderr | str trim)
        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_launch_module = ($fixture.runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
        let invocation_env = ($invocation | skip 3)
        let expected_env = [
            $"YAZELIX_RUNTIME_DIR=($fixture.runtime_dir)"
            "YAZELIX_DIR=unset"
            "IN_YAZELIX_SHELL=unset"
            "YAZELIX_TERMINAL=unset"
            "YAZELIX_MENU_POPUP=unset"
            "YAZELIX_POPUP_PANE=unset"
            "ZELLIJ_SESSION_NAME=unset"
            "YAZI_ID=unset"
        ]

        if (
            ($output.exit_code == 0)
            and ($stderr == "")
            and (($invocation | get -o 0 | default "") == $expected_launch_module)
            and (($invocation | get -o 1 | default "") == $fixture.fake_home)
            and (($invocation | get -o 2 | default "") == "--desktop-fast-path")
            and ($expected_env | all {|line| $line in $invocation_env })
        ) {
            print "  ✅ yzx desktop launch now reanchors to the installed runtime fast path and clears stale Yazelix launch state"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) invocation=(($invocation | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: desktop launch should fail loudly from the fast path instead of silently falling back to a second launch path.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_launch_propagates_fast_path_failures_without_fallback [] {
    print "🧪 Testing yzx desktop launch reports fast-path failures instead of hiding them behind a fallback launch..."

    let fixture = (setup_desktop_runtime_probe_fixture "yazelix_desktop_fallback_runtime")

    let result = (try {
        let fast_launch_module = ($fixture.runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
        let runtime_nu = ($fixture.runtime_dir | path join "bin" "nu")
        mkdir ($runtime_nu | path dirname)

        write_probe_nu $runtime_nu [
            "#!/bin/sh"
            $"printf '%s\\n' \"$1\" >> '($fixture.nu_log)'"
            "shift"
            "for arg in \"$@\"; do"
            $"  printf '%s\\n' \"$arg\" >> '($fixture.nu_log)'"
            "done"
            $"printf 'YAZELIX_RUNTIME_DIR=%s\\n' \"${YAZELIX_RUNTIME_DIR-unset}\" >> '($fixture.nu_log)'"
            $"printf 'IN_YAZELIX_SHELL=%s\\n' \"${IN_YAZELIX_SHELL-unset}\" >> '($fixture.nu_log)'"
            "printf 'Failure class: desktop-bootstrap-unavailable.\\n' >&2"
            "exit 91"
        ]

        let desktop_script = ($fixture.runtime_dir | path join "nushell" "scripts" "yzx" "desktop.nu")
        let output = (with-env {
            HOME: $fixture.fake_home
            IN_YAZELIX_SHELL: "true"
            YAZELIX_RUNTIME_DIR: ($fixture.tmpdir | path join "hostile_runtime")
        } {
            ^nu -c $"use \"($desktop_script)\" *; yzx desktop launch" | complete
        })
        let stderr = ($output.stderr | str trim)
        let invocation = (read_probe_lines $fixture.nu_log)

        if (
            ($output.exit_code == 1)
            and ($stderr | str contains "Failure class: desktop-bootstrap-unavailable.")
            and (($invocation | get -o 0 | default "") == $fast_launch_module)
            and (($invocation | get -o 1 | default "") == $fixture.fake_home)
            and (($invocation | get -o 2 | default "") == "--desktop-fast-path")
            and (($invocation | get -o 3 | default "") == $"YAZELIX_RUNTIME_DIR=($fixture.runtime_dir)")
            and (($invocation | get -o 4 | default "") == "IN_YAZELIX_SHELL=unset")
            and not ($invocation | any {|line| $line == "-c" })
        ) {
            print "  ✅ yzx desktop launch now surfaces fast-path failures directly instead of hiding them behind a second launch attempt"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) invocation=(($invocation | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: desktop fast path must not silently swap an explicit requested terminal for a different bootstrap terminal.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_desktop_fast_path_rejects_bootstrap_terminal_substitution_for_explicit_terminal [] {
    print "🧪 Testing desktop fast path refuses to substitute a different terminal when one was explicitly requested..."

    let tmpdir = (^mktemp -d /tmp/yazelix_desktop_terminal_override_XXXXXX | str trim)
    let real_nu = (which nu | get -o 0.path)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_bin

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "ghostty")
        ^chmod +x ($fake_bin | path join "ghostty")

        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            "try {"
            "    resolve_desktop_fast_path_candidates 'kitty' ['ghostty', 'kitty'] | ignore"
            "} catch {|err|"
            "    print $err.msg"
            "}"
        ] | str join "\n")
        let output = (with-env {
            PATH: $fake_bin
        } {
            ^$real_nu -c $snippet | complete
        })
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)

        if (
            ($output.exit_code == 0)
            and ($stderr == "")
            and ($stdout | str contains "Specified terminal 'kitty' is not available")
            and ($stdout | str contains "host-dependency")
            and (not ($stdout | str contains "ghostty"))
        ) {
            print "  ✅ Desktop fast path preserves an explicit terminal request instead of silently substituting another terminal"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: desktop fast path must not reuse stale managed wrappers when a rebuild is already needed.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_desktop_fast_path_uses_direct_host_terminal_during_reload_instead_of_stale_wrapper [] {
    print "🧪 Testing desktop fast path uses a direct host terminal during reload instead of a stale managed wrapper..."

    let tmpdir = (^mktemp -d /tmp/yazelix_desktop_wrapper_preference_XXXXXX | str trim)
    let real_nu = (which nu | get -o 0.path)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let config_dir = ($fake_home | path join ".config" "yazelix")
        let state_dir = ($fake_home | path join ".local" "share" "yazelix")
        let runtime_dir = ($tmpdir | path join "runtime")
        let fake_bin = ($tmpdir | path join "bin")
        let profile_dir = ($tmpdir | path join "profile")
        let profile_bin = ($profile_dir | path join "bin")
        let launch_state_path = ($state_dir | path join "state" "launch_state.json")

        mkdir $config_dir
        mkdir ($state_dir | path join "state")
        mkdir $runtime_dir
        mkdir ($runtime_dir | path join "nushell")
        mkdir ($runtime_dir | path join "shells")
        mkdir ($runtime_dir | path join "configs")
        mkdir ($runtime_dir | path join "docs")
        mkdir ($runtime_dir | path join "assets")
        mkdir $fake_bin
        mkdir $profile_bin

        ^ln -s (repo_path ".taplo.toml") ($runtime_dir | path join ".taplo.toml")
        "" | save --force --raw ($runtime_dir | path join "yazelix_default.toml")
        "" | save --force --raw ($runtime_dir | path join "CHANGELOG.md")
        {
            combined_hash: "ignored-for-fast-path-resolution"
            profile_path: $profile_dir
        } | to json | save --force $launch_state_path

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "ghostty")
        ^chmod +x ($fake_bin | path join "ghostty")

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($profile_bin | path join "yazelix-ghostty")
        ^chmod +x ($profile_bin | path join "yazelix-ghostty")

        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            "let candidates = (resolve_desktop_fast_path_candidates '' ['ghostty'])"
            "print ($candidates | to json -r)"
        ] | str join "\n")
        let output = (with-env {
            HOME: $fake_home
            PATH: ([$fake_bin, "/usr/bin", "/bin"] | str join (char esep))
            YAZELIX_RUNTIME_DIR: $runtime_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_CONFIG_DIR: $config_dir
        } {
            ^$real_nu -c $snippet | complete
        })
        let candidates = ($output.stdout | from json)
        let first_candidate = ($candidates | get -o 0 | default {})

        let chosen_command = ($first_candidate.command? | default "" | into string)

        if (
            ($output.exit_code == 0)
            and (($first_candidate.terminal? | default "") == "ghostty")
            and (($first_candidate.use_wrapper? | default false) == false)
            and (
                ($chosen_command == "ghostty")
                or ($chosen_command == ($fake_bin | path join "ghostty"))
            )
        ) {
            print "  ✅ Desktop fast path now uses a visible host bootstrap terminal instead of reusing a stale managed wrapper during reload"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: yzx edit must ignore stale ambient Helix wrapper paths and derive the canonical managed editor command.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
def test_yzx_edit_resolves_managed_helix_wrapper_from_canonical_launch_env [] {
    print "🧪 Testing yzx edit resolves the managed Helix wrapper from the canonical launch env..."

    let fixture = (setup_managed_config_fixture
        "yazelix_edit_canonical_launch_env"
        (open --raw (repo_path "yazelix_default.toml"))
    )

    let result = (try {
        let helper_script = (repo_path "nushell" "scripts" "utils" "editor_launch_context.nu")
        let repo_root = (repo_path)
        let snippet = ([
            $"source \"($helper_script)\""
            "let context = (resolve_editor_launch_context)"
            "print ($context.editor)"
        ] | str join "\n")
        let output = (run_nu_snippet $snippet {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            EDITOR: "/shells/posix/yazelix_hx.sh"
            YAZELIX_RUNTIME_DIR: $repo_root
        })
        let lines = ($output.stdout | lines)
        let expected_editor = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")

        if (
            ($output.exit_code == 0)
            and (($lines | get -o 0 | default "") == $expected_editor)
        ) {
            print "  ✅ yzx edit now ignores stale ambient wrapper paths and resolves the canonical managed editor wrapper"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx reveal must use the lightweight reveal helper instead of bootstrapping the full command suite.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_reveal_uses_lightweight_reveal_helper [] {
    print "🧪 Testing yzx CLI reveal uses the lightweight reveal helper..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_reveal_cli")

    let result = (try {
        let target_path = ($fixture.tmpdir | path join "target.txt")
        "" | save --force --raw $target_path

        install_argument_logging_probe $fixture

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {
            HOME: $fixture.fake_home
            NU_LOG: $fixture.nu_log
        } {
            ^$launcher_script reveal $target_path | complete
        })

        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_reveal_script = (repo_path "nushell" "scripts" "integrations" "reveal_in_yazi.nu")

        if (
            ($output.exit_code == 0)
            and (($invocation | get -o 0 | default "") == $expected_reveal_script)
            and (($invocation | get -o 1 | default "") == $target_path)
            and not ($invocation | any {|arg| $arg == "-c" })
            and not ($invocation | any {|arg| $arg | str contains "core/yazelix.nu" })
        ) {
            print "  ✅ yzx reveal now dispatches to the lightweight reveal helper instead of the full command suite"
            true
        } else {
            print $"  ❌ Unexpected yzx reveal invocation: exit=($output.exit_code) args=($invocation | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: yzx menu must use the lightweight menu module instead of bootstrapping the full command suite.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_menu_uses_lightweight_menu_module [] {
    print "🧪 Testing yzx CLI menu uses the lightweight menu module..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_menu_cli")

    let result = (try {
        install_argument_logging_probe $fixture

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {
            HOME: $fixture.fake_home
            NU_LOG: $fixture.nu_log
        } {
            ^$launcher_script menu --popup | complete
        })

        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_menu_script = (repo_path "nushell" "scripts" "yzx" "menu.nu")

        if (
            ($output.exit_code == 0)
            and (($invocation | get -o 0 | default "") == "-c")
            and (($invocation | get -o 1 | default "") | str contains $expected_menu_script)
            and (($invocation | get -o 1 | default "") | str contains "yzx menu --popup")
            and not (($invocation | get -o 1 | default "") | str contains "core/yazelix.nu")
        ) {
            print "  ✅ yzx menu now dispatches through the lightweight menu module instead of the full command suite"
            true
        } else {
            print $"  ❌ Unexpected yzx menu invocation: exit=($output.exit_code) args=($invocation | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: yzx popup must use the lightweight popup module instead of bootstrapping the full command suite.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_popup_uses_lightweight_popup_module [] {
    print "🧪 Testing yzx CLI popup uses the lightweight popup module..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_popup_cli")

    let result = (try {
        install_argument_logging_probe $fixture

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {
            HOME: $fixture.fake_home
            NU_LOG: $fixture.nu_log
        } {
            ^$launcher_script popup lazygit | complete
        })

        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_popup_script = (repo_path "nushell" "scripts" "yzx" "popup.nu")

        if (
            ($output.exit_code == 0)
            and (($invocation | get -o 0 | default "") == "-c")
            and (($invocation | get -o 1 | default "") | str contains $expected_popup_script)
            and (($invocation | get -o 1 | default "") | str contains "yzx popup lazygit")
            and not (($invocation | get -o 1 | default "") | str contains "core/yazelix.nu")
        ) {
            print "  ✅ yzx popup now dispatches through the lightweight popup module instead of the full command suite"
            true
        } else {
            print $"  ❌ Unexpected yzx popup invocation: exit=($output.exit_code) args=($invocation | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: yzx enter must use the lightweight enter module instead of bootstrapping the full command suite.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_enter_uses_lightweight_enter_module [] {
    print "🧪 Testing yzx CLI enter uses the lightweight enter module..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_enter_cli")

    let result = (try {
        let target_dir = ($fixture.tmpdir | path join "project")
        mkdir $target_dir

        install_argument_logging_probe $fixture

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {
            HOME: $fixture.fake_home
            NU_LOG: $fixture.nu_log
        } {
            ^$launcher_script enter --path $target_dir | complete
        })

        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_enter_script = (repo_path "nushell" "scripts" "yzx" "enter.nu")

        if (
            ($output.exit_code == 0)
            and (($invocation | get -o 0 | default "") == "-c")
            and (($invocation | get -o 1 | default "") | str contains $expected_enter_script)
            and (($invocation | get -o 1 | default "") | str contains "yzx enter --path")
            and not (($invocation | get -o 1 | default "") | str contains "core/yazelix.nu")
        ) {
            print "  ✅ yzx enter now dispatches through the lightweight enter module instead of the full command suite"
            true
        } else {
            print $"  ❌ Unexpected yzx enter invocation: exit=($output.exit_code) args=($invocation | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Defends: current-terminal startup uses the requested directory for nonpersistent sessions.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions [] {
    print "🧪 Testing non-persistent current-terminal startup keeps the requested directory for both launch and restart..."

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

# Regression: `yzx launch` no longer accepts current-terminal ownership through `--here`.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yzx_launch_rejects_removed_here_flag [] {
    print "🧪 Testing yzx launch rejects the removed --here flag..."

    let fixture = (setup_enter_forwarding_fixture "yazelix_launch_rejects_here")

    let result = (try {
        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_TEST_LAUNCH_LOG: $fixture.call_log
        } {
            ^nu -c $"use \"($fixture.launch_script)\" *; yzx launch --here" | complete
        })

        let stderr = ($output.stderr | str trim)
        let forwarded_exists = ($fixture.call_log | path exists)

        if (
            ($output.exit_code != 0)
            and ($stderr | str contains "doesn't have flag")
            and ($stderr | str contains "--here")
            and (not $forwarded_exists)
        ) {
            print "  ✅ yzx launch now fails clearly when asked to use the removed current-terminal flag"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) forwarded_exists=($forwarded_exists)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: persistent-session reuse warns when current-terminal startup ignores the requested directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_here_path_warns_when_existing_persistent_session_ignores_it [] {
    print "🧪 Testing current-terminal startup warns when an existing persistent session ignores the requested directory..."

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

# Regression: launch falls through to the next configured terminal after an immediate terminal failure.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_launch_falls_through_after_immediate_terminal_failure [] {
    print "🧪 Testing terminal launch falls through after immediate failure..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_fallback_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        let fake_home = ($tmpdir | path join "home")
        let fake_runtime = ($tmpdir | path join "runtime")
        let fake_shells = ($fake_runtime | path join "shells" "posix")
        let fake_terminal_configs = ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators")
        mkdir $fake_bin
        mkdir $fake_home
        mkdir $fake_runtime
        mkdir ($fake_runtime | path join "shells")
        mkdir $fake_shells
        mkdir ($fake_terminal_configs | path join "wezterm")
        mkdir ($fake_terminal_configs | path join "alacritty")
        ^ln -s (repo_path ".taplo.toml") ($fake_runtime | path join ".taplo.toml")
        "" | save --force --raw ($fake_runtime | path join "yazelix_default.toml")
        "" | save --force --raw ($fake_terminal_configs | path join "wezterm" ".wezterm.lua")
        "" | save --force --raw ($fake_terminal_configs | path join "alacritty" "alacritty.toml")

        [
            "#!/bin/sh"
            "echo wezterm-boom >&2"
            "exit 27"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "wezterm")
        ^chmod +x ($fake_bin | path join "wezterm")

        [
            "#!/bin/sh"
            "sleep 2"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "alacritty")
        ^chmod +x ($fake_bin | path join "alacritty")

        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            "let candidates = (resolve_terminal_candidates '' ['wezterm', 'alacritty'])"
            "if (($candidates | length) != 2) { error make { msg: ($candidates | to json -r) } }"
            "if (($candidates | get 0.command) != 'wezterm') { error make { msg: 'wezterm not selected first' } }"
            "if (($candidates | get 1.command) != 'alacritty') { error make { msg: 'alacritty not selected second' } }"
            "let launched = (launch_terminal_candidates $candidates 'yazelix' $env.PWD false $env.YAZELIX_RUNTIME_DIR false '')"
            "print ($launched.terminal)"
        ] | str join "\n")
        let output = (with-env {
            HOME: $fake_home
            YAZELIX_RUNTIME_DIR: $fake_runtime
            YAZELIX_STATE_DIR: ($tmpdir | path join "state")
            PATH: ([$fake_bin] | append $env.PATH)
        } {
            run_nu_snippet $snippet
        })
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "failed to start; trying Alacritty") and ($stdout | str ends-with "alacritty") and ($stderr == "") {
            print "  ✅ Terminal launch falls through to the next configured terminal after an immediate failure"
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

        if ($output.exit_code == 0) and ($stdout | str contains "Missing Yazelix generated Zellij layout") and ($stdout | str contains "yzx doctor") and ($stdout | str contains "Failure class: generated-state problem.") {
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

# Defends: new-window launch preflight requires the runtime launch script before deeper execution.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_requires_runtime_launch_script [] {
    print "🧪 Testing new-window launch preflight requires the runtime launch script..."

    try {
        let launch_script = (repo_path "nushell" "scripts" "yzx" "launch.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            'try {'
            '    require_launch_runtime_script "/tmp/yazelix_missing_launch_yazelix.nu" | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Missing Yazelix launch script") and ($stdout | str contains "Reinstall/regenerate Yazelix") and ($stdout | str contains "Failure class: generated-state problem.") {
            print "  ✅ New-window launch fails clearly when the runtime launch script is missing"
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

# Regression: workspace retarget should return plugin-owned editor/sidebar targeting truth in one response.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_retarget_workspace_for_path_returns_plugin_owned_sidebar_state_and_editor_status [] {
    print "🧪 Testing workspace retarget returns plugin-owned editor/sidebar targeting truth in one response..."

    let fixture = (setup_managed_config_fixture
        "yazelix_workspace_retarget_truth"
        '[core]
skip_welcome_screen = true
'
    )

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        let target_dir = ($fixture.tmp_home | path join "workspace")
        mkdir $fake_bin
        mkdir $target_dir

        write_probe_nu ($fake_bin | path join "zellij") [
            "#!/bin/sh"
            "for arg in \"$@\"; do"
            "  if [ \"$arg\" = \"retarget_workspace\" ]; then"
            "    printf '%s\\n' '{\"status\":\"ok\",\"editor_status\":\"ok\",\"sidebar_yazi_id\":\"plugin-sidebar-yazi-123\",\"sidebar_yazi_cwd\":\"/home/sidebar\"}'"
            "    exit 0"
            "  fi"
            "done"
            "printf '%s\\n' \"unexpected zellij args: $*\" >&2"
            "exit 1"
        ]

        let retarget_result = (with-env {
            HOME: $fixture.tmp_home
            PATH: ($env.PATH | prepend $fake_bin)
            ZELLIJ: "1"
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
        } {
            retarget_workspace_for_path $target_dir "helix" "workspace_truth.log"
        })

        if (
            ($retarget_result.status == "ok")
            and ($retarget_result.workspace_root == $target_dir)
            and ($retarget_result.tab_name == "workspace")
            and (($retarget_result.editor_status? | default "") == "ok")
            and (($retarget_result.sidebar_state.yazi_id? | default "") == "plugin-sidebar-yazi-123")
            and (($retarget_result.sidebar_state.cwd? | default "") == "/home/sidebar")
        ) {
            print "  ✅ workspace retarget now returns plugin-owned editor/sidebar targeting truth in one response"
            true
        } else {
            print $"  ❌ Unexpected retarget result: ($retarget_result | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

export def run_workspace_canonical_tests [] {
    [
        (test_yzx_cli_desktop_launch_ignores_hostile_shell_env)
        (test_yzx_desktop_launch_uses_leaf_launch_module_with_clean_env)
        (test_yzx_desktop_launch_propagates_fast_path_failures_without_fallback)
        (test_desktop_fast_path_rejects_bootstrap_terminal_substitution_for_explicit_terminal)
        (test_desktop_fast_path_uses_direct_host_terminal_during_reload_instead_of_stale_wrapper)
        (test_yzx_edit_resolves_managed_helix_wrapper_from_canonical_launch_env)
        (test_yzx_cli_reveal_uses_lightweight_reveal_helper)
        (test_yzx_cli_menu_uses_lightweight_menu_module)
        (test_yzx_cli_popup_uses_lightweight_popup_module)
        (test_yzx_cli_enter_uses_lightweight_enter_module)
        (test_startup_bootstrap_runtime_env_exports_state_and_logs_dirs)
        (test_startup_rejects_missing_working_dir)
        (test_launch_rejects_file_working_dir)
        (test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions)
        (test_yzx_launch_rejects_removed_here_flag)
        (test_launch_here_path_warns_when_existing_persistent_session_ignores_it)
        (test_launch_falls_through_after_immediate_terminal_failure)
        (test_startup_requires_generated_layout_path)
        (test_launch_requires_runtime_launch_script)
        (test_retarget_workspace_for_path_returns_plugin_owned_sidebar_state_and_editor_status)
        (test_yzx_cwd_requires_zellij)
    ]
}
