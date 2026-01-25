#!/usr/bin/env nu
# Shared environment bootstrap utilities for Yazelix
# Used by both start_yazelix.nu and yzx env to avoid duplication

use config_parser.nu parse_yazelix_config
use nix_detector.nu ensure_nix_available
use nix_env_helper.nu ensure_nix_in_environment
use common.nu [get_max_cores]
use config_state.nu [compute_config_state mark_config_state_applied]

# Check if already in Yazelix or Nix environment
export def check_environment_status [] {
    let already_in_env = (
        ($env.IN_YAZELIX_SHELL? == "true")
        or ($env.IN_NIX_SHELL? | is-not-empty)
    )

    {
        already_in_env: $already_in_env
        in_nix_shell: ($env.IN_NIX_SHELL? | is-not-empty)
        in_yazelix_shell: ($env.IN_YAZELIX_SHELL? == "true")
    }
}

# Ensure Nix environment is available
export def ensure_environment_available [] {
    let config = parse_yazelix_config
    let env_mode = ($config.environment_mode? | default "nix")
    if $env_mode == "system" {
        return
    }

    let env_status = check_environment_status

    if not $env_status.already_in_env {
        # If automatic setup fails, fall back to the detector with user interaction
        if not (ensure_nix_in_environment) {
            ensure_nix_available
        }
    }
}

# Run a command inside devenv shell
export def run_in_devenv_shell [
    command: string
    --env-only          # Set YAZELIX_ENV_ONLY=true
    --verbose           # Enable verbose output
    --quiet             # Run devenv with --quiet flag
    --skip-welcome      # Set YAZELIX_SKIP_WELCOME=true
    --force-refresh     # Force environment refresh
] {
    let config = parse_yazelix_config
    let env_mode = ($config.environment_mode? | default "nix")
    if $env_mode == "system" {
        print "Error: environment.mode = \"system\" disables devenv execution"
        exit 1
    }

    let env_status = check_environment_status
    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")

    if $verbose_mode {
        print $"🔁 IN_NIX_SHELL? ($env_status.in_nix_shell) | IN_YAZELIX_SHELL? ($env_status.in_yazelix_shell)"
    }

    if $env_status.already_in_env {
        # Already in a managed shell, run command directly to avoid recursive nesting
        if $verbose_mode {
            print "⚙️ Executing command directly in existing environment"
        }
        ^sh -c $command
    } else {
        # Not in managed shell, enter devenv first
        if (which devenv | is-empty) {
            print ""
            print "❌ devenv command not found."
            print "   Yazelix v11+ moved from flake-based `nix develop` shells to devenv."
            print "   Install devenv with:"
            print "     nix profile install github:cachix/devenv/latest"
            print "   After installing, relaunch Yazelix (or run `devenv shell --impure`)."
            print ""
            exit 1
        }

        if $verbose_mode {
            print "⚙️ Entering devenv shell before running command"
        }

        let yazelix_dir = "~/.config/yazelix"
        let max_cores = get_max_cores

        # Build devenv command with optional flags
        mut devenv_flags = ["--impure", "--cores", $max_cores]
        if $quiet {
            $devenv_flags = ($devenv_flags | prepend "--quiet")
        }

        let devenv_flags_str = ($devenv_flags | str join " ")
        let devenv_cmd = $"cd ($yazelix_dir) && devenv ($devenv_flags_str) shell -- sh -c '($command)'"

        # Build environment variables
        mut env_vars = {}
        if $env_only {
            $env_vars = ($env_vars | insert YAZELIX_ENV_ONLY "true")
        }
        if $skip_welcome {
            $env_vars = ($env_vars | insert YAZELIX_SKIP_WELCOME "true")
        }
        if $force_refresh {
            $env_vars = ($env_vars | insert YAZELIX_FORCE_REFRESH "true")
        }
        if $verbose_mode {
            $env_vars = ($env_vars | insert YAZELIX_VERBOSE "true")
        }

        if ($env_vars | is-empty) {
            ^sh -c $devenv_cmd
        } else {
            with-env $env_vars {
                ^sh -c $devenv_cmd
            }
        }
    }
}

