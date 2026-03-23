#!/usr/bin/env nu

use ../../../nushell/scripts/core/yazelix.nu *

def popup_plugin_toggle_result [] {
    let pipe_result = (^zellij pipe --name toggle_popup -- "" | complete)
    if $pipe_result.exit_code != 0 {
        error make {msg: $"Popup runner pipe failed: ($pipe_result.stderr | str trim)"}
    }

    $pipe_result.stdout | str trim
}

def popup_exists [] {
    let pipe_result = (^zellij pipe --name has_popup -- "" | complete)
    if $pipe_result.exit_code != 0 {
        error make {msg: $"Popup runner existence check failed: ($pipe_result.stderr | str trim)"}
    }

    match ($pipe_result.stdout | str trim) {
        "true" => true
        "false" => false
        "" => false
        "not_ready" => false
        _ => false
    }
}

export def resolve_popup_toggle_action [has_popup: bool, toggle_result?: string] {
    if not $has_popup {
        return { action: "open" }
    }

    let normalized_result = ($toggle_result | default "")

    match $normalized_result {
        "ok" => { action: "handled" }
        "permissions_denied" => { action: "error", message: "Popup runner permissions were denied. Restart Yazelix and accept the popup-runner plugin permissions." }
        "not_ready" => { action: "error", message: "Popup runner is not ready yet. Restart Yazelix and try again." }
        _ => { action: "error", message: $"Unexpected popup toggle result: ($normalized_result)" }
    }
}

def main [] {
    let has_popup = (popup_exists)
    let toggle_result = if $has_popup { popup_plugin_toggle_result } else { null }
    let action = (resolve_popup_toggle_action $has_popup $toggle_result)

    if $action.action == "open" {
        yzx popup
        return
    }

    if $action.action == "handled" {
        return
    }

    error make {msg: $action.message}
}
