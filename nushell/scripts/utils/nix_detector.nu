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

    # Ensure devenv command is available
    let devenv_available = try {
        let result = (^devenv --help | complete)
        $result.exit_code == 0
    } catch {
        false
    }

    if not $devenv_available {
        return {
            installed: true
            error: "devenv_not_found"
            message: "devenv command is not installed or not in PATH"
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
        
        "devenv_not_found" => {
            print $"($colors.yellow)🔍 Problem:($colors.reset) devenv CLI is not installed or not available in PATH."
            print ""
            print $"($colors.blue)💡 Solution:($colors.reset) Install devenv by following the official guide:"
            print $"($colors.cyan)https://devenv.sh/getting-started/($colors.reset)"
            print ""
            print "If devenv is already installed, ensure your shell sources the appropriate profile or restart your terminal."
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
export def ensure_nix_available [
    --non-interactive  # Skip interactive prompts (for testing)
] {
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

        # If non-interactive mode, just fail
        if $non_interactive {
            error make { msg: $"Nix not available: ($nix_status.error)" }
        }

        # Special handling for nix_not_in_path - offer to source it automatically
        if $nix_status.error == "nix_not_in_path" {
            print $"($colors.cyan)🔧 Quick fix options:($colors.reset)"
            print $"($colors.cyan)  s\) Try to source Nix profile automatically($colors.reset)"
            print $"($colors.cyan)  y\) Continue anyway \(bypass detection\)($colors.reset)"
            print $"($colors.cyan)  n\) Abort and fix manually($colors.reset)"
            print ""
            
            let response = try {
                (input $"($colors.cyan)Choose an option \(s/y/N\): ($colors.reset)")
            } catch { |err|
                print $"($colors.red)❌ Error getting user input: ($err.msg)($colors.reset)"
                print $"($colors.yellow)Defaulting to bypass mode...($colors.reset)"
                "y"
            }
            
            if ($response | str downcase) in ["s", "source"] {
                try {
                    print $"($colors.yellow)🔧 Attempting to fix Nix PATH...($colors.reset)"
                    
                    # Simple and safe approach: just add Nix bin directories to PATH
                    let nix_bin_paths = [
                        "/nix/var/nix/profiles/default/bin"
                        "~/.nix-profile/bin"
                    ]
                    
                    print $"($colors.cyan)Checking for Nix binary directories...($colors.reset)"
                        
                    # Find existing Nix binary directories
                    let existing_nix_paths = ($nix_bin_paths | where ($it | path expand | path exists) | each { |p| $p | path expand })
                    
                    if ($existing_nix_paths | is-empty) {
                        print $"($colors.red)❌ No Nix binary directories found($colors.reset)"
                        print $"($colors.yellow)Expected locations: ($nix_bin_paths | str join ', ')($colors.reset)"
                        return false
                    }
                    
                    print $"($colors.cyan)Found Nix directories: ($existing_nix_paths | str join ', ')($colors.reset)"
                    
                    # Safely update PATH
                    $env.PATH = ($existing_nix_paths | append $env.PATH | uniq)
                    print $"($colors.cyan)Updated PATH with Nix directories($colors.reset)"
                    
                    # Test if nix is now available
                    if (which nix | is-not-empty) {
                        let nix_version = try { (^nix --version | lines | first) } catch { "unknown" }
                        print $"($colors.green)✅ Success! Nix is now available: ($nix_version)($colors.reset)"
                        return true
                    } else {
                        print $"($colors.yellow)⚠️  PATH updated but nix command still not found($colors.reset)"
                        print $"($colors.yellow)   This might indicate a more complex issue($colors.reset)"
                        print $"($colors.yellow)   Continuing anyway...($colors.reset)"
                        return true
                    }
                } catch { |err|
                    print $"($colors.red)❌ DETAILED ERROR in source option:($colors.reset)"
                    print $"($colors.red)   Error message: ($err.msg)($colors.reset)"
                    print $"($colors.red)   Error debug: ($err.debug)($colors.reset)"
                    print $"($colors.red)   Error span: ($err.span)($colors.reset)"
                    print $"($colors.yellow)   Please copy this error and report it!($colors.reset)"
                    print $"($colors.yellow)   Falling back to bypass mode...($colors.reset)"
                    return true
                }
            } else if ($response | str downcase) in ["y", "yes"] {
                try {
                    print $"($colors.yellow)⚠️  Proceeding despite Nix detection issues...($colors.reset)"
                    return true
                } catch { |err|
                    print $"($colors.red)❌ DETAILED ERROR in bypass option:($colors.reset)"
                    print $"($colors.red)   Error message: ($err.msg)($colors.reset)"
                    print $"($colors.red)   Error debug: ($err.debug)($colors.reset)"
                    print $"($colors.red)   Error span: ($err.span)($colors.reset)"
                    print $"($colors.yellow)   Please copy this error and report it!($colors.reset)"
                    return true
                }
            } else {
                try {
                    print $"($colors.red)❌ Aborting. Please fix your Nix installation and try again.($colors.reset)"
                    exit 1
                } catch { |err|
                    print $"($colors.red)❌ DETAILED ERROR in abort option:($colors.reset)"
                    print $"($colors.red)   Error message: ($err.msg)($colors.reset)"
                    print $"($colors.red)   Error debug: ($err.debug)($colors.reset)"
                    print $"($colors.red)   Error span: ($err.span)($colors.reset)"
                    print $"($colors.yellow)   Please copy this error and report it!($colors.reset)"
                    exit 1
                }
            }
        } else {
            # For other errors, just offer bypass option
            print $"($colors.yellow)⚠️  If you believe your Nix installation is working correctly,($colors.reset)"
            print $"($colors.yellow)   this might be a detection issue.($colors.reset)"
            print ""
            
            let response = try {
                (input $"($colors.cyan)Do you want to try running Yazelix anyway? \(y/N\): ($colors.reset)")
            } catch { |err|
                print $"($colors.red)❌ Error getting user input: ($err.msg)($colors.reset)"
                print $"($colors.yellow)Defaulting to bypass mode...($colors.reset)"
                "y"
            }
            
            if ($response | str downcase) in ["y", "yes"] {
                try {
                    print $"($colors.yellow)⚠️  Proceeding despite Nix detection issues...($colors.reset)"
                    print $"($colors.yellow)   If Yazelix fails to start, please check your Nix installation.($colors.reset)"
                    return true
                } catch { |err|
                    print $"($colors.red)❌ DETAILED ERROR in general bypass option:($colors.reset)"
                    print $"($colors.red)   Error message: ($err.msg)($colors.reset)"
                    print $"($colors.red)   Error debug: ($err.debug)($colors.reset)"
                    print $"($colors.red)   Error span: ($err.span)($colors.reset)"
                    print $"($colors.yellow)   Please copy this error and report it!($colors.reset)"
                    return true
                }
            } else {
                try {
                    print $"($colors.red)❌ Aborting. Please fix your Nix installation and try again.($colors.reset)"
                    exit 1
                } catch { |err|
                    print $"($colors.red)❌ DETAILED ERROR in general abort option:($colors.reset)"
                    print $"($colors.red)   Error message: ($err.msg)($colors.reset)"
                    print $"($colors.red)   Error debug: ($err.debug)($colors.reset)"
                    print $"($colors.red)   Error span: ($err.span)($colors.reset)"
                    print $"($colors.yellow)   Please copy this error and report it!($colors.reset)"
                    exit 1
                }
            }
        }
    }
    
    # If we get here, Nix is properly installed
    return true
}
