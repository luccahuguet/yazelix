#!/usr/bin/env nu

use runtime_helper.nu [get_runtime_script_path run_runtime_nu_command]

let zellij_integration = (get_runtime_script_path "nushell/scripts/integrations/zellij.nu")
let command = ([
    $"use '($zellij_integration)' *"
    "let result = (toggle_editor_sidebar_focus)"
    "if $result.status in ['missing' 'not_ready'] {"
    "    exit 0"
    "}"
    "if $result.status != 'ok' {"
    "    print $'Error: toggle editor/sidebar focus failed \(status=($result.status)\)'"
    "    exit 1"
    "}"
] | str join "\n")

run_runtime_nu_command $command
