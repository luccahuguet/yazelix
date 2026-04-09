#!/usr/bin/env nu
# Zellij Configuration Merger
# Uses the Yazelix-managed user Zellij config when available, then native Zellij config, then Zellij defaults

use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS]
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/common.nu [get_yazelix_user_config_dir resolve_zellij_default_shell]
use ../utils/layout_generator.nu [render_custom_text_segment render_widget_tray_segment]
use ../utils/startup_profile.nu [profile_startup_step]
use ./zellij_plugin_paths.nu [
    PANE_ORCHESTRATOR_PLUGIN_ALIAS
    get_runtime_pane_orchestrator_wasm_path
    get_runtime_popup_runner_wasm_path
    get_runtime_zjstatus_wasm_path
    get_tracked_pane_orchestrator_wasm_path
    get_tracked_popup_runner_wasm_path
    get_tracked_zjstatus_wasm_path
    sync_pane_orchestrator_runtime_wasm
    sync_popup_runner_runtime_wasm
    sync_zjstatus_runtime_wasm
]

const zellij_generation_metadata_name = ".yazelix_generation.json"

# Fetch Zellij default configuration
def get_zellij_defaults [] {
    try { zellij setup --dump-config } catch {|err| 
        print $"❌ CRITICAL ERROR: Cannot fetch Zellij defaults: ($err.msg)"
        print "   Zellij must be available in PATH for Yazelix to work properly."
        print "   This indicates the merger is running outside the Nix environment."
        print "   Yazelix cannot function without proper Zellij configuration."
        exit 1
    }
}

def get_zellij_user_config_path [] {
    (get_yazelix_user_config_dir) | path join "zellij" "config.kdl"
}

def get_legacy_native_zellij_config_path [] {
    ($env.HOME | path join ".config" "zellij" "config.kdl")
}

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

def resolve_base_config_source [] {
    let managed_path = (get_zellij_user_config_path)
    let native_path = (get_legacy_native_zellij_config_path)

    if ($managed_path | path exists) {
        {
            source: "managed"
            path: $managed_path
            content: (read_text_if_exists $managed_path)
        }
    } else if ($native_path | path exists) {
        {
            source: "native"
            path: $native_path
            content: (read_text_if_exists $native_path)
        }
    } else {
        {
            source: "defaults"
            path: ""
            content: (get_zellij_defaults)
        }
    }
}

def describe_base_config_source [resolved: record] {
    match $resolved.source {
        "managed" => {
            print $"📥 Using managed Zellij config from ($resolved.path)"
        }
        "native" => {
            print $"📥 Using native Zellij config from ($resolved.path)"
        }
        _ => {
            print "📥 No user Zellij config found, fetching defaults..."
        }
    }
}

