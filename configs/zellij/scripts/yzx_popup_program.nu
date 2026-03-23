#!/usr/bin/env nu

use ../../../nushell/scripts/core/yazelix.nu *

def main [...popup_args: string] {
    $env.YAZELIX_POPUP_PANE = "true"
    yzx popup ...$popup_args
}
