#!/usr/bin/env nu

use ../../../nushell/scripts/integrations/yazi.nu consume_bootstrap_sidebar_cwd
use ../../../nushell/scripts/integrations/zellij.nu run_pane_orchestrator_command_raw

def bootstrap_workspace_root [target_dir: string] {
    let payload = ({workspace_root: $target_dir} | to json -r)

    mut attempts = 0
    loop {
        let response = (run_pane_orchestrator_command_raw "set_workspace_root" $payload "sidebar_bootstrap.log" | str trim)
        if $response == "ok" {
            return
        }

        if ($response not-in ["not_ready" "permissions_denied"]) or ($attempts >= 9) {
            return
        }

        $attempts = ($attempts + 1)
        sleep 100ms
    }
}

let bootstrap_dir = (consume_bootstrap_sidebar_cwd)
let target_dir = if ($bootstrap_dir | is-not-empty) {
    $bootstrap_dir
} else {
    pwd | path expand
}

bootstrap_workspace_root $target_dir
^yazi $target_dir
