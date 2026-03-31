#!/usr/bin/env nu
# Zellij Configuration Merger
# Uses the Yazelix-managed user Zellij config when available, falls back to Zellij defaults

use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS]
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/common.nu [get_yazelix_runtime_reference_dir get_yazelix_user_config_dir]
use ../utils/layout_generator.nu [render_custom_text_segment render_widget_tray_segment]
use ./zellij_plugin_paths.nu [PANE_ORCHESTRATOR_PLUGIN_ALIAS get_pane_orchestrator_wasm_path get_popup_runner_wasm_path]

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

def reconcile_zellij_user_config_path [] {
    let current_path = (get_zellij_user_config_path)
    let legacy_path = (get_legacy_native_zellij_config_path)
    let current_exists = ($current_path | path exists)
    let legacy_exists = ($legacy_path | path exists)

    if $current_exists and $legacy_exists {
        error make {
            msg: (
                [
                    "Yazelix found duplicate Zellij user config files in both user_configs and the native Zellij path."
                    $"user_configs path: ($current_path)"
                    $"native legacy path: ($legacy_path)"
                    ""
                    "Keep only the user_configs copy. Move or delete ~/.config/zellij/config.kdl so Yazelix has one clear managed owner."
                ] | str join "\n"
            )
        }
    }

    if $legacy_exists {
        mkdir ($current_path | path dirname)
        mv $legacy_path $current_path
    }

    $current_path
}

# Read the Yazelix-managed user Zellij config if it exists
def read_user_zellij_config [] {
    let user_config_path = (reconcile_zellij_user_config_path)
    if ($user_config_path | path exists) {
        try {
            print $"📥 Using existing Zellij config from ($user_config_path)"
            open $user_config_path
        } catch {|err|
            print $"⚠️  Could not read user config: ($err.msg)"
            ""
        }
    } else {
        ""
    }
}

# Choose the base config: Yazelix-managed user config if present, otherwise Zellij defaults
def get_base_config [] {
    let user_config = read_user_zellij_config
    if ($user_config | is-not-empty) {
        $user_config
    } else {
        print "📥 No user Zellij config found, fetching defaults..."
        get_zellij_defaults
    }
}

