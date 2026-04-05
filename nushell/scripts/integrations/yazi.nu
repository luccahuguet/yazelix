#!/usr/bin/env nu
# Yazi integration utilities for Yazelix

use ../utils/logging.nu log_to_file
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/editor_launch_context.nu [resolve_editor_launch_context]
use zellij.nu [open_in_existing_helix, open_in_existing_neovim, open_new_helix_pane, open_new_neovim_pane, get_workspace_root, set_workspace_for_path, focus_managed_pane, set_managed_editor_cwd, debug_editor_state]

def resolve_optional_command [configured: any, fallback: string] {
    let raw = ($configured | default "" | into string | str trim)
    if ($raw | is-empty) {
        $fallback
    } else {
        $raw
    }
}

def command_is_available [command: string] {
    let normalized = ($command | str trim)
    if ($normalized | is-empty) {
        false
    } else if (($normalized | str contains "/") or ($normalized | str starts-with "~")) {
        (($normalized | path expand) | path exists)
    } else {
        (which $normalized | is-not-empty)
    }
}

export def get_yazi_command [] {
    let config = parse_yazelix_config
    resolve_optional_command ($config.yazi_command? | default null) "yazi"
}

export def get_ya_command [] {
    let config = parse_yazelix_config
    resolve_optional_command ($config.yazi_ya_command? | default null) "ya"
}

def has_ya_command [] {
    command_is_available (get_ya_command)
}

def run_ya_emit_to [yazi_id: string, action: string, ...args: string] {
    let ya_command = (get_ya_command)
    run-external $ya_command "emit-to" $yazi_id $action ...$args
}

# Check if the editor command is Helix (supports both simple names and full paths)
# This allows yazelix to work with "hx", "helix", "/nix/store/.../bin/hx", "/usr/bin/hx", etc.
def is_helix_editor [editor: string] {
    let normalized = ($editor | str trim)
    let basename = if ($normalized | is-empty) { "" } else { $normalized | path basename }
    ($normalized | str ends-with "/hx")
        or ($normalized == "hx")
        or ($normalized | str ends-with "/helix")
        or ($normalized == "helix")
        or ($basename == "yazelix_hx.sh")
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
    let managed_helix_binary = ($env.YAZELIX_MANAGED_HELIX_BINARY? | default "" | into string | str trim)

    if ($managed_helix_binary | is-not-empty) or (is_helix_editor $editor) {
        "helix"
    } else if (is_neovim_editor $editor) {
        "neovim"
    } else {
        null
    }
}

export def is_sidebar_enabled [] {
    let config = parse_yazelix_config
    ($config.enable_sidebar? | default true)
}

