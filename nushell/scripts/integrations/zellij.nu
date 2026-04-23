#!/usr/bin/env nu
# Zellij integration utilities for Yazelix

use ../utils/logging.nu *
use ../utils/runtime_paths.nu [get_yazelix_runtime_dir]
use ../utils/yzx_core_bridge.nu [compute_runtime_env_via_yzx_core]

const FLOATING_WRAPPER_ENV_KEYS = [
    "PATH"
    "YAZELIX_RUNTIME_DIR"
    "IN_YAZELIX_SHELL"
    "NIX_CONFIG"
    "ZELLIJ_DEFAULT_LAYOUT"
    "YAZI_CONFIG_HOME"
    "YAZELIX_MANAGED_HELIX_BINARY"
    "EDITOR"
    "VISUAL"
    "HELIX_RUNTIME"
]

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

def serialize_wrapper_env_value [value: any] {
    let described = ($value | describe)

    if ($described | str starts-with "list") {
        $value | each {|entry| $entry | into string } | str join (char esep)
    } else {
        $value | into string
    }
}

def build_floating_wrapper_env_args [wrapper_env: record] {
    $wrapper_env
    | transpose key value
    | each {|row| $"($row.key)=(serialize_wrapper_env_value $row.value)" }
}

def get_floating_wrapper_env [] {
    let current_shell_env = (get_current_shell_wrapper_env)
    (compute_runtime_env_via_yzx_core) | merge $current_shell_env
}

export def get_new_editor_pane_launch_env [yazi_id: string = ""] {
    mut pane_env = (get_floating_wrapper_env)

    if ($yazi_id | str trim | is-not-empty) {
        $pane_env = ($pane_env | upsert YAZI_ID $yazi_id)
    }

    $pane_env
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
def get_tab_name [target_path: path] {
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

# Open a new pane and set up Neovim with Yazi integration
export def open_new_neovim_pane [file_path: path, yazi_id: string] {
    open_new_editor_pane $file_path $yazi_id "open_neovim.log"
}
