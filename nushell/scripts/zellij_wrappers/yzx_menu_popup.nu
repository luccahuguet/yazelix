#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

use ../yzx/menu.nu *

def main [] {
    with-env {YAZELIX_MENU_POPUP: "true"} {
        yzx menu
    }
}