# Run a command with args inside devenv shell (no string interpolation)
export def run_in_devenv_shell_command [
    command: string
    ...args: string
    --cwd: string      # Run command in this directory
    --env-only         # Set YAZELIX_ENV_ONLY=true
    --verbose          # Enable verbose output
    --quiet            # Run devenv with --quiet flag
    --skip-welcome     # Set YAZELIX_SKIP_WELCOME=true
    --force-refresh    # Force environment refresh
] {
    let config = parse_yazelix_config
    let env_mode = ($config.environment_mode? | default "nix")
    if $env_mode == "system" {
        print "Error: environment.mode = \"system\" disables devenv execution"
        exit 1
    }

    let env_status = check_environment_status
    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")

    if ($command | is-empty) {
        print "Error: No command provided"
        exit 1
    }

    if (which env | is-empty) {
        print "Error: env command not found - cannot run command in devenv shell"
        exit 1
    }

    let exec_cmd = if ($cwd | is-not-empty) {
        ["env", "-C", $cwd] | append $command | append $args
    } else {
        [$command] | append $args
    }
    let exec_bin = ($exec_cmd | first)
    let exec_args = ($exec_cmd | skip 1)

    if $env_status.already_in_env {
        if $verbose_mode {
            print "⚙️ Executing command directly in existing environment"
        }
        ^$exec_bin ...$exec_args
        return
    }

    if (which devenv | is-empty) {
        print ""
        print "❌ devenv command not found."
        print "   Yazelix v11+ moved from flake-based `nix develop` shells to devenv."
        print "   Install devenv with:"
        print "     nix profile install github:cachix/devenv/latest"
        print "   After installing, relaunch Yazelix (or run `devenv shell --impure`)."
        print ""
        exit 1
    }

    if $verbose_mode {
        print "⚙️ Entering devenv shell before running command"
    }

    let home = $env.HOME
    if ($home | is-empty) or (not ($home | path exists)) {
        print "Error: Cannot resolve HOME directory"
        exit 1
    }
    let yazelix_dir = $"($home)/.config/yazelix"
    if not ($yazelix_dir | path exists) {
        print $"Error: Cannot find Yazelix directory at ($yazelix_dir)"
        exit 1
    }
    let max_cores = get_max_cores

    mut devenv_flags = ["--impure", "--cores", $max_cores]
    if $quiet {
        $devenv_flags = ($devenv_flags | prepend "--quiet")
    }
    let devenv_cmd = (["env", "-C", $yazelix_dir, "devenv"] | append $devenv_flags | append ["shell", "--"] | append $exec_cmd)
    let devenv_bin = ($devenv_cmd | first)
    let devenv_args = ($devenv_cmd | skip 1)

    mut env_vars = {}
    if $env_only {
        $env_vars = ($env_vars | insert YAZELIX_ENV_ONLY "true")
    }
    if $skip_welcome {
        $env_vars = ($env_vars | insert YAZELIX_SKIP_WELCOME "true")
    }
    if $force_refresh {
        $env_vars = ($env_vars | insert YAZELIX_FORCE_REFRESH "true")
    }
    if $verbose_mode {
        $env_vars = ($env_vars | insert YAZELIX_VERBOSE "true")
    }

    if ($env_vars | is-empty) {
        ^$devenv_bin ...$devenv_args
    } else {
        with-env $env_vars {
            ^$devenv_bin ...$devenv_args
        }
    }
}

# Prepare environment (parse config, check state)
export def prepare_environment [--verbose] {
    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")

    # Parse configuration
    let config = parse_yazelix_config

    # Compute config state
    let config_state = compute_config_state

    if $verbose_mode {
        print "🔍 Environment prepared"
        print $"   Config file: ($config_state.config_file)"
        print $"   Needs refresh: ($config_state.needs_refresh)"
    }

    {
        config: $config
        config_state: $config_state
        needs_refresh: $config_state.needs_refresh
    }
}
