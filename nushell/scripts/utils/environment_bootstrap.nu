#!/usr/bin/env nu
# Shared environment bootstrap utilities for Yazelix
# Used by both start_yazelix.nu and yzx env to avoid duplication

use config_parser.nu parse_yazelix_config
use nix_detector.nu ensure_nix_available
use nix_env_helper.nu ensure_nix_in_environment
use common.nu [get_max_cores]
use config_state.nu [compute_config_state mark_config_state_applied]

# Check if unfree pack is enabled in yazelix.toml
export def is_unfree_enabled [] {
    let yazelix_dir = "~/.config/yazelix" | path expand
    let toml_file = ($yazelix_dir | path join "yazelix.toml")
    let default_toml = ($yazelix_dir | path join "yazelix_default.toml")
    let config_file = if ($toml_file | path exists) { $toml_file } else { $default_toml }
    let raw_config = open $config_file
    let pack_names = ($raw_config.packs?.enabled? | default [])
    $pack_names | any { |name| $name == "unfree" }
}

# Resolve absolute Yazelix directory from HOME
def resolve_yazelix_dir [] {
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
    $yazelix_dir
}

# Build a base devenv command from the canonical Yazelix directory
export def get_devenv_base_command [
    --quiet             # Include --quiet in devenv arguments
    --refresh-eval-cache  # Include --refresh-eval-cache in devenv arguments
] {
    let yazelix_dir = resolve_yazelix_dir
    let max_cores = get_max_cores

    mut cmd = [
        "env"
        "-C"
        $yazelix_dir
        "devenv"
        "--cores"
        ($max_cores | into string)
    ]

    if $quiet {
        $cmd = ($cmd | append "--quiet")
    }
    if $refresh_eval_cache {
        $cmd = ($cmd | append "--refresh-eval-cache")
    }

    $cmd
}

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
    let env_status = check_environment_status
    let verbose_mode = $verbose

    if $verbose_mode {
        print $"🔁 IN_NIX_SHELL? ($env_status.in_nix_shell) | IN_YAZELIX_SHELL? ($env_status.in_yazelix_shell)"
    }

    if $env_status.already_in_env and (not $force_refresh) {
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
            print "   After installing, relaunch Yazelix (or run `devenv shell`)."
            print ""
            exit 1
        }

        if $verbose_mode {
            print "⚙️ Entering devenv shell before running command"
        }

        let devenv_base = get_devenv_base_command --quiet=$quiet
        let devenv_cmd = ($devenv_base | append ["shell", "--", "sh", "-c", $command])
        let devenv_bin = ($devenv_cmd | first)
        let devenv_args = ($devenv_cmd | skip 1)

        # Build environment variables
        mut env_vars = {}
        if $env_only {
            $env_vars = ($env_vars | insert YAZELIX_ENV_ONLY "true")
        }
        if $skip_welcome {
            $env_vars = ($env_vars | insert YAZELIX_SKIP_WELCOME "true")
        }
        if (is_unfree_enabled) {
            $env_vars = ($env_vars | insert NIXPKGS_ALLOW_UNFREE "1")
        }

        if ($env_vars | is-empty) {
            ^$devenv_bin ...$devenv_args
        } else {
            with-env $env_vars {
                ^$devenv_bin ...$devenv_args
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
    let env_status = check_environment_status
    let verbose_mode = $verbose

    if ($command | is-empty) {
        print "Error: No command provided"
        exit 1
    }

    if (which env | is-empty) {
        print "Error: env command not found - cannot run command in devenv shell"
        exit 1
    }

    let resolved_cwd = if ($cwd | is-not-empty) { $cwd | path expand } else { "" }
    let exec_cmd = if ($resolved_cwd | is-not-empty) {
        ["env", "-C", $resolved_cwd] | append $command | append $args
    } else {
        [$command] | append $args
    }
    let exec_bin = ($exec_cmd | first)
    let exec_args = ($exec_cmd | skip 1)

    if $env_status.already_in_env and (not $force_refresh) {
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
        print "   After installing, relaunch Yazelix (or run `devenv shell`)."
        print ""
        exit 1
    }

    if $verbose_mode {
        print "⚙️ Entering devenv shell before running command"
    }

    let devenv_base = get_devenv_base_command --quiet=$quiet
    let devenv_cmd = ($devenv_base | append ["shell", "--"] | append $exec_cmd)
    let devenv_bin = ($devenv_cmd | first)
    let devenv_args = ($devenv_cmd | skip 1)

    mut env_vars = {}
    if $env_only {
        $env_vars = ($env_vars | insert YAZELIX_ENV_ONLY "true")
    }
    if $skip_welcome {
        $env_vars = ($env_vars | insert YAZELIX_SKIP_WELCOME "true")
    }
    if (is_unfree_enabled) {
        $env_vars = ($env_vars | insert NIXPKGS_ALLOW_UNFREE "1")
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
    let verbose_mode = $verbose

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
