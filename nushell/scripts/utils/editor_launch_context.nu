#!/usr/bin/env nu

use common.nu [get_yazelix_runtime_dir]
use yzx_core_bridge.nu compute_runtime_env_via_yzx_core

def normalize_editor_command [editor?: string, runtime_dir?: string] {
    let text = ($editor | default "" | into string | str trim)
    if ($text | is-empty) {
        return ""
    }

    if (($text | path basename) == "yazelix_hx.sh") {
        let resolved_runtime = if $runtime_dir == null {
            get_yazelix_runtime_dir
        } else {
            $runtime_dir | path expand
        }
        return ($resolved_runtime | path join "shells" "posix" "yazelix_hx.sh")
    }

    $text
}

export def resolve_editor_launch_context [] {
    mut launch_env = (compute_runtime_env_via_yzx_core)
    let runtime_dir = ($launch_env.YAZELIX_RUNTIME_DIR? | default (get_yazelix_runtime_dir))
    let editor = (normalize_editor_command ($launch_env.EDITOR? | default "") $runtime_dir)

    if ($editor | is-not-empty) {
        $launch_env = ($launch_env | upsert EDITOR $editor)
        return {
            editor: $editor
            launch_env: $launch_env
        }
    }

    let runtime_dir = (get_yazelix_runtime_dir)
    let ambient_editor = (normalize_editor_command ($env.EDITOR? | default "") $runtime_dir)
    if ($ambient_editor | is-empty) {
        error make {msg: "EDITOR is not set. Set it in yazelix.toml under [editor] command, or export EDITOR in your shell."}
    }

    mut launch_env = {}
    if ($env.YAZELIX_MANAGED_HELIX_BINARY? | default "" | str trim | is-not-empty) {
        $launch_env = ($launch_env | upsert YAZELIX_MANAGED_HELIX_BINARY $env.YAZELIX_MANAGED_HELIX_BINARY)
    }
    if ($env.HELIX_RUNTIME? | default "" | str trim | is-not-empty) {
        $launch_env = ($launch_env | upsert HELIX_RUNTIME $env.HELIX_RUNTIME)
    }
    if (($ambient_editor | path basename) == "yazelix_hx.sh") {
        $launch_env = ($launch_env | upsert YAZELIX_RUNTIME_DIR $runtime_dir)
    }

    {
        editor: $ambient_editor
        launch_env: $launch_env
    }
}
