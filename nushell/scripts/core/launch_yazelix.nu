#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
# Nushell version of the Yazelix launcher

use ../utils/config_state.nu compute_config_state
use ../utils/nix_detector.nu ensure_nix_available
use ../utils/terminal_configs.nu generate_all_terminal_configs
use ../utils/terminal_launcher.nu *
use ../utils/constants.nu [TERMINAL_METADATA]

def main [
    launch_cwd?: string
    --terminal(-t): string  # Override terminal selection (for sweep testing)
    --verbose               # Enable verbose logging
] {
    # Check if Nix is properly installed before proceeding
    ensure_nix_available

    # Resolve HOME using shell expansion
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")
    if $verbose_mode {
        print "🔍 launch_yazelix: verbose mode enabled"
        print $"Resolved HOME=($home)"
    }

    # Compute config state (auto-creates yazelix.toml if missing)
    let config_state = compute_config_state
    let config = $config_state.config
    let active_config_file = $config_state.config_file
    let current_hash = $config_state.current_hash
    let cached_hash = $config_state.cached_hash
    let needs_reload = $config_state.needs_refresh

    let legacy_nix_config = $"($home)/.config/yazelix/yazelix.nix"
    if ($legacy_nix_config | path exists) and ($legacy_nix_config != $active_config_file) {
        print ""
        print "⚠️  Detected legacy config: ~/.config/yazelix/yazelix.nix"
        print "   Yazelix now reads settings from ~/.config/yazelix/yazelix.toml."
        print "   Copy your custom options into the TOML file (see docs/customization.md) and remove the old file once migrated."
        print ""
    }

    if $verbose_mode {
        print $"🔍 Config hash check:"
        print $"   Current:  ($current_hash)"
        print $"   Cached:   ($cached_hash)"
        print $"   Reload:   ($needs_reload)"
    }

    # Use provided launch directory or fall back to current directory
    let working_dir = if ($launch_cwd | is-empty) { pwd } else { $launch_cwd }
    if $verbose_mode {
        print $"Launch directory: ($working_dir)"
    }

    let terminal_config_mode = $config.terminal_config_mode

    # Use terminal override if provided, otherwise use config preference
    let preferred_terminal = if ($terminal | is-not-empty) {
        $terminal
    } else {
        $config.preferred_terminal
    }

    # Generate all terminal configurations for safety and consistency
    generate_all_terminal_configs

    # Detect available terminal (wrappers preferred)
    # If terminal was explicitly specified via --terminal flag, force that specific terminal only
    let terminal_info = if ($terminal | is-not-empty) {
        # Strict mode: only try the specified terminal, no fallbacks
        let specified_terminal = $terminal  # Use the --terminal flag value
        let term_meta = $TERMINAL_METADATA | get $specified_terminal
        let wrapper_cmd = $term_meta.wrapper

        # Try wrapper first, then direct
        if (command_exists $wrapper_cmd) {
            {
                terminal: $specified_terminal
                name: $term_meta.name
                command: $wrapper_cmd
                use_wrapper: true
            }
        } else if (command_exists $specified_terminal) {
            {
                terminal: $specified_terminal
                name: $term_meta.name
                command: $specified_terminal
                use_wrapper: false
            }
        } else {
            print $"Error: Specified terminal '($specified_terminal)' is not installed"
            print "Please install it or choose a different terminal for testing"
            exit 1
        }
    } else {
        # Normal mode: use detect_terminal with fallbacks
        detect_terminal $preferred_terminal true
    }

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
    if $verbose_mode {
        print $"Using terminal: ($display_name)"
    }

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

    # Build launch command (pass needs_reload to control env var clearing)
    let launch_cmd = build_launch_command $terminal_info $terminal_config $terminal_config_mode $needs_reload

    # Print what we're running
    let terminal = $terminal_info.terminal
    if $verbose_mode {
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
    }

    # Launch terminal using bash to handle background processes properly
    # Pass YAZELIX_TERMINAL so verification scripts know which terminal launched
    if $terminal_info.use_wrapper {
        mut env_block = {
            YAZELIX_TERMINAL_CONFIG_MODE: $terminal_config_mode,
            YAZELIX_LAUNCH_CWD: $working_dir,
            YAZELIX_TERMINAL: $terminal_info.terminal
        }
        if $verbose_mode {
            $env_block = ($env_block | upsert YAZELIX_VERBOSE "true")
            print $"Launching wrapper command: ($launch_cmd)"
        }
        with-env $env_block {
            ^bash -c $launch_cmd
        }
    } else {
        mut env_block = {
            YAZELIX_LAUNCH_CWD: $working_dir,
            YAZELIX_TERMINAL: $terminal_info.terminal
        }
        if $verbose_mode {
            $env_block = ($env_block | upsert YAZELIX_VERBOSE "true")
            print $"Launching command: ($launch_cmd)"
        }
        with-env $env_block {
            ^bash -c $launch_cmd
        }
    }
}
