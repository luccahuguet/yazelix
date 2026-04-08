#!/usr/bin/env nu

use ../utils/logging.nu log_to_file
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/editor_launch_context.nu [resolve_editor_launch_context]
use ./zellij.nu [open_in_existing_helix, open_in_existing_neovim, open_new_helix_pane, open_new_neovim_pane, get_workspace_root, set_workspace_for_path, set_managed_editor_cwd]
use ./yazi.nu [get_ya_command, is_sidebar_enabled, sync_active_sidebar_yazi_to_directory]

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
        let ya_command = (get_ya_command)
        ^$ya_command "emit-to" $yazi_id "cd" $target_dir
        log_to_file $log_file $"Successfully navigated yazi to directory: ($target_dir)"
    } catch {|err|
        log_to_file $log_file $"Failed to navigate yazi: ($err.msg)"
    }
}

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
        sync_yazi_to_directory $file_path $yazi_id $log_file
        log_to_file $log_file $"No-sidebar mode: leaving Yazi pane open, no close operation needed"
    }

    log_to_file $log_file $"open_with_($editor_name) function completed"
}

def open_with_helix [file_path: path, yazi_id: string] {
    open_with_editor_integration $file_path $yazi_id "Helix" "open_helix.log" {|path| open_in_existing_helix $path} {|path, id| open_new_helix_pane $path $id}
}

def open_with_neovim [file_path: path, yazi_id: string] {
    open_with_editor_integration $file_path $yazi_id "Neovim" "open_neovim.log" {|path| open_in_existing_neovim $path} {|path, id| open_new_neovim_pane $path $id}
}

def open_with_generic_editor [file_path: path, editor: string, yazi_id: string] {
    log_to_file "open_generic.log" $"open_with_generic_editor called with file_path: '($file_path)', editor: '($editor)'"

    let workspace_root = (get_workspace_root $file_path)
    log_to_file "open_generic.log" $"Using workspace root: ($workspace_root)"

    try {
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

    let sidebar_enabled = is_sidebar_enabled
    log_to_file "open_editor.log" $"Sidebar enabled: ($sidebar_enabled)"

    let yazi_id = ($env.YAZI_ID? | default "")
    if ($yazi_id | is-empty) {
        let warning_msg = "YAZI_ID not set in this environment. Yazi navigation may fail."
        log_to_file "open_editor.log" $"WARNING: ($warning_msg)"
        print $"Warning: ($warning_msg)"
    } else {
        log_to_file "open_editor.log" $"YAZI_ID found: '($yazi_id)'"
    }

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
