#!/usr/bin/env nu
# Yazelix Command Suite
# Consolidated commands for managing and interacting with yazelix

use ../utils/config_manager.nu *
use ../utils/constants.nu *
use ../utils/version_info.nu *
use ../utils/config_parser.nu parse_yazelix_config
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ./start_yazelix.nu [start_yazelix_session]

# =============================================================================
# YAZELIX COMMANDS WITH NATIVE SUBCOMMAND SUPPORT
# =============================================================================

# Yazelix Command Suite - Yazi + Zellij + Helix integrated terminal environment
#
# Manage yazelix sessions, run diagnostics, and configure your setup.
# Supports: bash, nushell, fish, zsh
#
# Common commands:
#   yzx launch    - Start a new yazelix session
#   yzx doctor    - Run health checks
#   yzx profile   - Profile launch performance
#   yzx test      - Run test suite
#   yzx versions  - Show tool versions
export def yzx [
    --version (-V)  # Show version information
] {
    if $version {
        print $"Yazelix ($YAZELIX_VERSION)"
        return
    }
    help yzx
}

# Elevator pitch: Why Yazelix
export def "yzx why" [] {
    print "Yazelix is a reproducible terminal IDE (Yazi + Zellij + Helix) with:"
    print "• Zero‑conflict keybindings, zjstatus, smooth Yazi↔editor flows"
    print "• Top terminals (Ghostty/WezTerm/Kitty/Alacritty) and shells (Bash/Zsh/Fish/Nushell)"
    print "• One‑file config (Nix) with sane defaults and curated packs"
    print "• Remote‑ready over SSH; same superterminal on barebones hosts"
    print "• Git and tooling preconfigured (lazygit, starship, zoxide, carapace)"
    print "Get everything running in <10 minutes. No extra deps, only Nix."
    print "Install once, get the same environment everywhere."
}

# Show configuration status (canonical, no aliases)
export def "yzx config_status" [shell?: string] {
    if ($shell | is-empty) {
        show_config_status ~/.config/yazelix
    } else {
        let config_file = ($SHELL_CONFIGS | get $shell | str replace "~" $env.HOME)
        if not ($config_file | path exists) {
            print $"❌ Config file not found: ($config_file)"
            return
        }
        let section = extract_yazelix_section $config_file
        if $section.exists {
            print $"=== Yazelix Section in ($shell) ==="
            print $section.content
            print "=================================="
        } else {
            print $"❌ No yazelix section found in ($config_file)"
        }
        $section
    }
}

# List available versions
export def "yzx versions" [] {
    nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu
}

# Show system info
export def "yzx info" [] {
    # Parse configuration using the shared module
    let config = parse_yazelix_config

    print "=== Yazelix Information ==="
    print $"Version: ($YAZELIX_VERSION)"
    print $"Description: ($YAZELIX_DESCRIPTION)"
    print $"Directory: ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME)"
    print $"Logs: ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)"
    print $"Default Shell: ($config.default_shell)"
    print $"Preferred Terminal: ($config.preferred_terminal)"
    print $"Helix Mode: ($config.helix_mode)"
    print $"Persistent Sessions: ($config.persistent_sessions)"
    if ($config.persistent_sessions == "true") {
        print $"Session Name: ($config.session_name)"
    }
    print "=========================="
}

