#!/usr/bin/env nu

use runtime_helper.nu [run_runtime_nu_script]

def main [] {
    run_runtime_nu_script "nushell/scripts/zellij_wrappers/open_workspace_terminal.nu"
}
