#!/usr/bin/env nu

use ../yzx/popup.nu *

def main [...popup_args: string] {
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action rename-pane "yzx_popup" | complete | ignore
    }

    with-env {
        YAZELIX_POPUP_PANE: "true"
    } {
        yzx popup ...$popup_args
    }
}