def resolve_zellij_plugin_artifacts [yazelix_dir: string] {
    let tracked_paths = [
        {
            name: "pane_orchestrator"
            tracked_path: (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)
            runtime_path: (get_runtime_pane_orchestrator_wasm_path)
            missing_label: "Tracked pane orchestrator wasm"
        }
        {
            name: "popup_runner"
            tracked_path: (get_tracked_popup_runner_wasm_path $yazelix_dir)
            runtime_path: (get_runtime_popup_runner_wasm_path)
            missing_label: "Tracked popup runner wasm"
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

def build_zellij_generation_fingerprint [
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

def record_generation_fingerprint [merged_config_dir: string, fingerprint: string] {
    let metadata_path = (get_zellij_generation_metadata_path $merged_config_dir)
    let temp_path = $"($metadata_path).tmp"
    {
        fingerprint: $fingerprint
        generated_at: (date now | format date "%Y-%m-%dT%H:%M:%S%.3f%:z")
    } | to json | save --force $temp_path
    mv --force $temp_path $metadata_path
}

def can_reuse_generated_zellij_state [
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

# Dynamic overrides sourced from yazelix.toml (takes precedence over user config)
def get_dynamic_overrides [config: record] {
    let pane_frames = ($config | get -o zellij_pane_frames | default "true")
    let pane_frames_value = if ($pane_frames | str starts-with "false") {
        "false"
    } else {
        "true"
    }

    let rounded = ($config | get -o zellij_rounded_corners | default "true")
    let rounded_value = if ($rounded | str starts-with "false") {
        "false"
    } else {
        "true"
    }

    # Zellij built-in themes (37 total: 28 dark + 9 light)
    let zellij_themes = [
        "ansi", "ao", "atelier-sulphurpool", "ayu_mirage", "ayu_dark",
        "catppuccin-frappe", "catppuccin-macchiato", "cyber-noir", "blade-runner",
        "retro-wave", "dracula", "everforest-dark", "gruvbox-dark", "iceberg-dark",
        "kanagawa", "lucario", "menace", "molokai-dark", "night-owl", "nightfox",
        "nord", "one-half-dark", "onedark", "solarized-dark", "tokyo-night-dark",
        "tokyo-night-storm", "tokyo-night", "vesper",
        "ayu_light", "catppuccin-latte", "everforest-light", "gruvbox-light",
        "iceberg-light", "dayfox", "pencil-light", "solarized-light", "tokyo-night-light"
    ]

    let theme_config = ($config | get -o zellij_theme | default "default")
    let theme = if $theme_config == "random" {
        $zellij_themes | shuffle | first
    } else {
        $theme_config
    }

    # disable_tips in yazelix.toml → show_startup_tips in Zellij config (inverted logic)
    let disable_tips = ($config | get -o disable_zellij_tips | default "true")
    let show_tips_value = if ($disable_tips | str starts-with "false") {
        "true"
    } else {
        "false"
    }

    let persistent_sessions = ($config | get -o persistent_sessions | default "false")
    let on_force_close_value = if ($persistent_sessions | str starts-with "true") {
        "detach"
    } else {
        "quit"
    }

    [
        "// === YAZELIX DYNAMIC SETTINGS (from yazelix.toml) ===",
        $"theme \"($theme)\"",
        $"show_startup_tips ($show_tips_value)",
        "show_release_notes false",
        $"on_force_close \"($on_force_close_value)\"",
        $"pane_frames ($pane_frames_value)",
        "ui {",
        "    pane_frames {",
        $"        rounded_corners ($rounded_value)",
        "    }",
        "}"
    ] | str join "\n"
}

# Ensure directory exists
def ensure_dir [path: string] {
    let dir = ($path | path dirname)
    if not ($dir | path exists) {
        mkdir $dir
    }
}

def extract_semantic_config_blocks [config_content: string] {
    mut stripped_lines = []
    mut load_plugin_lines = []
    mut plugin_lines = []
    mut keybind_lines = []
    mut active_block = ""
    mut brace_depth = 0

    for line in ($config_content | lines) {
        let trimmed = ($line | str trim)
        let open_braces = (($line | split chars | where {|char| $char == "{"}) | length)
        let close_braces = (($line | split chars | where {|char| $char == "}"}) | length)

        if ($active_block | is-empty) {
            let matched_block = (
                ["load_plugins", "plugins", "keybinds"]
                | where {|block_name| $trimmed | str starts-with $block_name }
                | get 0?
                | default ""
            )

            if ($matched_block | is-not-empty) {
                $active_block = $matched_block
                $brace_depth = ($open_braces - $close_braces)

                # Preserve compact one-line forms like:
                # keybinds { normal { bind "f1" { WriteChars "fixture"; } } }
                if $brace_depth <= 0 {
                    let inline_body = (
                        $trimmed
                        | str replace -r $"^($matched_block)\\s*\\{" ""
                        | str replace -r "\\}\\s*$" ""
                        | str trim
                    )
                    if ($inline_body | is-not-empty) {
                        match $matched_block {
                            "load_plugins" => {
                                $load_plugin_lines = ($load_plugin_lines | append $inline_body)
                            }
                            "plugins" => {
                                $plugin_lines = ($plugin_lines | append $inline_body)
                            }
                            "keybinds" => {
                                $keybind_lines = ($keybind_lines | append $inline_body)
                            }
                        }
                    }
                    $active_block = ""
                    $brace_depth = 0
                }
            } else {
                $stripped_lines = ($stripped_lines | append $line)
            }
        } else {
            $brace_depth = ($brace_depth + $open_braces - $close_braces)
            if $brace_depth > 0 {
                match $active_block {
                    "load_plugins" => {
                        $load_plugin_lines = ($load_plugin_lines | append $line)
                    }
                    "plugins" => {
                        $plugin_lines = ($plugin_lines | append $line)
                    }
                    "keybinds" => {
                        $keybind_lines = ($keybind_lines | append $line)
                    }
                }
            } else {
                $active_block = ""
            }
        }
    }

    {
        config_without_semantic_blocks: ($stripped_lines | str join "\n")
        load_plugin_lines: $load_plugin_lines
        plugin_lines: $plugin_lines
        keybind_lines: $keybind_lines
    }
}

def build_yazelix_load_plugins_block [
    existing_load_plugin_lines: list<string>
    pane_orchestrator_alias: string
    popup_runner_wasm_path: string
] {
    mut merged_plugin_lines = ($existing_load_plugin_lines | flatten)
    let pane_orchestrator_entry = $"  ($pane_orchestrator_alias)"
    let pane_orchestrator_present = ($merged_plugin_lines | any {|line| ($line | str trim) == $pane_orchestrator_alias })
    if not $pane_orchestrator_present {
        $merged_plugin_lines = ($merged_plugin_lines | append $pane_orchestrator_entry)
    }

    let popup_runner_entry = $"  \"file:($popup_runner_wasm_path)\""
    let popup_runner_present = ($merged_plugin_lines | any {|line| $line | str contains $popup_runner_wasm_path })
    if not $popup_runner_present {
        $merged_plugin_lines = ($merged_plugin_lines | append $popup_runner_entry)
    }

    (
        [
            "load_plugins {"
            ...($merged_plugin_lines | flatten)
            "}"
        ]
        | str join "\n"
    )
}

def build_yazelix_plugins_block [
    existing_plugin_lines: list<string>
    pane_orchestrator_alias: string
    pane_orchestrator_wasm_path: string
    widget_tray_segment: string
    custom_text_segment: string
    sidebar_width_percent: int
] {
    let escaped_widget_tray = ($widget_tray_segment | to json -r)
    let escaped_custom_text = ($custom_text_segment | to json -r)
    let escaped_sidebar_width_percent = ($sidebar_width_percent | into string | to json -r)
    let pane_alias_present = ($existing_plugin_lines | any {|line| $line | str contains $"($pane_orchestrator_alias) location=" })
    mut merged_plugin_lines = $existing_plugin_lines

    if not $pane_alias_present {
        $merged_plugin_lines = ($merged_plugin_lines | append [
            $"    ($pane_orchestrator_alias) location=\"file:($pane_orchestrator_wasm_path)\" {"
            $"        widget_tray_segment ($escaped_widget_tray)"
            $"        custom_text_segment ($escaped_custom_text)"
            $"        sidebar_width_percent ($escaped_sidebar_width_percent)"
            "    }"
        ])
    }

    if ($merged_plugin_lines | is-empty) {
        ""
    } else {
        (
            [
                "plugins {"
                ...($merged_plugin_lines | flatten)
                "}"
            ]
            | str join "\n"
        )
    }
}

def build_merged_keybinds_block [
    existing_keybind_lines: list<string>
    yazelix_keybind_lines: list<string>
] {
    let merged_keybind_lines = ($existing_keybind_lines | append $yazelix_keybind_lines | flatten)
    if ($merged_keybind_lines | is-empty) {
        ""
    } else {
        (
            [
                "keybinds {"
                ...$merged_keybind_lines
                "}"
            ]
            | str join "\n"
        )
    }
}

def read_yazelix_overrides [
    yazelix_dir: string
    pane_orchestrator_plugin_url: string
]: nothing -> record {
    let overrides_path = ($yazelix_dir | path join $ZELLIJ_CONFIG_PATHS.yazelix_overrides)

    if not ($overrides_path | path exists) {
        error make {msg: $"Missing Yazelix Zellij overrides file: ($overrides_path)"}
    }

    let runtime_ref = ($yazelix_dir | path expand)
    let resolved_overrides = (
        (open $overrides_path)
        | str replace -a "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__" $pane_orchestrator_plugin_url
        | str replace -a "__YAZELIX_RUNTIME_DIR__" $runtime_ref
    )
    let extracted_blocks = (extract_semantic_config_blocks $resolved_overrides)
    {
        overrides_without_keybinds: $extracted_blocks.config_without_semantic_blocks
        keybind_lines: $extracted_blocks.keybind_lines
    }
}

# Main function: Generate merged Zellij configuration
export def generate_merged_zellij_config [yazelix_dir: string, merged_config_dir_override?: string] {
    let merged_config_dir = if ($merged_config_dir_override | is-not-empty) {
        $merged_config_dir_override | path expand
    } else {
        $ZELLIJ_CONFIG_PATHS.merged_config_dir | path expand
    }
    let merged_config_path = ($merged_config_dir | path join "config.kdl")
    let yazelix_layout_dir = $"($merged_config_dir)/layouts"
    let config = parse_yazelix_config
    let widget_tray = ($config.zellij_widget_tray? | default ["editor", "shell", "term", "cpu", "ram"])
    let custom_text = ($config.zellij_custom_text? | default "")
    let kitty_protocol = ($config | get -o support_kitty_keyboard_protocol | default "true")
    let kitty_protocol_value = if ($kitty_protocol | str starts-with "false") { "false" } else { "true" }
    let default_shell = ($config.default_shell? | default "nu")
    let resolved_default_shell = (resolve_zellij_default_shell $yazelix_dir $default_shell)
    let default_mode = ($config.zellij_default_mode? | default "normal")
    let default_layout_name = if ($config.enable_sidebar? | default true) { "yzx_side" } else { "yzx_no_side" }
    let sidebar_width_percent = ($config.sidebar_width_percent? | default 20)
    let source_layouts_dir = $"($yazelix_dir)/($ZELLIJ_CONFIG_PATHS.layouts_dir)"
    let pane_orchestrator_plugin_url = $PANE_ORCHESTRATOR_PLUGIN_ALIAS
    let plugin_artifacts = (profile_startup_step "zellij_config" "resolve_plugin_artifacts" {
        resolve_zellij_plugin_artifacts $yazelix_dir
    })
    let base_config_source = (profile_startup_step "zellij_config" "load_base_config" {
        resolve_base_config_source
    })
    # `zellij_theme = "random"` is documented to pick a different theme on each
    # Yazelix restart, so warm-state reuse must stay disabled for that mode.
    let reuse_allowed = (($config.zellij_theme? | default "default") != "random")
    let generation_fingerprint = (
        profile_startup_step "zellij_config" "build_generation_fingerprint" {
            (
                build_zellij_generation_fingerprint
                    $config
                    $yazelix_dir
                    $base_config_source
                    $resolved_default_shell
                    $source_layouts_dir
                    $plugin_artifacts
            )
        }
    )

    if $reuse_allowed and (profile_startup_step "zellij_config" "check_generation_reuse" {
        (
            can_reuse_generated_zellij_state
                $merged_config_dir
                $merged_config_path
                $source_layouts_dir
                $generation_fingerprint
                $plugin_artifacts
        )
    }) {
        return $merged_config_path
    }

    describe_base_config_source $base_config_source
    print "🔄 Regenerating Zellij configuration..."

    # Ensure output directory exists
    ensure_dir $merged_config_path

    let pane_orchestrator_wasm_path = (profile_startup_step "zellij_config" "sync_pane_orchestrator_plugin" {
        sync_pane_orchestrator_runtime_wasm $yazelix_dir
    })
    let popup_runner_wasm_path = (profile_startup_step "zellij_config" "sync_popup_runner_plugin" {
        sync_popup_runner_runtime_wasm $yazelix_dir
    })
    let zjstatus_wasm_path = (profile_startup_step "zellij_config" "sync_zjstatus_plugin" {
        sync_zjstatus_runtime_wasm $yazelix_dir
    })
    let zjstatus_plugin_url = $"file:($zjstatus_wasm_path)"

    let yazelix_overrides = (profile_startup_step "zellij_config" "load_overrides" {
        read_yazelix_overrides $yazelix_dir $pane_orchestrator_plugin_url
    })
    let widget_tray_segment = (render_widget_tray_segment $widget_tray)
    let custom_text_segment = (render_custom_text_segment $custom_text)

    let target_layouts_dir = $"($merged_config_dir)/layouts"
    if ($source_layouts_dir | path exists) {
        use ../utils/layout_generator.nu
        if ($custom_text | is-not-empty) {
            print $"ℹ️  zjstatus custom text badge: '($custom_text)'"
        }
        profile_startup_step "zellij_config" "generate_layouts" {
            layout_generator generate_all_layouts $source_layouts_dir $target_layouts_dir $widget_tray $custom_text $pane_orchestrator_plugin_url $zjstatus_plugin_url $yazelix_dir $sidebar_width_percent
        }
    }

    let extracted_blocks = (profile_startup_step "zellij_config" "extract_semantic_blocks" {
        extract_semantic_config_blocks $base_config_source.content
    })

    # Remove any settings we control from base config (yazelix.toml takes precedence)
    # This prevents conflicts when multiple declarations of the same setting exist
    let base_config = ($extracted_blocks.config_without_semantic_blocks | lines | where {|line|
        let trimmed = ($line | str trim)
        not (
            ($trimmed | str starts-with "theme ") or
            ($trimmed | str starts-with "pane_frames ") or
            ($trimmed | str starts-with "support_kitty_keyboard_protocol ") or
            ($trimmed | str starts-with "default_mode ") or
            ($trimmed | str starts-with "default_layout ") or
            ($trimmed | str starts-with "layout_dir ") or
            ($trimmed | str starts-with "on_force_close ") or
            ($trimmed | str starts-with "show_startup_tips ") or
            ($trimmed | str starts-with "default_shell ")
        )
    } | str join "\n")
    let merged_keybinds_block = (build_merged_keybinds_block $extracted_blocks.keybind_lines $yazelix_overrides.keybind_lines)
    let merged_config = [
        "// ========================================",
        "// GENERATED ZELLIJ CONFIG (YAZELIX)",
        "// ========================================",
        "// Source preference:",
        "//   1) ~/.config/yazelix/user_configs/zellij/config.kdl (user-managed)",
        "//   2) ~/.config/zellij/config.kdl (native fallback, read-only)",
        "//   3) zellij setup --dump-config (defaults)",
        "//",
        $"// Generated: (date now | format date '%Y-%m-%d %H:%M:%S')",
        "// ========================================",
        "",
        $base_config,
        "",
        $yazelix_overrides.overrides_without_keybinds,
        "",
        $merged_keybinds_block,
        "",
        (build_yazelix_plugins_block
            $extracted_blocks.plugin_lines
            $PANE_ORCHESTRATOR_PLUGIN_ALIAS
            $pane_orchestrator_wasm_path
            $widget_tray_segment
            $custom_text_segment
            $sidebar_width_percent
        ),
        "",
        (get_dynamic_overrides $config),
        "",
        "// === YAZELIX ENFORCED SETTINGS ===",
        $"support_kitty_keyboard_protocol ($kitty_protocol_value)",
        $"default_mode \"($default_mode)\"",
        $"default_shell \"($resolved_default_shell)\"",
        $"default_layout \"($yazelix_layout_dir)/($default_layout_name).kdl\"",
        $"layout_dir \"($yazelix_layout_dir)\"",
        "",
        "// === YAZELIX BACKGROUND PLUGINS ===",
        (build_yazelix_load_plugins_block $extracted_blocks.load_plugin_lines $PANE_ORCHESTRATOR_PLUGIN_ALIAS $popup_runner_wasm_path)
    ] | str join "\n"
    
    # Write atomically (write to temp file, then move)
    let temp_path = $"($merged_config_path).tmp"
    try {
        $merged_config | save $temp_path
        mv $temp_path $merged_config_path
        record_generation_fingerprint $merged_config_dir $generation_fingerprint
        print $"✅ Zellij configuration generated successfully!"
        print $"   📁 Config saved to: ($merged_config_path)"
        print "   🔄 Config will auto-regenerate when source files change"
    } catch {|err|
        print $"❌ Failed to write merged config: ($err.msg)"
        # Clean up temp file if it exists
        if ($temp_path | path exists) {
            rm $temp_path
        }
        exit 1
    }
    
    $merged_config_path
}

# Export main function for external use
export def main [yazelix_dir: string] {
    generate_merged_zellij_config $yazelix_dir | ignore
}
