#!/usr/bin/env nu

use ../integrations/yazi.nu [consume_bootstrap_sidebar_cwd resolve_reveal_target_path]
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

def test_consume_bootstrap_sidebar_cwd [] {
    print "🧪 Testing restart-only sidebar Yazi cwd bootstrap..."

    let tmpdir = (^mktemp -d /tmp/yazelix_sidebar_bootstrap_XXXXXX | str trim)

    let result = (try {
        let workspace_dir = ($tmpdir | path join "workspace")
        mkdir $workspace_dir
        let bootstrap_file = ($tmpdir | path join "sidebar_cwd.txt")
        $workspace_dir | save --force --raw $bootstrap_file

        let resolved = (with-env {YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: $bootstrap_file} {
            consume_bootstrap_sidebar_cwd
        })

        if ($resolved == $workspace_dir) and (not ($bootstrap_file | path exists)) {
            print "  ✅ Sidebar Yazi bootstrap cwd is consumed exactly once"
            true
        } else {
            print $"  ❌ Unexpected result: resolved=($resolved) file_exists=(($bootstrap_file | path exists))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_restart_uses_home_for_future_tab_defaults [] {
    print "🧪 Testing restart keeps pane and tab defaults at HOME..."

    try {
        let start_inner = (repo_path "nushell" "scripts" "core" "start_yazelix_inner.nu")
        let snippet = ([
            $"source \"($start_inner)\""
            'with-env {HOME: "/tmp/yazelix-home", YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: "/tmp/sidebar-bootstrap"} {'
            '    print ({'
            '        session_default: (resolve_session_default_cwd "/tmp/restart-workspace")'
            '        launch_process: (resolve_launch_process_cwd "/tmp/restart-workspace")'
            '    } | to json -r)'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == '{"session_default":"/tmp/yazelix-home","launch_process":"/tmp/yazelix-home"}') {
            print "  ✅ Restart keeps both the launch process and future tab defaults at HOME"
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

def test_terminal_launch_requires_bash [] {
    print "🧪 Testing terminal launch reports missing bash..."

    let tmpdir = (^mktemp -d /tmp/yazelix_launch_bash_test_XXXXXX | str trim)

    let result = (try {
        let nu_bin = (which nu | get 0.path)
        let isolated_path = ($tmpdir | path join "bin")
        mkdir $isolated_path
        ^ln -s $nu_bin ($isolated_path | path join "nu")
        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            'try {'
            '    run_detached_terminal_launch "exit 0" "Test Terminal"'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet {PATH: $isolated_path})
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "bash is not available in PATH") and ($stdout | str contains "Failure class: host-dependency problem.") {
            print "  ✅ Terminal launch fails clearly when bash is unavailable"
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

def test_terminal_launch_reports_immediate_failure [] {
    print "🧪 Testing terminal launch reports immediate command failures..."

    try {
        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            'try {'
            '    run_detached_terminal_launch "echo launch-broke >&2; exit 23" "Test Terminal"'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Failed to launch Test Terminal") and ($stdout | str contains "exit code: 23") and ($stdout | str contains "launch-broke") {
            print "  ✅ Terminal launch failures include exit code and stderr context"
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

def test_posix_startup_launcher_reports_missing_runtime_script [] {
    print "🧪 Testing POSIX startup launcher reports missing runtime script..."

    let tmpdir = (^mktemp -d /tmp/yazelix_posix_startup_test_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_home
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")
        ^ln -s /bin/sh ($fake_bin | path join "sh")

        let startup_script = (repo_path "shells" "posix" "start_yazelix.sh")
        let output = (with-env {HOME: $fake_home, PATH: $fake_bin} {
            ^sh $startup_script | complete
        })
        let stderr = ($output.stderr | str trim)

        if ($output.exit_code == 1) and ($stderr | str contains "Missing Yazelix startup script") and ($stderr | str contains "runtime looks incomplete") and ($stderr | str contains "Failure class: generated-state problem.") {
            print "  ✅ POSIX startup launcher fails clearly when the runtime script is missing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_posix_desktop_launcher_reports_missing_runtime_script [] {
    print "🧪 Testing POSIX desktop launcher reports missing runtime script..."

    let tmpdir = (^mktemp -d /tmp/yazelix_posix_desktop_test_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_home
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")
        ^ln -s /bin/sh ($fake_bin | path join "sh")

        let launcher_script = (repo_path "shells" "posix" "desktop_launcher.sh")
        let output = (with-env {HOME: $fake_home, PATH: $fake_bin} {
            ^sh $launcher_script | complete
        })
        let stderr = ($output.stderr | str trim)

        if ($output.exit_code == 1) and ($stderr | str contains "Missing Yazelix desktop launcher") and ($stderr | str contains "runtime looks incomplete") and ($stderr | str contains "Failure class: generated-state problem.") {
            print "  ✅ POSIX desktop launcher fails clearly when the runtime script is missing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
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

def test_sidebar_layout_uses_wrapper_launcher [] {
    print "🧪 Testing sidebar layouts use the Yazi wrapper launcher..."

    try {
        let side_layout = (open --raw (repo_path "configs" "zellij" "layouts" "yzx_side.kdl"))
        let no_side_layout = (open --raw (repo_path "configs" "zellij" "layouts" "yzx_no_side.kdl"))
        let swap_fragment = (open --raw (repo_path "configs" "zellij" "layouts" "fragments" "swap_sidebar_open.kdl"))

        if ($side_layout | str contains "launch_sidebar_yazi.nu") and ($no_side_layout | str contains "launch_sidebar_yazi.nu") and ($swap_fragment | str contains "launch_sidebar_yazi.nu") {
            print "  ✅ Sidebar layouts launch Yazi through the restart-aware wrapper"
            true
        } else {
            print "  ❌ One or more sidebar layouts still launch Yazi directly"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sidebar_wrapper_bootstraps_workspace_root [] {
    print "🧪 Testing sidebar Yazi wrapper bootstraps the tab workspace root..."

    try {
        let wrapper = (open --raw (repo_path "configs" "zellij" "scripts" "launch_sidebar_yazi.nu"))

        if ($wrapper | str contains 'bootstrap_workspace_root $target_dir') and ($wrapper | str contains '^yazi $target_dir') and ($wrapper | str contains 'pwd | path expand') {
            print "  ✅ Sidebar Yazi wrapper always roots the tab to the launched Yazi directory"
            true
        } else {
            print "  ❌ Sidebar Yazi wrapper is missing the always-bootstrap target-dir flow"
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

def test_resolve_reveal_target_path_from_relative_buffer [] {
    print "🧪 Testing reveal target resolution for relative buffer paths..."

    try {
        let expected_readme = ((get_repo_root) | path join "README.md")
        let stdout = (do {
            cd (get_repo_config_dir)
            resolve_reveal_target_path "README.md"
        } | str trim)

        if $stdout == $expected_readme {
            print "  ✅ Reveal target resolution expands relative buffer paths against the current cwd"
            true
        } else {
            print $"  ❌ Unexpected result: stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_workspace_tests [] {
    [
        (test_consume_bootstrap_sidebar_cwd)
        (test_restart_uses_home_for_future_tab_defaults)
        (test_startup_rejects_missing_working_dir)
        (test_launch_rejects_file_working_dir)
        (test_terminal_launch_requires_bash)
        (test_terminal_launch_reports_immediate_failure)
        (test_posix_startup_launcher_reports_missing_runtime_script)
        (test_posix_desktop_launcher_reports_missing_runtime_script)
        (test_startup_requires_generated_layout_path)
        (test_sidebar_layout_uses_wrapper_launcher)
        (test_sidebar_wrapper_bootstraps_workspace_root)
        (test_yzx_cwd_requires_zellij)
        (test_yzx_cwd_resolves_zoxide_query)
        (test_resolve_reveal_target_path_from_relative_buffer)
    ]
}
