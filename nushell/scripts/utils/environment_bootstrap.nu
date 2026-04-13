#!/usr/bin/env nu
# Shared environment preparation utilities for Yazelix.

use config_parser.nu parse_yazelix_config
use config_state.nu compute_config_state
use startup_profile.nu [profile_startup_step]
use common.nu require_yazelix_runtime_dir

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
