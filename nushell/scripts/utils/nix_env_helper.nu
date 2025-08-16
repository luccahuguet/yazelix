#!/usr/bin/env nu
# Helper script to set up Nix environment

# Attempt to source Nix profile and make it available
export def setup_nix_environment [] {
    # Common Nix profile locations
    let nix_profiles = [
        "~/.nix-profile/etc/profile.d/nix.sh"
        "/nix/var/nix/profiles/default/etc/profile.d/nix.sh"
    ]
    
    # Find the first existing profile script
    let nix_profile = ($nix_profiles | where ($it | path expand | path exists) | first)
    
    if ($nix_profile | is-empty) {
        return {
            success: false
            message: "No Nix profile script found"
        }
    }
    
    # Source the profile script using bash and capture the environment
    let expanded_profile = ($nix_profile | path expand)
    
    try {
        # Use bash to source the profile and export the environment
        let env_output = (bash -c $"source ($expanded_profile) && env")
        
        # Parse the environment variables
        let nix_vars = ($env_output | lines | where ($it | str contains "NIX_") | parse "{key}={value}")
        let path_line = ($env_output | lines | where ($it | str starts-with "PATH=") | first)
        
        if not ($path_line | is-empty) {
            let new_path = ($path_line | str replace "PATH=" "")
            $env.PATH = ($new_path | split row ":")
        }
        
        # Set NIX environment variables
        for $var in $nix_vars {
            load-env {($var.key): $var.value}
        }
        
        return {
            success: true
            message: $"Nix environment loaded from ($expanded_profile)"
        }
    } catch { |err|
        return {
            success: false
            message: $"Failed to source Nix profile: ($err.msg)"
        }
    }
}

# Check if Nix is available after environment setup
export def ensure_nix_in_environment [] {
    # First check if nix is already available
    if (which nix | is-not-empty) {
        return true
    }
    
    # Try to set up the environment
    let setup_result = setup_nix_environment
    
    if not $setup_result.success {
        print $"⚠️  Could not set up Nix environment: ($setup_result.message)"
        return false
    }
    
    # Check again after setup
    if (which nix | is-not-empty) {
        print $"✅ ($setup_result.message)"
        return true
    } else {
        print "⚠️  Nix environment setup completed but nix command still not available"
        return false
    }
}