#!/usr/bin/env nu

use ../integrations/yazi.nu [consume_bootstrap_sidebar_cwd]
use ./test_yzx_helpers.nu [CLEAN_ZELLIJ_ENV_PREFIX get_repo_config_dir]

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
        let output = (^nu -c "source ~/.config/yazelix/nushell/scripts/core/start_yazelix_inner.nu; with-env {HOME: '/tmp/yazelix-home', YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: '/tmp/sidebar-bootstrap'} { print ({ session_default: (resolve_session_default_cwd '/tmp/restart-workspace'), launch_process: (resolve_launch_process_cwd '/tmp/restart-workspace') } | to json -r) }" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == '{"session_default":"/tmp/yazelix-home","launch_process":"/tmp/yazelix-home"}') {
            print "  ✅ Restart keeps both the launch process and future tab defaults at HOME"
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

def test_sidebar_layout_uses_wrapper_launcher [] {
    print "🧪 Testing sidebar layouts use the Yazi wrapper launcher..."

    try {
        let side_layout = (open --raw ~/.config/yazelix/configs/zellij/layouts/yzx_side.kdl)
        let no_side_layout = (open --raw ~/.config/yazelix/configs/zellij/layouts/yzx_no_side.kdl)
        let swap_fragment = (open --raw ~/.config/yazelix/configs/zellij/layouts/fragments/swap_sidebar_open.kdl)

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
        let wrapper = (open --raw ~/.config/yazelix/configs/zellij/scripts/launch_sidebar_yazi.nu)

        if ($wrapper | str contains 'set_workspace_root') and ($wrapper | str contains 'bootstrap_workspace_root') {
            print "  ✅ Sidebar Yazi wrapper updates the tab workspace root before launch"
            true
        } else {
            print "  ❌ Sidebar Yazi wrapper is missing workspace-root bootstrap logic"
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
        let output = (^bash -lc $"($CLEAN_ZELLIJ_ENV_PREFIX) nu -c 'use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx cwd .'" | complete)
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
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; resolve_yzx_cwd_target yazelix" | complete)
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
        let expected_readme = ((get_repo_config_dir) | path join "README.md")
        let output = (^bash -lc 'cd ~/.config/yazelix && nu -c "use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; print (resolve_reveal_target_path README.md)"' | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == $expected_readme) {
            print "  ✅ Reveal target resolution expands relative buffer paths against the current cwd"
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

export def run_workspace_tests [] {
    [
        (test_consume_bootstrap_sidebar_cwd)
        (test_restart_uses_home_for_future_tab_defaults)
        (test_sidebar_layout_uses_wrapper_launcher)
        (test_sidebar_wrapper_bootstraps_workspace_root)
        (test_yzx_cwd_requires_zellij)
        (test_yzx_cwd_resolves_zoxide_query)
        (test_resolve_reveal_target_path_from_relative_buffer)
    ]
}
