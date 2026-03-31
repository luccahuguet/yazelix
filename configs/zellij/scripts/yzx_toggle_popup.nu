#!/usr/bin/env nu

def get_runtime_script_path [relative_path: string] {
    let runtime_dir = ($env.YAZELIX_RUNTIME_DIR? | default "" | str trim)
    if ($runtime_dir | is-empty) {
        error make {msg: "Missing YAZELIX_RUNTIME_DIR for Yazelix Zellij helper script."}
    }

    let expanded_runtime_dir = ($runtime_dir | path expand)
    if not ($expanded_runtime_dir | path exists) {
        error make {msg: $"Configured YAZELIX_RUNTIME_DIR does not exist: ($expanded_runtime_dir)"}
    }

    ($expanded_runtime_dir | path join $relative_path)
}

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
        "ok" => { action: "handled" }
        "permissions_denied" => { action: "error", message: "Popup runner permissions were denied. Restart Yazelix and accept the popup-runner plugin permissions." }
        "not_ready" => { action: "error", message: "Popup runner is not ready yet. Restart Yazelix and try again." }
        _ => { action: "error", message: $"Unexpected popup toggle result: ($normalized_result)" }
    }
}

def main [] {
    let action = (resolve_popup_toggle_action (popup_plugin_toggle_result))

    if $action.action == "open" {
        let core_script = (get_runtime_script_path "nushell/scripts/core/yazelix.nu")
        let command = ([
            $"use '($core_script)' *"
            "yzx popup"
        ] | str join "\n")
        run-external nu "-c" $command
        return
    }

    if $action.action == "handled" {
        return
    }

    error make {msg: $action.message}
}
