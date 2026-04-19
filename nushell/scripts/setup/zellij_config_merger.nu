#!/usr/bin/env nu
# Zellij Configuration Merger
# Uses the Yazelix-managed user Zellij config when available, then native Zellij config, then Zellij defaults

use ../utils/constants.nu [DEFAULT_SHELL ZELLIJ_CONFIG_PATHS]
use ../utils/atomic_writes.nu write_text_atomic
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/common.nu resolve_zellij_default_shell
use ../utils/layout_generator.nu [render_custom_text_segment render_widget_tray_segment]
use ../utils/startup_profile.nu [profile_startup_step]
use ./zellij_base_config.nu [
    describe_base_config_source
    resolve_base_config_source
]
use ./zellij_generation_state.nu [
    build_zellij_generation_fingerprint
    can_reuse_generated_zellij_state
    record_generation_fingerprint
    resolve_zellij_plugin_artifacts
]
use ./zellij_owned_settings.nu [
    build_yazelix_ui_block
    render_yazelix_top_level_settings_block
    strip_yazelix_owned_top_level_settings
]
use ../utils/zellij_render_plan.nu [compute_zellij_render_plan]
use ./zellij_semantic_blocks.nu [
    build_merged_keybinds_block
    build_yazelix_load_plugins_block
    build_yazelix_plugins_block
    extract_semantic_config_blocks
    read_yazelix_override_keybinds
]
use ./zellij_plugin_paths.nu [
    PANE_ORCHESTRATOR_PLUGIN_ALIAS
    cleanup_legacy_popup_runner_artifacts
    sync_pane_orchestrator_runtime_wasm
    sync_zjstatus_runtime_wasm
]

# Ensure directory exists
def ensure_dir [path: string] {
    let dir = ($path | path dirname)
    if not ($dir | path exists) {
        mkdir $dir
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
    let default_shell = ($config.default_shell? | default $DEFAULT_SHELL)
    let resolved_default_shell = (resolve_zellij_default_shell $yazelix_dir $default_shell)
    let source_layouts_dir = $"($yazelix_dir)/($ZELLIJ_CONFIG_PATHS.layouts_dir)"
    let pane_orchestrator_plugin_url = $PANE_ORCHESTRATOR_PLUGIN_ALIAS
    let plugin_artifacts = (profile_startup_step "zellij_config" "resolve_plugin_artifacts" {
        resolve_zellij_plugin_artifacts $yazelix_dir
    })
    profile_startup_step "zellij_config" "cleanup_legacy_popup_runner" {
        cleanup_legacy_popup_runner_artifacts
    } | ignore
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

    let render_plan = (compute_zellij_render_plan $yazelix_dir $config $yazelix_layout_dir $resolved_default_shell)
    let widget_tray = $render_plan.widget_tray
    let custom_text = $render_plan.custom_text
    let resolved_owned_settings = {
        rounded_value: $render_plan.rounded_value
        dynamic_top_level_settings: $render_plan.dynamic_top_level_settings
        enforced_top_level_settings: $render_plan.enforced_top_level_settings
        owned_top_level_setting_names: $render_plan.owned_top_level_setting_names
    }

    describe_base_config_source $base_config_source
    print "🔄 Regenerating Zellij configuration..."

    # Ensure output directory exists
    ensure_dir $merged_config_path

    let pane_orchestrator_wasm_path = (profile_startup_step "zellij_config" "sync_pane_orchestrator_plugin" {
        sync_pane_orchestrator_runtime_wasm $yazelix_dir
    })
    let zjstatus_wasm_path = (profile_startup_step "zellij_config" "sync_zjstatus_plugin" {
        sync_zjstatus_runtime_wasm $yazelix_dir
    })
    let zjstatus_plugin_url = $"file:($zjstatus_wasm_path)"

    let yazelix_override_keybinds = (profile_startup_step "zellij_config" "load_override_keybinds" {
        read_yazelix_override_keybinds $yazelix_dir $pane_orchestrator_plugin_url
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
            layout_generator generate_all_layouts $source_layouts_dir $target_layouts_dir $widget_tray $custom_text $pane_orchestrator_plugin_url $zjstatus_plugin_url $yazelix_dir $render_plan.layout_percentages
        }
    }

    let extracted_blocks = (profile_startup_step "zellij_config" "extract_semantic_blocks" {
        extract_semantic_config_blocks $base_config_source.content
    })

    # Current upstream Zellij config parsing is first-match for these top-level
    # options, so Yazelix must strip and replace the settings it owns.
    let dynamic_top_level_settings = $resolved_owned_settings.dynamic_top_level_settings
    let enforced_top_level_settings = $resolved_owned_settings.enforced_top_level_settings
    let owned_top_level_setting_names = $resolved_owned_settings.owned_top_level_setting_names
    let base_config = (
        strip_yazelix_owned_top_level_settings
            $extracted_blocks.config_without_semantic_blocks
            $owned_top_level_setting_names
    )
    let merged_keybinds_block = (build_merged_keybinds_block $extracted_blocks.keybind_lines $yazelix_override_keybinds)
    let merged_ui_block = (build_yazelix_ui_block $extracted_blocks.ui_lines $resolved_owned_settings.rounded_value)
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
        $merged_keybinds_block,
        "",
        (build_yazelix_plugins_block
            $extracted_blocks.plugin_lines
            $PANE_ORCHESTRATOR_PLUGIN_ALIAS
            $pane_orchestrator_wasm_path
            $widget_tray_segment
            $custom_text_segment
            $render_plan.sidebar_width_percent
            $yazelix_dir
            $render_plan.popup_width_percent
            $render_plan.popup_height_percent
        ),
        "",
        $merged_ui_block,
        "",
        (render_yazelix_top_level_settings_block "// === YAZELIX DYNAMIC SETTINGS (from yazelix.toml) ===" $dynamic_top_level_settings),
        "",
        (render_yazelix_top_level_settings_block "// === YAZELIX ENFORCED SETTINGS ===" $enforced_top_level_settings),
        "",
        "// === YAZELIX BACKGROUND PLUGINS ===",
        (build_yazelix_load_plugins_block $extracted_blocks.load_plugin_lines $PANE_ORCHESTRATOR_PLUGIN_ALIAS)
    ] | str join "\n"
    
    try {
        write_text_atomic $merged_config_path $merged_config --raw | ignore
        record_generation_fingerprint $merged_config_dir $generation_fingerprint
        print $"✅ Zellij configuration generated successfully!"
        print $"   📁 Config saved to: ($merged_config_path)"
        print "   🔄 Config will auto-regenerate when source files change"
    } catch {|err|
        print $"❌ Failed to write merged config: ($err.msg)"
        exit 1
    }
    
    $merged_config_path
}

# Export main function for external use
export def main [yazelix_dir: string] {
    generate_merged_zellij_config $yazelix_dir | ignore
}
