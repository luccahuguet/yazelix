#!/usr/bin/env nu
# Zellij integration utilities for Yazelix

use ../utils/logging.nu *
use ../setup/zellij_plugin_paths.nu PANE_ORCHESTRATOR_PLUGIN_ALIAS
use ../utils/common.nu [get_yazelix_runtime_dir resolve_yazelix_nu_bin]
use ../utils/config_parser.nu [parse_yazelix_config]
use ../utils/launch_state.nu [get_launch_env]

const FLOATING_WRAPPER_ENV_KEYS = [
    "DEVENV_PROFILE"
    "PATH"
    "YAZELIX_RUNTIME_DIR"
    "IN_YAZELIX_SHELL"
    "IN_NIX_SHELL"
    "NIX_CONFIG"
    "ZELLIJ_DEFAULT_LAYOUT"
    "YAZI_CONFIG_HOME"
    "YAZELIX_MANAGED_HELIX_BINARY"
    "EDITOR"
    "HELIX_RUNTIME"
]

def get_pane_orchestrator_plugin_target [] {
    $PANE_ORCHESTRATOR_PLUGIN_ALIAS
}

def get_current_shell_wrapper_env [] {
    mut wrapper_env = {}

    for key in $FLOATING_WRAPPER_ENV_KEYS {
        let value = ($env | get -o $key | default null)
        if $value != null {
            let text = ($value | into string)
            if ($text | is-not-empty) {
                $wrapper_env = ($wrapper_env | upsert $key $text)
            }
        }
    }

    $wrapper_env
}

def get_current_shell_launch_profile [] {
    let profile_path = ($env.DEVENV_PROFILE? | default "" | into string | str trim)
    if ($profile_path | is-not-empty) and ($profile_path | path exists) {
        $profile_path | path expand
    } else {
        ""
    }
}

def serialize_wrapper_env_value [value: any] {
    let described = ($value | describe)

    if ($described | str starts-with "list") {
        $value | each {|entry| $entry | into string } | str join (char esep)
    } else {
        $value | into string
    }
}

export def build_floating_wrapper_env_args [wrapper_env: record] {
    $wrapper_env
    | transpose key value
    | each {|row| $"($row.key)=(serialize_wrapper_env_value $row.value)" }
}

export def get_floating_wrapper_env [] {
    let current_shell_env = (get_current_shell_wrapper_env)
    let profile_path = (get_current_shell_launch_profile)

    if ($profile_path | is-empty) {
        return $current_shell_env
    }

    let config = (parse_yazelix_config)
    get_launch_env $config $profile_path
}

export def get_new_editor_pane_launch_env [yazi_id: string = ""] {
    mut pane_env = (get_floating_wrapper_env)

    if ($yazi_id | str trim | is-not-empty) {
        $pane_env = ($pane_env | upsert YAZI_ID $yazi_id)
    }

    $pane_env
}

def run_pane_orchestrator_command [command_name: string, log_file: string, payload: string = ""] {
    let plugin_target = (get_pane_orchestrator_plugin_target)
    let pipe_result = (^zellij action pipe --plugin $plugin_target --name $command_name -- $payload | complete)

    if $pipe_result.exit_code != 0 {
        let stderr = ($pipe_result.stderr | str trim)
        log_to_file $log_file $"Pane orchestrator pipe failed for '($command_name)': ($stderr)"
        error make {msg: $"Pane orchestrator pipe failed for '($command_name)': ($stderr)"}
    }

    let response = ($pipe_result.stdout | str trim)
    log_to_file $log_file $"Pane orchestrator response for '($command_name)': ($response)"
    $response
}

export def run_pane_orchestrator_command_raw [command_name: string, payload: string = "", log_file: string = "zellij_plugin_debug.log"] {
    run_pane_orchestrator_command $command_name $log_file $payload
}

