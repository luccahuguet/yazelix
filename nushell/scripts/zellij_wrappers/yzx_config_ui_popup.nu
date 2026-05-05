#!/usr/bin/env nu

use ../utils/runtime_paths.nu [require_yazelix_runtime_dir]
use ../utils/transient_pane_contract.nu [
    close_current_transient_pane
    get_transient_pane_mode_env
    rename_current_transient_pane
]

def main [] {
    rename_current_transient_pane "config"

    let runtime_dir = (require_yazelix_runtime_dir)
    let yzx_cli = ($runtime_dir | path join "shells" "posix" "yzx_cli.sh")

    with-env (get_transient_pane_mode_env "config") {
        ^$yzx_cli config ui
    }

    try { close_current_transient_pane }
}
