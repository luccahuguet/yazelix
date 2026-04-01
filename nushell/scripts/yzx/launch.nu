#!/usr/bin/env nu
# yzx launch command - Launch Yazelix in new or current terminal

use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/environment_bootstrap.nu [prepare_environment rebuild_yazelix_environment run_in_devenv_shell_command get_refresh_output_mode]
use ../utils/launch_state.nu [get_launch_env get_launch_profile require_reused_launch_profile]
use ../utils/doctor.nu print_runtime_version_drift_warning
use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../core/start_yazelix.nu [start_yazelix_session]
use ../utils/common.nu [describe_build_parallelism get_yazelix_runtime_dir]
use ../utils/constants.nu [TERMINAL_METADATA]

def launch_profile_supports_configured_terminal [config: record, profile_path: string] {
    let manage_terminals = ($config.manage_terminals? | default true)
    if not $manage_terminals {
        return true
    }

    let terminals = ($config.terminals? | default ["ghostty"] | uniq)
    if ($terminals | is-empty) {
        return true
    }

    let preferred_terminal = ($terminals | first | into string)
    let terminal_meta = ($TERMINAL_METADATA | get -o $preferred_terminal | default {})
    let wrapper_name = ($terminal_meta.wrapper? | default "")
    let profile_bin_dir = ($profile_path | path join "bin")
    let wrapper_path = if ($wrapper_name | is-not-empty) {
        $profile_bin_dir | path join $wrapper_name
    } else {
        ""
    }
    let direct_terminal_path = ($profile_bin_dir | path join $preferred_terminal)

    (
        (($wrapper_path | is-not-empty) and ($wrapper_path | path exists))
        or ($direct_terminal_path | path exists)
    )
}

def current_environment_supports_configured_terminal [config: record] {
    let manage_terminals = ($config.manage_terminals? | default true)
    if not $manage_terminals {
        return true
    }

    let terminals = ($config.terminals? | default ["ghostty"] | uniq)
    if ($terminals | is-empty) {
        return true
    }

    let preferred_terminal = ($terminals | first | into string)
    let terminal_meta = ($TERMINAL_METADATA | get -o $preferred_terminal | default {})
    let wrapper_name = ($terminal_meta.wrapper? | default "")

    (
        (($wrapper_name | is-not-empty) and (which $wrapper_name | is-not-empty))
        or (which $preferred_terminal | is-not-empty)
    )
}