# Launch yazelix
export def "yzx launch" [
    --here             # Start in current terminal instead of launching new terminal
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --terminal(-t): string  # Override terminal selection (for sweep testing)
    --verbose          # Enable verbose logging
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")
    if $verbose_mode {
        print "🔍 yzx launch: verbose mode enabled"
    }

    let config_state = compute_config_state
    let needs_refresh = $config_state.needs_refresh
    if $verbose_mode {
        print $"🔍 Config hash changed? ($needs_refresh)"
    }

    if $here {
        # Start in current terminal without spawning a new process
        $env.YAZELIX_ENV_ONLY = "false"

        let cwd_override = if $home {
            $env.HOME
        } else if ($path != null) {
            $path
        } else {
            null
        }

        if ($cwd_override != null) {
            if $verbose {
                if $needs_refresh {
                    with-env {YAZELIX_FORCE_REFRESH: "true"} {
                        start_yazelix_session $cwd_override --verbose
                    }
                } else {
                    start_yazelix_session $cwd_override --verbose
                }
            } else {
                if $needs_refresh {
                    with-env {YAZELIX_FORCE_REFRESH: "true"} {
                        start_yazelix_session $cwd_override
                    }
                } else {
                    start_yazelix_session $cwd_override
                }
            }
        } else if $verbose {
            if $needs_refresh {
                with-env {YAZELIX_FORCE_REFRESH: "true"} {
                    start_yazelix_session --verbose
                }
            } else {
                start_yazelix_session --verbose
            }
        } else {
            if $needs_refresh {
                with-env {YAZELIX_FORCE_REFRESH: "true"} {
                    start_yazelix_session
                }
            } else {
                start_yazelix_session
            }
        }
        if $needs_refresh {
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

        # Check if already in Yazelix environment to skip redundant setup
        let in_yazelix_shell = ($env.IN_YAZELIX_SHELL? == "true")

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
                let env_record = if $needs_refresh {
                    {YAZELIX_VERBOSE: "true", YAZELIX_FORCE_REFRESH: "true"}
                } else {
                    {YAZELIX_VERBOSE: "true"}
                }
                with-env $env_record {
                    ^nu ...$run_args
                }
            } else {
                let final_args = $mut_args
                if $needs_refresh {
                    with-env {YAZELIX_FORCE_REFRESH: "true"} {
                        ^nu ...$final_args
                    }
                } else {
                    ^nu ...$final_args
                }
            }
        } else {
            # Not in Yazelix environment - wrap with devenv shell
            let quote_single = {|text|
                let escaped = ($text | str replace "'" "'\"'\"'")
                $"'" + $escaped + "'"
            }

            mut segments = ["nu"]
            $segments = ($segments | append (do $quote_single $launch_script))
            if ($launch_cwd | is-not-empty) {
                $segments = ($segments | append (do $quote_single $launch_cwd))
            }
            if ($terminal | is-not-empty) {
                $segments = ($segments | append "--terminal")
                $segments = ($segments | append (do $quote_single $terminal))
            }
            if $verbose_mode {
                $segments = ($segments | append "--verbose")
            }

            let launch_cmd = ($segments | str join " ")
            # Build environment variable exports for bash
            let env_exports = [
                (if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) { $"export YAZELIX_CONFIG_OVERRIDE='($env.YAZELIX_CONFIG_OVERRIDE)'; " } else { "" })
                (if ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) { $"export ZELLIJ_DEFAULT_LAYOUT='($env.ZELLIJ_DEFAULT_LAYOUT)'; " } else { "" })
                (if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) { $"export YAZELIX_SWEEP_TEST_ID='($env.YAZELIX_SWEEP_TEST_ID)'; " } else { "" })
                (if ($env.YAZELIX_SKIP_WELCOME? | is-not-empty) { $"export YAZELIX_SKIP_WELCOME='($env.YAZELIX_SKIP_WELCOME)'; " } else { "" })
                (if ($env.YAZELIX_TERMINAL? | is-not-empty) { $"export YAZELIX_TERMINAL='($env.YAZELIX_TERMINAL)'; " } else { "" })
                (if $needs_refresh { "export YAZELIX_FORCE_REFRESH='true'; " } else { "" })
                (if $verbose_mode { "export YAZELIX_VERBOSE='true'; " } else { "" })
            ] | str join ""

            let full_cmd = $"($env_exports)($launch_cmd)"
            if (which devenv | is-empty) {
                print "❌ devenv command not found - install devenv to launch Yazelix."
                print "   See https://devenv.sh/getting-started/ for installation instructions."
                exit 1
            }
            if $verbose_mode {
                print $"⚙️ devenv shell command: ($full_cmd)"
            }

            # Must run devenv from the directory containing devenv.nix
            let yazelix_dir = "~/.config/yazelix"
            if $needs_refresh and $verbose_mode {
                print "♻️  Config changed since last launch – rebuilding environment"
            }
            let devenv_cmd = $"cd ($yazelix_dir) && devenv shell --impure -- bash -c '($full_cmd)'"
            ^bash -c $devenv_cmd
            if $needs_refresh {
                mark_config_state_applied $config_state
            }
        }
}

