#!/usr/bin/env nu

use ../../../nushell/scripts/core/yazelix.nu *

def main [...popup_args: string] {
    $env.YAZELIX_POPUP_PANE = "true"
    if ($env.ZELLIJ? | is-not-empty) {
        ^zellij action rename-pane "yzx_popup" | complete | ignore
    }
    yzx popup ...$popup_args
}
