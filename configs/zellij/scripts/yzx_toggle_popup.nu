#!/usr/bin/env nu

use ../../../nushell/scripts/integrations/yazi.nu [refresh_active_sidebar_yazi]
use ../../../nushell/scripts/yzx/popup.nu *

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

def refresh_sidebar_after_popup_close [] {
    refresh_active_sidebar_yazi | ignore
}

def main [] {
    let action = (resolve_popup_toggle_action (popup_plugin_toggle_result))

    if $action.action == "open" {
        yzx popup
        return
    }

    if $action.action == "handled" {
        if ($action.refresh_sidebar? | default false) {
            refresh_sidebar_after_popup_close
        }
        return
    }

    error make {msg: $action.message}
}