# Dynamic overrides sourced from yazelix.toml (takes precedence over user config)
def get_dynamic_overrides [] {
    let config = (try {
        parse_yazelix_config
    } catch {
        {zellij_rounded_corners: "true", zellij_theme: "default", disable_zellij_tips: "true", zellij_default_mode: "normal"}
    })

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

    [
        "// === YAZELIX DYNAMIC SETTINGS (from yazelix.toml) ===",
        $"theme \"($theme)\"",
        $"show_startup_tips ($show_tips_value)",
        "show_release_notes false",
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

def split_top_level_block [config_content: string, block_name: string] {
    mut stripped_lines = []
    mut block_lines = []
    mut in_named_block = false
    mut brace_depth = 0

    for line in ($config_content | lines) {
        let trimmed = ($line | str trim)
        let open_braces = (($line | split chars | where {|char| $char == "{"}) | length)
        let close_braces = (($line | split chars | where {|char| $char == "}"}) | length)

        if not $in_named_block {
            if ($trimmed | str starts-with $block_name) {
                $in_named_block = true
                $brace_depth = ($open_braces - $close_braces)
            } else {
                $stripped_lines = ($stripped_lines | append $line)
            }
        } else {
            $brace_depth = ($brace_depth + $open_braces - $close_braces)
            if $brace_depth > 0 {
                $block_lines = ($block_lines | append $line)
            } else {
                $in_named_block = false
            }
        }
    }

    {
        config_without_block: ($stripped_lines | str join "\n")
        block_lines: $block_lines
    }
}

def split_load_plugins_block [config_content: string] {
    let split = (split_top_level_block $config_content "load_plugins")
    {
        config_without_load_plugins: $split.config_without_block
        load_plugin_lines: $split.block_lines
    }
}

def split_keybinds_block [config_content: string] {
    let split = (split_top_level_block $config_content "keybinds")
    {
        config_without_keybinds: $split.config_without_block
        keybind_lines: $split.block_lines
    }
}

def split_plugins_block [config_content: string] {
    let split = (split_top_level_block $config_content "plugins")
    {
        config_without_plugins: $split.config_without_block
        plugin_lines: $split.block_lines
    }
}

def build_yazelix_load_plugins_block [
    existing_load_plugin_lines: list<string>
    pane_orchestrator_alias: string
    popup_runner_wasm_path: string
] {
    mut merged_plugin_lines = $existing_load_plugin_lines
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
            ...$merged_plugin_lines
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

    let runtime_ref = (get_yazelix_runtime_reference_dir)
    let resolved_overrides = (
        (open $overrides_path)
        | str replace -a "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__" $pane_orchestrator_plugin_url
        | str replace -a "__YAZELIX_RUNTIME_DIR__" $runtime_ref
    )
    let split_keybinds = (split_keybinds_block $resolved_overrides)
    {
        overrides_without_keybinds: $split_keybinds.config_without_keybinds
        keybind_lines: $split_keybinds.keybind_lines
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
    let default_mode = ($config.zellij_default_mode? | default "normal")
    let default_layout_name = if ($config.enable_sidebar? | default true) { "yzx_side" } else { "yzx_no_side" }
    let sidebar_width_percent = ($config.sidebar_width_percent? | default 20)
    
    print "🔄 Regenerating Zellij configuration..."
    
    # Ensure output directory exists
    ensure_dir $merged_config_path
    
    let pane_orchestrator_wasm_path = (get_pane_orchestrator_wasm_path $yazelix_dir)
    let pane_orchestrator_plugin_url = $PANE_ORCHESTRATOR_PLUGIN_ALIAS
    let popup_runner_wasm_path = (get_popup_runner_wasm_path $yazelix_dir)
    let yazelix_overrides = (read_yazelix_overrides $yazelix_dir $pane_orchestrator_plugin_url)
    let widget_tray_segment = (render_widget_tray_segment $widget_tray)
    let custom_text_segment = (render_custom_text_segment $custom_text)

    if not ($pane_orchestrator_wasm_path | path exists) {
        error make {msg: $"Pane orchestrator runtime wasm not found at: ($pane_orchestrator_wasm_path)"}
    }
    if not ($popup_runner_wasm_path | path exists) {
        error make {msg: $"Popup runner runtime wasm not found at: ($popup_runner_wasm_path)"}
    }

    # Copy layouts directory to merged config
    let source_layouts_dir = $"($yazelix_dir)/($ZELLIJ_CONFIG_PATHS.layouts_dir)"
    let target_layouts_dir = $"($merged_config_dir)/layouts"
    if ($source_layouts_dir | path exists) {
        # Copy layouts to merged config directory
        use ../utils/layout_generator.nu
        if ($custom_text | is-not-empty) {
            print $"ℹ️  zjstatus custom text badge: '($custom_text)'"
        }
        layout_generator generate_all_layouts $source_layouts_dir $target_layouts_dir $widget_tray $custom_text $pane_orchestrator_plugin_url $yazelix_dir $sidebar_width_percent
    }
    
    # Generate configuration from user config or defaults
    let base_config_raw = get_base_config
    let extracted_load_plugins = (split_load_plugins_block $base_config_raw)
    let extracted_plugins = (split_plugins_block $extracted_load_plugins.config_without_load_plugins)
    let extracted_keybinds = (split_keybinds_block $extracted_plugins.config_without_plugins)
    # Remove any settings we control from base config (yazelix.toml takes precedence)
    # This prevents conflicts when multiple declarations of the same setting exist
    let base_config = ($extracted_keybinds.config_without_keybinds | lines | where {|line|
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
    let merged_keybinds_block = (build_merged_keybinds_block $extracted_keybinds.keybind_lines $yazelix_overrides.keybind_lines)
    let merged_config = [
        "// ========================================",
        "// GENERATED ZELLIJ CONFIG (YAZELIX)",
        "// ========================================",
        "// Source preference:",
        "//   1) ~/.config/yazelix/user_configs/zellij/config.kdl (user-managed)",
        "//   2) zellij setup --dump-config (defaults)",
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
            $extracted_plugins.plugin_lines
            $PANE_ORCHESTRATOR_PLUGIN_ALIAS
            $pane_orchestrator_wasm_path
            $widget_tray_segment
            $custom_text_segment
            $sidebar_width_percent
        ),
        "",
        (get_dynamic_overrides),
        "",
        "// === YAZELIX ENFORCED SETTINGS ===",
        "pane_frames false",
        $"support_kitty_keyboard_protocol ($kitty_protocol_value)",
        $"default_mode \"($default_mode)\"",
        $"default_shell \"($default_shell)\"",
        $"default_layout \"($yazelix_layout_dir)/($default_layout_name).kdl\"",
        $"layout_dir \"($yazelix_layout_dir)\"",
        "",
        "// === YAZELIX BACKGROUND PLUGINS ===",
        (build_yazelix_load_plugins_block $extracted_load_plugins.load_plugin_lines $PANE_ORCHESTRATOR_PLUGIN_ALIAS $popup_runner_wasm_path)
    ] | str join "\n"
    
    # Write atomically (write to temp file, then move)
    let temp_path = $"($merged_config_path).tmp"
    try {
        $merged_config | save $temp_path
        mv $temp_path $merged_config_path
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
