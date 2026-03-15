#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/environment_bootstrap.nu *
use ../utils/launch_state.nu [activate_launch_profile get_launch_profile require_reused_launch_profile]
use ../utils/common.nu [describe_build_parallelism]

def _start_yazelix_impl [cwd_override?: string, --verbose, --setup-only, --reuse] {
    # Capture original directory before any cd commands
    let original_dir = pwd

    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 start_yazelix: verbose mode enabled"
    }

    # Resolve HOME using Nushell's built-in
    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }

    # Set absolute path for Yazelix directory
    let yazelix_dir = $"($home)/.config/yazelix"

    # Navigate to Yazelix directory
    # This is important for devenv to find devenv.nix in the current directory
    if not ($yazelix_dir | path exists) {
        print $"Error: Cannot find Yazelix directory at ($yazelix_dir)"
        exit 1
    }

    let env_prep = prepare_environment --verbose=$verbose_mode
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh
    let refresh_output = get_refresh_output_mode $config
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let build_parallelism_description = (describe_build_parallelism $build_cores $max_jobs)
    let env_status = check_environment_status
    let reuse_mode = $reuse
    mut activated_profile = false

    if $reuse_mode and $needs_refresh {
        print "⚡ Reuse mode enabled - using the last built Yazelix profile without rebuild."
        print "   Local config/input changes since the last refresh are not applied."
    }

    if (not $env_status.already_in_env) and ((not $needs_refresh) or $reuse_mode) {
        let profile_path = if $reuse_mode {
            require_reused_launch_profile $env_prep.config_state "yzx launch --here --reuse"
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

        run_in_devenv_shell_command "sh" "-c" "echo '✅ Setup complete! Shell hooks installed.'" --max-jobs $max_jobs --build-cores $build_cores --cwd $yazelix_dir --skip-welcome --verbose=$verbose_mode --force-refresh=$needs_refresh

        print ""
        print "📝 Next steps:"
        print "   1. Restart your shell (or source your shell config)"
        print "   2. Run 'yzx launch' to start Yazelix"
        print ""
        return
    }

    # For Zellij config, create a placeholder for now - will be generated inside Nix environment
    let merged_zellij_dir = $"($env.HOME)/.local/share/yazelix/configs/zellij"

    # Determine which directory to use as default CWD
    # Priority: 1. cwd_override parameter 2. original directory
    let working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else {
        $original_dir
    }

    # Resolve layout from yazelix.toml; explicit override wins for sweep/test flows.
    let configured_layout = if ($config.enable_sidebar? | default true) { "yzx_side" } else { "yzx_no_side" }
    let layout = if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
        $env.YAZELIX_LAYOUT_OVERRIDE
    } else if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) and ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
        $env.ZELLIJ_DEFAULT_LAYOUT
    } else {
        $configured_layout
    }
    # Resolve layout to an absolute file path so it works even if user config overrides layout_dir
    let layout_path = if ($layout | str contains "/") or ($layout | str ends-with ".kdl") {
        $layout
    } else {
        $"($merged_zellij_dir)/layouts/($layout).kdl"
    }

    let inner_script = $"($yazelix_dir)/nushell/scripts/core/start_yazelix_inner.nu"
    let base_args = if ($working_dir | is-not-empty) {
        ["-i", $inner_script, $working_dir, $layout_path]
    } else {
        ["-i", $inner_script, "", $layout_path]
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

    with-env {HOME: $home} {
        if $use_activated_profile {
            if $verbose_mode {
                print "⚡ Reusing activated profile without entering devenv shell"
            }
            nu $"($yazelix_dir)/nushell/scripts/setup/environment.nu" --welcome-source start
        }

        if $needs_refresh {
            if $verbose_mode {
                print $"♻️  Config changed - rebuilding environment using ($build_parallelism_description)"
            } else if $refresh_output != "quiet" {
                print $"♻️  Config changed - rebuilding environment using ($build_parallelism_description)"
            }
        }

        # Use shared devenv runner (consolidates with yzx env)
        run_in_devenv_shell_command "nu" ...$inner_args --max-jobs $max_jobs --build-cores $build_cores --cwd $yazelix_dir --skip-welcome --verbose=$verbose_mode --force-refresh=($needs_refresh and (not $reuse_mode)) --refresh-output-mode $refresh_output
    }
}

export def start_yazelix_session [cwd_override?: string, --verbose, --setup-only, --reuse] {
    if ($cwd_override | is-not-empty) {
        if $setup_only {
            if $verbose {
                _start_yazelix_impl $cwd_override --verbose --setup-only --reuse=$reuse
            } else {
                _start_yazelix_impl $cwd_override --setup-only --reuse=$reuse
            }
        } else if $verbose {
            _start_yazelix_impl $cwd_override --verbose --reuse=$reuse
        } else {
            _start_yazelix_impl $cwd_override --reuse=$reuse
        }
    } else if $setup_only {
        if $verbose {
            _start_yazelix_impl --verbose --setup-only --reuse=$reuse
        } else {
            _start_yazelix_impl --setup-only --reuse=$reuse
        }
    } else if $verbose {
        _start_yazelix_impl --verbose --reuse=$reuse
    } else {
        _start_yazelix_impl --reuse=$reuse
    }
}

export def main [cwd_override?: string, --verbose, --setup-only, --reuse] {
    if ($cwd_override | is-not-empty) {
        if $setup_only {
            if $verbose {
                _start_yazelix_impl $cwd_override --verbose --setup-only --reuse=$reuse
            } else {
                _start_yazelix_impl $cwd_override --setup-only --reuse=$reuse
            }
        } else if $verbose {
            _start_yazelix_impl $cwd_override --verbose --reuse=$reuse
        } else {
            _start_yazelix_impl $cwd_override --reuse=$reuse
        }
    } else if $setup_only {
        if $verbose {
            _start_yazelix_impl --verbose --setup-only --reuse=$reuse
        } else {
            _start_yazelix_impl --setup-only --reuse=$reuse
        }
    } else if $verbose {
        _start_yazelix_impl --verbose --reuse=$reuse
    } else {
        _start_yazelix_impl --reuse=$reuse
    }
}
