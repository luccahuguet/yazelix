#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
# Nushell version of the Yazelix launcher

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/terminal_launcher.nu *
use ../utils/constants.nu [DEFAULT_TERMINAL SUPPORTED_TERMINALS, TERMINAL_METADATA]
use ../utils/common.nu [get_yazelix_runtime_dir normalize_path_entries require_yazelix_runtime_dir]
use ../utils/startup_profile.nu [profile_startup_step propagate_startup_profile_env]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface compute_config_state_via_yzx_core generate_ghostty_materialization_via_yzx_core generate_terminal_materialization_via_yzx_core run_yzx_core_request_json_command]

const STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND = "startup-launch-preflight.evaluate"

def ghostty_cursor_random_requested [config: record] {
    [
        ($config.ghostty_trail_color? | default "")
        ($config.ghostty_trail_effect? | default "")
        ($config.ghostty_mode_effect? | default "")
    ] | any {|value| ($value | into string | str trim) == "random" }
}

def get_launch_command_search_paths [] {
    normalize_path_entries ($env.PATH? | default [])
}

def normalize_selected_terminals [selected_terminals: list<string>] {
    $selected_terminals | where {|terminal| $terminal in $SUPPORTED_TERMINALS} | uniq
}

def materialize_selected_terminal_configs [
    selected_terminals: list<string>
    runtime_dir?: string
    config_file?: string
] {
    let resolved_runtime_dir = (($runtime_dir | default (get_yazelix_runtime_dir)) | path expand)
    let terminals = (normalize_selected_terminals $selected_terminals)
    if ($terminals | is-empty) {
        return
    }

    print "Generating bundled terminal configurations..."

    generate_terminal_materialization_via_yzx_core $terminals $resolved_runtime_dir $config_file | ignore

    let generated = ($terminals | each {|t| ($TERMINAL_METADATA | get -o $t | default {} | get -o name | default $t) })
    let generated_list = ($generated | str join ", ")
    print $"✓ Generated terminal configurations ($generated_list)"
    print "📋 Static example configs for other terminals in configs/terminal_emulators/"
}

# Compatibility wrapper used by tests and install validation.
export def generate_selected_terminal_configs [selected_terminals: list<string>, runtime_dir?: string] {
    materialize_selected_terminal_configs $selected_terminals $runtime_dir
}

# Compatibility wrapper used by tests and install validation.
export def generate_all_terminal_configs [runtime_dir?: string] {
    let config = parse_yazelix_config
    let terminals = ($config.terminals? | default [$DEFAULT_TERMINAL])
    if ($terminals | is-empty) {
        error make {msg: "terminal.terminals must include at least one terminal"}
    }

    materialize_selected_terminal_configs $terminals $runtime_dir
}

def reroll_ghostty_random_cursor_config_for_launch [
    config: record
    runtime_dir?: string
    config_file?: string
    --quiet
] {
    if not (ghostty_cursor_random_requested $config) {
        return false
    }

    let resolved_runtime_dir = (($runtime_dir | default (get_yazelix_runtime_dir)) | path expand)
    let resolved_config_file = ($config_file | default "")

    if not $quiet {
        print "🎲 Rerolling Ghostty random cursor settings for this Yazelix window..."
    }

    generate_ghostty_materialization_via_yzx_core $resolved_runtime_dir $resolved_config_file | ignore

    if not $quiet {
        print "✓ Rerolled Ghostty cursor settings"
    }

    true
}

def validate_launch_working_dir [working_dir: string] {
    (run_launch_preflight $working_dir "" [$DEFAULT_TERMINAL]).working_dir
}

def resolve_terminal_candidates [requested_terminal: string, terminals: list<string>] {
    (run_launch_preflight (pwd) $requested_terminal $terminals).terminal_candidates
}

def resolve_desktop_fast_path_candidates [requested_terminal: string, terminals: list<string>] {
    resolve_terminal_candidates $requested_terminal $terminals
}

def run_launch_preflight [working_dir: string, requested_terminal: string, terminals: list<string>] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let data = (run_yzx_core_request_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND
        {
            launch: {
                working_dir: ($working_dir | path expand)
                requested_terminal: $requested_terminal
                terminals: $terminals
                command_search_paths: (get_launch_command_search_paths)
            }
        }
        "Yazelix Rust startup-launch-preflight helper returned invalid JSON.")

    if ($data.kind? | default "") != "launch" {
        error make {msg: "Unexpected startup-launch-preflight response \(expected launch\)."}
    }

    {
        working_dir: $data.working_dir
        terminal_candidates: ($data.terminal_candidates? | default [])
    }
}

