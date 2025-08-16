#!/usr/bin/env nu
# Nix installation detector and graceful failure handler

# Check if Nix is installed and properly configured
export def check_nix_installation [] {
    # Check if nix command is available in PATH
    let nix_available = (which nix | is-not-empty)
    
    # If not in PATH, check common Nix installation locations
    let nix_locations = [
        "/nix/var/nix/profiles/default/bin/nix"
        "~/.nix-profile/bin/nix"
        "/run/current-system/sw/bin/nix"
    ]
    
    let nix_found = if $nix_available {
        true
    } else {
        # Check if Nix exists in common locations
        $nix_locations | any { |path| ($path | path expand | path exists) }
    }
    
    if not $nix_found {
        return {
            installed: false
            error: "nix_not_found"
            message: "Nix package manager is not installed or not in PATH"
        }
    }
    
    # If Nix exists but not in PATH, suggest sourcing profile
    if not $nix_available and $nix_found {
        return {
            installed: true
            error: "nix_not_in_path"
            message: "Nix is installed but not in PATH - shell profile may need to be sourced"
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
    let colors = {
        red: $"\u{1b}[31m"
        yellow: $"\u{1b}[33m"
        blue: $"\u{1b}[34m"
        green: $"\u{1b}[32m"
        cyan: $"\u{1b}[36m"
        reset: $"\u{1b}[0m"
    }
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
        
        "nix_not_in_path" => {
            print $"($colors.yellow)🔍 Problem:($colors.reset) Nix is installed but not available in your current shell's PATH."
            print ""
            print $"($colors.blue)💡 Solution:($colors.reset) Your shell needs to load the Nix profile. Try one of these:"
            print ""
            print "Option 1 - Source Nix profile in current session:"
            print $"($colors.cyan)source ~/.nix-profile/etc/profile.d/nix.sh($colors.reset)"
            print ""
            print "Option 2 - Restart your terminal emulator (recommended)"
            print "Option 3 - Start a login shell:"
            print $"($colors.cyan)bash -l($colors.reset)"
            print ""
            print "This issue can happen when:"
            print "  • Using certain terminal emulators that don't load login shells"
            print "  • Shell configuration files weren't properly updated during Nix installation"
            print "  • Using non-standard shell configurations"
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
    let colors = {
        red: $"\u{1b}[31m"
        yellow: $"\u{1b}[33m"
        blue: $"\u{1b}[34m"
        green: $"\u{1b}[32m"
        cyan: $"\u{1b}[36m"
        reset: $"\u{1b}[0m"
    }
    
    let nix_status = check_nix_installation
    
    if not $nix_status.installed or ($nix_status.error | is-not-empty) {
        show_nix_installation_help $nix_status.error
        print ""
        print $"($colors.yellow)⚠️  If you believe your Nix installation is working correctly,($colors.reset)"
        print $"($colors.yellow)   this might be a detection issue.($colors.reset)"
        print ""
        
        let response = (input $"($colors.cyan)Do you want to try running Yazelix anyway? \(y/N\): ($colors.reset)")
        
        if ($response | str downcase) in ["y", "yes"] {
            print $"($colors.yellow)⚠️  Proceeding despite Nix detection issues...($colors.reset)"
            print $"($colors.yellow)   If Yazelix fails to start, please check your Nix installation.($colors.reset)"
            return true
        } else {
            print $"($colors.red)❌ Aborting. Please fix your Nix installation and try again.($colors.reset)"
            exit 1
        }
    }
    
    # If we get here, Nix is properly installed
    return true
}