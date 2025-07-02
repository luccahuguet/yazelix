#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/launch-yazelix.nu
# Nushell version of the Yazelix launcher

def main [] {
    # Resolve HOME using shell expansion
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    print $"Resolved HOME=($home)"

    # Read preference from environment (set by Nix shellHook)
    let preferred_terminal = ($env.YAZELIX_PREFERRED_TERMINAL? | default "wezterm")

    # Check if a supported terminal is installed
    let terminal_info = if ($preferred_terminal == "wezterm") and ((which wezterm | length) > 0) {
        {
            terminal: "wezterm"
            config: $"($home)/.config/yazelix/terminal_configs/wezterm/.wezterm.lua"
        }
    } else if ($preferred_terminal == "ghostty") and ((which ghostty | length) > 0) {
        {
            terminal: "ghostty"
            config: $"($home)/.config/yazelix/terminal_configs/ghostty/config"
        }
    } else if (which wezterm | length) > 0 {
        # Fallback to wezterm if preferred terminal not available
        {
            terminal: "wezterm"
            config: $"($home)/.config/yazelix/terminal_configs/wezterm/.wezterm.lua"
        }
    } else if (which ghostty | length) > 0 {
        # Fallback to ghostty if wezterm not available
        {
            terminal: "ghostty"
            config: $"($home)/.config/yazelix/terminal_configs/ghostty/config"
        }
    } else {
        print "Error: Neither Ghostty nor WezTerm is installed. Please install one of these terminals to use Yazelix."
        print "  - Ghostty: https://ghostty.org/"
        print "  - WezTerm: https://wezfurlong.org/wezterm/"
        exit 1
    }

    let terminal = $terminal_info.terminal
    let terminal_config = $terminal_info.config

    # Print which terminal is being used and the preferred terminal
    print ("Using terminal: " + $terminal + " (preferred: " + $preferred_terminal + ")")

    # Check if terminal config exists
    if not ($terminal_config | path exists) {
        print $"Error: ($terminal) config not found at ($terminal_config)"
        exit 1
    }

    # Launch terminal using bash to handle background processes properly
    if $terminal == "ghostty" {
        print ("Running: ghostty --config " + $terminal_config)
        ^bash -c $"nohup ghostty --config ($terminal_config) >/dev/null 2>&1 &"
    } else if $terminal == "wezterm" {
        print ("Running: wezterm --config-file " + $terminal_config + " start")
        ^bash -c $"nohup wezterm --config-file ($terminal_config) start >/dev/null 2>&1 &"
    }
}

# Export the main function so it can be called
export def launch_yazelix [] {
    main
} 