#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
# Nushell version of the Yazelix launcher

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/nix_detector.nu ensure_nix_available
use ../utils/terminal_configs.nu generate_all_terminal_configs

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

    # Helper to resolve which config file to use for a terminal
    def resolve_config [term: string] {
        let home = $env.HOME
        # Yazelix-generated configs now live in XDG state dir
        let yz = match $term {
            "wezterm" => $"($home)/.local/share/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua",
            "ghostty" => $"($home)/.local/share/yazelix/configs/terminal_emulators/ghostty/config",
            "kitty" => $"($home)/.local/share/yazelix/configs/terminal_emulators/kitty/kitty.conf",
            "alacritty" => $"($home)/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty.toml",
            "foot" => $"($home)/.local/share/yazelix/configs/terminal_emulators/foot/foot.ini"
            _ => null
        }
        let user = match $term {
            "wezterm" => (if ($"($home)/.wezterm.lua" | path exists) { $"($home)/.wezterm.lua" } else { $"($home)/.config/wezterm/wezterm.lua" }),
            "ghostty" => $"($home)/.config/ghostty/config",
            "kitty" => $"($home)/.config/kitty/kitty.conf",
            "alacritty" => $"($home)/.config/alacritty/alacritty.toml",
            "foot" => $"($home)/.config/foot/foot.ini"
            _ => null
        }
        let mode = $terminal_config_mode
        if $mode == "yazelix" {
            $yz
        } else if $mode == "user" {
            if ($user | path exists) { $user } else { $yz }
        } else {
            # auto
            if ($user | path exists) { $user } else { $yz }
        }
    }

    # Prefer wrappers when available (they handle nixGL and respect config mode)
    let prefer_wrappers = true

    # Check for yazelix included terminals first only if preferred
    let terminal_info = if $prefer_wrappers and ($preferred_terminal == "kitty") and ((which yazelix-kitty | length) > 0) {
        print "Using Yazelix - Kitty (with nixGL acceleration)"
        {
            terminal: "yazelix-kitty"
            config: null # Config is handled internally by the wrapper
        }
    } else if $prefer_wrappers and ($preferred_terminal == "wezterm") and ((which yazelix-wezterm | length) > 0) {
        print "Using Yazelix - WezTerm (with nixGL acceleration)"
        {
            terminal: "yazelix-wezterm"
            config: null # Config is handled internally by the wrapper
        }
    } else if $prefer_wrappers and ($preferred_terminal == "alacritty") and ((which yazelix-alacritty | length) > 0) {
        print "Using Yazelix - Alacritty (with nixGL acceleration)"
        {
            terminal: "yazelix-alacritty"
            config: null # Config is handled internally by the wrapper
        }
    } else if $prefer_wrappers and (which yazelix-ghostty | length) > 0 {
        print "Using Yazelix - Ghostty (with nixGL acceleration)"
        {
            terminal: "yazelix-ghostty"
            config: null # Config is handled internally by the wrapper
        }
    } else if $prefer_wrappers and (which yazelix-kitty | length) > 0 {
        print "Using Yazelix - Kitty (with nixGL acceleration)"
        {
            terminal: "yazelix-kitty"
            config: null # Config is handled internally by the wrapper
        }
    } else if $prefer_wrappers and (which yazelix-wezterm | length) > 0 {
        print "Using Yazelix - WezTerm (with nixGL acceleration)"
        {
            terminal: "yazelix-wezterm"
            config: null # Config is handled internally by the wrapper
        }
    } else if $prefer_wrappers and (which yazelix-alacritty | length) > 0 {
        print "Using Yazelix - Alacritty (with nixGL acceleration)"
        {
            terminal: "yazelix-alacritty"
            config: null # Config is handled internally by the wrapper
        }
    } else if $prefer_wrappers and (which yazelix-foot | length) > 0 {
        print "Using Yazelix - Foot (with nixGl acceleration)"
        {
            terminal: "yazelix-foot",
            config: null # Config is handled internally by the wrapper
        }
    } else if ($preferred_terminal == "wezterm") and ((which wezterm | length) > 0) {
        print $"Using terminal: wezterm \(preferred: ($preferred_terminal)\)"
        {
            terminal: "wezterm"
            config: (resolve_config wezterm)
        }
    } else if ($preferred_terminal == "ghostty") and ((which ghostty | length) > 0) {
        print $"Using terminal: ghostty \(preferred: ($preferred_terminal)\)"
        {
            terminal: "ghostty"
            config: (resolve_config ghostty)
        }
    } else if ($preferred_terminal == "kitty") and ((which kitty | length) > 0) {
        print $"Using terminal: kitty \(preferred: ($preferred_terminal)\)"
        {
            terminal: "kitty"
            config: (resolve_config kitty)
        }
    } else if ($preferred_terminal == "foot") and ((which foot | length) > 0) {
        print $"Using terminal: foot \(preferred: ($preferred_terminal)\)"
        {
            terminal: "foot",
            config: (resolve_config foot)
        }
    } else if ($preferred_terminal == "alacritty") and ((which alacritty | length) > 0) {
        print $"Using terminal: alacritty \(preferred: ($preferred_terminal)\)"
        {
            terminal: "alacritty"
            config: (resolve_config alacritty)
        }
    } else if ($preferred_terminal == "wezterm") and ((which wezterm | length) > 0) {
        {
            terminal: "wezterm"
            config: (resolve_config wezterm)
        }
    } else if ($preferred_terminal == "ghostty") and ((which ghostty | length) > 0) {
        {
            terminal: "ghostty"
            config: (resolve_config ghostty)
        }
    } else if ($preferred_terminal == "kitty") and ((which kitty | length) > 0) {
        {
            terminal: "kitty"
            config: (resolve_config kitty)
        }
    } else if ($preferred_terminal == "foot") and ((which foot | length) > 0) {
        {
            terminal: "foot",
            config: (resolve_config foot)
        }
    } else if ($preferred_terminal == "alacritty") and ((which alacritty | length) > 0) {
        {
            terminal: "alacritty"
            config: (resolve_config alacritty)
        }
    } else if (which wezterm | length) > 0 {
        print $"Using terminal: wezterm \(preferred: ($preferred_terminal)\)"
        {
            terminal: "wezterm"
            config: (resolve_config wezterm)
        }
    } else if (which ghostty | length) > 0 {
        print $"Using terminal: ghostty \(preferred: ($preferred_terminal)\)"
        {
            terminal: "ghostty"
            config: (resolve_config ghostty)
        }
    } else if (which kitty | length) > 0 {
        print $"Using terminal: kitty \(preferred: ($preferred_terminal)\)"
        {
            terminal: "kitty"
            config: (resolve_config kitty)
        }
    } else if (which alacritty | length) > 0 {
        print $"Using terminal: alacritty \(preferred: ($preferred_terminal)\)"
        {
            terminal: "alacritty"
            config: (resolve_config alacritty)
        }
    } else if (which foot | length) > 0 {
        print $"Using terminal: foot \(preferred: ($preferred_terminal)\)"
        {
            terminal: "foot",
            config: (resolve_config foot)
        }
    } else {
        print "Error: None of the supported terminals (WezTerm, Ghostty, Kitty, Alacritty, Foot) are installed. Please install one of these terminals to use Yazelix."
        print "  - WezTerm: https://wezfurlong.org/wezterm/"
        print "  - Ghostty: https://ghostty.org/"
        print "  - Kitty: https://sw.kovidgoyal.net/kitty/"
        print "  - Alacritty: https://alacritty.org/"
        print " - Foot: https://codeberg.org/dnkl/foot"
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
        with-env { YAZELIX_TERMINAL_CONFIG_MODE: $terminal_config_mode } {
            ^bash -c "nohup yazelix-ghostty >/dev/null 2>&1 &"
        }
    } else if $terminal == "yazelix-kitty" {
        print "Running: yazelix-kitty (with nixGL auto-detection)"
        with-env { YAZELIX_TERMINAL_CONFIG_MODE: $terminal_config_mode } {
            ^bash -c "nohup yazelix-kitty >/dev/null 2>&1 &"
        }
    } else if $terminal == "yazelix-wezterm" {
        print "Running: yazelix-wezterm (with nixGL auto-detection)"
        with-env { YAZELIX_TERMINAL_CONFIG_MODE: $terminal_config_mode } {
            ^bash -c "nohup yazelix-wezterm >/dev/null 2>&1 &"
        }
    } else if $terminal == "yazelix-alacritty" {
        print "Running: yazelix-alacritty (with nixGL auto-detection)"
        with-env { YAZELIX_TERMINAL_CONFIG_MODE: $terminal_config_mode } {
            ^bash -c "nohup yazelix-alacritty >/dev/null 2>&1 &"
        }
    } else if $terminal == "yazelix-foot" {
        print "Running: yazelix-foot (with nixGL auto-detection)"
        with-env { YAZELIX_TERMINAL_CONFIG_MODE: $terminal_config_mode } {
            ^bash -c "nohup yazelix-foot >/dev/null 2>&1 &"
        }
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
    } else if $terminal == "foot" {
        print ("Running: foot --config " + $terminal_config + " --app-id com.yazelix.Yazelix")
        ^bash -c $"nohup foot --config ($terminal_config) --app-id com.yazelix.Yazelix >/dev/null 2>&1 &"
    }
}

# Export the main function so it can be called
export def launch_yazelix [] {
    main
}
