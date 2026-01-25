#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/constants.nu YAZELIX_ENV_VARS
use ../utils/environment_bootstrap.nu *
use ../utils/system_mode.nu [assert_no_packs require_command]

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
    let env_mode = ($config.environment_mode? | default "nix")

    if $env_mode == "system" {
        assert_no_packs $config
        require_command "zellij" "zellij"
        require_command "yazi" "yazi"
        require_command ($config.default_shell? | default "nu") "shell.default_shell"

        let editor_cmd = ($config.editor_command? | default "")
        if ($editor_cmd | is-empty) {
            print "Error: environment.mode = \"system\" requires editor.command"
            print "Set [editor].command to a system editor (e.g., \"hx\" or \"nvim\")."
            exit 1
        }
        require_command $editor_cmd "editor.command"
    }

    # If setup-only mode, just run devenv shell to install hooks and exit
    if $setup_only {
        if $env_mode == "system" {
            print "Error: --setup-only is not available in system mode"
            exit 1
        }
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

    let inner_script = $"($yazelix_dir)/nushell/scripts/core/start_yazelix_inner.nu"
    let inner_args = if ($working_dir | is-not-empty) {
        [$inner_script, $working_dir, $layout_path]
    } else {
        [$inner_script, "", $layout_path]
    }
    let cmd = $"nu -i \"($inner_args | get 0)\" \"($inner_args | get 1)\" \"($inner_args | get 2)\""

    # Run devenv shell with explicit HOME.
    # The default shell is dynamically read from yazelix.toml configuration
    # and passed directly to the zellij command.
    with-env {HOME: $home, YAZELIX_WELCOME_SOURCE: "start"} {
        if $verbose_mode and $needs_refresh {
            print "♻️  Config changed – rebuilding environment"
        }

        if $env_mode == "system" {
            if $verbose_mode and $needs_refresh {
                print "♻️  Config changed – rebuild skipped in system mode"
            }
            ^nu -i ($inner_args | get 0) ($inner_args | get 1) ($inner_args | get 2)
        } else {
            # Use shared devenv runner (consolidates with yzx env)
            run_in_devenv_shell $cmd --verbose=$verbose_mode --force-refresh=$needs_refresh
        }
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
