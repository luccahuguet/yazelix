#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

use runtime_helper.nu [get_runtime_script_path run_runtime_nu_command]

let core_script = (get_runtime_script_path "nushell/scripts/core/yazelix.nu")
let command = $"use '($core_script)' *; yzx menu"

with-env {YAZELIX_MENU_POPUP: "true"} {
    run_runtime_nu_command $command
}
