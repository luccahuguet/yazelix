#!/usr/bin/env nu
# Yazi integration utilities for Yazelix

use ../utils/logging.nu log_to_file
use ../utils/config_parser.nu parse_yazelix_config
use zellij.nu [open_in_existing_helix, open_in_existing_neovim, open_new_helix_pane, open_new_neovim_pane, get_workspace_root, set_workspace_for_path, focus_managed_pane, set_managed_editor_cwd, debug_editor_state]

# Check if the editor command is Helix (supports both simple names and full paths)
# This allows yazelix to work with "hx", "helix", "/nix/store/.../bin/hx", "/usr/bin/hx", etc.
def is_helix_editor [editor: string] {
    ($editor | str ends-with "/hx") or ($editor == "hx") or ($editor | str ends-with "/helix") or ($editor == "helix")
}

# Check if the editor command is Neovim (supports both simple names and full paths)
# This allows yazelix to work with "nvim", "neovim", "/nix/store/.../bin/nvim", "/usr/bin/nvim", etc.
def is_neovim_editor [editor: string] {
    ($editor | str ends-with "/nvim") or ($editor == "nvim") or ($editor | str ends-with "/neovim") or ($editor == "neovim")
}

export def get_managed_editor_kind [] {
    let config = parse_yazelix_config
    let configured_editor = ($config.editor_command? | default null)
    let editor = if ($configured_editor != null) and (($configured_editor | into string | str trim) | is-not-empty) {
        $configured_editor | into string
    } else {
        $env.EDITOR? | default ""
    }

    if (is_helix_editor $editor) {
        "helix"
    } else if (is_neovim_editor $editor) {
        "neovim"
    } else {
        null
    }
}

def is_sidebar_enabled [] {
    let config = parse_yazelix_config
    ($config.enable_sidebar? | default true)
}

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

export def get_sidebar_yazi_state_path [session_name: string, pane_id: string] {
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
            yazi_id: $yazi_id
            cwd: ($state_lines | get -o 1 | default "" | str trim)
        }
    }
}

def read_active_sidebar_state [] {
    let session_name = (get_current_zellij_session_name)
    if ($session_name | is-empty) {
        return null
    }

    let sidebar_pane_id = (
        try {
            let state = (debug_editor_state)
            let pane_id = ($state.sidebar_pane_id? | default "" | into string | str trim)
            if ($pane_id | is-empty) { null } else { $pane_id }
        } catch {
            null
        }
    )
    if ($sidebar_pane_id | is-empty) {
        return null
    }

    read_sidebar_state_file (get_sidebar_yazi_state_path $session_name $sidebar_pane_id)
}

export def get_active_sidebar_cwd [] {
    let sidebar_state = (read_active_sidebar_state)
    if ($sidebar_state | is-empty) {
        null
    } else {
        let cwd = ($sidebar_state.cwd? | default "" | str trim)
        if ($cwd | is-empty) {
            null
        } else {
            $cwd
        }
    }
}

export def sync_active_sidebar_yazi_to_directory [target_path: path, log_file: string = "yazi_sync.log"] {
    if not (is_sidebar_enabled) {
        return {status: "skipped", reason: "sidebar_disabled"}
    }

    if ($env.ZELLIJ? | is-empty) {
        return {status: "skipped", reason: "outside_zellij"}
    }

    if (which ya | is-empty) {
        return {status: "skipped", reason: "ya_missing"}
    }

    let sidebar_state = (read_active_sidebar_state)
    if ($sidebar_state | is-empty) {
        return {status: "skipped", reason: "sidebar_yazi_missing"}
    }

    let expanded_target_path = ($target_path | path expand)
    let target_dir = if (($expanded_target_path | path type) == "dir") {
        $expanded_target_path
    } else {
        $expanded_target_path | path dirname
    }

    try {
        ya emit-to $sidebar_state.yazi_id cd $target_dir
        log_to_file $log_file $"Synced active sidebar Yazi to directory: ($target_dir)"
        {status: "ok", target_dir: $target_dir}
    } catch {|err|
        log_to_file $log_file $"Failed to sync active sidebar Yazi to directory '($target_dir)': ($err.msg)"
        {status: "error", reason: $err.msg, target_dir: $target_dir}
    }
}

export def sync_managed_editor_cwd [target_path: path, log_file: string = "editor_sync.log"] {
    if ($env.ZELLIJ? | is-empty) {
        return {status: "skipped", reason: "outside_zellij"}
    }

    let editor_kind = (get_managed_editor_kind)
    if ($editor_kind | is-empty) {
        return {status: "skipped", reason: "unsupported_editor"}
    }

    let result = (set_managed_editor_cwd $editor_kind $target_path $log_file)
    match $result.status {
        "ok" => {
            log_to_file $log_file $"Synced managed editor cwd to: ($result.working_dir)"
            $result
        }
        "missing" => {
            {status: "skipped", reason: "editor_missing", editor: $editor_kind}
        }
        "unsupported_editor" => {
            {status: "skipped", reason: "unsupported_editor", editor: $editor_kind}
        }
        _ => $result
    }
}

