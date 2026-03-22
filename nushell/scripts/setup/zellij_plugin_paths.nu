#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_runtime_dir]

const pane_orchestrator_plugin_prefix = "yazelix_pane_orchestrator"
const pane_orchestrator_wasm_name = "yazelix_pane_orchestrator.wasm"

def atomic_copy [source_path: string, target_path: string] {
    let target_dir = ($target_path | path dirname)
    if not ($target_dir | path exists) {
        mkdir $target_dir
    }

    let tmp_path = $"($target_path).tmp"
    cp --force $source_path $tmp_path
    mv --force $tmp_path $target_path
}

def get_runtime_plugins_dir [] {
    $env.HOME | path join ".local" "share" "yazelix" "configs" "zellij" "plugins"
}

export def get_tracked_pane_orchestrator_wasm_path [yazelix_dir?: string] {
    let root = (($yazelix_dir | default (get_yazelix_runtime_dir)) | path expand)
    $root | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}

export def sync_pane_orchestrator_runtime_wasm [yazelix_dir?: string] {
    let tracked_path = (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)
    if not ($tracked_path | path exists) {
        error make {msg: $"Tracked pane orchestrator wasm not found at: ($tracked_path)"}
    }

    let wasm_hash = (open --raw $tracked_path | hash sha256 | str substring 0..<12)
    let runtime_dir = (get_runtime_plugins_dir)
    let runtime_file_name = $"($pane_orchestrator_plugin_prefix)_($wasm_hash).wasm"
    let runtime_path = ($runtime_dir | path join $runtime_file_name)

    if not ($runtime_path | path exists) {
        atomic_copy $tracked_path $runtime_path
    }

    if ($runtime_dir | path exists) {
        let plugin_name_pattern = ("^" + $pane_orchestrator_plugin_prefix + "(_[0-9a-f]+)?\\.wasm$")
        let stale_runtime_plugins = (
            ls $runtime_dir
            | where type == file
            | each {|entry|
                let full_path = $entry.name
                let file_name = ($full_path | path basename)
                {
                    full_path: $full_path
                    file_name: $file_name
                }
            }
            | where file_name =~ $plugin_name_pattern
            | where full_path != $runtime_path
            | get full_path
        )

        if ($stale_runtime_plugins | length) > 0 {
            rm --force ...$stale_runtime_plugins
        }
    }

    $runtime_path
}

export def get_pane_orchestrator_wasm_path [yazelix_dir?: string] {
    sync_pane_orchestrator_runtime_wasm $yazelix_dir
}
