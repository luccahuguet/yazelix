#!/usr/bin/env nu

use runtime_helper.nu [get_runtime_script_path run_runtime_nu_command]

def popup_plugin_toggle_result [] {
    let pipe_result = (^zellij pipe --name toggle_popup -- "" | complete)
    if $pipe_result.exit_code != 0 {
        error make {msg: $"Popup runner pipe failed: ($pipe_result.stderr | str trim)"}
    }

    $pipe_result.stdout | str trim
}

export def resolve_popup_toggle_action [toggle_result?: string] {
    let normalized_result = ($toggle_result | default "")

    match $normalized_result {
        "missing" => { action: "open" }
        "ok" => { action: "handled", refresh_sidebar: false }
        "focused" => { action: "handled", refresh_sidebar: false }
        "closed" => { action: "handled", refresh_sidebar: true }
        "permissions_denied" => { action: "error", message: "Popup runner permissions were denied. Restart Yazelix and accept the popup-runner plugin permissions." }
        "not_ready" => { action: "error", message: "Popup runner is not ready yet. Restart Yazelix and try again." }
        _ => { action: "error", message: $"Unexpected popup toggle result: ($normalized_result)" }
    }
}

def maybe_refresh_active_sidebar_yazi [action: record] {
    if not ($action.refresh_sidebar? | default false) {
        return
    }

    # Zellij reports popup closure before focus restoration fully settles.
    # Give the sidebar instance a moment to become active again before emitting into Yazi.
    sleep 150ms

    let yazi_integration = (get_runtime_script_path "nushell/scripts/integrations/yazi.nu")
    let command = ([
        $"use '($yazi_integration)' [refresh_active_sidebar_yazi]"
        "refresh_active_sidebar_yazi | ignore"
    ] | str join "\n")
    run_runtime_nu_command $command
}

def main [] {
    let action = (resolve_popup_toggle_action (popup_plugin_toggle_result))

    if $action.action == "open" {
        let popup_script = (get_runtime_script_path "nushell/scripts/yzx/popup.nu")
        let command = ([
            $"use '($popup_script)' *"
            "yzx popup"
        ] | str join "\n")
        run_runtime_nu_command $command
        return
    }

    if $action.action == "handled" {
        maybe_refresh_active_sidebar_yazi $action
        return
    }

    error make {msg: $action.message}
}
