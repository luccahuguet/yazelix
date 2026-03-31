#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
# Nushell version of the Yazelix launcher

use ../utils/config_state.nu compute_config_state
use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../utils/failure_classes.nu [format_failure_classification]
use ../utils/nix_detector.nu ensure_nix_available
use ../utils/terminal_configs.nu generate_all_terminal_configs
use ../utils/terminal_launcher.nu *
use ../utils/constants.nu [SUPPORTED_TERMINALS, TERMINAL_METADATA]
use ../utils/common.nu [get_yazelix_runtime_dir]

def validate_launch_working_dir [working_dir: string] {
    let resolved = ($working_dir | path expand)

    if not ($resolved | path exists) {
        error make {msg: $"Launch directory does not exist: ($resolved)\nUse an existing directory, or use --home to start from HOME."}
    }

    if (($resolved | path type) != "dir") {
        error make {msg: $"Launch path is not a directory: ($resolved)\nPass a directory to yzx launch --path."}
    }

    $resolved
}

def run_detached_terminal_launch [launch_cmd: string, terminal_name: string, --verbose] {
    if (which bash | is-empty) {
        let classification = (format_failure_classification "host-dependency" "Install bash or fix PATH, then retry the launch.")
        error make {msg: $"Cannot launch ($terminal_name): bash is not available in PATH.\nYazelix uses bash to detach new terminal windows.\n($classification)"}
    }

    let output = (^bash -c $launch_cmd | complete)
    if $output.exit_code != 0 {
        let stderr_tail = (
            $output.stderr
            | default ""
            | lines
            | last 5
            | str join "\n"
            | str trim
        )
        let details = if ($stderr_tail | is-empty) {
            "No stderr output was captured."
        } else {
            $stderr_tail
        }

        error make {msg: $"Failed to launch ($terminal_name) \(exit code: ($output.exit_code)\)\n($details)"}
    }

    if $verbose {
        print $"✅ Launch request sent to ($terminal_name)"
    }
}

def main [
    launch_cwd?: string
    --terminal(-t): string  # Override terminal selection (for sweep testing)
    --verbose               # Enable verbose logging
] {
    # Check if Nix is properly installed before proceeding
    ensure_nix_available
    run_entrypoint_config_migration_preflight "Yazelix launch" --allow-noninteractive | ignore

    # Resolve HOME using shell expansion
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    let verbose_mode = $verbose
    let requested_terminal = ($terminal | default "")
    if $verbose_mode {
        print "🔍 launch_yazelix: verbose mode enabled"
        print $"Resolved HOME=($home)"
    }

    # Compute config state (auto-creates yazelix.toml if missing)
    let config_state = compute_config_state
    let config = $config_state.config
    let active_config_file = $config_state.config_file
    let current_hash = $config_state.combined_hash
    let cached_hash = $config_state.cached_hash
    let needs_reload = $config_state.needs_refresh
    let legacy_nix_config = $"($home)/.config/yazelix/yazelix.nix"
    if ($legacy_nix_config | path exists) and ($legacy_nix_config != $active_config_file) {
        print ""
        print "⚠️  Detected legacy config: ~/.config/yazelix/yazelix.nix"
        print "   Yazelix now reads settings from ~/.config/yazelix/user_configs/yazelix.toml."
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
    let requested_working_dir = if ($launch_cwd | is-empty) { pwd } else { $launch_cwd }
    let working_dir = (validate_launch_working_dir $requested_working_dir)
    if $verbose_mode {
        print $"Launch directory: ($working_dir)"
    }

    let terminal_config_mode = $config.terminal_config_mode
    let manage_terminals = ($config.manage_terminals? | default true)
    mut terminals = ($config.terminals? | default ["ghostty"] | uniq)
    if ($terminals | is-empty) {
        if $manage_terminals {
            let available = (
                $SUPPORTED_TERMINALS
                | where {|t|
                    let meta = ($TERMINAL_METADATA | get -o $t | default {})
                    (which $meta.wrapper | is-not-empty) or (which $t | is-not-empty)
                }
            )
            let available_str = if ($available | is-empty) {
                "none detected"
            } else {
                $available | str join ", "
            }
            print "Error: terminal.terminals must include at least one terminal"
            print $"Detected terminals: ($available_str)"
            print "Set [terminal].terminals in ~/.config/yazelix/user_configs/yazelix.toml"
            exit 1
        } else {
            $terminals = $SUPPORTED_TERMINALS
        }
    }

    # Generate all terminal configurations for safety and consistency
    generate_all_terminal_configs

    # Detect available terminal (wrappers preferred)
    # If terminal was explicitly specified via --terminal flag, force that specific terminal only
    let terminal_info = if ($requested_terminal | is-not-empty) {
        # Strict mode: only try the specified terminal, no fallbacks
        let specified_terminal = $requested_terminal  # Use the --terminal flag value
        let term_meta = ($TERMINAL_METADATA | get -o $specified_terminal)
        if $term_meta == null {
            print $"Error: Unsupported terminal '($specified_terminal)'"
            print $"Supported terminals: ($SUPPORTED_TERMINALS | str join ', ')"
            exit 1
        }
        let wrapper_cmd = $term_meta.wrapper

        # Prefer the direct terminal binary first so source-tree launches do not
        # depend on stale built wrapper scripts. Fall back to the wrapper when
        # the direct binary is not available.
        if (command_exists $specified_terminal) {
            {
                terminal: $specified_terminal
                name: $term_meta.name
                command: $specified_terminal
                use_wrapper: false
            }
        } else if (command_exists $wrapper_cmd) {
            {
                terminal: $specified_terminal
                name: $term_meta.name
                command: $wrapper_cmd
                use_wrapper: true
            }
        } else {
            print $"Error: Specified terminal '($specified_terminal)' is not installed"
            print "Please install it or choose a different terminal for testing"
            exit 1
        }
    } else {
        # Normal mode: use detect_terminal with fallbacks
        detect_terminal $terminals true
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
    let launch_cmd = build_launch_command $terminal_info $terminal_config $working_dir $needs_reload

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
    # Preserve sweep/test env vars when present so the launched session can select
    # the test layout and write verification results.
    let runtime_dir = (get_yazelix_runtime_dir)
    mut propagated_env = {
        YAZELIX_TERMINAL: $terminal_info.terminal
        YAZELIX_RUNTIME_DIR: $runtime_dir
        YAZELIX_DIR: $runtime_dir
    }
    if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) {
        $propagated_env = ($propagated_env | upsert YAZELIX_SWEEP_TEST_ID $env.YAZELIX_SWEEP_TEST_ID)
    }
    if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
        $propagated_env = ($propagated_env | upsert YAZELIX_LAYOUT_OVERRIDE $env.YAZELIX_LAYOUT_OVERRIDE)
    }
    if $terminal_info.use_wrapper {
        let env_block = ($propagated_env | upsert YAZELIX_TERMINAL_CONFIG_MODE $terminal_config_mode)
        if $verbose_mode {
            print $"Launching wrapper command: ($launch_cmd)"
        }
        with-env $env_block {
            run_detached_terminal_launch $launch_cmd $display_name --verbose=$verbose_mode
        }
    } else {
        if $verbose_mode {
            print $"Launching command: ($launch_cmd)"
        }
        with-env $propagated_env {
            run_detached_terminal_launch $launch_cmd $display_name --verbose=$verbose_mode
        }
    }
}
