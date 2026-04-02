#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_runtime_dir]

const pane_orchestrator_plugin_prefix = "yazelix_pane_orchestrator"
const pane_orchestrator_wasm_name = "yazelix_pane_orchestrator.wasm"
const popup_runner_plugin_prefix = "yazelix_popup_runner"
const popup_runner_wasm_name = "yazelix_popup_runner.wasm"
const zjstatus_wasm_name = "zjstatus.wasm"
const zjframes_plugin_prefix = "zjframes"
const zjframes_wasm_name = "zjframes.wasm"
export const PANE_ORCHESTRATOR_PLUGIN_ALIAS = "yazelix_pane_orchestrator"
const pane_orchestrator_required_permissions = [
    "ReadApplicationState"
    "OpenTerminalsOrPlugins"
    "ChangeApplicationState"
    "WriteToStdin"
    "ReadCliPipes"
]
const popup_runner_required_permissions = [
    "ReadApplicationState"
    "ChangeApplicationState"
    "ReadCliPipes"
]
const zjstatus_plugin_prefix = "zjstatus"
const zjstatus_required_permissions = [
    "ReadApplicationState"
    "ChangeApplicationState"
    "RunCommands"
]
const zjframes_required_permissions = [
    "ReadApplicationState"
    "ChangeApplicationState"
]

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
    let cache_dir = ($permissions_cache_path | path dirname)
    if not ($cache_dir | path exists) {
        mkdir $cache_dir
    }
    $updated_content | save --force --raw $permissions_cache_path
    $permissions_cache_path
}