export def open_floating_runtime_wrapper [
    pane_name: string
    wrapper_name: string
    cwd: string
    extra_env: record = {}
    command_args: list<string> = []
    width_percent: int = 90
    height_percent: int = 90
] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let wrapper = ($runtime_dir | path join "configs" "zellij" "scripts" $wrapper_name)
    let runtime_nu = (resolve_yazelix_nu_bin)
    if not ($wrapper | path exists) {
        error make {msg: $"Floating wrapper script not found at: ($wrapper)"}
    }
    if not ($runtime_nu | path exists) {
        error make {msg: $"Resolved Yazelix Nushell binary not found at: ($runtime_nu)"}
    }

    let wrapper_env = ((get_floating_wrapper_env) | merge $extra_env)
    let env_args = (build_floating_wrapper_env_args $wrapper_env)
    let width_arg = $"($width_percent)%"
    let height_arg = $"($height_percent)%"
    let x_offset = (((100 - $width_percent) / 2) | math floor | into int)
    let y_offset = (((100 - $height_percent) / 2) | math floor | into int)
    let x_arg = $"($x_offset)%"
    let y_arg = $"($y_offset)%"

    ^zellij run --name $pane_name --floating --close-on-exit --width $width_arg --height $height_arg --x $x_arg --y $y_arg --cwd $cwd -- env ...$env_args $runtime_nu $wrapper ...$command_args
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

export def toggle_editor_sidebar_focus [log_file: string = "zellij_plugin.log"] {
    try {
        let response = (run_pane_orchestrator_command "toggle_editor_sidebar_focus" $log_file)
        parse_pane_orchestrator_response $response
    } catch {|err|
        {status: "error", reason: $err.msg}
    }
}

def parse_pane_orchestrator_response [response: string] {
    match $response {
        "ok" => {status: "ok"}
        "focused_editor" => {status: "ok", target: "editor"}
        "focused_sidebar" => {status: "ok", target: "sidebar"}
        "opened_sidebar" => {status: "ok", target: "sidebar", opened: true}
        "missing" => {status: "missing"}
        "missing_workspace" => {status: "missing_workspace"}
        "not_ready" => {status: "not_ready"}
        "permissions_denied" => {status: "permissions_denied"}
        "invalid_payload" => {status: "invalid_payload"}
        "unknown_layout" => {status: "unknown_layout"}
        "unsupported_editor" => {status: "unsupported_editor"}
        _ => {status: "error", reason: $response}
    }
}

# Get the stable workspace root for a target path.
# Inside a Git repo, this is the repo root. Otherwise, it is the target directory itself.
def resolve_target_dir [target_path: path] {
    let expanded_target_path = ($target_path | path expand)
    if ($expanded_target_path | path exists) and (($expanded_target_path | path type) == "dir") {
        $expanded_target_path
    } else {
        $expanded_target_path | path dirname
    }
}

export def get_workspace_root [target_path: path] {
    let target_dir = (resolve_target_dir $target_path)
    try {
        let git_result = (^git -C $target_dir rev-parse --show-toplevel | complete)
        let git_root = ($git_result.stdout | str trim)
        if ($git_result.exit_code == 0) and ($git_root | is-not-empty) {
            $git_root | path expand
        } else {
            $target_dir
        }
    } catch {
        $target_dir
    }
}

# Get the tab name based on an already-resolved workspace root.
export def get_tab_name [target_path: path] {
    let basename = ($target_path | path expand | str trim | path basename)
    if ($basename | is-empty) {
        "unnamed"
    } else {
        $basename
    }
}

def get_workspace_context [target_path: path, log_file: string] {
    let workspace_root = (get_workspace_root $target_path)
    let tab_name = (get_tab_name $workspace_root)
    log_to_file $log_file $"Resolved workspace_root: ($workspace_root)"
    log_to_file $log_file $"Calculated tab_name: ($tab_name)"
    {
        workspace_root: $workspace_root
        tab_name: $tab_name
    }
}

export def resolve_tab_cwd_target [
    target?: string  # Directory path or zoxide query for the current tab (defaults to the current directory)
] {
    let requested_target = if ($target | is-not-empty) {
        $target
    } else {
        pwd
    }

    if ($requested_target == (pwd)) {
        return $requested_target
    }

    if (which zoxide | is-not-empty) {
        let zoxide_result = (^zoxide query -- $requested_target | complete)
        if $zoxide_result.exit_code == 0 {
            return ($zoxide_result.stdout | str trim)
        }
    }

    if ($requested_target | path exists) {
        return $requested_target
    }

    if (which zoxide | is-not-empty) {
        error make {msg: $"Could not resolve '($requested_target)' with zoxide or as an existing path."}
    } else {
        error make {msg: $"zoxide is not available and '($requested_target)' is not an existing path."}
    }
}

