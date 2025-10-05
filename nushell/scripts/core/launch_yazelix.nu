#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
# Nushell version of the Yazelix launcher

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/nix_detector.nu ensure_nix_available
use ../utils/terminal_configs.nu generate_all_terminal_configs
use ../utils/terminal_launcher.nu *

def main [] {
    # Check if Nix is properly installed before proceeding
    ensure_nix_available

    # Resolve HOME using shell expansion
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    print $"Resolved HOME=($home)"

    # Always read preference directly from config file to avoid stale environment variables
    let config = parse_yazelix_config
    let preferred_terminal = $config.preferred_terminal
    let terminal_config_mode = $config.terminal_config_mode

    # Generate all terminal configurations for safety and consistency
    generate_all_terminal_configs

    # Detect available terminal (wrappers preferred)
    let terminal_info = detect_terminal $preferred_terminal true

    if $terminal_info == null {
        print "Error: None of the supported terminals (WezTerm, Ghostty, Kitty, Alacritty, Foot) are installed. Please install one of these terminals to use Yazelix."
        print "  - WezTerm: https://wezfurlong.org/wezterm/"
        print "  - Ghostty: https://ghostty.org/"
        print "  - Kitty: https://sw.kovidgoyal.net/kitty/"
        print "  - Alacritty: https://alacritty.org/"
        print " - Foot: https://codeberg.org/dnkl/foot"
        exit 1
    }

    # Get display name and print
    let display_name = get_terminal_display_name $terminal_info
    print $"Using terminal: ($display_name)"

    # Resolve config path (skip for wrappers which handle internally)
    let terminal_config = if $terminal_info.use_wrapper {
        null
    } else {
        resolve_terminal_config $terminal_info.terminal $terminal_config_mode
    }

    # Check if terminal config exists (skip for wrappers)
    if ($terminal_config != null) and (not ($terminal_config | path exists)) {
        print $"Error: ($terminal_info.name) config not found at ($terminal_config)"
        exit 1
    }

    # Build launch command
    let launch_cmd = build_launch_command $terminal_info $terminal_config $terminal_config_mode

    # Print what we're running
    let terminal = $terminal_info.terminal
    if $terminal_info.use_wrapper {
        print $"Running: ($terminal_info.command) \(with nixGL auto-detection\)"
    } else {
        if $terminal == "wezterm" {
            print $"Running: wezterm --config-file ($terminal_config) start --class=com.yazelix.Yazelix"
        } else if $terminal == "ghostty" {
            print $"Running: ghostty --config-file=($terminal_config)"
        } else if $terminal == "kitty" {
            print $"Running: kitty --config=($terminal_config) --class=com.yazelix.Yazelix"
        } else if $terminal == "alacritty" {
            print $"Running: alacritty --config-file=($terminal_config)"
        } else if $terminal == "foot" {
            print $"Running: foot --config ($terminal_config) --app-id com.yazelix.Yazelix"
        }
    }

    # Launch terminal using bash to handle background processes properly
    if $terminal_info.use_wrapper {
        with-env { YAZELIX_TERMINAL_CONFIG_MODE: $terminal_config_mode } {
            ^bash -c $launch_cmd
        }
    } else {
        ^bash -c $launch_cmd
    }
}

# Export the main function so it can be called
export def launch_yazelix [] {
    main
}
