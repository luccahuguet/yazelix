#!/usr/bin/env nu

use ../integrations/yazi.nu [resolve_reveal_target_path]
use ./test_yzx_helpers.nu [CLEAN_ZELLIJ_ENV_PREFIX get_repo_config_dir get_repo_root repo_path]

def run_nu_snippet [snippet: string, extra_env?: record] {
    let result = if ($extra_env | is-empty) {
        ^nu -c $snippet | complete
    } else {
        with-env $extra_env {
            ^nu -c $snippet | complete
        }
    }
    $result
}

def setup_launch_path_fixture [label: string, persistent_sessions: bool, existing_session: bool] {
    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let fake_bin = ($tmp_home | path join "bin")
    let zellij_log = ($tmp_home | path join "zellij.log")
    let existing_session_flag = if $existing_session { "true" } else { "false" }
    let real_nu = (which nu | get 0.path)

    mkdir $runtime_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir
    mkdir $fake_bin

    for entry in ["nushell", "shells", "configs", "devenv.lock", "yazelix_default.toml", "docs", "CHANGELOG.md", "assets"] {
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

def test_posix_desktop_launcher_direct_exec_ignores_hostile_shell_env [] {
    print "🧪 Testing POSIX desktop launcher direct exec ignores hostile shell env..."

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

        let launcher_script = (repo_path "shells" "posix" "desktop_launcher.sh")
        let output = (with-env {HOME: $fake_home, BASH_ENV: $env_file, ENV: $env_file} {
            ^$launcher_script | complete
        })
        let stderr = ($output.stderr | str trim)
        let nu_invocation = if ($nu_log | path exists) {
            open --raw $nu_log | str trim
        } else {
            ""
        }

        if ($output.exit_code == 0) and ($stderr == "") and ($nu_invocation | str ends-with "nushell/scripts/core/desktop_launcher.nu") {
            print "  ✅ POSIX desktop launcher reaches Nushell without sourcing hostile shell env files"
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

def test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions [] {
    print "🧪 Testing yzx launch --here --path honors the requested directory for non-persistent sessions..."

    let fixture = (setup_launch_path_fixture "yazelix_launch_here_path_nonpersistent" false false)

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

        if ($output.exit_code == 0) and ($zellij_log | str contains $"options --default-cwd ($target_dir)") and (not ($stdout | str contains "--path ignored")) {
            print "  ✅ Non-persistent sessions pass the requested launch path through to Zellij"
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

def test_startup_requires_generated_layout_path [] {
    print "🧪 Testing startup requires an existing Zellij layout..."

    try {
        let start_inner = (repo_path "nushell" "scripts" "core" "start_yazelix_inner.nu")
        let snippet = ([
            $"source \"($start_inner)\""
            'try {'
            '    require_existing_layout "/tmp/yazelix_missing_layout.kdl" | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Zellij layout not found") and ($stdout | str contains "yzx refresh") and ($stdout | str contains "Failure class: generated-state problem.") {
            print "  ✅ Startup fails clearly when the generated layout is missing"
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

def test_yzx_cwd_resolves_zoxide_query [] {
    print "🧪 Testing yzx cwd zoxide resolution..."

    try {
        if (which zoxide | is-empty) {
            print "  ℹ️  Skipping zoxide resolution test because zoxide is not available"
            return true
        }

        let repo_dir = (get_repo_config_dir)
        ^zoxide add $repo_dir
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let output = (^nu -c $"use \"($yzx_script)\" *; resolve_yzx_cwd_target yazelix" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == $repo_dir) {
            print "  ✅ yzx cwd resolves zoxide queries before updating the tab directory"
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
        (test_posix_desktop_launcher_direct_exec_ignores_hostile_shell_env)
        (test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions)
        (test_launch_here_path_warns_when_existing_persistent_session_ignores_it)
        (test_startup_rejects_missing_working_dir)
        (test_launch_rejects_file_working_dir)
        (test_startup_requires_generated_layout_path)
        (test_yzx_cwd_requires_zellij)
        (test_yzx_cwd_resolves_zoxide_query)
    ]
}

export def run_workspace_tests [] {
    run_workspace_canonical_tests
}
