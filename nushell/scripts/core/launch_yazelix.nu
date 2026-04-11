#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
# Nushell version of the Yazelix launcher

use ../utils/config_state.nu compute_config_state
use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../utils/terminal_configs.nu [generate_all_terminal_configs generate_selected_terminal_configs]
use ../utils/terminal_launcher.nu *
use ../utils/constants.nu [SUPPORTED_TERMINALS, TERMINAL_METADATA]
use ../utils/common.nu [get_yazelix_runtime_dir]
use ../utils/runtime_contract_checker.nu [
    check_launch_terminal_support
    check_launch_working_dir
    require_runtime_check
]

def validate_launch_working_dir [working_dir: string] {
    let check = (check_launch_working_dir $working_dir)
    require_runtime_check $check | ignore
    $check.path
}

def resolve_terminal_candidates [requested_terminal: string, terminals: list<string>] {
    let check = (check_launch_terminal_support $requested_terminal $terminals)
    require_runtime_check $check | ignore
    ($check.candidates? | default [])
}
def resolve_desktop_fast_path_candidates [requested_terminal: string, terminals: list<string>] {
    resolve_terminal_candidates $requested_terminal $terminals
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
        generate_selected_terminal_configs $candidate_terminals $runtime_dir
    }
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
        let display_name = (get_terminal_display_name $terminal_info)
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

        let env_block = $propagated_env

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
            let next_name = (get_terminal_display_name ($terminal_candidates | get $index))
            print $"⚠️  ($display_name) failed to start; trying ($next_name)..."
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
    --terminal(-t): string  # Override terminal selection (for sweep testing)
    --verbose               # Enable verbose logging
    --desktop-fast-path     # Launch the terminal immediately and let startup rebuild inside it
] {
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
    mut terminals = ($config.terminals? | default ["ghostty"] | uniq)
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
    let terminal_candidates = if $desktop_fast_path {
        resolve_desktop_fast_path_candidates $requested_terminal $terminals
    } else {
        resolve_terminal_candidates $requested_terminal $terminals
    }
    if $desktop_fast_path {
        ensure_terminal_configs_available_for_candidates $terminal_candidates $terminal_config_mode $runtime_dir
    } else {
        # Generate all terminal configurations for safety and consistency
        generate_all_terminal_configs
    }
    launch_terminal_candidates $terminal_candidates $terminal_config_mode $working_dir $needs_reload $runtime_dir $verbose_mode $requested_terminal | ignore
}
