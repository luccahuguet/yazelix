#!/usr/bin/env nu

use ./zellij.nu debug_editor_state

def get_sidebar_yazi_state_dir [] {
    $env.HOME | path join ".local" "share" "yazelix" "state" "yazi" "sidebar"
}

def sanitize_sidebar_state_component [value: string] {
    $value | str replace -ra '[^A-Za-z0-9._-]' '_'
}

def normalize_sidebar_pane_id [pane_id: string] {
    if ($pane_id | str contains ":") {
        $pane_id
    } else {
        $"terminal:($pane_id)"
    }
}

def get_sidebar_yazi_state_path [session_name: string, pane_id: string] {
    let sanitized_session = (sanitize_sidebar_state_component $session_name)
    let sanitized_pane = (sanitize_sidebar_state_component (normalize_sidebar_pane_id $pane_id))
    (get_sidebar_yazi_state_dir | path join $"($sanitized_session)__($sanitized_pane).txt")
}

def get_current_zellij_session_name [] {
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

def read_sidebar_state_file [state_path: string] {
    if not ($state_path | path exists) {
        return null
    }

    let state_lines = (open --raw $state_path | lines)
    let yazi_id = ($state_lines | get -o 0 | default "" | str trim)
    if ($yazi_id | is-empty) {
        null
    } else {
        {
            path: $state_path
            yazi_id: $yazi_id
            cwd: ($state_lines | get -o 1 | default "" | str trim)
        }
    }
}

def get_session_sidebar_state_files [session_name: string] {
    let state_dir = (get_sidebar_yazi_state_dir)
    if not ($state_dir | path exists) {
        return []
    }

    let session_prefix = ($session_name | str trim)
    if ($session_prefix | is-empty) {
        return []
    }

    ls $state_dir
    | where type == file
    | where { |entry|
        let name = ($entry.name | path basename)
        ($name | str starts-with $"($session_prefix)__") and ($name | str ends-with ".txt")
    }
    | sort-by modified --reverse
    | get name
}

def get_sidebar_pane_state_files [pane_id: string] {
    let state_dir = (get_sidebar_yazi_state_dir)
    if not ($state_dir | path exists) {
        return []
    }

    let normalized_pane_id = ($pane_id | str trim)
    if ($normalized_pane_id | is-empty) {
        return []
    }

    let sanitized_pane = (sanitize_sidebar_state_component (normalize_sidebar_pane_id $normalized_pane_id))
    if ($sanitized_pane | is-empty) {
        return []
    }

    ls $state_dir
    | where type == file
    | where { |entry|
        let name = ($entry.name | path basename)
        ($name | str ends-with $"__($sanitized_pane).txt")
    }
    | sort-by modified --reverse
    | get name
}

export def get_active_sidebar_state [] {
    let sidebar_pane_id = (
        try {
            let state = (debug_editor_state)
            let pane_id = ($state.sidebar_pane_id? | default "" | into string | str trim)
            if ($pane_id | is-empty) { null } else { $pane_id }
        } catch {
            null
        }
    )

    let session_name = (get_current_zellij_session_name)
    let exact_state_path = if (($session_name | is-not-empty) and ($sidebar_pane_id | is-not-empty)) {
        let candidate = (get_sidebar_yazi_state_path $session_name $sidebar_pane_id)
        if ($candidate | path exists) {
            $candidate
        } else {
            null
        }
    } else {
        null
    }

    if ($exact_state_path | is-not-empty) {
        let exact_state = (read_sidebar_state_file $exact_state_path)
        if ($exact_state | is-not-empty) {
            return $exact_state
        }
    }

    let session_paths = if ($session_name | is-not-empty) {
        get_session_sidebar_state_files $session_name
    } else {
        []
    }
    let pane_paths = if ($sidebar_pane_id | is-not-empty) {
        get_sidebar_pane_state_files $sidebar_pane_id
    } else {
        []
    }
    let candidate_paths = ($session_paths ++ $pane_paths | uniq)

    for state_path in $candidate_paths {
        let sidebar_state = (read_sidebar_state_file $state_path)
        if ($sidebar_state | is-not-empty) {
            return $sidebar_state
        }
    }

    null
}
