#!/usr/bin/env nu
# yzx refresh command - Refresh Yazelix devenv cache/environment without launching UI

use ../utils/environment_bootstrap.nu prepare_environment
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/common.nu [get_max_cores]

# Refresh devenv evaluation cache without launching Yazelix UI
export def "yzx refresh" [
    --force(-f)    # Force refresh even when no config/input changes are detected
    --verbose(-v)  # Show detailed refresh diagnostics
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let env_prep = prepare_environment --verbose=$verbose
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let needs_refresh = $config_state.needs_refresh

    if (not $force) and (not $needs_refresh) {
        print "✅ Yazelix environment is already up to date."
        print "   Nothing to refresh."
        return
    }

    let refresh_reason = if $force {
        "manual refresh requested"
    } else {
        $config_state.refresh_reason? | default "config or devenv inputs changed since last launch"
    }

    print $"♻️  Refreshing Yazelix environment \(($refresh_reason)\)..."

    let yazelix_dir = "~/.config/yazelix" | path expand
    if not ($yazelix_dir | path exists) {
        print $"❌ Yazelix directory not found: ($yazelix_dir)"
        exit 1
    }

    let max_cores = get_max_cores
    let allow_unfree = (($config.pack_names? | default []) | any { |name| $name == "unfree" })

    mut devenv_cmd = [
        "env"
        "-C"
        $yazelix_dir
        "devenv"
        "--impure"
        "--cores"
        ($max_cores | into string)
    ]

    if not $verbose {
        $devenv_cmd = ($devenv_cmd | append "--quiet")
    }

    $devenv_cmd = ($devenv_cmd | append "--refresh-eval-cache" | append "shell" | append "--" | append "true")

    let cmd_bin = ($devenv_cmd | first)
    let cmd_args = ($devenv_cmd | skip 1)

    if $verbose {
        print $"⚙️ Running: ($devenv_cmd | str join ' ')"
    }

    let result = if $allow_unfree {
        with-env {NIXPKGS_ALLOW_UNFREE: "1"} {
            ^$cmd_bin ...$cmd_args | complete
        }
    } else {
        ^$cmd_bin ...$cmd_args | complete
    }

    if $result.exit_code != 0 {
        let stderr = ($result.stderr | str trim)
        print $"❌ Refresh failed \(exit code: ($result.exit_code)\)"
        if ($stderr | is-not-empty) {
            print $stderr
        }
        exit 1
    }

    mark_config_state_applied (compute_config_state)

    print "✅ Refresh completed."
    print "⚠️  Your current shell keeps its existing environment."
    print "   To use updated env vars/tools now, run 'yzx restart' or open a new Zellij pane."
}