def update_tab_workspace [command_name: string, target_path: path, log_file: string] {
    let expanded_target_path = ($target_path | path expand)
    if not ($expanded_target_path | path exists) {
        error make {msg: $"Path does not exist: ($expanded_target_path)"}
    }

    let target_dir = if (($expanded_target_path | path type) == "dir") {
        $expanded_target_path
    } else {
        $expanded_target_path | path dirname
    }
    let payload = {
        workspace_root: $target_dir
    } | to json -r

    log_to_file $log_file $"Setting tab cwd to: ($target_dir)"

    try {
        let response = (run_pane_orchestrator_command $command_name $log_file $payload)
        {
            workspace_root: $target_dir
            tab_name: (get_tab_name $target_dir)
        } | merge (parse_pane_orchestrator_response $response)
    } catch {|err|
        {
            workspace_root: $target_dir
            tab_name: (get_tab_name $target_dir)
            status: "error"
            reason: $err.msg
        }
    }
}

export def set_tab_cwd [target_path: path, log_file: string = "zellij_plugin.log"] {
    update_tab_workspace "set_workspace_root_and_cd_focused_pane" $target_path $log_file
}

export def set_tab_workspace_root [target_path: path, log_file: string = "zellij_plugin.log"] {
    update_tab_workspace "set_workspace_root" $target_path $log_file
}

export def set_workspace_for_path [target_path: path, log_file: string = "zellij_plugin.log"] {
    let workspace = (get_workspace_context $target_path $log_file)
    let payload = {
        workspace_root: $workspace.workspace_root
    } | to json -r

    try {
        let response = (run_pane_orchestrator_command "set_workspace_root" $log_file $payload)
        $workspace | merge (parse_pane_orchestrator_response $response)
    } catch {|err|
        $workspace | merge {status: "error", reason: $err.msg}
    }
}

def open_file_in_managed_editor [editor_kind: string, file_path: path, log_file: string] {
    let expanded_file_path = ($file_path | path expand)
    let workspace = (get_workspace_context $expanded_file_path $log_file)
    let payload = {
        editor: $editor_kind
        file_path: $expanded_file_path
        working_dir: $workspace.workspace_root
    } | to json -r

    try {
        let response = (run_pane_orchestrator_command "open_file" $log_file $payload)
        parse_pane_orchestrator_response $response
    } catch {|err|
        {status: "error", reason: $err.msg}
    }
}

export def debug_editor_state [] {
    let response = (run_pane_orchestrator_command_raw "debug_editor_state")
    try {
        $response | from json
    } catch {
        {raw: $response}
    }
}

def read_current_tab_workspace_root [--include-bootstrap] {
    let state = try {
        debug_editor_state
    } catch {
        null
    }

    if ($state | is-empty) {
        null
    } else {
        let workspace_root_source = ($state.workspace_root_source? | default "" | into string | str trim)
        if ((not $include_bootstrap) and ($workspace_root_source == "bootstrap")) {
            return null
        }

        let workspace_root = ($state.workspace_root? | default "" | into string | str trim)
        if ($workspace_root | is-empty) {
            null
        } else {
            $workspace_root
        }
    }
}

export def get_current_tab_workspace_root [] {
    read_current_tab_workspace_root
}

export def get_current_tab_workspace_root_including_bootstrap [] {
    read_current_tab_workspace_root --include-bootstrap
}

