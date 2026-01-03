#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZI_CONFIG_PATHS, YAZELIX_ENV_VARS]
use ../utils/environment_bootstrap.nu *
use ../setup/zellij_config_merger.nu generate_merged_zellij_config
use ../setup/yazi_config_merger.nu generate_merged_yazi_config

def _start_yazelix_impl [cwd_override?: string, --verbose, --setup-only] {
    # Capture original directory before any cd commands
    let original_dir = pwd

    # Ensure environment is available (shared with yzx env)
    ensure_environment_available

    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")
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

    cd $yazelix_dir

    # Parse configuration using the shared module
    let env_prep = prepare_environment --verbose=$verbose_mode
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh

    # If setup-only mode, just run devenv shell to install hooks and exit
    if $setup_only {
        print "🔧 Setting up Yazelix environment (installing shell hooks and dependencies)..."
        print "   This may take several minutes on first run."

        run_in_devenv_shell "echo '✅ Setup complete! Shell hooks installed.'" --verbose=$verbose_mode --force-refresh=$needs_refresh

        print ""
        print "📝 Next steps:"
        print "   1. Restart your shell (or source your shell config)"
        print "   2. Run 'yzx launch' to start Yazelix"
        print ""
        return
    }

    # Generate merged Yazi configuration (doesn't need zellij)
    print "🔧 Preparing Yazi configuration..."
    let merged_yazi_dir = if $verbose_mode {
        generate_merged_yazi_config $yazelix_dir
    } else {
        generate_merged_yazi_config $yazelix_dir --quiet
    }
    
    # For Zellij config, create a placeholder for now - will be generated inside Nix environment
    let merged_zellij_dir = $"($env.HOME)/.local/share/yazelix/configs/zellij"

    # Determine which directory to use as default CWD
    # Priority: 1. cwd_override parameter 2. YAZELIX_LAUNCH_CWD env var 3. original directory
    let working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else if ($env.YAZELIX_LAUNCH_CWD? | is-not-empty) {
        $env.YAZELIX_LAUNCH_CWD
    } else {
        $original_dir
    }

    # Build the command that first generates the zellij config, then starts zellij
    let zellij_merger_cmd = $"nu ($yazelix_dir)/nushell/scripts/setup/zellij_config_merger.nu ($yazelix_dir)"

    # Check for layout override (for testing), default to constant
    let layout = if ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
        $env.ZELLIJ_DEFAULT_LAYOUT
    } else {
        $YAZELIX_ENV_VARS.ZELLIJ_DEFAULT_LAYOUT
    }
    # Resolve layout to an absolute file path so it works even if user config overrides layout_dir
    let layout_path = if ($layout | str contains "/") or ($layout | str ends-with ".kdl") {
        $layout
    } else {
        $"($merged_zellij_dir)/layouts/($layout).kdl"
    }

    let cmd = if ($config.persistent_sessions == "true") {
        # Use zellij attach with create flag for persistent sessions
        [
            $zellij_merger_cmd "&&"
            "zellij"
            "--config-dir" $merged_zellij_dir
            "attach"
            "-c" $config.session_name
            "options"
            "--default-cwd" $working_dir
            "--default-layout" $layout_path
            "--pane-frames" "false"
            "--default-shell" $config.default_shell
        ] | str join " "
    } else {
        # For new sessions, apply options explicitly
        [
            $zellij_merger_cmd "&&"
            "zellij"
            "--config-dir" $merged_zellij_dir
            "options"
            "--default-cwd" $working_dir
            "--default-layout" $layout_path
            "--pane-frames" "false"
            "--default-shell" $config.default_shell
        ] | str join " "
    }

    if $verbose_mode {
        print $"🔁 zellij command: ($cmd)"
    }

    # Run devenv shell with explicit HOME.
    # The default shell is dynamically read from yazelix.toml configuration
    # and passed directly to the zellij command.
    with-env {HOME: $home} {
        if $verbose_mode and $needs_refresh {
            print "♻️  Config changed – rebuilding environment"
        }

        # Use shared devenv runner (consolidates with yzx env)
        run_in_devenv_shell $cmd --verbose=$verbose_mode --force-refresh=$needs_refresh
    }
}

export def start_yazelix_session [cwd_override?: string, --verbose, --setup-only] {
    if ($cwd_override | is-not-empty) {
        if $setup_only {
            if $verbose {
                _start_yazelix_impl $cwd_override --verbose --setup-only
            } else {
                _start_yazelix_impl $cwd_override --setup-only
            }
        } else if $verbose {
            _start_yazelix_impl $cwd_override --verbose
        } else {
            _start_yazelix_impl $cwd_override
        }
    } else if $setup_only {
        if $verbose {
            _start_yazelix_impl --verbose --setup-only
        } else {
            _start_yazelix_impl --setup-only
        }
    } else if $verbose {
        _start_yazelix_impl --verbose
    } else {
        _start_yazelix_impl
    }
}

export def main [cwd_override?: string, --verbose, --setup-only] {
    if ($cwd_override | is-not-empty) {
        if $setup_only {
            if $verbose {
                _start_yazelix_impl $cwd_override --verbose --setup-only
            } else {
                _start_yazelix_impl $cwd_override --setup-only
            }
        } else if $verbose {
            _start_yazelix_impl $cwd_override --verbose
        } else {
            _start_yazelix_impl $cwd_override
        }
    } else if $setup_only {
        if $verbose {
            _start_yazelix_impl --verbose --setup-only
        } else {
            _start_yazelix_impl --setup-only
        }
    } else if $verbose {
        _start_yazelix_impl --verbose
    } else {
        _start_yazelix_impl
    }
}
