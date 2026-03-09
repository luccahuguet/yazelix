#!/usr/bin/env nu
# yzx refresh command - Refresh Yazelix devenv cache/environment without launching UI

use ../utils/environment_bootstrap.nu [prepare_environment get_devenv_base_command is_unfree_enabled get_refresh_output_mode]
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]

def summarize_values [values max_items: int] {
    let normalized = ($values | each { |value| $value | into string })
    let total = ($normalized | length)

    if $total == 0 {
        "none"
    } else if $total <= $max_items {
        $normalized | str join ", "
    } else {
        let shown = ($normalized | first $max_items | str join ", ")
        let remaining = ($total - $max_items)
        $"($shown), +($remaining) more"
    }
}

def get_requested_package_scope [config] {
    let enabled_packs = ($config.pack_names? | default [])
    let pack_declarations = ($config.pack_declarations? | default {})
    let pack_packages = (
        $enabled_packs
        | each { |pack_name|
            $pack_declarations | get -o $pack_name | default []
        }
        | flatten
    )
    let user_packages = ($config.user_packages? | default [])
    let top_level_packages = ($pack_packages | append $user_packages | flatten | uniq)

    {
        enabled_packs: $enabled_packs
        top_level_packages: $top_level_packages
    }
}

def run_refresh_command [devenv_cmd allow_unfree --stream-output] {
    let cmd_bin = ($devenv_cmd | first)
    let cmd_args = ($devenv_cmd | skip 1)

    if $stream_output {
        let exit_code = if $allow_unfree {
            with-env {NIXPKGS_ALLOW_UNFREE: "1"} {
                ^$cmd_bin ...$cmd_args
                ($env.LAST_EXIT_CODE? | default 0)
            }
        } else {
            ^$cmd_bin ...$cmd_args
            ($env.LAST_EXIT_CODE? | default 0)
        }

        {
            exit_code: $exit_code
            stderr: ""
        }
    } else {
        let result = if $allow_unfree {
            with-env {NIXPKGS_ALLOW_UNFREE: "1"} {
                ^$cmd_bin ...$cmd_args | complete
            }
        } else {
            ^$cmd_bin ...$cmd_args | complete
        }

        {
            exit_code: $result.exit_code
            stderr: ($result.stderr | default "")
        }
    }
}

# Refresh devenv evaluation cache without launching Yazelix UI
export def "yzx refresh" [
    --force(-f)    # Force refresh even when no config/input changes are detected
    --verbose(-v)  # Show package scope and concise refresh progress
    --very-verbose(-V)  # Show full build logs during refresh (-vv equivalent)
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let needs_refresh = $config_state.needs_refresh
    let refresh_output = if $very_verbose {
        "full"
    } else if $verbose {
        "normal"
    } else {
        get_refresh_output_mode $config
    }
    let show_progress = ($refresh_output != "quiet")

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

    if $show_progress {
        let scope = get_requested_package_scope $config
        let packs_text = summarize_values $scope.enabled_packs 12
        let packages_text = summarize_values $scope.top_level_packages 20
        let recommended_deps = ($config | get -o recommended_deps | default true)
        let yazi_extensions = ($config | get -o yazi_extensions | default true)
        print $"📦 Requested packs: ($packs_text)"
        print $"📦 Top-level packages: ($packages_text)"
        print $"   Optional bundles: recommended_deps=($recommended_deps), yazi_extensions=($yazi_extensions)"
        print "   Note: Nix builds transitive dependencies in addition to these top-level packages."
    }

    let allow_unfree = is_unfree_enabled
    if $needs_refresh or $force {
        print $"♻️  Refreshing Yazelix environment \(($refresh_reason)\)..."

        let devenv_base = (get_devenv_base_command --quiet=($refresh_output == "quiet") --devenv-verbose=($refresh_output == "full") --refresh-eval-cache)
        let devenv_cmd = ($devenv_base | append ["build", "shell"])

        if $show_progress {
            print $"⚙️ Running: ($devenv_cmd | str join ' ')"
        }

        let refresh_result = run_refresh_command $devenv_cmd $allow_unfree --stream-output=$show_progress
        if $refresh_result.exit_code != 0 {
            print $"❌ Refresh failed \(exit code: ($refresh_result.exit_code)\)"
            let stderr = ($refresh_result.stderr | str trim)
            if ($stderr | is-not-empty) {
                print $stderr
            }
            exit 1
        }
    }
    mark_config_state_applied (compute_config_state)

    print "✅ Refresh completed."
    print "⚠️  Your current Yazelix session keeps its existing environment."
    print "   Run 'yzx restart' to switch this window to the refreshed profile."
    print "   Or run 'yzx launch' to open a separate Yazelix window on the refreshed profile."
}
