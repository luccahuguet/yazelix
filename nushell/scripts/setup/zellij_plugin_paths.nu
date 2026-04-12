#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_runtime_dir]
use ../utils/atomic_writes.nu [copy_file_atomic write_text_atomic]

const pane_orchestrator_plugin_prefix = "yazelix_pane_orchestrator"
const pane_orchestrator_wasm_name = "yazelix_pane_orchestrator.wasm"
const zjstatus_wasm_name = "zjstatus.wasm"
export const PANE_ORCHESTRATOR_PLUGIN_ALIAS = "yazelix_pane_orchestrator"
const pane_orchestrator_required_permissions = [
    "ReadApplicationState"
    "OpenTerminalsOrPlugins"
    "ChangeApplicationState"
    "RunCommands"
    "WriteToStdin"
    "ReadCliPipes"
]
const zjstatus_plugin_prefix = "zjstatus"
const zjstatus_required_permissions = [
    "ReadApplicationState"
    "ChangeApplicationState"
    "RunCommands"
]

def get_runtime_plugins_dir [] {
    $env.HOME | path join ".local" "share" "yazelix" "configs" "zellij" "plugins"
}

def get_runtime_pane_orchestrator_target_path [] {
    (get_runtime_plugins_dir) | path join $pane_orchestrator_wasm_name
}

def get_runtime_zjstatus_target_path [] {
    (get_runtime_plugins_dir) | path join $zjstatus_wasm_name
}

def get_permissions_cache_path [] {
    $env.HOME | path join ".cache" "zellij" "permissions.kdl"
}

def parse_permission_blocks [content: string] {
    mut blocks = []
    let lines = ($content | lines)
    mut current_path = null
    mut current_permissions = []

    for line in $lines {
        let trimmed = ($line | str trim)

        if $current_path == null {
            let parsed = ($trimmed | parse --regex '^"(?<path>.+)"\s*\{$')
            let parsed_path = ($parsed | get -o 0.path)
            if $parsed_path != null {
                $current_path = $parsed_path
                $current_permissions = []
            }
            continue
        }

        if $trimmed == "}" {
            $blocks = ($blocks | append {
                path: $current_path
                permissions: $current_permissions
            })
            $current_path = null
            $current_permissions = []
            continue
        }

        if ($trimmed | is-not-empty) {
            $current_permissions = ($current_permissions | append $trimmed)
        }
    }

    $blocks
}

def build_permission_block [plugin_path: string, permissions: list<string>] {
    (
        [
            $"\"($plugin_path)\" {"
            ...($permissions | each {|permission| $"    ($permission)" })
            "}"
        ]
        | str join "\n"
    )
}

def upsert_permission_blocks [blocks: list<string>] {
    let permissions_cache_path = (get_permissions_cache_path)
    let existing_blocks = if ($permissions_cache_path | path exists) {
        parse_permission_blocks (open --raw $permissions_cache_path)
    } else {
        []
    }
    let target_paths = (
        $blocks
        | each {|block|
            (
                $block
                | lines
                | first
                | parse --regex '^"(?<path>.+)"\s*\{$'
                | get -o 0.path
            )
        }
        | where {|path| $path != null }
    )
    let retained_text = (
        $existing_blocks
        | where {|block| $block.path not-in $target_paths }
        | each {|block| build_permission_block $block.path $block.permissions }
    )
    let updated_content = ($retained_text | append $blocks | str join "\n\n")
    write_text_atomic $permissions_cache_path $updated_content --raw | ignore
    $permissions_cache_path
}

def preserve_plugin_permissions [
    plugin_prefix: string
    tracked_path: string
    runtime_path: string
    required_permissions: list<string>
] {
    let permissions_cache_path = (get_permissions_cache_path)
    if not ($permissions_cache_path | path exists) {
        return { status: "missing_cache" }
    }

    let raw_content = (open --raw $permissions_cache_path)
    let blocks = (parse_permission_blocks $raw_content)
    let matching_blocks = (
        $blocks
        | where {|block|
            let file_name = ($block.path | path basename)
            $file_name =~ ("^" + $plugin_prefix + "(_[0-9a-f]+)?\\.wasm$")
        }
    )

    if ($matching_blocks | is-empty) {
        return { status: "no_existing_source" }
    }

    let retained_blocks = (
        $blocks
        | where {|block|
            let file_name = ($block.path | path basename)
            not ($file_name =~ ("^" + $plugin_prefix + "(_[0-9a-f]+)?\\.wasm$"))
        }
    )
    let target_blocks = [
        (build_permission_block $tracked_path $required_permissions)
        (build_permission_block $runtime_path $required_permissions)
    ]
    let retained_text = ($retained_blocks | each {|block| build_permission_block $block.path $block.permissions })
    let updated_content = (
        $retained_text
        | append $target_blocks
        | str join "\n\n"
    )

    write_text_atomic $permissions_cache_path $updated_content --raw | ignore
    {
        status: "updated"
        source_path: (($matching_blocks | get 0.path))
    }
}

def preserve_pane_orchestrator_permissions [tracked_path: string, runtime_path: string] {
    preserve_plugin_permissions $pane_orchestrator_plugin_prefix $tracked_path $runtime_path $pane_orchestrator_required_permissions
}

