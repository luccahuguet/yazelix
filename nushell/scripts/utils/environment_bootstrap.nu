#!/usr/bin/env nu
# Shared environment preparation utilities for Yazelix.

use config_parser.nu parse_yazelix_config
use config_state.nu compute_config_state
use startup_profile.nu [profile_startup_step]
use common.nu require_yazelix_runtime_dir

def setup_nix_environment [] {
    let nix_profiles = [
        "~/.nix-profile/etc/profile.d/nix.sh"
        "/nix/var/nix/profiles/default/etc/profile.d/nix.sh"
    ]

    let nix_profile = ($nix_profiles | where ($it | path expand | path exists) | first)
    if ($nix_profile | is-empty) {
        return {
            success: false
            message: "No Nix profile script found"
        }
    }

    let expanded_profile = ($nix_profile | path expand)

    try {
        let env_output = (bash -c $"source ($expanded_profile) && env")
        let nix_vars = ($env_output | lines | where ($it | str contains "NIX_") | parse "{key}={value}")
        let path_line = ($env_output | lines | where ($it | str starts-with "PATH=") | first)

        if not ($path_line | is-empty) {
            let new_path = ($path_line | str replace "PATH=" "")
            $env.PATH = ($new_path | split row ":")
        }

        for $var in $nix_vars {
            load-env {($var.key): $var.value}
        }

        {
            success: true
            message: $"Nix environment loaded from ($expanded_profile)"
        }
    } catch { |err|
        {
            success: false
            message: $"Failed to source Nix profile: ($err.msg)"
        }
    }
}

def ensure_nix_in_environment [] {
    if (which nix | is-not-empty) {
        return true
    }

    let setup_result = setup_nix_environment
    if not $setup_result.success {
        print $"⚠️  Could not set up Nix environment: ($setup_result.message)"
        return false
    }

    if (which nix | is-not-empty) {
        print $"✅ ($setup_result.message)"
        true
    } else {
        print "⚠️  Nix environment setup completed but nix command still not available"
        false
    }
}

export def ensure_environment_available [] {
    require_yazelix_runtime_dir | ignore
}

export def prepare_environment [--verbose] {
    let verbose_mode = $verbose

    let config = (profile_startup_step "bootstrap" "prepare.parse_config" {
        parse_yazelix_config
    })

    let config_state = (profile_startup_step "bootstrap" "prepare.compute_config_state" {
        compute_config_state
    })

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
