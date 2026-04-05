#!/usr/bin/env nu

use runtime_helper.nu [get_runtime_script_path run_runtime_nu_command]

export def build_toggle_editor_sidebar_focus_command [] {
    let zellij_integration = (get_runtime_script_path "nushell/scripts/integrations/zellij.nu")
    let yazi_integration = (get_runtime_script_path "nushell/scripts/integrations/yazi.nu")

    [
        $"use '($zellij_integration)' [toggle_editor_sidebar_focus]"
        $"use '($yazi_integration)' [refresh_active_sidebar_yazi]"
        "let result = (toggle_editor_sidebar_focus)"
        "if $result.status in ['missing' 'not_ready'] {"
        "    exit 0"
        "}"
        "if $result.status != 'ok' {"
        "    print $\"Error: toggle editor/sidebar focus failed \\(status=($result.status)\\)\""
        "    exit 1"
        "}"
        "if (($result.target? | default '') == 'sidebar') {"
        "    refresh_active_sidebar_yazi | ignore"
        "}"
    ] | str join "\n"
}

def main [] {
    run_runtime_nu_command (build_toggle_editor_sidebar_focus_command)
}