def ensure_terminal_configs_available_for_candidates [terminal_candidates: list<record>, terminal_config_mode: string, runtime_dir: string] {
    if $terminal_config_mode != "yazelix" {
        return
    }

    let candidate_terminals = ($terminal_candidates | each {|candidate| $candidate.terminal } | uniq)
    let needs_generation = (
        $candidate_terminals
        | any {|terminal|
            let config_path = (resolve_terminal_config $terminal $terminal_config_mode)
            not ($config_path | path exists)
        }
    )

    if $needs_generation {
        materialize_selected_terminal_configs $candidate_terminals $runtime_dir
    }
}

def reroll_ghostty_random_cursor_config_for_launch_candidates [
    terminal_candidates: list<record>
    terminal_config_mode: string
    runtime_dir: string
    config: record
    verbose_mode: bool
] {
    if $terminal_config_mode != "yazelix" {
        return false
    }

    let will_try_ghostty = ($terminal_candidates | any {|candidate| $candidate.terminal == "ghostty" })
    if not $will_try_ghostty {
        return false
    }

    reroll_ghostty_random_cursor_config_for_launch $config $runtime_dir --quiet=(not $verbose_mode)
}

def describe_terminal_invocation [terminal_info: record, terminal_config] {
    let terminal = $terminal_info.terminal
    if $terminal == "wezterm" {
        $"Running: wezterm --config-file ($terminal_config) start --class=com.yazelix.Yazelix"
    } else if $terminal == "ghostty" {
        $"Running: ghostty --config-file=($terminal_config)"
    } else if $terminal == "kitty" {
        $"Running: kitty --config=($terminal_config) --class=com.yazelix.Yazelix"
    } else if $terminal == "alacritty" {
        $"Running: alacritty --config-file=($terminal_config)"
    } else if $terminal == "foot" {
        $"Running: foot --config ($terminal_config) --app-id com.yazelix.Yazelix"
    } else {
        $"Running: ($terminal)"
    }
}

def get_terminal_candidate_display_name [terminal_info: record] {
    $terminal_info.name? | default $terminal_info.terminal
}

def launch_terminal_candidates [
    terminal_candidates: list<record>
    terminal_config_mode: string
    working_dir: string
    needs_reload: bool
    runtime_dir: string
    verbose_mode: bool
    requested_terminal: string
] {
    mut failures = []
    mut index = 0

    for terminal_info in $terminal_candidates {
        let display_name = (get_terminal_candidate_display_name $terminal_info)
        let terminal_config = (resolve_terminal_config $terminal_info.terminal $terminal_config_mode)

        if ($terminal_config != null) and (not ($terminal_config | path exists)) {
            let msg = $"($terminal_info.name) config not found at ($terminal_config)"
            $failures = ($failures | append {name: $display_name, reason: $msg})
            continue
        }

        let launch_cmd = (build_launch_command $terminal_info $terminal_config $working_dir $needs_reload)

        if $verbose_mode {
            print $"Using terminal: ($display_name)"
            print (describe_terminal_invocation $terminal_info $terminal_config)
        }

        mut propagated_env = {
            YAZELIX_TERMINAL: $terminal_info.terminal
            YAZELIX_RUNTIME_DIR: $runtime_dir
        }
        if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) {
            $propagated_env = ($propagated_env | upsert YAZELIX_SWEEP_TEST_ID $env.YAZELIX_SWEEP_TEST_ID)
        }
        if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
            $propagated_env = ($propagated_env | upsert YAZELIX_LAYOUT_OVERRIDE $env.YAZELIX_LAYOUT_OVERRIDE)
        }
        let env_block = (propagate_startup_profile_env $propagated_env)

        if $verbose_mode {
            print $"Launching command: ($launch_cmd)"
        }

        let launch_attempt = (try {
            with-env $env_block {
                run_detached_terminal_launch $launch_cmd $display_name --verbose=$verbose_mode
            }
            {ok: true}
        } catch {|err|
            {ok: false, err: $err}
        })

        if $launch_attempt.ok {
            return $terminal_info
        }

        let err_msg = ($launch_attempt.err.msg | default ($launch_attempt.err | to nuon))
        $failures = ($failures | append {name: $display_name, reason: $err_msg})
        $index = $index + 1

        if ($requested_terminal | is-empty) and ($index < ($terminal_candidates | length)) {
            let next_candidate = ($terminal_candidates | get -o $index)
            if $next_candidate != null {
                let next_name = (get_terminal_candidate_display_name $next_candidate)
                print $"⚠️  ($display_name) failed to start; trying ($next_name)..."
            }
        }
    }

    let failure_summary = (
        $failures
        | each {|failure|
            let tail = (
                $failure.reason
                | lines
                | last 2
                | str join " "
                | str trim
            )
            $"  - ($failure.name): ($tail)"
        }
        | str join "\n"
    )

    if ($requested_terminal | is-not-empty) {
        error make {msg: $"Failed to launch requested terminal '($requested_terminal)'.\n($failure_summary)"}
    } else {
        error make {msg: $"Failed to launch any configured terminal.\n($failure_summary)"}
    }
}

