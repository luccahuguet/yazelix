#!/usr/bin/env nu
# Modular terminal configuration generator for yazelix

use config_parser.nu parse_yazelix_config
use ./atomic_writes.nu write_text_atomic
use ./constants.nu *
use ./common.nu get_yazelix_runtime_dir
use ./terminal_ghostty_assets.nu sync_generated_ghostty_shader_assets
use ./terminal_renderers.nu [
    generate_ghostty_config
    generate_wezterm_config
    generate_kitty_config
    generate_alacritty_base_config
    generate_alacritty_config
    generate_foot_config
]

def write_generated_terminal_config [file_path: string, content: string] {
    write_text_atomic $file_path $content --raw | ignore
}

export def generate_selected_terminal_configs [selected_terminals: list<string>, runtime_dir?: string] {
    let config = parse_yazelix_config
    let resolved_runtime_dir = (($runtime_dir | default (get_yazelix_runtime_dir)) | path expand)
    let terminals = ($selected_terminals | where {|terminal| $terminal in $SUPPORTED_TERMINALS} | uniq)
    if ($terminals | is-empty) {
        return
    }

    let should_generate_ghostty = ($terminals | any {|t| $t == "ghostty" })
    let should_generate_foot = ($terminals | any {|t| $t == "foot" })
    let should_generate_wezterm = ($terminals | any {|t| $t == "wezterm" })
    let should_generate_kitty = ($terminals | any {|t| $t == "kitty" })
    let generated_dir = ($YAZELIX_GENERATED_CONFIGS_DIR | str replace "~" $env.HOME)
    let configs_dir = ($generated_dir | path join "terminal_emulators")

    print "Generating bundled terminal configurations..."

    # Ghostty (optional)
    if $should_generate_ghostty {
        let ghostty_dir = ($configs_dir | path join "ghostty")
        mkdir $ghostty_dir
        write_generated_terminal_config ($ghostty_dir | path join "config") (generate_ghostty_config)
        let glow_level = ($config.ghostty_trail_glow? | default "medium")
        sync_generated_ghostty_shader_assets $resolved_runtime_dir $ghostty_dir $glow_level
    }

    # Alacritty (conditional)
    if ($terminals | any {|t| $t == "alacritty" }) {
        let alacritty_dir = ($configs_dir | path join "alacritty")
        mkdir $alacritty_dir
        write_generated_terminal_config ($alacritty_dir | path join "alacritty_base.toml") (generate_alacritty_base_config)
        write_generated_terminal_config ($alacritty_dir | path join "alacritty.toml") (generate_alacritty_config)
    }

    mut generated = []
    if $should_generate_ghostty { $generated = ($generated | append "Ghostty") }
    if ($terminals | any {|t| $t == "alacritty" }) { $generated = ($generated | append "Alacritty") }

    # WezTerm (conditional)
    if $should_generate_wezterm {
        let wezterm_dir = ($configs_dir | path join "wezterm")
        mkdir $wezterm_dir
        write_generated_terminal_config ($wezterm_dir | path join ".wezterm.lua") (generate_wezterm_config)
        $generated = ($generated | append "WezTerm")
    }

    # Kitty (conditional)
    if $should_generate_kitty {
        let kitty_dir = ($configs_dir | path join "kitty")
        mkdir $kitty_dir
        write_generated_terminal_config ($kitty_dir | path join "kitty.conf") (generate_kitty_config)
        $generated = ($generated | append "Kitty")
    }

    # Foot (conditional)
    if $should_generate_foot {
        let foot_dir = ($configs_dir | path join "foot")
        mkdir $foot_dir
        write_generated_terminal_config ($foot_dir | path join "foot.ini") (generate_foot_config)
        $generated = ($generated | append "Foot")
    }

    let generated_list = ($generated | str join ", ")
    print $"✓ Generated terminal configurations ($generated_list)"
    print "📋 Static example configs for other terminals in configs/terminal_emulators/"
}

export def generate_all_terminal_configs [runtime_dir?: string] {
    let config = parse_yazelix_config
    mut terminals = ($config.terminals? | default [$DEFAULT_TERMINAL])
    if ($terminals | is-empty) {
        error make {msg: "terminal.terminals must include at least one terminal"}
    }

    generate_selected_terminal_configs $terminals $runtime_dir
}
