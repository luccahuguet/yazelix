#!/usr/bin/env nu
# Open a directory in the managed editor pane
# Called from the zoxide-editor Yazi plugin with the selected path

use ../utils/logging.nu log_to_file
use ./zellij.nu [set_managed_editor_cwd, set_workspace_for_path, open_new_managed_editor_in_cwd]
use ./managed_editor.nu get_managed_editor_kind
use ./yazi.nu [sync_active_sidebar_yazi_to_directory, is_sidebar_enabled]

const LOG = "zoxide_open_in_editor.log"

export def main [target_dir: string] {
    log_to_file $LOG $"Called with target_dir: ($target_dir)"

    if not ($target_dir | path exists) {
        log_to_file $LOG $"ERROR: Path does not exist: ($target_dir)"
        error make {msg: $"Path does not exist: ($target_dir)"}
    }

    let editor_kind = (get_managed_editor_kind)
    if $editor_kind == null {
        log_to_file $LOG "ERROR: No managed editor detected"
        error make {msg: "No managed editor detected"}
    }

    let cwd_result = (set_managed_editor_cwd $editor_kind $target_dir $LOG)
    match ($cwd_result.status? | default "error") {
        "ok" => {
            log_to_file $LOG $"Set ($editor_kind) cwd to ($target_dir)"
        }
        "missing" => {
            let yazi_id = ($env.YAZI_ID? | default "" | into string | str trim)
            log_to_file $LOG "Managed editor pane missing, opening a new managed editor pane"
            open_new_managed_editor_in_cwd $editor_kind $target_dir $yazi_id $LOG
        }
        _ => {
            log_to_file $LOG $"ERROR: Failed to set editor cwd: ($cwd_result)"
            error make {msg: $"Failed to set editor cwd: ($cwd_result)"}
        }
    }

    let workspace_result = (set_workspace_for_path $target_dir $LOG)
    if $workspace_result.status == "ok" {
        log_to_file $LOG $"Updated workspace root to: ($workspace_result.workspace_root)"
    } else {
        log_to_file $LOG $"WARNING: Failed to update workspace root \(status=($workspace_result.status)\)"
    }

    if (is_sidebar_enabled) {
        let sync_result = (sync_active_sidebar_yazi_to_directory $target_dir $LOG)
        if $sync_result.status == "ok" {
            log_to_file $LOG $"Synced sidebar Yazi to: ($sync_result.target_dir)"
        } else {
            log_to_file $LOG $"WARNING: Sidebar sync skipped \(status=($sync_result.status)\)"
        }
    }
}
