#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/environment_bootstrap.nu *
use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../utils/launch_state.nu [activate_launch_profile get_launch_profile require_reused_launch_profile]
use ../utils/common.nu [describe_build_parallelism require_yazelix_dir]
use ../utils/startup_profile.nu [profile_startup_step]
use ../utils/runtime_contract_checker.nu [
    check_generated_layout
    check_runtime_script
    check_startup_working_dir
    require_runtime_check
    resolve_expected_layout_path
]

def validate_startup_working_dir [working_dir: string] {
    let check = (check_startup_working_dir $working_dir)
    require_runtime_check $check | ignore
    $check.path
}

def require_runtime_script [script_path: string, label: string] {
    let check = (check_runtime_script $script_path "startup_runtime_script" $label "startup")
    require_runtime_check $check | ignore
    $check.path
}

def require_generated_layout [layout_path: string] {
    let check = (check_generated_layout $layout_path "startup")
    require_runtime_check $check | ignore
    $check.path
}

def _start_yazelix_impl [cwd_override?: string, --verbose, --setup-only, --reuse, --skip-refresh, --force-reenter] {
    # Capture original directory before any cd commands
    let original_dir = pwd

    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 start_yazelix: verbose mode enabled"
    }

    let yazelix_dir = try {
        require_yazelix_dir
    } catch {|err|
        print $"Error: ($err.msg)"
        exit 1
    }

    profile_startup_step "startup" "entrypoint.config_migration_preflight" {
        run_entrypoint_config_migration_preflight "Yazelix startup" | ignore
    }

    let env_prep = prepare_environment --verbose=$verbose_mode
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh
    let should_refresh = ($needs_refresh and (not $reuse) and (not $skip_refresh))
    let refresh_output = get_refresh_output_mode $config
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let build_parallelism_description = (describe_build_parallelism $build_cores $max_jobs)
    let env_status = check_environment_status
    let reuse_mode = $reuse
    let skip_refresh_mode = $skip_refresh
    let force_reenter_mode = $force_reenter
    mut activated_profile = false

    if $reuse_mode and $needs_refresh {
        print "⚡ Reuse mode enabled - using the last built Yazelix profile without rebuild."
        print "   Local config/input changes since the last refresh are not applied."
    } else if $skip_refresh_mode and $needs_refresh {
        print "⚠️  Skipping explicit refresh trigger; environment may be stale."
        print "   If tools/env vars look outdated, rerun without --skip-refresh or run 'yzx refresh'."
    }

    if (not $env_status.already_in_env) and ((not $should_refresh) or $reuse_mode) and (not $force_reenter_mode) {
        let profile_path = if $reuse_mode {
            require_reused_launch_profile $env_prep.config_state "yzx enter --reuse"
        } else {
            get_launch_profile $env_prep.config_state
        }
        if $profile_path != null {
            if $verbose_mode {
                print $"⚡ Activating Yazelix profile: ($profile_path)"
            }
            activate_launch_profile $config $profile_path
            $activated_profile = true
        }
    }

    # Ensure environment is available when direct activation is not possible.
    if not $activated_profile {
        ensure_environment_available
    }

    # If setup-only mode, just run devenv shell to install hooks and exit
    if $setup_only {
        print "🔧 Setting up Yazelix environment (installing shell hooks and dependencies)..."
        print "   This may take several minutes on first run."

        run_in_devenv_shell_command "sh" "-c" "echo '✅ Setup complete! Shell hooks installed.'" --max-jobs $max_jobs --build-cores $build_cores --cwd $yazelix_dir --runtime-dir $yazelix_dir --skip-welcome --force-shell=true --verbose=$verbose_mode --force-refresh=$should_refresh

        print ""
        print "📝 Next steps:"
        print "   1. Restart your shell (or source your shell config)"
        print "   2. Run 'yzx launch' to open Yazelix in a new window, or 'yzx enter' to start it here"
        print ""
        return
    }

    # For Zellij config, create a placeholder for now - will be generated inside Nix environment
    let merged_zellij_dir = $"($env.HOME)/.local/share/yazelix/configs/zellij"

    # Determine which directory to use as default CWD
    # Priority: 1. cwd_override parameter 2. original directory
    let requested_working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else {
        $original_dir
    }
    let working_dir = (validate_startup_working_dir $requested_working_dir)

    # Resolve layout from yazelix.toml; explicit override wins for sweep/test flows.
    let layout_path = (resolve_expected_layout_path $config $"($merged_zellij_dir)/layouts")
    let resolved_layout_path = (require_generated_layout $layout_path)

    let inner_script = (require_runtime_script $"($yazelix_dir)/nushell/scripts/core/start_yazelix_inner.nu" "startup script")
    let base_args = if ($working_dir | is-not-empty) {
        ["-i", $inner_script, $working_dir, $resolved_layout_path]
    } else {
        ["-i", $inner_script, "", $resolved_layout_path]
    }
    let inner_args = if $verbose_mode {
        $base_args | append "--verbose"
    } else {
        $base_args
    }

    # Run devenv shell with explicit HOME.
    # The default shell is dynamically read from yazelix.toml configuration
    # and passed directly to the zellij command.
    let use_activated_profile = $activated_profile

    with-env {HOME: $env.HOME} {
        if $use_activated_profile {
            if $verbose_mode {
                print "⚡ Reusing activated profile without entering devenv shell"
            }
            nu $"($yazelix_dir)/nushell/scripts/setup/environment.nu" --welcome-source start
        }

        if $should_refresh {
            if $verbose_mode {
                print $"♻️  Config changed - rebuilding environment using ($build_parallelism_description)"
            } else if $refresh_output != "quiet" {
                print $"♻️  Config changed - rebuilding environment using ($build_parallelism_description)"
            }
        }

        # Use shared devenv runner (consolidates with yzx env)
        run_in_devenv_shell_command "nu" ...$inner_args --max-jobs $max_jobs --build-cores $build_cores --cwd $yazelix_dir --runtime-dir $yazelix_dir --skip-welcome --force-shell=$force_reenter_mode --verbose=$verbose_mode --force-refresh=$should_refresh --refresh-output-mode $refresh_output
    }
}

export def start_yazelix_session [cwd_override?: string, --verbose, --setup-only, --reuse, --skip-refresh, --force-reenter] {
    if ($cwd_override | is-not-empty) {
        _start_yazelix_impl $cwd_override --verbose=$verbose --setup-only=$setup_only --reuse=$reuse --skip-refresh=$skip_refresh --force-reenter=$force_reenter
    } else {
        _start_yazelix_impl --verbose=$verbose --setup-only=$setup_only --reuse=$reuse --skip-refresh=$skip_refresh --force-reenter=$force_reenter
    }
}

export def main [cwd_override?: string, --verbose, --setup-only, --reuse, --skip-refresh, --force-reenter] {
    if ($cwd_override | is-not-empty) {
        start_yazelix_session $cwd_override --verbose=$verbose --setup-only=$setup_only --reuse=$reuse --skip-refresh=$skip_refresh --force-reenter=$force_reenter
    } else {
        start_yazelix_session --verbose=$verbose --setup-only=$setup_only --reuse=$reuse --skip-refresh=$skip_refresh --force-reenter=$force_reenter
    }
}
