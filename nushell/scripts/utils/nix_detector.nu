#!/usr/bin/env nu
# Nix installation detector and graceful failure handler

# Check if Nix is installed and properly configured
export def check_nix_installation [] {
    # Check if nix command is available
    let nix_available = (which nix | is-not-empty)
    
    if not $nix_available {
        return {
            installed: false
            error: "nix_not_found"
            message: "Nix package manager is not installed or not in PATH"
        }
    }
    
    # Check if nix develop command works (basic functionality test)
    let nix_develop_works = try {
        let result = (^nix develop --help | complete)
        $result.exit_code == 0
    } catch {
        false
    }
    
    if not $nix_develop_works {
        return {
            installed: true
            error: "nix_develop_unavailable"
            message: "Nix is installed but 'nix develop' command is not available (flakes may not be enabled)"
        }
    }
    
    # Check if flakes are enabled
    let flakes_enabled = try {
        let result = (^nix flake --help | complete)
        $result.exit_code == 0
    } catch {
        false
    }
    
    if not $flakes_enabled {
        return {
            installed: true
            error: "flakes_disabled"
            message: "Nix is installed but flakes are not enabled"
        }
    }
    
    return {
        installed: true
        error: null
        message: "Nix is properly installed and configured"
    }
}

# Display helpful error message and installation instructions
export def show_nix_installation_help [error_type: string] {
    use ascii_art.nu get_yazelix_colors
    let colors = get_yazelix_colors
    
    # Add red color since it's not in the standard Yazelix palette
    let red = (ansi red)
    
    print $"($red)❌ Yazelix requires Nix but it's not properly set up!($colors.reset)"
    print ""
    
    match $error_type {
        "nix_not_found" => {
            print $"($colors.yellow)🔍 Problem:($colors.reset) Nix package manager is not installed or not in your PATH."
            print ""
            print $"($colors.blue)💡 Solution:($colors.reset) Install Nix using the Determinate Systems installer:"
            print $"($colors.cyan)curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install($colors.reset)"
            print ""
            print "This installer:"
            print "  • Installs Nix with flakes enabled by default"
            print "  • Sets up proper file permissions and system integration"
            print "  • Provides a reliable uninstaller if needed"
            print ""
            print "After installation, restart your shell or run:"
            print $"($colors.cyan)source ~/.nix-profile/etc/profile.d/nix.sh($colors.reset)"
        }
        
        "nix_develop_unavailable" => {
            print $"($colors.yellow)🔍 Problem:($colors.reset) Nix is installed but 'nix develop' is not available."
            print ""
            print $"($colors.blue)💡 Solution:($colors.reset) This usually means you have an older Nix installation."
            print "Update Nix to a recent version that supports flakes:"
            print $"($colors.cyan)nix upgrade-nix($colors.reset)"
            print ""
            print "Or reinstall with the modern installer:"
            print $"($colors.cyan)curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install($colors.reset)"
        }
        
        "flakes_disabled" => {
            print $"($colors.yellow)🔍 Problem:($colors.reset) Nix is installed but flakes are not enabled."
            print ""
            print $"($colors.blue)💡 Solution:($colors.reset) Enable flakes by adding this to your Nix configuration:"
            print ""
            print "Create or edit ~/.config/nix/nix.conf:"
            print $"($colors.cyan)mkdir -p ~/.config/nix($colors.reset)"
            print $"($colors.cyan)echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf($colors.reset)"
            print ""
            print "Or reinstall with the modern installer that enables flakes by default:"
            print $"($colors.cyan)curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install($colors.reset)"
        }
    }
    
    print ""
    print $"($colors.green)📚 More help:($colors.reset)"
    print "  • Yazelix documentation: https://github.com/luccahuguet/yazelix"
    print "  • Nix installation guide: https://nixos.org/download.html"
    print "  • Determinate Systems installer: https://install.determinate.systems/"
    print ""
    print $"($colors.yellow)💡 Tip:($colors.reset) After installing Nix, you can verify it works by running:"
    print $"($colors.cyan)nix --version && nix flake --help($colors.reset)"
}

# Main function to check Nix and fail gracefully if not available
export def ensure_nix_available [] {
    let nix_status = check_nix_installation
    
    if not $nix_status.installed or ($nix_status.error | is-not-empty) {
        show_nix_installation_help $nix_status.error
        exit 1
    }
    
    # If we get here, Nix is properly installed
    return true
}