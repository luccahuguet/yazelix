#!/usr/bin/env nu
# yzx launch command - Launch Yazelix in a new terminal window

use ../utils/config_state.nu [compute_config_state record_materialized_state]
use ../utils/build_policy.nu [describe_build_parallelism]
use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/devenv_backend.nu [advance_runtime_state_after_rebuild check_environment_status get_refresh_output_mode print_refresh_request_guidance rebuild_yazelix_environment resolve_launch_transition resolve_refresh_request resolve_runtime_entry_state run_in_devenv_shell_command]
use ../utils/launch_state.nu [get_launch_env get_launch_profile require_reused_launch_profile resolve_runtime_owned_profile]
use ../utils/doctor.nu print_runtime_version_drift_warning
use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../utils/common.nu [require_yazelix_runtime_dir]
use ../utils/constants.nu [TERMINAL_METADATA]
use ../utils/runtime_contract_checker.nu [check_runtime_script require_runtime_check]

def wrapper_store_paths_exist [wrapper_path: string] {
    if not ($wrapper_path | path exists) {
        return false
    }

    let store_paths = (
        open --raw $wrapper_path
        | parse -r '(?<path>/nix/store/[^" \n]+)'
        | get -o path
        | default []
        | uniq
    )

    if ($store_paths | is-empty) {
        true
    } else {
        $store_paths | all {|path_ref| $path_ref | path exists }
    }
}

def resolve_terminal_for_support_check [config: record, requested_terminal: string] {
    if ($requested_terminal | is-not-empty) {
        return $requested_terminal
    }

    let terminals = ($config.terminals? | default ["ghostty"] | uniq)
    if ($terminals | is-empty) {
        "unknown"
    } else {
        ($terminals | first | into string)
    }
}

def launch_profile_supports_configured_terminal [config: record, profile_path: string, requested_terminal: string] {
    let manage_terminals = ($config.manage_terminals? | default true)
    if not $manage_terminals {
        return true
    }

    let preferred_terminal = (resolve_terminal_for_support_check $config $requested_terminal)
    let terminal_meta = ($TERMINAL_METADATA | get -o $preferred_terminal | default {})
    let wrapper_name = ($terminal_meta.wrapper? | default "")
    let profile_bin_dir = ($profile_path | path join "bin")
    let wrapper_path = if ($wrapper_name | is-not-empty) {
        $profile_bin_dir | path join $wrapper_name
    } else {
        ""
    }

    (($wrapper_path | is-not-empty) and (wrapper_store_paths_exist $wrapper_path))
}

def current_environment_supports_configured_terminal [config: record, requested_terminal: string] {
    let manage_terminals = ($config.manage_terminals? | default true)
    if not $manage_terminals {
        return true
    }

    let preferred_terminal = (resolve_terminal_for_support_check $config $requested_terminal)
    let terminal_meta = ($TERMINAL_METADATA | get -o $preferred_terminal | default {})
    let wrapper_name = ($terminal_meta.wrapper? | default "")

    if ($wrapper_name | is-empty) {
        return false
    }

    let current_profile = ($env.DEVENV_PROFILE? | default "" | into string | str trim)
    let wrapper_path = if ($current_profile | is-empty) {
        ""
    } else {
        $current_profile | path join "bin" $wrapper_name
    }
    (($wrapper_path | is-not-empty) and (wrapper_store_paths_exist $wrapper_path))
}

def require_launch_runtime_script [script_path: string] {
    let check = (check_runtime_script $script_path "launch_runtime_script" "launch script" "launch")
    require_runtime_check $check | ignore
    $check.path
}

