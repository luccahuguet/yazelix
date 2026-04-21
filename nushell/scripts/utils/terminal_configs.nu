#!/usr/bin/env nu
# Thin wrapper for Yazelix terminal configuration generation.
# The real owner is terminal-materialization.generate in yzx_core.

use config_parser.nu [parse_yazelix_config run_yzx_core_json_command]
use config_surfaces.nu load_active_config_surface
use ./constants.nu *
use ./common.nu [get_yazelix_config_dir get_yazelix_runtime_dir]

def ghostty_cursor_random_requested [config: record] {
    [
        ($config.ghostty_trail_color? | default "")
        ($config.ghostty_trail_effect? | default "")
        ($config.ghostty_mode_effect? | default "")
    ] | any {|value| ($value | into string | str trim) == "random" }
}

export def generate_selected_terminal_configs [selected_terminals: list<string>, runtime_dir?: string] {
    let resolved_runtime_dir = (($runtime_dir | default (get_yazelix_runtime_dir)) | path expand)
    let config_surface = (load_active_config_surface)
    let config_dir = (get_yazelix_config_dir)
    let state_dir = ($YAZELIX_STATE_DIR | str replace "~" $env.HOME | path expand)
    let config = parse_yazelix_config
    let terminals = ($selected_terminals | where {|terminal| $terminal in $SUPPORTED_TERMINALS} | uniq)
    if ($terminals | is-empty) {
        return
    }

    print "Generating bundled terminal configurations..."

    run_yzx_core_json_command $resolved_runtime_dir {display_config_path: "" config_file: ""} [
        "terminal-materialization.generate"
        "--config" $config_surface.config_file
        "--default-config" $config_surface.default_config_path
        "--contract" ($resolved_runtime_dir | path join "config_metadata" "main_config_contract.toml")
        "--runtime-dir" $resolved_runtime_dir
        "--state-dir" $state_dir
        "--terminals-json" ($terminals | to json -r)
    ] "Yazelix Rust terminal-materialization helper returned invalid JSON." | ignore

    let generated = ($terminals | each {|t| ($TERMINAL_METADATA | get -o $t | default {} | get -o name | default $t) })
    let generated_list = ($generated | str join ", ")
    print $"✓ Generated terminal configurations ($generated_list)"
    print "📋 Static example configs for other terminals in configs/terminal_emulators/"
}

export def generate_all_terminal_configs [runtime_dir?: string] {
    let config = parse_yazelix_config
    let terminals = ($config.terminals? | default [$DEFAULT_TERMINAL])
    if ($terminals | is-empty) {
        error make {msg: "terminal.terminals must include at least one terminal"}
    }

    generate_selected_terminal_configs $terminals $runtime_dir
}

export def reroll_ghostty_random_cursor_config_for_launch [
    config: record
    runtime_dir?: string
    --quiet
] {
    if not (ghostty_cursor_random_requested $config) {
        return false
    }

    let resolved_runtime_dir = (($runtime_dir | default (get_yazelix_runtime_dir)) | path expand)
    let config_dir = (get_yazelix_config_dir)
    let state_dir = ($YAZELIX_STATE_DIR | str replace "~" $env.HOME | path expand)

    if not $quiet {
        print "🎲 Rerolling Ghostty random cursor settings for this Yazelix window..."
    }

    run_yzx_core_json_command $resolved_runtime_dir {display_config_path: "" config_file: ""} [
        "ghostty-materialization.generate"
        "--runtime-dir" $resolved_runtime_dir
        "--config-dir" $config_dir
        "--state-dir" $state_dir
        "--transparency" ($config.transparency? | default "none")
        "--ghostty-trail-glow" ($config.ghostty_trail_glow? | default "medium")
        "--ghostty-trail-color" ($config.ghostty_trail_color? | default "")
        "--ghostty-trail-effect" ($config.ghostty_trail_effect? | default "")
        "--ghostty-mode-effect" ($config.ghostty_mode_effect? | default "")
    ] "Yazelix Rust ghostty-materialization helper returned invalid JSON." | ignore

    if not $quiet {
        print "✓ Rerolled Ghostty cursor settings"
    }

    true
}
