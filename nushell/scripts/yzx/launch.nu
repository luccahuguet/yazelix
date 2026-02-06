#!/usr/bin/env nu
# yzx launch command - Launch Yazelix in new or current terminal

use ../utils/config_state.nu [mark_config_state_applied]
use ../utils/common.nu [get_max_cores]
use ../utils/environment_bootstrap.nu prepare_environment
use ../core/start_yazelix.nu [start_yazelix_session]

# Check if unfree pack is enabled in yazelix.toml
def is_unfree_enabled [] {
    let yazelix_dir = "~/.config/yazelix" | path expand
    let toml_file = ($yazelix_dir | path join "yazelix.toml")
    let default_toml = ($yazelix_dir | path join "yazelix_default.toml")
    let config_file = if ($toml_file | path exists) { $toml_file } else { $default_toml }
    let raw_config = open $config_file
    let pack_names = ($raw_config.packs?.enabled? | default [])
    $pack_names | any { |name| $name == "unfree" }
}

# Launch yazelix
export def "yzx launch" [
    --here             # Start in current terminal instead of launching new terminal
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --terminal(-t): string  # Override terminal selection (for sweep testing)
    --verbose          # Enable verbose logging
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let verbose_mode = $verbose or ($env.YAZELIX_VERBOSE? == "true")
    if $verbose_mode {
        print "🔍 yzx launch: verbose mode enabled"
    }

    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    mut needs_refresh = $env_prep.needs_refresh
    let manage_terminals = ($config.manage_terminals? | default true)
    if $verbose_mode {
        print $"🔍 Config hash changed? ($needs_refresh)"
    }

    let force_reenter = ($env.YAZELIX_FORCE_REENTER? == "true")
    mut in_yazelix_shell = ($env.IN_YAZELIX_SHELL? == "true")
    if $manage_terminals and $needs_refresh and $in_yazelix_shell {
        # Only print if not called from yzx restart (which already printed the message)
        if not $force_reenter {
            print "🔄 Configuration changed - rebuilding environment to install terminals..."
        }
        $in_yazelix_shell = false
    }
    if $force_reenter {
        $in_yazelix_shell = false
    }

    if $here {
        # Start in current terminal without spawning a new process
        $env.YAZELIX_ENV_ONLY = "false"

        # Determine directory override: explicit --home or --path, else let start_yazelix handle it
        let cwd_override = if $home {
            $env.HOME
        } else if ($path != null) {
            $path
        } else {
            null
        }

        if $verbose {
            if $needs_refresh {
                with-env {YAZELIX_FORCE_REFRESH: "true"} {
                    if ($cwd_override != null) {
                        start_yazelix_session $cwd_override --verbose
                    } else {
                        start_yazelix_session --verbose
                    }
                }
            } else {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override --verbose
                } else {
                    start_yazelix_session --verbose
                }
            }
        } else {
            if $needs_refresh {
                with-env {YAZELIX_FORCE_REFRESH: "true"} {
                    if ($cwd_override != null) {
                        start_yazelix_session $cwd_override
                    } else {
                        start_yazelix_session
                    }
                }
            } else {
                if ($cwd_override != null) {
                    start_yazelix_session $cwd_override
                } else {
                    start_yazelix_session
                }
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
                let reason = ($config_state.refresh_reason? | default "config or devenv inputs changed since last launch")
                print $"♻️  ($reason) – rebuilding environment"
            }
            let max_cores = get_max_cores
            let unfree_prefix = if (is_unfree_enabled) { "NIXPKGS_ALLOW_UNFREE=1 " } else { "" }
            let devenv_cmd = $"cd ($yazelix_dir) && ($unfree_prefix)devenv --impure --cores ($max_cores) shell -- sh -c '($full_cmd)'"
            ^sh -c $devenv_cmd
            if $needs_refresh {
                mark_config_state_applied $config_state
            }
        }
}
