#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

use ../yzx/menu.nu *

def rename_transient_pane [name: string] {
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action rename-pane $name | complete | ignore
    }
}

def close_transient_pane [] {
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action close-pane | complete | ignore
    }
}

def main [] {
    rename_transient_pane "yzx_menu"

    let result = (try {
        with-env {YAZELIX_MENU_POPUP: "true"} {
            yzx menu
        }
        {ok: true}
    } catch {|err|
        {ok: false, msg: $err.msg}
    })

    if $result.ok {
        close_transient_pane
        return
    }

    error make {msg: $result.msg}
}