# Load yazelix environment without UI
export def "yzx env" [
    --no-shell(-n)  # Keep current shell instead of launching configured shell
    --command(-c): string  # Run a command in the Yazelix environment
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    if (which devenv | is-empty) {
        print "❌ devenv command not found - install devenv to load the Yazelix environment."
        print "   See https://devenv.sh/getting-started/ for installation instructions."
        exit 1
    }

    let config_state = (try {
        compute_config_state
    } catch {|err|
        print $"❌ Failed to evaluate Yazelix config: ($err.msg)"
        exit 1
    })
    let needs_refresh = $config_state.needs_refresh
    let config = $config_state.config

    let yazelix_dir = "~/.config/yazelix"

    if ($command | is-not-empty) {
        # Run command in Yazelix environment (skip welcome screen for automation)
        with-env {YAZELIX_ENV_ONLY: "true", YAZELIX_SKIP_WELCOME: "true"} {
            let devenv_cmd = $"cd ($yazelix_dir) && devenv shell --impure -- bash -c '($command)'"
            if $needs_refresh {
                with-env {YAZELIX_FORCE_REFRESH: "true"} {
                    ^bash -c $devenv_cmd
                }
            } else {
                ^bash -c $devenv_cmd
            }
        }
        if $needs_refresh {
            mark_config_state_applied $config_state
        }
    } else if $no_shell {
        with-env {YAZELIX_ENV_ONLY: "true"} {
            let devenv_cmd = $"cd ($yazelix_dir) && devenv shell --impure"
            if $needs_refresh {
                with-env {YAZELIX_FORCE_REFRESH: "true"} {
                    ^bash -c $devenv_cmd
                }
            } else {
                ^bash -c $devenv_cmd
            }
        }
        if $needs_refresh {
            mark_config_state_applied $config_state
        }
    } else {
        let shell_name = ($config.default_shell? | default "nu" | str downcase)
        let shell_command = match $shell_name {
            "nu" => ["nu" "--login"]
            "bash" => ["bash" "--login"]
            "fish" => ["fish" "-l"]
            "zsh" => ["zsh" "-l"]
            _ => [$shell_name]
        }
        let shell_exec = ($shell_command | first)
        let command_str = ($shell_command | str join " ")
        let exec_command = $"exec ($command_str)"
        with-env {YAZELIX_ENV_ONLY: "true", SHELL: $shell_exec} {
            try {
                let devenv_cmd = $"cd ($yazelix_dir) && devenv shell --impure -- bash -lc '($exec_command)'"
                if $needs_refresh {
                    with-env {YAZELIX_FORCE_REFRESH: "true"} {
                        ^bash -c $devenv_cmd
                    }
                } else {
                    ^bash -c $devenv_cmd
                }
            } catch {|err|
                print $"❌ Failed to launch configured shell: ($err.msg)"
                print "   Tip: rerun with 'yzx env --no-shell' to stay in your current shell."
                exit 1
            }
        }
        if $needs_refresh {
            mark_config_state_applied $config_state
        }
    }
}

