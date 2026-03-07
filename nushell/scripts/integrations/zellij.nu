#!/usr/bin/env nu
# Zellij integration utilities for Yazelix

use ../utils/logging.nu *
use ../setup/zellij_plugin_paths.nu get_pane_orchestrator_wasm_path

def get_pane_orchestrator_plugin_url [] {
    let wasm_path = (get_pane_orchestrator_wasm_path)
    if not ($wasm_path | path exists) {
        error make {msg: $"Yazelix pane orchestrator plugin not found at: ($wasm_path)"}
    }

    $"file:($wasm_path)"
}

def run_pane_orchestrator_command [command_name: string, log_file: string, payload: string = ""] {
    let plugin_url = (get_pane_orchestrator_plugin_url)
    let pipe_result = (^zellij action pipe --plugin $plugin_url --name $command_name -- $payload | complete)

    if $pipe_result.exit_code != 0 {
        let stderr = ($pipe_result.stderr | str trim)
        log_to_file $log_file $"Pane orchestrator pipe failed for '($command_name)': ($stderr)"
        error make {msg: $"Pane orchestrator pipe failed for '($command_name)': ($stderr)"}
    }

    let response = ($pipe_result.stdout | str trim)
    log_to_file $log_file $"Pane orchestrator response for '($command_name)': ($response)"
    $response
}

export def focus_managed_pane [pane_name: string, log_file: string = "zellij_plugin.log"] {
    let command_name = match $pane_name {
        "editor" => "focus_editor"
        "sidebar" => "focus_sidebar"
        _ => {
            error make {msg: $"Unsupported managed pane name: ($pane_name)"}
        }
    }

    try {
        let response = (run_pane_orchestrator_command $command_name $log_file)
        parse_pane_orchestrator_response $response
    } catch {|err|
        {status: "error", reason: $err.msg}
    }
}

def parse_pane_orchestrator_response [response: string] {
    match $response {
        "ok" => {status: "ok"}
        "missing" => {status: "missing"}
        "not_ready" => {status: "not_ready"}
        "permissions_denied" => {status: "permissions_denied"}
        "invalid_payload" => {status: "invalid_payload"}
        "unsupported_editor" => {status: "unsupported_editor"}
        _ => {status: "error", reason: $response}
    }
}

# Get the tab name based on Git repo or working directory
export def get_tab_name [working_dir: path] {
    try {
        let git_root = (bash -c $"cd '($working_dir)' && git rev-parse --show-toplevel 2>/dev/null" | str trim)
        if ($git_root | is-not-empty) and (not ($git_root | str starts-with "fatal:")) {
            log_to_file "open_helix.log" $"Git root found: ($git_root)"
            $git_root | path basename
        } else {
            let basename = ($working_dir | str trim | path basename)
            log_to_file "open_helix.log" $"No valid Git repo, using basename of ($working_dir): ($basename)"
            if ($basename | is-empty) {
                "unnamed"
            } else {
                $basename
            }
        }
    } catch {
        $working_dir | path basename
    }
}

def open_file_in_managed_editor [editor_kind: string, file_path: path, log_file: string] {
    let expanded_file_path = ($file_path | path expand)
    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $expanded_file_path
    } else {
        $expanded_file_path | path dirname
    }

    let payload = {
        editor: $editor_kind
        file_path: $expanded_file_path
        working_dir: $working_dir
    } | to json -r

    try {
        let response = (run_pane_orchestrator_command "open_file" $log_file $payload)
        parse_pane_orchestrator_response $response
    } catch {|err|
        {status: "error", reason: $err.msg}
    }
}

# Open a file in an existing managed Helix pane through the pane orchestrator
export def open_in_existing_helix [file_path: path] {
    open_file_in_managed_editor "helix" $file_path "open_helix.log"
}

# Generic function to open a new editor pane with Yazi integration
def open_new_editor_pane [file_path: path, yazi_id: string, log_file: string] {
    let working_dir = if ($file_path | path exists) and ($file_path | path type) == "dir" {
        $file_path
    } else {
        $file_path | path dirname
    }

    log_to_file $log_file $"Attempting to open new pane with YAZI_ID=($yazi_id) for file=($file_path)"

    let tab_name = get_tab_name $working_dir
    log_to_file $log_file $"Calculated tab_name: ($tab_name)"

    # Use the configured editor from environment, preserving YAZI_ID
    let editor = $env.EDITOR
    let cmd = $"env YAZI_ID=($yazi_id) ($editor) '($file_path)'"

    log_to_file $log_file $"Full command to execute: ($cmd)"

    let pane_name = "editor"
    log_to_file $log_file $"Preparing command: nu -c \"($cmd)\" with pane name: ($pane_name)"
    zellij run --name $pane_name --cwd $working_dir -- nu -c $cmd
    log_to_file $log_file $"Command executed successfully: nu -c \"($cmd)\" with pane name: ($pane_name)"

    zellij action rename-tab $tab_name
    log_to_file $log_file $"Renamed tab to: ($tab_name)"
}

# Open a new pane and set up Helix with Yazi integration, renaming tab
export def open_new_helix_pane [file_path: path, yazi_id: string] {
    open_new_editor_pane $file_path $yazi_id "open_helix.log"
}

# Open a file in an existing managed Neovim pane through the pane orchestrator
export def open_in_existing_neovim [file_path: path] {
    open_file_in_managed_editor "neovim" $file_path "open_neovim.log"
}

# Open a new pane and set up Neovim with Yazi integration, renaming tab
export def open_new_neovim_pane [file_path: path, yazi_id: string] {
    open_new_editor_pane $file_path $yazi_id "open_neovim.log"
}