# Sync yazi's directory to match the opened file's location
# This keeps yazi's view synchronized with the tab name and editor context
def sync_yazi_to_directory [file_path: path, yazi_id: string, log_file: string] {
    if ($yazi_id | is-empty) {
        log_to_file $log_file "YAZI_ID not set, skipping yazi navigation"
        return
    }

    let target_dir = if ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    try {
        ya emit-to $yazi_id cd $target_dir
        log_to_file $log_file $"Successfully navigated yazi to directory: ($target_dir)"
    } catch {|err|
        log_to_file $log_file $"Failed to navigate yazi: ($err.msg)"
    }
}

# Navigate Yazi to the directory of the current Helix buffer
export def reveal_in_yazi [buffer_name: string] {
    log_to_file "reveal_in_yazi.log" $"reveal_in_yazi called with buffer_name: '($buffer_name)'"

    # Check if sidebar mode is enabled
    let sidebar_enabled = is_sidebar_enabled
    if (not $sidebar_enabled) {
        let friendly_msg = "📂 Reveal in Yazi only works in sidebar mode. You're currently using no-sidebar mode."
        let tip_msg = "💡 Tip: Use your editor-local file picker in no-sidebar mode, or enable sidebar mode in yazelix.toml"
        print $"($friendly_msg)\n($tip_msg)"
        log_to_file "reveal_in_yazi.log" "Sidebar mode disabled - reveal_in_yazi not available"
        return
    }

    if ($buffer_name | is-empty) {
        let error_msg = "Buffer name not provided"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }

    log_to_file "reveal_in_yazi.log" $"Normalized buffer name: '($normalized_buffer_name)'"

    let full_path = ($env.PWD | path join $normalized_buffer_name | path expand)
    log_to_file "reveal_in_yazi.log" $"Resolved full path: '($full_path)'"

    if not ($full_path | path exists) {
        let error_msg = $"Resolved path '($full_path)' does not exist."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    if ($env.YAZI_ID | is-empty) {
        let error_msg = "YAZI_ID not set. reveal_in_yazi requires that you open helix from yazelix's yazi sidebar"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    log_to_file "reveal_in_yazi.log" $"YAZI_ID found: '($env.YAZI_ID)'"

    try {
        # Use 'reveal' command instead of 'cd' to both navigate to directory and select the file
        ya emit-to $env.YAZI_ID reveal $full_path
        log_to_file "reveal_in_yazi.log" $"Successfully sent 'reveal ($full_path)' command to yazi instance ($env.YAZI_ID)"

        let focus_result = (focus_managed_pane "sidebar" "reveal_in_yazi.log")
        if $focus_result.status == "ok" {
            log_to_file "reveal_in_yazi.log" "Successfully focused managed sidebar pane"
        } else {
            let error_msg = $"Managed sidebar pane focus failed \(status=($focus_result.status)\). Ensure the Yazelix pane orchestrator plugin is loaded and the sidebar pane title is 'sidebar'."
            log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
            print $"Error: ($error_msg)"
        }
    } catch {|err|
        let error_msg = $"Failed to execute yazi/zellij commands: ($err.msg)"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
    }
}


# Generic function to find and open file with editor integration
def open_with_editor_integration [
    file_path: path
    yazi_id: string
    editor_name: string
    log_file: string
    open_in_existing: closure
    open_new_pane: closure
] {
    log_to_file $log_file $"open_with_($editor_name) called with file_path: '($file_path)'"

    let open_result = (do $open_in_existing $file_path)

    if $open_result.status == "ok" {
        log_to_file $log_file $"Managed editor pane found for ($editor_name), opening in existing instance through pane orchestrator"
        print $"($editor_name) pane found, opening in existing instance"
    } else if $open_result.status == "missing" {
        log_to_file $log_file $"Managed editor pane missing for ($editor_name), opening new pane"
        print $"($editor_name) pane not found, opening new pane"
        do $open_new_pane $file_path $yazi_id
    } else {
        let error_msg = $"Managed editor open failed for ($editor_name) \(status=($open_result.status)\). Ensure the Yazelix pane orchestrator plugin is loaded and the editor pane title is 'editor'."
        log_to_file $log_file $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    let workspace_result = (set_workspace_for_path $file_path $log_file)
    if $workspace_result.status == "ok" {
        log_to_file $log_file $"Updated workspace root to: ($workspace_result.workspace_root)"
    } else {
        log_to_file $log_file $"WARNING: Failed to update workspace root \(status=($workspace_result.status)\)"
    }

    # Sync yazi's directory to match the opened file's location
    sync_yazi_to_directory $file_path $yazi_id $log_file

    # In no-sidebar mode, we leave the Yazi pane open - no need to close it
    let sidebar_enabled = is_sidebar_enabled
    if (not $sidebar_enabled) {
        log_to_file $log_file $"No-sidebar mode: leaving Yazi pane open, no close operation needed"
    }

    log_to_file $log_file $"open_with_($editor_name) function completed"
}

# Open file with Helix (with full Yazelix integration)
def open_with_helix [file_path: path, yazi_id: string] {
    open_with_editor_integration $file_path $yazi_id "Helix" "open_helix.log" {|path| open_in_existing_helix $path} {|path, id| open_new_helix_pane $path $id}
}

# Open file with Neovim (with full Yazelix integration)
def open_with_neovim [file_path: path, yazi_id: string] {
    open_with_editor_integration $file_path $yazi_id "Neovim" "open_neovim.log" {|path| open_in_existing_neovim $path} {|path, id| open_new_neovim_pane $path $id}
}

# Open file with generic editor (basic Zellij integration)
def open_with_generic_editor [file_path: path, editor: string, yazi_id: string] {
    log_to_file "open_generic.log" $"open_with_generic_editor called with file_path: '($file_path)', editor: '($editor)'"

    let workspace_root = (get_workspace_root $file_path)
    log_to_file "open_generic.log" $"Using workspace root: ($workspace_root)"

    try {
        # Create a new pane with the editor
        zellij action new-pane --cwd $workspace_root -- $editor $file_path

        log_to_file "open_generic.log" $"Successfully opened ($file_path) with ($editor) in new pane"
        print $"Opened ($file_path) with ($editor) in new pane"

        let workspace_result = (set_workspace_for_path $file_path "open_generic.log")
        if $workspace_result.status == "ok" {
            log_to_file "open_generic.log" $"Updated workspace root to: ($workspace_result.workspace_root)"
        } else {
            log_to_file "open_generic.log" $"WARNING: Failed to update workspace root \(status=($workspace_result.status)\)"
        }
    } catch {|err|
        let error_msg = $"Failed to open file with ($editor): ($err.msg)"
        log_to_file "open_generic.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
    }

    # Sync yazi's directory to match the opened file's location
    sync_yazi_to_directory $file_path $yazi_id "open_generic.log"

    log_to_file "open_generic.log" "open_with_generic_editor function completed"
}


# Main file opening function - dispatches to appropriate editor handler
export def open_file_with_editor [file_path: path] {
    log_to_file "open_editor.log" $"open_file_with_editor called with file_path: '($file_path)'"
    print $"DEBUG: file_path received: ($file_path), type: ($file_path | path type)"

    if not ($file_path | path exists) {
        let error_msg = $"File path ($file_path) does not exist"
        log_to_file "open_editor.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    # Get the configured editor
    let editor = $env.EDITOR
    if ($editor | is-empty) {
        let error_msg = "EDITOR environment variable is not set"
        log_to_file "open_editor.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    log_to_file "open_editor.log" $"Using editor: ($editor)"

    # Check if sidebar is enabled
    let sidebar_enabled = is_sidebar_enabled
    log_to_file "open_editor.log" $"Sidebar enabled: ($sidebar_enabled)"

    # Capture YAZI_ID from Yazi's pane
    let yazi_id = $env.YAZI_ID
    if ($yazi_id | is-empty) {
        let warning_msg = "YAZI_ID not set in this environment. Yazi navigation may fail."
        log_to_file "open_editor.log" $"WARNING: ($warning_msg)"
        print $"Warning: ($warning_msg)"
    } else {
        log_to_file "open_editor.log" $"YAZI_ID found: '($yazi_id)'"
    }

    # For no-sidebar mode, we still use the multi-pane approach since we start with the editor.
    # Editor-local file pickers handle the "open in same pane" workflow outside the sidebar layout.

    # Dispatch to the appropriate editor handler
    if (is_helix_editor $editor) {
        log_to_file "open_editor.log" "Detected Helix editor, using Helix-specific logic"
        open_with_helix $file_path $yazi_id
    } else if (is_neovim_editor $editor) {
        log_to_file "open_editor.log" "Detected Neovim editor, using Neovim-specific logic"
        open_with_neovim $file_path $yazi_id
    } else {
        log_to_file "open_editor.log" $"Using generic editor approach for: ($editor)"
        open_with_generic_editor $file_path $editor $yazi_id
    }

    log_to_file "open_editor.log" "open_file_with_editor function completed"
}
