#!/usr/bin/env nu
# yzx launch command - Launch Yazelix in new or current terminal

use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/environment_bootstrap.nu [prepare_environment rebuild_yazelix_environment run_in_devenv_shell_command]
use ../utils/launch_state.nu [get_launch_env get_launch_profile]
use ../utils/doctor.nu print_runtime_version_drift_warning
use ../core/start_yazelix.nu [start_yazelix_session]

# Launch yazelix
export def "yzx launch" [
    --here             # Start in current terminal instead of launching new terminal
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --terminal(-t): string  # Override terminal selection (for sweep testing)
    --verbose          # Enable verbose logging
    --skip-refresh(-s) # Skip explicit refresh trigger and allow potentially stale environment
    --force-reenter    # Force re-entering devenv before launch
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available
    print_runtime_version_drift_warning

    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 yzx launch: verbose mode enabled"
    }

    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    mut needs_refresh = $env_prep.needs_refresh
    let should_refresh = ($needs_refresh and (not $skip_refresh))
    let launch_profile = if $should_refresh {
        null
    } else {
        get_launch_profile $config_state
    }
    let manage_terminals = ($config.manage_terminals? | default true)
    if $verbose_mode {
        print $"🔍 Config hash changed? ($needs_refresh)"
    }
    if $skip_refresh and $needs_refresh {
        print "⚠️  Skipping explicit refresh trigger; environment may be stale."
        print "   If tools/env vars look outdated, rerun without --skip-refresh or run 'yzx refresh'."
    }

    let force_reenter = $force_reenter
    mut in_yazelix_shell = ($env.IN_YAZELIX_SHELL? == "true")
    if $manage_terminals and $should_refresh and $in_yazelix_shell {
        # Only print if not called from yzx restart (which already printed the message)
        if not $force_reenter {
            print "🔄 Configuration changed - rebuilding environment..."
        }
        $in_yazelix_shell = false
    }
    if $force_reenter {
        $in_yazelix_shell = false
    }

    if $here {
        # Start in current terminal without spawning a new process
        $env.YAZELIX_ENV_ONLY = "false"

        # Determine directory override: explicit --home or --path, else let start_yazelix handle it
        let cwd_override = if $home {
            $env.HOME
        } else if ($path != null) {
            $path
        } else {
            null
        }

        if $verbose_mode {
            if $should_refresh {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override --verbose
                } else {
                    start_yazelix_session --verbose
                }
            } else {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override --verbose
                } else {
                    start_yazelix_session --verbose
                }
            }
        } else {
            if $should_refresh {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override
                } else {
                    start_yazelix_session
                }
            } else {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override
                } else {
                    start_yazelix_session
                }
            }
        }
        if $should_refresh {
            mark_config_state_applied $config_state
        }
        return
    }

    # Launch new terminal
    let launch_cwd = if $home {
            $env.HOME
        } else if ($path | is-not-empty) {
            $path
        } else {
            pwd
        }

        let launch_script = $"($env.HOME)/.config/yazelix/nushell/scripts/core/launch_yazelix.nu"

        if $in_yazelix_shell {
            # Already in Yazelix environment - run directly via bash
            let base_args = [$launch_script]
            let mut_args = if ($launch_cwd | is-not-empty) {
                $base_args | append $launch_cwd
            } else {
                $base_args
            }
            let mut_args = if ($terminal | is-not-empty) {
                $mut_args | append "--terminal" | append $terminal
            } else {
                $mut_args
            }
            if $verbose_mode {
                let run_args = ($mut_args | append "--verbose")
                print $"⚙️ Executing launch_yazelix.nu inside Yazelix shell - cwd: ($launch_cwd)"
                ^nu ...$run_args
            } else {
                let final_args = $mut_args
                ^nu ...$final_args
            }
        } else {
            # Not in Yazelix environment - wrap with devenv shell
            if $should_refresh {
                rebuild_yazelix_environment --refresh-eval-cache
                $needs_refresh = false
            }

            let fresh_state = if $should_refresh {
                compute_config_state
            } else {
                $config_state
            }
            let fresh_launch_profile = get_launch_profile $fresh_state

            if $fresh_launch_profile != null {
                let base_args = [$launch_script]
                let launch_args = if ($launch_cwd | is-not-empty) {
                    $base_args | append $launch_cwd
                } else {
                    $base_args
                }
                let launch_args = if ($terminal | is-not-empty) {
                    $launch_args | append "--terminal" | append $terminal
                } else {
                    $launch_args
                }
                let launch_args = if $verbose_mode {
                    $launch_args | append "--verbose"
                } else {
                    $launch_args
                }

                if $verbose_mode {
                    print $"⚡ Using activated Yazelix profile: ($fresh_launch_profile)"
                }
                with-env (get_launch_env $config $fresh_launch_profile) {
                    ^nu ...$launch_args
                }
                return
            }

            let yazelix_dir = ("~/.config/yazelix" | path expand)
            if $should_refresh and $verbose_mode {
                let reason = ($config_state.refresh_reason? | default "config or devenv inputs changed since last launch")
                print $"♻️  ($reason) – rebuilding environment"
            }

            mut launch_args = [$launch_script]
            if ($launch_cwd | is-not-empty) {
                $launch_args = ($launch_args | append $launch_cwd)
            }
            if ($terminal | is-not-empty) {
                $launch_args = ($launch_args | append "--terminal" | append $terminal)
            }
            if $verbose_mode {
                $launch_args = ($launch_args | append "--verbose")
            }
            let final_launch_args = $launch_args

            mut env_block = {}
            if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) {
                $env_block = ($env_block | upsert YAZELIX_CONFIG_OVERRIDE $env.YAZELIX_CONFIG_OVERRIDE)
            }
            if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
                $env_block = ($env_block | upsert YAZELIX_LAYOUT_OVERRIDE $env.YAZELIX_LAYOUT_OVERRIDE)
            }
            if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) {
                $env_block = ($env_block | upsert YAZELIX_SWEEP_TEST_ID $env.YAZELIX_SWEEP_TEST_ID)
            }
            if ($env.YAZELIX_SKIP_WELCOME? | is-not-empty) {
                $env_block = ($env_block | upsert YAZELIX_SKIP_WELCOME $env.YAZELIX_SKIP_WELCOME)
            }
            if ($env.YAZELIX_TERMINAL? | is-not-empty) {
                $env_block = ($env_block | upsert YAZELIX_TERMINAL $env.YAZELIX_TERMINAL)
            }
            with-env $env_block {
                run_in_devenv_shell_command "nu" ...$final_launch_args --cwd $yazelix_dir --skip-welcome --force-refresh=$should_refresh --verbose=$verbose_mode
            }
            if $should_refresh {
                mark_config_state_applied $fresh_state
            }
        }
}
