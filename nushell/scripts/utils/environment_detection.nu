#!/usr/bin/env nu
# Environment Detection Functions
# Helper functions for detecting yazelix environment configuration

use constants.nu YAZELIX_CONFIG_DIR

# Check if yazelix config directory is read-only
export def is_read_only_config [] {
    let config_dir = ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME)
    try {
        # Test write access by trying to create a temporary file
        let test_file = $"($config_dir)/.yazelix_write_test"
        touch $test_file
        rm $test_file
        false
    } catch {
        true
    }
}

# Check if running in a home-manager environment
export def is_home_manager_environment [] {
    # Check for common home-manager indicators
    let home_manager_indicators = [
        ($env.HOME + "/.local/state/nix/profiles/home-manager")
        ($env.HOME + "/.nix-profile/etc/profile.d/hm-session-vars.sh")
        $env.NIX_PROFILE?
    ]
    $home_manager_indicators | where ($it != null) | any { |path| $path | path exists }
}

# Detect the current yazelix environment type
export def detect_environment [] {
    let is_readonly = (is_read_only_config)
    let is_hm = (is_home_manager_environment)

    {
        read_only_config: $is_readonly
        home_manager: $is_hm
        environment_type: (
            if $is_hm { "home-manager" }
            else if $is_readonly { "read-only" }
            else { "standard" }
        )
    }
}
