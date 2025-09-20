#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
# Nushell version of the Yazelix launcher

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/nix_detector.nu ensure_nix_available

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

    # Check for yazelix included terminals first, prioritizing preferred terminal
    let terminal_info = if ($preferred_terminal == "kitty") and ((which yazelix-kitty | length) > 0) {
        print "Using Yazelix - Kitty (with nixGL acceleration)"
        {
            terminal: "yazelix-kitty"
            config: null # Config is handled internally by the wrapper
        }
    } else if ($preferred_terminal == "wezterm") and ((which yazelix-wezterm | length) > 0) {
        print "Using Yazelix - WezTerm (with nixGL acceleration)"
        {
            terminal: "yazelix-wezterm"
            config: null # Config is handled internally by the wrapper
        }
    } else if ($preferred_terminal == "alacritty") and ((which yazelix-alacritty | length) > 0) {
        print "Using Yazelix - Alacritty (with nixGL acceleration)"
        {
            terminal: "yazelix-alacritty"
            config: null # Config is handled internally by the wrapper
        }
    } else if (which yazelix-ghostty | length) > 0 {
        print "Using Yazelix - Ghostty (with nixGL acceleration)"
        {
            terminal: "yazelix-ghostty"
            config: null # Config is handled internally by the wrapper
        }
    } else if (which yazelix-kitty | length) > 0 {
        print "Using Yazelix - Kitty (with nixGL acceleration)"
        {
            terminal: "yazelix-kitty"
            config: null # Config is handled internally by the wrapper
        }
    } else if (which yazelix-wezterm | length) > 0 {
        print "Using Yazelix - WezTerm (with nixGL acceleration)"
        {
            terminal: "yazelix-wezterm"
            config: null # Config is handled internally by the wrapper
        }
    } else if (which yazelix-alacritty | length) > 0 {
        print "Using Yazelix - Alacritty (with nixGL acceleration)"
        {
            terminal: "yazelix-alacritty"
            config: null # Config is handled internally by the wrapper
        }
    } else if ($preferred_terminal == "wezterm") and ((which wezterm | length) > 0) {
        {
            terminal: "wezterm"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua"
        }
    } else if ($preferred_terminal == "ghostty") and ((which ghostty | length) > 0) {
        {
            terminal: "ghostty"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/ghostty/config"
        }
    } else if ($preferred_terminal == "kitty") and ((which kitty | length) > 0) {
        {
            terminal: "kitty"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/kitty/kitty.conf"
        }
    } else if ($preferred_terminal == "alacritty") and ((which alacritty | length) > 0) {
        {
            terminal: "alacritty"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/alacritty/alacritty.toml"
        }
    } else if (which ghostty | length) > 0 {
        # Fallback to ghostty if preferred terminal not available
        {
            terminal: "ghostty"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/ghostty/config"
        }
    } else if (which wezterm | length) > 0 {
        # Fallback to wezterm if ghostty not available
        {
            terminal: "wezterm"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua"
        }
    } else if (which kitty | length) > 0 {
        # Fallback to kitty if ghostty and wezterm not available
        {
            terminal: "kitty"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/kitty/kitty.conf"
        }
    } else if (which alacritty | length) > 0 {
        # Fallback to alacritty if other terminals not available
        {
            terminal: "alacritty"
            config: $"($home)/.config/yazelix/configs/terminal_emulators/alacritty/alacritty.toml"
        }
    } else {
        print "Error: None of the supported terminals (WezTerm, Ghostty, Kitty, Alacritty) are installed. Please install one of these terminals to use Yazelix."
        print "  - WezTerm: https://wezfurlong.org/wezterm/"
        print "  - Ghostty: https://ghostty.org/"
        print "  - Kitty: https://sw.kovidgoyal.net/kitty/"
        print "  - Alacritty: https://alacritty.org/"
        exit 1
    }

    let terminal = $terminal_info.terminal
    let terminal_config = $terminal_info.config

    # Print which terminal is being used and the preferred terminal
    print ("Using terminal: " + $terminal + " (preferred: " + $preferred_terminal + ")")

    # Check if terminal config exists (skip for yazelix-ghostty which handles config internally)
    if ($terminal_config != null) and (not ($terminal_config | path exists)) {
        print $"Error: ($terminal) config not found at ($terminal_config)"
        exit 1
    }

    # Launch terminal using bash to handle background processes properly
    if $terminal == "yazelix-ghostty" {
        print "Running: yazelix-ghostty (with nixGL auto-detection)"
        ^bash -c "nohup yazelix-ghostty >/dev/null 2>&1 &"
    } else if $terminal == "yazelix-kitty" {
        print "Running: yazelix-kitty (with nixGL auto-detection)"
        ^bash -c "nohup yazelix-kitty >/dev/null 2>&1 &"
    } else if $terminal == "yazelix-wezterm" {
        print "Running: yazelix-wezterm (with nixGL auto-detection)"
        ^bash -c "nohup yazelix-wezterm >/dev/null 2>&1 &"
    } else if $terminal == "yazelix-alacritty" {
        print "Running: yazelix-alacritty (with nixGL auto-detection)"
        ^bash -c "nohup yazelix-alacritty >/dev/null 2>&1 &"
    } else if $terminal == "ghostty" {
        print ("Running: ghostty --config-file=" + $terminal_config)
        ^bash -c $"nohup ghostty --config-file=($terminal_config) >/dev/null 2>&1 &"
    } else if $terminal == "wezterm" {
        print ("Running: wezterm --config-file " + $terminal_config + " start --class=com.yazelix.Yazelix")
        ^bash -c $"nohup wezterm --config-file ($terminal_config) start --class=com.yazelix.Yazelix >/dev/null 2>&1 &"
    } else if $terminal == "kitty" {
        print ("Running: kitty --config=" + $terminal_config + " --class=com.yazelix.Yazelix")
        ^bash -c $"nohup kitty --config=($terminal_config) --class=com.yazelix.Yazelix >/dev/null 2>&1 &"
    } else if $terminal == "alacritty" {
        print ("Running: alacritty --config-file=" + $terminal_config)
        ^bash -c $"nohup alacritty --config-file ($terminal_config) >/dev/null 2>&1 &"
    }
}

# Export the main function so it can be called
export def launch_yazelix [] {
    main
}