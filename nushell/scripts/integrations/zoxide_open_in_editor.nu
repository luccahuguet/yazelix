#!/usr/bin/env nu
# Open a directory in the managed editor pane
# Called from the zoxide-editor Yazi plugin with the selected path

use ../utils/logging.nu log_to_file
use ./zellij.nu [retarget_workspace_for_path, open_new_managed_editor_in_cwd]
use ./managed_editor.nu [get_managed_editor_kind, sync_post_retarget_workspace_state]

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

    let retarget_result = (retarget_workspace_for_path $target_dir $editor_kind $LOG)
    match ($retarget_result.status? | default "error") {
        "ok" => {
            let yazi_id = ($env.YAZI_ID? | default "" | into string | str trim)
            sync_post_retarget_workspace_state $retarget_result $target_dir $LOG $editor_kind "" {
                log_to_file $LOG "Managed editor pane missing, opening a new managed editor pane"
                open_new_managed_editor_in_cwd $editor_kind $target_dir $yazi_id $LOG
            } | ignore
        }
        _ => {
            log_to_file $LOG $"ERROR: Failed to retarget workspace/editor state: ($retarget_result)"
            error make {msg: $"Failed to retarget workspace/editor state: ($retarget_result)"}
        }
    }
}