def main [
    launch_cwd?: string
    --terminal(-t): string = ""  # Override terminal selection (for sweep testing)
    --verbose               # Enable verbose logging
    --desktop-fast-path     # Launch the terminal immediately and let startup rebuild inside it
] {
    let component = if $desktop_fast_path { "desktop_fast_path" } else { "launch" }

    # Resolve HOME using shell expansion
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    let verbose_mode = $verbose
    let requested_terminal = $terminal
    if $verbose_mode {
        print "🔍 launch_yazelix: verbose mode enabled"
        print $"Resolved HOME=($home)"
    }

    # Compute config state (auto-creates yazelix.toml if missing)
    let config_state = (profile_startup_step $component "compute_config_state" {
        compute_config_state_via_yzx_core
    })
    let config = $config_state.config
    let active_config_file = $config_state.config_file
    let current_hash = $config_state.combined_hash
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
        print $"   Reload:   ($needs_reload)"
    }

    # Use provided launch directory or fall back to current directory
    let requested_working_dir = if ($launch_cwd | is-empty) { pwd } else { $launch_cwd }
    let launch_preflight = (profile_startup_step $component "validate_working_dir" {
        run_launch_preflight $requested_working_dir $requested_terminal ($config.terminals? | default [$DEFAULT_TERMINAL] | uniq)
    })
    let working_dir = $launch_preflight.working_dir
    if $verbose_mode {
        print $"Launch directory: ($working_dir)"
    }

    let terminal_config_mode = $config.terminal_config_mode
    let terminals = ($config.terminals? | default [$DEFAULT_TERMINAL] | uniq)
    if ($terminals | is-empty) {
        let available = ($SUPPORTED_TERMINALS | where {|t| which $t | is-not-empty })
        let available_str = if ($available | is-empty) {
            "none detected"
        } else {
            $available | str join ", "
        }
        print "Error: terminal.terminals must include at least one terminal"
        print $"Detected terminals: ($available_str)"
        print "Set [terminal].terminals in ~/.config/yazelix/user_configs/yazelix.toml"
        exit 1
    }

    let runtime_dir = (get_yazelix_runtime_dir)
    let terminal_candidates = (profile_startup_step $component "resolve_terminals" {
        $launch_preflight.terminal_candidates
    })
    if $desktop_fast_path {
        profile_startup_step $component "generate_terminal_configs" {
            ensure_terminal_configs_available_for_candidates $terminal_candidates $terminal_config_mode $runtime_dir
        } | ignore
        profile_startup_step $component "reroll_ghostty_cursor" {
            reroll_ghostty_random_cursor_config_for_launch_candidates $terminal_candidates $terminal_config_mode $runtime_dir $config $verbose_mode
        } | ignore
    } else {
        # Generate all terminal configurations for safety and consistency
        profile_startup_step $component "generate_all_terminal_configs" {
            let terminals = ($config.terminals? | default [$DEFAULT_TERMINAL])
            if ($terminals | is-empty) {
                error make {msg: "terminal.terminals must include at least one terminal"}
            }
            materialize_selected_terminal_configs $terminals $runtime_dir $active_config_file
        } | ignore
    }
    profile_startup_step $component "launch_terminal" {
        launch_terminal_candidates $terminal_candidates $terminal_config_mode $working_dir $needs_reload $runtime_dir $verbose_mode $requested_terminal
    } | ignore
}
