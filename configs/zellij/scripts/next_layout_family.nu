#!/usr/bin/env nu

use runtime_helper.nu [run_runtime_nu_script]

def main [] {
    run_runtime_nu_script "nushell/scripts/zellij_wrappers/next_layout_family.nu"
}
