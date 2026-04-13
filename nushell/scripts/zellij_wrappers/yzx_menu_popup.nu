#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

use ../yzx/menu.nu *
use ../utils/transient_pane_contract.nu [
    close_current_transient_pane
    get_transient_pane_mode_env
    rename_current_transient_pane
]

def main [] {
    rename_current_transient_pane "menu"

    let result = (try {
        with-env (get_transient_pane_mode_env "menu") {
            yzx menu
        }
        {ok: true}
    } catch {|err|
        {ok: false, msg: $err.msg}
    })

    if $result.ok {
        close_current_transient_pane
        return
    }

    error make {msg: $result.msg}
}