# Launch yazelix
export def "yzx launch" [
    --here             # Start in current terminal instead of launching new terminal
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --terminal(-t): string  # Override terminal selection (for sweep testing)
    --verbose          # Enable verbose logging
    --reuse            # Reuse the last built profile without rebuilding
    --skip-refresh(-s) # Skip explicit refresh trigger and allow potentially stale environment
    --force-reenter    # Force re-entering devenv before launch
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available
    print_runtime_version_drift_warning
    run_entrypoint_config_migration_preflight "yzx launch" | ignore

    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 yzx launch: verbose mode enabled"
    }

    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    mut needs_refresh = $env_prep.needs_refresh
    let reuse_mode = $reuse
    let should_refresh = ($needs_refresh and (not $skip_refresh) and (not $reuse_mode))
    let refresh_output = get_refresh_output_mode $config
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let build_parallelism_description = (describe_build_parallelism $build_cores $max_jobs)
    let show_refresh_notice = ($refresh_output != "quiet")
    let manage_terminals = ($config.manage_terminals? | default true)
    let requested_path = ($path | default "")
    let requested_terminal = ($terminal | default "")
    mut printed_refresh_notice = false
    if $verbose_mode {
        print $"🔍 Config hash changed? ($needs_refresh)"
    }
    if $reuse_mode and $needs_refresh {
        print "⚡ Reuse mode enabled - using the last built Yazelix profile without rebuild."
        print "   Local config/input changes since the last refresh are not applied."
    } else if $skip_refresh and $needs_refresh {
        print "⚠️  Skipping explicit refresh trigger; environment may be stale."
        print "   If tools/env vars look outdated, rerun without --skip-refresh or run 'yzx refresh'."
    }

    let force_reenter = $force_reenter
    mut in_yazelix_shell = ($env.IN_YAZELIX_SHELL? == "true")
    if $manage_terminals and $should_refresh and $in_yazelix_shell {
        # Only print if not called from yzx restart (which already printed the message)
        if (not $force_reenter) and $show_refresh_notice {
            print $"🔄 Configuration changed - rebuilding environment using ($build_parallelism_description)..."
            $printed_refresh_notice = true
        }
        $in_yazelix_shell = false
    }
    if $force_reenter {
        $in_yazelix_shell = false
    }
    if $in_yazelix_shell and (not (current_environment_supports_configured_terminal $config)) {
        if $verbose_mode {
            print "⚠️  Current Yazelix shell does not include the configured terminal; re-entering a fresh environment."
        }
        $in_yazelix_shell = false
    }

    if $here {
        # Start in current terminal without spawning a new process
        $env.YAZELIX_ENV_ONLY = "false"

        # Determine directory override: explicit --home or --path, else let start_yazelix handle it
        let cwd_override = if $home {
            $env.HOME
        } else if ($requested_path | is-not-empty) {
            $requested_path
        } else {
            null
        }

        if $verbose_mode {
            if $should_refresh {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override --verbose --reuse=$reuse_mode
                } else {
                    start_yazelix_session --verbose --reuse=$reuse_mode
                }
            } else {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override --verbose --reuse=$reuse_mode
                } else {
                    start_yazelix_session --verbose --reuse=$reuse_mode
                }
            }
        } else {
            if $should_refresh {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override --reuse=$reuse_mode
                } else {
                    start_yazelix_session --reuse=$reuse_mode
                }
            } else {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override --reuse=$reuse_mode
                } else {
                    start_yazelix_session --reuse=$reuse_mode
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
        } else if ($requested_path | is-not-empty) {
            $requested_path
        } else {
            pwd
        }

        let runtime_dir = (get_yazelix_runtime_dir)
        let launch_script = ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")

        if $in_yazelix_shell {
            # Already in Yazelix environment - run directly via bash
            let base_args = [$launch_script]
            let mut_args = if ($launch_cwd | is-not-empty) {
                $base_args | append $launch_cwd
            } else {
                $base_args
            }
            let mut_args = if ($requested_terminal | is-not-empty) {
                $mut_args | append "--terminal" | append $requested_terminal
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
            let reused_launch_profile = if $reuse_mode {
                require_reused_launch_profile $config_state "yzx launch --reuse"
            } else {
                null
            }

            if $should_refresh {
                if $show_refresh_notice and (not $printed_refresh_notice) {
                    print $"🔄 Configuration changed - rebuilding environment using ($build_parallelism_description)..."
                    $printed_refresh_notice = true
                }
                rebuild_yazelix_environment --max-jobs $max_jobs --build-cores $build_cores --refresh-eval-cache --output-mode $refresh_output
                $needs_refresh = false
            }

            let fresh_state = if $should_refresh {
                compute_config_state
            } else {
                $config_state
            }
            # After a forced re-enter (used by restart after rebuild), prefer a
            # fresh devenv shell over the cached fast-launch profile. The cached
            # profile can lag behind the rebuilt shell and miss newly selected
            # terminal packages such as Kitty.
            let fresh_launch_profile = if $force_reenter {
                null
            } else if $reuse_mode {
                $reused_launch_profile
            } else {
                get_launch_profile $fresh_state
            }

            let fresh_launch_profile = if ($fresh_launch_profile != null) and (not (launch_profile_supports_configured_terminal $config $fresh_launch_profile)) {
                if $verbose_mode {
                    print "⚠️  Cached Yazelix profile does not include the currently configured terminal; re-entering devenv instead."
                }
                null
            } else {
                $fresh_launch_profile
            }

            if $fresh_launch_profile != null {
                let base_args = [$launch_script]
                let launch_args = if ($launch_cwd | is-not-empty) {
                    $base_args | append $launch_cwd
                } else {
                    $base_args
                }
                let launch_args = if ($requested_terminal | is-not-empty) {
                    $launch_args | append "--terminal" | append $requested_terminal
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

            let yazelix_dir = (get_yazelix_runtime_dir)
            if $should_refresh and $verbose_mode {
                let reason = ($config_state.refresh_reason? | default "config or devenv inputs changed since last launch")
                print $"♻️  Re-entering Yazelix after rebuild \(($reason), ($build_parallelism_description)\)"
            }

            mut launch_args = [$launch_script]
            if ($launch_cwd | is-not-empty) {
                $launch_args = ($launch_args | append $launch_cwd)
            }
            if ($requested_terminal | is-not-empty) {
                $launch_args = ($launch_args | append "--terminal" | append $requested_terminal)
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
            if ($env.YAZELIX_TERMINAL? | is-not-empty) {
                $env_block = ($env_block | upsert YAZELIX_TERMINAL $env.YAZELIX_TERMINAL)
            }
            with-env $env_block {
                run_in_devenv_shell_command "nu" ...$final_launch_args --max-jobs $max_jobs --build-cores $build_cores --cwd $yazelix_dir --runtime-dir $yazelix_dir --skip-welcome --force-refresh=($should_refresh or $force_reenter) --verbose=$verbose_mode --refresh-output-mode $refresh_output
            }
            if $should_refresh {
                mark_config_state_applied $fresh_state
            }
        }
}
