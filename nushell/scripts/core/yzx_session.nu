#!/usr/bin/env nu
# Session and restart commands that still live in Nushell.

use ../utils/atomic_writes.nu write_text_atomic
use ../utils/launcher_resolution.nu resolve_stable_yzx_wrapper_path
use ../yzx/launch.nu ["yzx launch"]

def print_completed_output [result: record] {
    let stdout_text = ($result.stdout | default "")
    let stderr_text = ($result.stderr | default "")

    if ($stdout_text | is-not-empty) {
        print --raw $stdout_text
    }

    if ($stderr_text | is-not-empty) {
        print --stderr --raw $stderr_text
    }
}

def get_current_zellij_session [] {
    if ($env.ZELLIJ_SESSION_NAME? | is-not-empty) {
        return $env.ZELLIJ_SESSION_NAME
    }

    try {
        let current_line = (
            zellij list-sessions
            | lines
            | where {|line| ($line =~ '\bcurrent\b')}
            | first
        )

        let clean_line = (
            $current_line
            | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' ''
            | str replace -r '^>\s*' ''
            | str trim
        )

        if ($clean_line | is-empty) {
            return null
        }

        return (
            $clean_line
            | split row " "
            | where {|token| $token != ""}
            | first
        )
    } catch {
        return null
    }
}

def kill_zellij_session [session_name?: string] {
    try {
        if ($session_name | is-empty) {
            print "⚠️  No Zellij session detected to close"
            return null
        }

        print $"Killing Zellij session: ($session_name)"
        zellij kill-session $session_name
        return $session_name
    } catch {|err|
        print $"❌ Failed to kill session: ($err.msg)"
        return null
    }
}

def create_restart_sidebar_bootstrap_file [target_dir: string] {
    let state_dir = ($env.HOME | path join ".local" "share" "yazelix" "state" "restart")
    mkdir $state_dir

    let bootstrap_file = (^mktemp ($state_dir | path join "sidebar_cwd_XXXXXX") | str trim)
    write_text_atomic $bootstrap_file ($target_dir | path expand) --raw | ignore
    $bootstrap_file
}

# Restart yazelix
export def "yzx restart" [] {
    let session_to_kill = get_current_zellij_session
    let restart_sidebar_cwd_file = (create_restart_sidebar_bootstrap_file (pwd))
    let restart_env = {
        YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: $restart_sidebar_cwd_file
    }

    let is_yazelix_terminal = ($env.YAZELIX_TERMINAL? | is-not-empty)

    if $is_yazelix_terminal {
        print "🔄 Restarting Yazelix..."
    } else {
        print "🔄 Restarting Yazelix \(opening new window\)..."
    }

    let stable_wrapper = (resolve_stable_yzx_wrapper_path)
    if $stable_wrapper != null {
        let launch_output = (with-env $restart_env {
            ^$stable_wrapper launch | complete
        })
        if $launch_output.exit_code != 0 {
            print_completed_output $launch_output
            print "❌ Failed to relaunch Yazelix through the stable owner wrapper."
            exit $launch_output.exit_code
        }
    } else {
        with-env $restart_env {
            yzx launch
        }
    }

    sleep 1sec
    kill_zellij_session $session_to_kill
}