def permission_block_is_sufficient [permissions: list<string>, required_permissions: list<string>] {
    $required_permissions
    | all {|permission| $permission in $permissions }
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
    let pane_orchestrator_blocks = (
        $blocks
        | where {|block|
            let file_name = ($block.path | path basename)
            $file_name =~ ("^" + $plugin_prefix + "(_[0-9a-f]+)?\\.wasm$")
        }
    )

    let source_block = (
        $pane_orchestrator_blocks
        | where {|block| permission_block_is_sufficient $block.permissions $required_permissions }
        | get -o 0
    )

    if $source_block == null {
        return { status: "no_granted_source" }
    }

    let retained_blocks = (
        $blocks
        | where {|block| ($block.path != $tracked_path) and ($block.path != $runtime_path) }
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

    $updated_content | save --force --raw $permissions_cache_path
    {
        status: "updated"
        source_path: $source_block.path
    }
}

def preserve_pane_orchestrator_permissions [tracked_path: string, runtime_path: string] {
    preserve_plugin_permissions $pane_orchestrator_plugin_prefix $tracked_path $runtime_path $pane_orchestrator_required_permissions
}

def preserve_popup_runner_permissions [tracked_path: string, runtime_path: string] {
    preserve_plugin_permissions $popup_runner_plugin_prefix $tracked_path $runtime_path $popup_runner_required_permissions
}

def preserve_zjstatus_permissions [tracked_path: string, runtime_path: string] {
    preserve_plugin_permissions $zjstatus_plugin_prefix $tracked_path $runtime_path $zjstatus_required_permissions
}

def preserve_zjframes_permissions [tracked_path: string, runtime_path: string] {
    preserve_plugin_permissions $zjframes_plugin_prefix $tracked_path $runtime_path $zjframes_required_permissions
}

export def get_tracked_pane_orchestrator_wasm_path [yazelix_dir?: string] {
    let root = (($yazelix_dir | default (get_yazelix_runtime_dir)) | path expand)
    $root | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}

export def get_tracked_popup_runner_wasm_path [yazelix_dir?: string] {
    let root = (($yazelix_dir | default (get_yazelix_runtime_dir)) | path expand)
    $root | path join "configs" "zellij" "plugins" $popup_runner_wasm_name
}

export def get_tracked_zjstatus_wasm_path [yazelix_dir?: string] {
    let root = (($yazelix_dir | default (get_yazelix_runtime_dir)) | path expand)
    $root | path join "configs" "zellij" "plugins" $zjstatus_wasm_name
}

export def get_tracked_zjframes_wasm_path [yazelix_dir?: string] {
    let root = (($yazelix_dir | default (get_yazelix_runtime_dir)) | path expand)
    $root | path join "configs" "zellij" "plugins" $zjframes_wasm_name
}

export def sync_pane_orchestrator_runtime_wasm [yazelix_dir?: string] {
    let tracked_path = (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)
    if not ($tracked_path | path exists) {
        error make {msg: $"Tracked pane orchestrator wasm not found at: ($tracked_path)"}
    }

    let runtime_dir = (get_runtime_plugins_dir)
    let runtime_file_name = $pane_orchestrator_wasm_name
    let runtime_path = ($runtime_dir | path join $runtime_file_name)

    atomic_copy $tracked_path $runtime_path

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

export def sync_popup_runner_runtime_wasm [yazelix_dir?: string] {
    let tracked_path = (get_tracked_popup_runner_wasm_path $yazelix_dir)
    if not ($tracked_path | path exists) {
        error make {msg: $"Tracked popup runner wasm not found at: ($tracked_path)"}
    }

    let runtime_dir = (get_runtime_plugins_dir)
    let runtime_file_name = $popup_runner_wasm_name
    let runtime_path = ($runtime_dir | path join $runtime_file_name)

    atomic_copy $tracked_path $runtime_path

    if ($runtime_dir | path exists) {
        let plugin_name_pattern = ("^" + $popup_runner_plugin_prefix + "(_[0-9a-f]+)?\\.wasm$")
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

    preserve_popup_runner_permissions $tracked_path $runtime_path | ignore

    $runtime_path
}

export def sync_zjstatus_runtime_wasm [yazelix_dir?: string] {
    let tracked_path = (get_tracked_zjstatus_wasm_path $yazelix_dir)
    if not ($tracked_path | path exists) {
        error make {msg: $"Tracked zjstatus wasm not found at: ($tracked_path)"}
    }

    let runtime_dir = (get_runtime_plugins_dir)
    let runtime_path = ($runtime_dir | path join $zjstatus_wasm_name)

    atomic_copy $tracked_path $runtime_path

    preserve_zjstatus_permissions $tracked_path $runtime_path | ignore

    $runtime_path
}

export def sync_zjframes_runtime_wasm [yazelix_dir?: string] {
    let tracked_path = (get_tracked_zjframes_wasm_path $yazelix_dir)
    if not ($tracked_path | path exists) {
        error make {msg: $"Tracked zjframes wasm not found at: ($tracked_path)"}
    }

    let runtime_dir = (get_runtime_plugins_dir)
    let runtime_path = ($runtime_dir | path join $zjframes_wasm_name)

    atomic_copy $tracked_path $runtime_path

    if ($runtime_dir | path exists) {
        let plugin_name_pattern = ("^" + $zjframes_plugin_prefix + "(_[0-9a-f]+)?\\.wasm$")
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

    preserve_zjframes_permissions $tracked_path $runtime_path | ignore

    $runtime_path
}

export def get_pane_orchestrator_wasm_path [yazelix_dir?: string] {
    sync_pane_orchestrator_runtime_wasm $yazelix_dir
}

export def get_popup_runner_wasm_path [yazelix_dir?: string] {
    sync_popup_runner_runtime_wasm $yazelix_dir
}

export def get_zjstatus_wasm_path [yazelix_dir?: string] {
    sync_zjstatus_runtime_wasm $yazelix_dir
}

export def get_zjframes_wasm_path [yazelix_dir?: string] {
    sync_zjframes_runtime_wasm $yazelix_dir
}

export def seed_yazelix_plugin_permissions [yazelix_dir?: string] {
    let tracked_pane_orchestrator = (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)
    let tracked_popup_runner = (get_tracked_popup_runner_wasm_path $yazelix_dir)
    let tracked_zjstatus = (get_tracked_zjstatus_wasm_path $yazelix_dir)
    let tracked_zjframes = (get_tracked_zjframes_wasm_path $yazelix_dir)
    let runtime_pane_orchestrator = (sync_pane_orchestrator_runtime_wasm $yazelix_dir)
    let runtime_popup_runner = (sync_popup_runner_runtime_wasm $yazelix_dir)
    let runtime_zjstatus = (sync_zjstatus_runtime_wasm $yazelix_dir)
    let runtime_zjframes = (sync_zjframes_runtime_wasm $yazelix_dir)

    let blocks = [
        (build_permission_block $tracked_pane_orchestrator $pane_orchestrator_required_permissions)
        (build_permission_block $runtime_pane_orchestrator $pane_orchestrator_required_permissions)
        (build_permission_block $tracked_popup_runner $popup_runner_required_permissions)
        (build_permission_block $runtime_popup_runner $popup_runner_required_permissions)
        (build_permission_block $tracked_zjstatus $zjstatus_required_permissions)
        (build_permission_block $runtime_zjstatus $zjstatus_required_permissions)
        (build_permission_block $tracked_zjframes $zjframes_required_permissions)
        (build_permission_block $runtime_zjframes $zjframes_required_permissions)
    ]
    let permissions_cache_path = (upsert_permission_blocks $blocks)

    {
        permissions_cache_path: $permissions_cache_path
        tracked_pane_orchestrator: $tracked_pane_orchestrator
        runtime_pane_orchestrator: $runtime_pane_orchestrator
        tracked_popup_runner: $tracked_popup_runner
        runtime_popup_runner: $runtime_popup_runner
        tracked_zjstatus: $tracked_zjstatus
        runtime_zjstatus: $runtime_zjstatus
        tracked_zjframes: $tracked_zjframes
        runtime_zjframes: $runtime_zjframes
    }
}
