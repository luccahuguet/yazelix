#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZI_CONFIG_PATHS, YAZELIX_ENV_VARS]
use ../utils/nix_detector.nu ensure_nix_available
use ../utils/common.nu [get_max_cores]
use ../setup/zellij_config_merger.nu generate_merged_zellij_config
use ../setup/yazi_config_merger.nu generate_merged_yazi_config

def _start_yazelix_impl [cwd_override?: string, --verbose] {
    # Try to set up Nix environment automatically when outside Yazelix/nix shells
    use ../utils/nix_env_helper.nu ensure_nix_in_environment

    let already_in_env = (
        ($env.IN_YAZELIX_SHELL? == "true")
        or ($env.IN_NIX_SHELL? | is-not-empty)
    )

    if not $already_in_env {
        # If automatic setup fails, fall back to the detector with user interaction
        if not (ensure_nix_in_environment) {
            ensure_nix_available
        }
    }

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
    let config = parse_yazelix_config

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
    # Priority: 1. cwd_override parameter 2. YAZELIX_LAUNCH_CWD env var 3. current directory 4. home
    let working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else if ($env.YAZELIX_LAUNCH_CWD? | is-not-empty) {
        $env.YAZELIX_LAUNCH_CWD
    } else {
        pwd
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
    # Guard against recursive environment initialization when already in a managed shell
    with-env {HOME: $home} {
        let in_nix_shell = ($env.IN_NIX_SHELL? | is-not-empty)
        let in_yazelix_shell = ($env.IN_YAZELIX_SHELL? == "true")

        if $verbose_mode {
            print $"🔁 IN_NIX_SHELL? ($in_nix_shell) | IN_YAZELIX_SHELL? ($in_yazelix_shell)"
            if ($in_nix_shell or $in_yazelix_shell) {
                print "⚙️ Executing zellij command directly"
            } else {
                print "⚙️ Entering devenv shell before running zellij command"
            }
        }

        if ($in_nix_shell or $in_yazelix_shell) {
            # Already in a managed shell, run command directly to avoid recursive nesting
            ^bash -c $cmd
        } else {
            # Not in managed shell, enter devenv first
            if (which devenv | is-empty) {
                print ""
                print "❌ devenv command not found."
                print "   Yazelix v11+ moved from flake-based `nix develop` shells to devenv."
                print "   Install devenv with:"
                print "     nix profile install github:cachix/devenv/latest"
                print "   After installing, relaunch Yazelix (or run `devenv shell --impure`)."
                print "   Old commands like `nix develop` are no longer supported."
                print ""
                exit 1
            }
            # Must run devenv from the directory containing devenv.nix
            if ($env.YAZELIX_FORCE_REFRESH? == "true") and $verbose_mode {
                print "♻️  Config changed – rebuilding environment"
            }
            let max_cores = get_max_cores
            let devenv_cmd = $"cd ($yazelix_dir) && devenv --impure --cores ($max_cores) shell -- bash -c '($cmd)'"
            ^bash -c $devenv_cmd
        }
    }
}

export def start_yazelix_session [cwd_override?: string, --verbose] {
    if ($cwd_override | is-not-empty) {
        if $verbose {
            _start_yazelix_impl $cwd_override --verbose
        } else {
            _start_yazelix_impl $cwd_override
        }
    } else if $verbose {
        _start_yazelix_impl --verbose
    } else {
        _start_yazelix_impl
    }
}

export def main [cwd_override?: string, --verbose] {
    if ($cwd_override | is-not-empty) {
        if $verbose {
            _start_yazelix_impl $cwd_override --verbose
        } else {
            _start_yazelix_impl $cwd_override
        }
    } else if $verbose {
        _start_yazelix_impl --verbose
    } else {
        _start_yazelix_impl
    }
}
