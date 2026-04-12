#!/usr/bin/env nu

use ../utils/constants.nu ZELLIJ_CONFIG_PATHS
use ../utils/atomic_writes.nu write_text_atomic
use ./zellij_plugin_paths.nu [
    get_runtime_pane_orchestrator_wasm_path
    get_runtime_zjstatus_wasm_path
    get_tracked_pane_orchestrator_wasm_path
    get_tracked_zjstatus_wasm_path
]

const zellij_generation_metadata_name = ".yazelix_generation.json"

def read_text_if_exists [path_value: string] {
    if not ($path_value | path exists) {
        return ""
    }

    open --raw $path_value
}

def hash_text [value: string] {
    $value | hash sha256
}

def hash_file [path_value: string] {
    open --raw $path_value | hash sha256
}

def get_zellij_generation_metadata_path [merged_config_dir: string] {
    $merged_config_dir | path join $zellij_generation_metadata_name
}

def list_source_layout_files [source_layouts_dir: string] {
    let source_root = ($source_layouts_dir | path expand)
    if not ($source_root | path exists) {
        return []
    }
    let fragment_dir = ($source_root | path join "fragments")
    let top_level_files = (
        ls $source_root
        | where type == file
        | get name
        | where {|path_value| (($path_value | path parse | get extension | default "") == "kdl") }
    )
    let fragment_files = if ($fragment_dir | path exists) {
        ls $fragment_dir
        | where type == file
        | get name
        | where {|path_value| (($path_value | path parse | get extension | default "") == "kdl") }
    } else {
        []
    }

    ($top_level_files | append $fragment_files) | sort
}

def list_expected_layout_targets [source_layouts_dir: string, merged_config_dir: string] {
    if not (($source_layouts_dir | path expand) | path exists) {
        return []
    }

    let layout_names = (
        ls ($source_layouts_dir | path expand)
        | where type == file
        | get name
        | where {|path_value| (($path_value | path parse | get extension | default "") == "kdl") }
        | each {|path_value| $path_value | path basename }
        | sort
    )

    $layout_names | each {|layout_name| ($merged_config_dir | path join "layouts" $layout_name | path expand) }
}

export def resolve_zellij_plugin_artifacts [yazelix_dir: string] {
    let tracked_paths = [
        {
            name: "pane_orchestrator"
            tracked_path: (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)
            runtime_path: (get_runtime_pane_orchestrator_wasm_path)
            missing_label: "Tracked pane orchestrator wasm"
        }
        {
            name: "zjstatus"
            tracked_path: (get_tracked_zjstatus_wasm_path $yazelix_dir)
            runtime_path: (get_runtime_zjstatus_wasm_path)
            missing_label: "Tracked zjstatus wasm"
        }
    ]

    $tracked_paths | each {|artifact|
        if not ($artifact.tracked_path | path exists) {
            error make {msg: $"($artifact.missing_label) not found at: ($artifact.tracked_path)"}
        }

        {
            name: $artifact.name
            tracked_path: $artifact.tracked_path
            tracked_hash: (hash_file $artifact.tracked_path)
            runtime_path: $artifact.runtime_path
        }
    }
}

export def build_zellij_generation_fingerprint [
    config: record
    yazelix_dir: string
    base_config_source: record
    resolved_default_shell: string
    source_layouts_dir: string
    plugin_artifacts: list<record>
] {
    let overrides_path = ($yazelix_dir | path join $ZELLIJ_CONFIG_PATHS.yazelix_overrides)
    let relevant_config = {
        zellij_widget_tray: ($config.zellij_widget_tray? | default ["editor", "shell", "term", "cpu", "ram"])
        zellij_custom_text: ($config.zellij_custom_text? | default "")
        support_kitty_keyboard_protocol: ($config.support_kitty_keyboard_protocol? | default "true")
        default_shell: ($config.default_shell? | default "nu")
        resolved_default_shell: $resolved_default_shell
        zellij_default_mode: ($config.zellij_default_mode? | default "normal")
        enable_sidebar: ($config.enable_sidebar? | default true)
        sidebar_width_percent: ($config.sidebar_width_percent? | default 20)
        popup_width_percent: ($config.popup_width_percent? | default 90)
        popup_height_percent: ($config.popup_height_percent? | default 90)
        disable_zellij_tips: ($config.disable_zellij_tips? | default "true")
        zellij_pane_frames: ($config.zellij_pane_frames? | default "true")
        zellij_rounded_corners: ($config.zellij_rounded_corners? | default "true")
        zellij_theme: ($config.zellij_theme? | default "default")
        persistent_sessions: ($config.persistent_sessions? | default "false")
    }
    let layout_sources = (
        list_source_layout_files $source_layouts_dir
        | each {|path_value|
            {
                path: $path_value
                hash: (open --raw $path_value | hash sha256)
            }
        }
    )

    {
        schema_version: 1
        runtime_dir: ($yazelix_dir | path expand)
        relevant_config: $relevant_config
        base_config: {
            source: $base_config_source.source
            path: ($base_config_source.path? | default "")
            hash: (hash_text $base_config_source.content)
        }
        overrides_hash: (hash_text (read_text_if_exists $overrides_path))
        layout_sources: $layout_sources
        plugins: (
            $plugin_artifacts
            | each {|artifact|
                {
                    name: $artifact.name
                    tracked_path: ($artifact.tracked_path | path expand)
                    tracked_hash: $artifact.tracked_hash
                    runtime_path: ($artifact.runtime_path | path expand)
                }
            }
        )
    } | to json -r | hash sha256
}

def load_cached_generation_fingerprint [merged_config_dir: string] {
    let metadata_path = (get_zellij_generation_metadata_path $merged_config_dir)
    if not ($metadata_path | path exists) {
        return ""
    }

    try {
        open $metadata_path | get fingerprint | into string
    } catch {
        ""
    }
}

export def record_generation_fingerprint [merged_config_dir: string, fingerprint: string] {
    let metadata_path = (get_zellij_generation_metadata_path $merged_config_dir)
    write_text_atomic $metadata_path ({
        fingerprint: $fingerprint
        generated_at: (date now | format date "%Y-%m-%dT%H:%M:%S%.3f%:z")
    } | to json) --raw | ignore
}

export def can_reuse_generated_zellij_state [
    merged_config_dir: string
    merged_config_path: string
    source_layouts_dir: string
    fingerprint: string
    plugin_artifacts: list<record>
] {
    let expected_layout_targets = (list_expected_layout_targets $source_layouts_dir $merged_config_dir)
    let cached_fingerprint = (load_cached_generation_fingerprint $merged_config_dir)
    let required_paths = (
        [$merged_config_path]
        | append ($plugin_artifacts | get runtime_path)
        | append $expected_layout_targets
    )
    let runtime_plugins_match = (
        $plugin_artifacts
        | all {|artifact|
            ($artifact.runtime_path | path exists) and ((hash_file $artifact.runtime_path) == $artifact.tracked_hash)
        }
    )

    ($cached_fingerprint == $fingerprint) and ($required_paths | all {|path_value| $path_value | path exists }) and $runtime_plugins_match
}
