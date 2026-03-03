#!/usr/bin/env nu
# yzx refresh command - Refresh Yazelix devenv cache/environment without launching UI

use ../utils/environment_bootstrap.nu [prepare_environment get_devenv_base_command is_unfree_enabled]
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]

# Refresh devenv evaluation cache without launching Yazelix UI
export def "yzx refresh" [
    --force(-f)    # Force refresh even when no config/input changes are detected
    --verbose(-v)  # Show detailed refresh diagnostics
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let env_prep = prepare_environment --verbose=$verbose
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

    let allow_unfree = is_unfree_enabled
    let devenv_base = get_devenv_base_command --quiet=(not $verbose) --refresh-eval-cache
    let devenv_cmd = ($devenv_base | append ["shell", "--", "true"])

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
