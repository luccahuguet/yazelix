#!/usr/bin/env nu

use runtime_helper.nu [run_runtime_nu_script]

def main [...popup_args: string] {
    run_runtime_nu_script "nushell/scripts/zellij_wrappers/yzx_popup_program.nu" ...$popup_args
}
