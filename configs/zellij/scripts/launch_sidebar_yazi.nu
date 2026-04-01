#!/usr/bin/env nu

use runtime_helper.nu [get_runtime_script_path run_runtime_nu_command]

let yazi_integration = (get_runtime_script_path "nushell/scripts/integrations/yazi.nu")
let command = ([
    $"use '($yazi_integration)' [consume_bootstrap_sidebar_cwd get_yazi_command]"
    "let bootstrap_dir = (consume_bootstrap_sidebar_cwd)"
    "let target_dir = if ($bootstrap_dir | is-not-empty) { $bootstrap_dir } else { pwd | path expand }"
    "let yazi_command = (get_yazi_command)"
    "run-external $yazi_command $target_dir"
] | str join "\n")

run_runtime_nu_command $command
