#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

use runtime_helper.nu [run_runtime_nu_script]

def main [] {
    run_runtime_nu_script "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu"
}