# Launch yazelix
export def "yzx launch" [
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
    let reuse_mode = $reuse
    let requested_path = ($path | default "")
    let requested_terminal = ($terminal | default "")

    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let needs_refresh = $env_prep.needs_refresh
    let refresh_request = (resolve_refresh_request $needs_refresh --reuse=$reuse_mode --skip-refresh=$skip_refresh)
    let refresh_output = get_refresh_output_mode $config
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let build_parallelism_description = (describe_build_parallelism $build_cores $max_jobs)
    let show_refresh_notice = ($refresh_output != "quiet")
    let manage_terminals = ($config.manage_terminals? | default true)
    let built_profile = (resolve_runtime_owned_profile)
    let terminal_profile_needs_repair = (
        $manage_terminals
        and (not $skip_refresh)
        and (not $reuse_mode)
        and ($built_profile | is-not-empty)
        and (not (launch_profile_supports_configured_terminal $config $built_profile $requested_terminal))
    )
    let launch_refresh_request = if $terminal_profile_needs_repair {
        $refresh_request
        | upsert should_refresh true
        | upsert mode "refresh"
    } else {
        $refresh_request
    }
    let should_refresh = ($launch_refresh_request.should_refresh? | default false)
    let env_status = (check_environment_status)
    let force_reenter = $force_reenter
    let runtime_state = (
        resolve_runtime_entry_state
        $launch_refresh_request
        --already-in-env=$env_status.already_in_env
        --in-yazelix-shell=$env_status.in_yazelix_shell
        --force-reenter=$force_reenter
    )
    mut printed_refresh_notice = false
    if $verbose_mode {
        print $"🔍 Config hash changed? ($needs_refresh)"
        if $terminal_profile_needs_repair {
            print "🔍 Managed terminal wrapper repair needed: true"
        }
    }
    print_refresh_request_guidance $refresh_request
    if $terminal_profile_needs_repair {
        print "🔄 Managed terminal wrapper is stale or missing required runtime dependencies."
        print $"   Rebuilding environment using ($build_parallelism_description)..."
        $printed_refresh_notice = true
    }

    let current_session_eligible = (
        (($runtime_state.activation_surface | default "external_process") == "live_yazelix_session")
        and (current_environment_supports_configured_terminal $config $requested_terminal)
    )
    if (($runtime_state.activation_surface | default "external_process") == "live_yazelix_session") and (not $current_session_eligible) {
        if $verbose_mode {
            print "⚠️  Current Yazelix shell does not include the configured terminal; re-entering a fresh environment."
        }
    }
    let cached_launch_profile = if (($runtime_state.profile_request | default "none") == "reused_recorded_profile") {
        require_reused_launch_profile $config_state "yzx launch --reuse"
    } else if (($runtime_state.profile_request | default "none") == "verified_recorded_profile") {
        get_launch_profile $config_state
    } else {
        null
    }
    let cached_launch_profile = if ($cached_launch_profile != null) and (not (launch_profile_supports_configured_terminal $config $cached_launch_profile $requested_terminal)) {
        if $verbose_mode {
            print "⚠️  Cached Yazelix profile does not include the currently configured terminal; re-entering devenv instead."
        }
        null
    } else {
        $cached_launch_profile
    }
    let launch_transition = (
        resolve_launch_transition
        $runtime_state
        --current-session-eligible=$current_session_eligible
        --profile-available=($cached_launch_profile != null)
    )

    # Launch new terminal
    let launch_cwd = if $home {
            $env.HOME
        } else if ($requested_path | is-not-empty) {
            $requested_path
        } else {
            pwd
        }

        let runtime_dir = (require_yazelix_runtime_dir)
        let launch_script = (require_launch_runtime_script ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu"))

        if $launch_transition.execution == "current_session" {
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
            let active_launch_transition = if $launch_transition.rebuild_before_exec {
                if $show_refresh_notice and (not $printed_refresh_notice) {
                    print $"🔄 Configuration changed - rebuilding environment using ($build_parallelism_description)..."
                    $printed_refresh_notice = true
                }
                rebuild_yazelix_environment --max-jobs $max_jobs --build-cores $build_cores --refresh-eval-cache --output-mode $refresh_output
                let post_refresh_state = (advance_runtime_state_after_rebuild $runtime_state)
                let fresh_state = compute_config_state
                let fresh_launch_profile = if ($post_refresh_state.profile_request == "verified_recorded_profile") {
                    get_launch_profile $fresh_state
                } else {
                    null
                }
                let fresh_launch_profile = if ($fresh_launch_profile != null) and (not (launch_profile_supports_configured_terminal $config $fresh_launch_profile $requested_terminal)) {
                    if $verbose_mode {
                        print "⚠️  Cached Yazelix profile does not include the currently configured terminal; re-entering devenv instead."
                    }
                    null
                } else {
                    $fresh_launch_profile
                }

                let post_refresh_transition = (
                    resolve_launch_transition
                    $post_refresh_state
                    --current-session-eligible=false
                    --profile-available=($fresh_launch_profile != null)
                )
                {
                    transition: $post_refresh_transition
                    state: $fresh_state
                    profile: $fresh_launch_profile
                }
            } else {
                {
                    transition: $launch_transition
                    state: $config_state
                    profile: $cached_launch_profile
                }
            }

            let final_transition = $active_launch_transition.transition
            let final_state = $active_launch_transition.state
            let final_launch_profile = $active_launch_transition.profile

            if $final_transition.execution == "launch_profile" {
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
                    print $"⚡ Using activated Yazelix profile: ($final_launch_profile)"
                }
                with-env (get_launch_env $config $final_launch_profile) {
                    ^nu ...$launch_args
                }
                return
            }

            if $launch_transition.rebuild_before_exec and $verbose_mode {
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
            with-env $env_block {
                run_in_devenv_shell_command "nu" ...$final_launch_args --max-jobs $max_jobs --build-cores $build_cores --cwd $runtime_dir --runtime-dir $runtime_dir --skip-welcome --force-shell --force-refresh=$final_transition.rebuild_before_exec --verbose=$verbose_mode --refresh-output-mode $refresh_output
            }
            if $launch_transition.rebuild_before_exec {
                record_materialized_state $final_state
            }
        }
}