export def consume_bootstrap_sidebar_cwd [] {
    let cwd_file = ($env.YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE? | default "" | str trim)
    if ($cwd_file | is-empty) {
        return null
    }

    let expanded_file = ($cwd_file | path expand)
    if not ($expanded_file | path exists) {
        return null
    }

    let requested_path = (open --raw $expanded_file | str trim)
    rm -f $expanded_file

    if ($requested_path | is-empty) {
        return null
    }

    let expanded_path = ($requested_path | path expand)
    if not ($expanded_path | path exists) {
        return null
    }

    if (($expanded_path | path type) == "dir") {
        $expanded_path
    } else {
        $expanded_path | path dirname
    }
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

def read_active_sidebar_state [] {
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
    let pane_paths = if ($sidebar_pane_id | is-not-empty) {
        get_sidebar_pane_state_files $sidebar_pane_id
    } else {
        []
    }
    let session_paths = if ($session_name | is-not-empty) {
        get_session_sidebar_state_files $session_name
    } else {
        []
    }
    let candidate_paths = ($pane_paths ++ $session_paths | uniq)

    for state_path in $candidate_paths {
        let sidebar_state = (read_sidebar_state_file $state_path)
        if ($sidebar_state | is-not-empty) {
            return $sidebar_state
        }
    }

    null
}

export def get_active_sidebar_yazi_id [] {
    let sidebar_state = (read_active_sidebar_state)
    if ($sidebar_state | is-empty) {
        null
    } else {
        let yazi_id = ($sidebar_state.yazi_id? | default "" | str trim)
        if ($yazi_id | is-empty) {
            null
        } else {
            $yazi_id
        }
    }
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

export def resolve_reveal_target_path [buffer_name: string] {
    if ($buffer_name | is-empty) {
        error make {msg: "Buffer name not provided"}
    }

    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }

    let full_path = if ($normalized_buffer_name | path type) != "relative" {
        $normalized_buffer_name | path expand
    } else {
        ($env.PWD | path join $normalized_buffer_name | path expand)
    }

    if not ($full_path | path exists) {
        error make {msg: $"Resolved path '($full_path)' does not exist."}
    }

    $full_path
}

export def sync_active_sidebar_yazi_to_directory [target_path: path, log_file: string = "yazi_sync.log"] {
    if not (is_sidebar_enabled) {
        return {status: "skipped", reason: "sidebar_disabled"}
    }

    if ($env.ZELLIJ? | is-empty) {
        return {status: "skipped", reason: "outside_zellij"}
    }

    if not (has_ya_command) {
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
        run_ya_emit_to $sidebar_state.yazi_id "cd" $target_dir
        if ($sidebar_state.path? | is-not-empty) {
            $"($sidebar_state.yazi_id)\n($target_dir)\n" | save --force $sidebar_state.path
            log_to_file $log_file $"Updated sidebar state cache: ($sidebar_state.path)"
        }
        log_to_file $log_file $"Synced active sidebar Yazi to directory: ($target_dir)"
        {status: "ok", target_dir: $target_dir}
    } catch {|err|
        log_to_file $log_file $"Failed to sync active sidebar Yazi to directory '($target_dir)': ($err.msg)"
        {status: "error", reason: $err.msg, target_dir: $target_dir}
    }
}

export def refresh_active_sidebar_yazi [log_file: string = "yazi_refresh.log"] {
    if not (is_sidebar_enabled) {
        return {status: "skipped", reason: "sidebar_disabled"}
    }

    if ($env.ZELLIJ? | is-empty) {
        return {status: "skipped", reason: "outside_zellij"}
    }

    if not (has_ya_command) {
        return {status: "skipped", reason: "ya_missing"}
    }

    let sidebar_state = (read_active_sidebar_state)
    if ($sidebar_state | is-empty) {
        return {status: "skipped", reason: "sidebar_yazi_missing"}
    }

    try {
        run_ya_emit_to $sidebar_state.yazi_id "refresh"
        log_to_file $log_file $"Refreshed active sidebar Yazi instance: ($sidebar_state.yazi_id)"
        {status: "ok", yazi_id: $sidebar_state.yazi_id}
    } catch {|err|
        log_to_file $log_file $"Failed to refresh active sidebar Yazi instance '($sidebar_state.yazi_id)': ($err.msg)"
        {status: "error", reason: $err.msg, yazi_id: $sidebar_state.yazi_id}
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

export def resolve_managed_editor_open_strategy [status: string] {
    match $status {
        "ok" => {action: "reuse_managed"}
        "missing" => {action: "open_new_managed"}
        _ => {action: "error", status: $status}
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
        run_ya_emit_to $yazi_id "cd" $target_dir
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

    if ($env.ZELLIJ? | is-empty) {
        let error_msg = "Reveal in Yazi only works inside a Yazelix/Zellij session."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    if not (has_ya_command) {
        let error_msg = $"The configured Yazi CLI `\(get_ya_command\)` is not available in this environment."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    let full_path = try {
        resolve_reveal_target_path $buffer_name
    } catch {|err|
        log_to_file "reveal_in_yazi.log" $"ERROR: ($err.msg)"
        print $"Error: ($err.msg)"
        return
    }

    log_to_file "reveal_in_yazi.log" $"Resolved full path: '($full_path)'"

    let sidebar_yazi_id = (get_active_sidebar_yazi_id)
    if ($sidebar_yazi_id | is-empty) {
        let error_msg = "Managed sidebar Yazi is not available in the current tab. Open the sidebar and try again."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    log_to_file "reveal_in_yazi.log" $"Managed sidebar Yazi ID found: '($sidebar_yazi_id)'"

    try {
        # Use 'reveal' command instead of 'cd' to both navigate to directory and select the file
        run_ya_emit_to $sidebar_yazi_id "reveal" $full_path
        log_to_file "reveal_in_yazi.log" $"Successfully sent 'reveal ($full_path)' command to managed sidebar yazi instance ($sidebar_yazi_id)"

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
    let open_strategy = (resolve_managed_editor_open_strategy $open_result.status)

    if $open_strategy.action == "reuse_managed" {
        log_to_file $log_file $"Managed editor pane found for ($editor_name), opening in existing instance through pane orchestrator"
        print $"($editor_name) pane found, opening in existing instance"
    } else if $open_strategy.action == "open_new_managed" {
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

    let sidebar_enabled = is_sidebar_enabled
    if $sidebar_enabled {
        let sidebar_sync_result = (sync_active_sidebar_yazi_to_directory $file_path $log_file)
        if $sidebar_sync_result.status == "ok" {
            log_to_file $log_file $"Synced active sidebar Yazi to directory: ($sidebar_sync_result.target_dir)"
        } else {
            log_to_file $log_file $"WARNING: Active sidebar Yazi sync skipped \(status=($sidebar_sync_result.status)\)"
        }
    } else {
        # In no-sidebar mode, keep the originating Yazi instance aligned instead.
        sync_yazi_to_directory $file_path $yazi_id $log_file
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

    let sidebar_enabled = is_sidebar_enabled
    if $sidebar_enabled {
        let sidebar_sync_result = (sync_active_sidebar_yazi_to_directory $file_path "open_generic.log")
        if $sidebar_sync_result.status == "ok" {
            log_to_file "open_generic.log" $"Synced active sidebar Yazi to directory: ($sidebar_sync_result.target_dir)"
        } else {
            log_to_file "open_generic.log" $"WARNING: Active sidebar Yazi sync skipped \(status=($sidebar_sync_result.status)\)"
        }
    } else {
        sync_yazi_to_directory $file_path $yazi_id "open_generic.log"
    }

    log_to_file "open_generic.log" "open_with_generic_editor function completed"
}


# Main file opening function - dispatches to appropriate editor handler
export def open_file_with_editor [file_path: path] {
    log_to_file "open_editor.log" $"open_file_with_editor called with file_path: '($file_path)'"

    if not ($file_path | path exists) {
        let error_msg = $"File path ($file_path) does not exist"
        log_to_file "open_editor.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    let editor_context = try {
        resolve_editor_launch_context
    } catch {|err|
        let error_msg = $err.msg
        log_to_file "open_editor.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }
    let editor = $editor_context.editor

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
    let editor_kind = (get_managed_editor_kind)

    if $editor_kind == "helix" {
        log_to_file "open_editor.log" "Detected Helix editor, using Helix-specific logic"
        open_with_helix $file_path $yazi_id
    } else if $editor_kind == "neovim" {
        log_to_file "open_editor.log" "Detected Neovim editor, using Neovim-specific logic"
        open_with_neovim $file_path $yazi_id
    } else {
        log_to_file "open_editor.log" $"Using generic editor approach for: ($editor)"
        open_with_generic_editor $file_path $editor $yazi_id
    }

    log_to_file "open_editor.log" "open_file_with_editor function completed"
}