export def set_managed_editor_cwd [editor_kind: string, target_path: path, log_file: string = "zellij_plugin.log"] {
    let expanded_target_path = ($target_path | path expand)
    if not ($expanded_target_path | path exists) {
        error make {msg: $"Path does not exist: ($expanded_target_path)"}
    }

    let target_dir = if (($expanded_target_path | path type) == "dir") {
        $expanded_target_path
    } else {
        $expanded_target_path | path dirname
    }
    let payload = {
        editor: $editor_kind
        working_dir: $target_dir
    } | to json -r

    try {
        let response = (run_pane_orchestrator_command "set_managed_editor_cwd" $log_file $payload)
        {
            working_dir: $target_dir
            editor: $editor_kind
        } | merge (parse_pane_orchestrator_response $response)
    } catch {|err|
        {
            working_dir: $target_dir
            editor: $editor_kind
            status: "error"
            reason: $err.msg
        }
    }
}

export def next_layout_family [] {
    let response = (run_pane_orchestrator_command_raw "next_family")
    parse_pane_orchestrator_response $response
}

export def previous_layout_family [] {
    let response = (run_pane_orchestrator_command_raw "previous_family")
    parse_pane_orchestrator_response $response
}

export def toggle_sidebar_layout [] {
    let response = (run_pane_orchestrator_command_raw "toggle_sidebar")
    parse_pane_orchestrator_response $response
}

# Open a file in an existing managed Helix pane through the pane orchestrator
export def open_in_existing_helix [file_path: path] {
    open_file_in_managed_editor "helix" $file_path "open_helix.log"
}

# Generic function to open a new editor pane with Yazi integration
def open_new_editor_pane [file_path: path, yazi_id: string, log_file: string] {
    let expanded_file_path = ($file_path | path expand)
    let workspace = (get_workspace_context $expanded_file_path $log_file)
    let pane_env = (get_new_editor_pane_launch_env $yazi_id)
    let env_args = (build_floating_wrapper_env_args $pane_env)
    let editor = ($pane_env.EDITOR? | default "" | str trim)
    if ($editor | is-empty) {
        error make {msg: "EDITOR environment variable is not set in the canonical launch env"}
    }

    log_to_file $log_file $"Attempting to open new pane with YAZI_ID=($yazi_id) for file=($expanded_file_path)"
    log_to_file $log_file $"Launching editor pane with editor=($editor), workspace_root=($workspace.workspace_root), file=($expanded_file_path)"

    let pane_name = "editor"
    zellij run --name $pane_name --cwd $workspace.workspace_root -- env ...$env_args $editor $expanded_file_path
    log_to_file $log_file $"Command executed successfully with pane name: ($pane_name)"
}

export def open_new_managed_editor_in_cwd [
    editor_kind: string
    target_dir: path
    yazi_id: string = ""
    log_file: string = "open_editor.log"
] {
    let expanded_target_dir = ($target_dir | path expand)
    if not ($expanded_target_dir | path exists) {
        error make {msg: $"Target directory does not exist: ($expanded_target_dir)"}
    }

    let working_dir = if (($expanded_target_dir | path type) == "dir") {
        $expanded_target_dir
    } else {
        $expanded_target_dir | path dirname
    }

    let pane_env = (get_new_editor_pane_launch_env $yazi_id)
    let env_args = (build_floating_wrapper_env_args $pane_env)
    let editor = ($pane_env.EDITOR? | default "" | str trim)
    if ($editor | is-empty) {
        error make {msg: "EDITOR environment variable is not set in the canonical launch env"}
    }

    let editor_args = if $editor_kind == "helix" {
        [$working_dir]
    } else {
        []
    }

    log_to_file $log_file $"Launching new managed editor pane in cwd=($working_dir) with editor=($editor), editor_kind=($editor_kind)"
    zellij run --name "editor" --cwd $working_dir -- env ...$env_args $editor ...$editor_args
    log_to_file $log_file "Command executed successfully with pane name: editor"
}

# Open a new pane and set up Helix with Yazi integration
export def open_new_helix_pane [file_path: path, yazi_id: string] {
    open_new_editor_pane $file_path $yazi_id "open_helix.log"
}

# Open a file in an existing managed Neovim pane through the pane orchestrator
export def open_in_existing_neovim [file_path: path] {
    open_file_in_managed_editor "neovim" $file_path "open_neovim.log"
}

# Open a new pane and set up Neovim with Yazi integration
export def open_new_neovim_pane [file_path: path, yazi_id: string] {
    open_new_editor_pane $file_path $yazi_id "open_neovim.log"
}