def preserve_zjstatus_permissions [tracked_path: string, runtime_path: string] {
    preserve_plugin_permissions $zjstatus_plugin_prefix $tracked_path $runtime_path $zjstatus_required_permissions
}

export def get_tracked_pane_orchestrator_wasm_path [yazelix_dir?: string] {
    let root = (($yazelix_dir | default (get_yazelix_runtime_dir)) | path expand)
    $root | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}

export def get_tracked_zjstatus_wasm_path [yazelix_dir?: string] {
    let root = (($yazelix_dir | default (get_yazelix_runtime_dir)) | path expand)
    $root | path join "configs" "zellij" "plugins" $zjstatus_wasm_name
}

def remove_runtime_plugins_by_prefix [plugin_prefix: string] {
    let runtime_dir = (get_runtime_plugins_dir)
    if not ($runtime_dir | path exists) {
        return []
    }

    let plugin_name_pattern = ("^" + $plugin_prefix + "(_[0-9a-f]+)?\\.wasm$")
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
        | get full_path
    )

    if ($stale_runtime_plugins | length) > 0 {
        rm --force ...$stale_runtime_plugins
    }

    $stale_runtime_plugins
}

def remove_permission_blocks_by_prefix [plugin_prefix: string] {
    let permissions_cache_path = (get_permissions_cache_path)
    if not ($permissions_cache_path | path exists) {
        return {status: "missing_cache"}
    }

    let blocks = (parse_permission_blocks (open --raw $permissions_cache_path))
    let retained_blocks = (
        $blocks
        | where {|block|
            let file_name = ($block.path | path basename)
            not ($file_name =~ ("^" + $plugin_prefix + "(_[0-9a-f]+)?\\.wasm$"))
        }
    )

    if ($retained_blocks | length) == ($blocks | length) {
        return {status: "no_matches"}
    }

    let updated_content = (
        $retained_blocks
        | each {|block| build_permission_block $block.path $block.permissions }
        | str join "\n\n"
    )
    write_text_atomic $permissions_cache_path $updated_content --raw | ignore
    {status: "updated"}
}

export def cleanup_legacy_popup_runner_artifacts [] {
    {
        removed_runtime_plugins: (remove_runtime_plugins_by_prefix "yazelix_popup_runner")
        permissions: (remove_permission_blocks_by_prefix "yazelix_popup_runner")
    }
}

export def sync_pane_orchestrator_runtime_wasm [yazelix_dir?: string] {
    let tracked_path = (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)
    if not ($tracked_path | path exists) {
        error make {msg: $"Tracked pane orchestrator wasm not found at: ($tracked_path)"}
    }

    let runtime_dir = (get_runtime_plugins_dir)
    let runtime_path = (get_runtime_pane_orchestrator_target_path)

    copy_file_atomic $tracked_path $runtime_path | ignore

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

    preserve_pane_orchestrator_permissions $tracked_path $runtime_path | ignore

    $runtime_path
}

export def sync_zjstatus_runtime_wasm [yazelix_dir?: string] {
    let tracked_path = (get_tracked_zjstatus_wasm_path $yazelix_dir)
    if not ($tracked_path | path exists) {
        error make {msg: $"Tracked zjstatus wasm not found at: ($tracked_path)"}
    }

    let runtime_path = (get_runtime_zjstatus_target_path)

    copy_file_atomic $tracked_path $runtime_path | ignore

    preserve_zjstatus_permissions $tracked_path $runtime_path | ignore

    $runtime_path
}

export def get_zjstatus_wasm_path [yazelix_dir?: string] {
    sync_zjstatus_runtime_wasm $yazelix_dir
}

export def get_runtime_pane_orchestrator_wasm_path [] {
    get_runtime_pane_orchestrator_target_path
}

export def get_runtime_zjstatus_wasm_path [] {
    get_runtime_zjstatus_target_path
}

export def seed_yazelix_plugin_permissions [yazelix_dir?: string] {
    let legacy_popup_runner_cleanup = (cleanup_legacy_popup_runner_artifacts)
    let tracked_pane_orchestrator = (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)
    let tracked_zjstatus = (get_tracked_zjstatus_wasm_path $yazelix_dir)
    let runtime_pane_orchestrator = (sync_pane_orchestrator_runtime_wasm $yazelix_dir)
    let runtime_zjstatus = (sync_zjstatus_runtime_wasm $yazelix_dir)

    let blocks = [
        (build_permission_block $tracked_pane_orchestrator $pane_orchestrator_required_permissions)
        (build_permission_block $runtime_pane_orchestrator $pane_orchestrator_required_permissions)
        (build_permission_block $tracked_zjstatus $zjstatus_required_permissions)
        (build_permission_block $runtime_zjstatus $zjstatus_required_permissions)
    ]
    let permissions_cache_path = (upsert_permission_blocks $blocks)

    {
        permissions_cache_path: $permissions_cache_path
        legacy_popup_runner_cleanup: $legacy_popup_runner_cleanup
        tracked_pane_orchestrator: $tracked_pane_orchestrator
        runtime_pane_orchestrator: $runtime_pane_orchestrator
        tracked_zjstatus: $tracked_zjstatus
        runtime_zjstatus: $runtime_zjstatus
    }
}
