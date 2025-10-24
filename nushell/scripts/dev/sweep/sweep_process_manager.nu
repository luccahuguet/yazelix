#!/usr/bin/env nu
# Sweep Testing - Process Management Utilities
# Handles terminal process tracking and cleanup for visual tests

# Get current terminal processes for a given terminal type
export def get_terminal_pids [terminal: string]: nothing -> list<int> {
    try {
        ps | where name =~ $terminal | get pid
    } catch {
        []
    }
}

# Kill Zellij session by name pattern
export def cleanup_zellij_session [session_pattern: string]: nothing -> nothing {
    try {
        let sessions = (zellij list-sessions | lines | where $it =~ $session_pattern)
        if not ($sessions | is-empty) {
            let session_line = ($sessions | first)
            let session_id = ($session_line | split row " " | first | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' '')
            print $"   Cleaning up session: ($session_id)"
            zellij kill-session $session_id
        }
    } catch {
        print "   Session cleanup skipped"
    }
}

# Kill terminal processes that are new compared to baseline
export def cleanup_terminal_processes [
    terminal: string,
    before_pids: list<int>
]: nothing -> nothing {
    try {
        # Wait for session cleanup to complete
        sleep 1sec

        # Find terminal processes that were started after our baseline
        let after_pids = get_terminal_pids $terminal
        let new_pids = $after_pids | where $it not-in $before_pids

        if not ($new_pids | is-empty) {
            for $pid in $new_pids {
                print $"   Terminating terminal process: ($pid)"
                try {
                    # Graceful termination first (SIGTERM = 15)
                    kill --signal 15 $pid
                    sleep 300ms
                    # Force kill if still running
                    let still_running = try {
                        (ps | where pid == $pid | length) > 0
                    } catch { false }
                    if $still_running {
                        kill --force $pid
                    }
                } catch {
                    print $"   Failed to kill process ($pid)"
                }
            }
        } else {
            print $"   No new terminal processes detected for cleanup"
        }
    } catch {
        print "   Terminal cleanup failed"
    }
}

# Complete cleanup: both session and terminal processes
export def cleanup_visual_test [
    session_name: string,
    terminal: string,
    before_pids: list<int>,
    delay: duration
]: nothing -> nothing {
    print $"   Waiting ($delay) before cleanup..."
    sleep $delay

    # Clean up Zellij session first
    cleanup_zellij_session $session_name

    # Then clean up terminal processes
    cleanup_terminal_processes $terminal $before_pids
}