# Restart yazelix
export def "yzx restart" [] {
    # Parse configuration using the shared module
    let config = parse_yazelix_config

    if ($config.persistent_sessions == "true") {
        print $"Persistent sessions are enabled \(session: ($config.session_name)\)"
        print "yzx restart is disabled when persistent sessions are enabled."
        print "Your session will persist automatically - no restart needed."
        print ""
        print "To start a new session, use: yzx launch --here"
        print $"To kill the current session, use: zellij kill-session ($config.session_name)"
    } else {
        print "Attempting to kill the current Zellij session..."
        let current_session = (zellij list-sessions | lines | where $it =~ 'current' | first | split row " " | first)
        let clean_session = ($current_session | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' '')
        print "Restarting Yazelix..."
        yzx launch
        print "Waiting for Zellij to shut down..."
        sleep 1sec
        if ($clean_session | is-empty) {
            print "No current Zellij session detected. Skipping kill step."
        } else {
            print $"Killing Zellij session: ($clean_session)"
            try { zellij kill-session $clean_session } catch { print $"Failed to kill session: ($clean_session)" }
        }
    }
}

# Run health checks and diagnostics
export def "yzx doctor" [
    --verbose(-v)  # Show detailed information
    --fix(-f)      # Attempt to auto-fix issues
] {
    use ../utils/doctor.nu run_doctor_checks
    run_doctor_checks $verbose $fix
}

# Update dependencies and inputs
export def "yzx update" [] {
    print "Yazelix update commands:"
    print "  yzx update devenv  # Refresh devenv.lock using devenv update"
    print "  yzx update nix     # Alias for devenv update (refresh Yazelix dependencies)"
}

export def "yzx update devenv" [
    --verbose  # Show the underlying devenv command
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    if (which devenv | is-empty) {
        print "❌ devenv command not found - install devenv to manage Yazelix dependencies."
        print "   See https://devenv.sh/getting-started/ for installation instructions."
        exit 1
    }

    let yazelix_dir = "~/.config/yazelix" | path expand
    let command = $"cd ($yazelix_dir) && devenv update"

    if $verbose {
        print $"⚙️ Running: ($command)"
    } else {
        print "🔄 Updating devenv inputs (this may take a moment)..."
    }

    try {
        ^bash -c $command
        print "✅ devenv.lock updated. Review and commit the changes if everything looks good."
    } catch {|err|
        print $"❌ devenv update failed: ($err.msg)"
        print "   Check your network connection and devenv.yaml inputs, then try again."
        exit 1
    }
}

export def "yzx update nix" [
    --verbose  # Show the underlying devenv command
] {
    if $verbose {
        yzx update devenv --verbose
    } else {
        yzx update devenv
    }
}

# Run configuration sweep tests across shell/terminal combinations
export def "yzx sweep" [] {
    print "Run 'yzx sweep --help' to see available subcommands"
}

# Test all shell combinations
export def "yzx sweep shells" [
    --verbose(-v)  # Show detailed output
] {
    use ../dev/test_config_sweep.nu run_all_sweep_tests

    if $verbose {
        run_all_sweep_tests --verbose
    } else {
        run_all_sweep_tests
    }
}

# Test all terminal launches
export def "yzx sweep terminals" [
    --verbose(-v)       # Show detailed output
    --delay: int = 3    # Delay between terminal launches in seconds
] {
    use ../dev/test_config_sweep.nu run_all_sweep_tests

    run_all_sweep_tests --visual --verbose=$verbose --visual-delay $delay
}

# Run all sweep tests (shells + terminals)
export def "yzx sweep all" [
    --verbose(-v)       # Show detailed output
    --delay: int = 3    # Delay between terminal launches in seconds
] {
    print "=== Running All Sweep Tests ==="
    print "Phase 1: Shell combinations (fast)"
    print ""

    yzx sweep shells --verbose=$verbose

    print ""
    print "=== Phase 2: Terminal launches (slow) ==="
    print ""

    yzx sweep terminals --verbose=$verbose --delay $delay
}

# Run Yazelix test suite
export def "yzx test" [
    --verbose(-v)  # Show detailed test output
    --new-window(-n)  # Run tests in a new Yazelix window
    --all(-a)  # Include visual terminal sweep tests
] {
    use ../utils/test_runner.nu run_all_tests
    run_all_tests --verbose=$verbose --new-window=$new_window --all=$all
}

# Benchmark terminal launch performance
export def "yzx bench" [
    --iterations(-n): int = 1  # Number of iterations per terminal
    --terminal(-t): string     # Test only specific terminal
    --verbose(-v)              # Show detailed output
] {
    mut args = ["--iterations", $iterations]

    if ($terminal | is-not-empty) {
        $args = ($args | append ["--terminal", $terminal])
    }

    if $verbose {
        $args = ($args | append "--verbose")
    }

    nu $"($env.HOME)/.config/yazelix/nushell/scripts/dev/benchmark_terminals.nu" ...$args
}

# Profile launch sequence and identify bottlenecks
export def "yzx profile" [
    --cold(-c)        # Profile cold launch from vanilla terminal (emulates desktop entry or fresh terminal launch)
    --clear-cache     # Toggle yazelix.toml option and clear cache to force full Nix re-evaluation (simulates config change)
] {
    use ../utils/profile.nu *

    if $cold {
        profile_cold_launch --clear-cache=$clear_cache
    } else {
        profile_launch
    }
}
