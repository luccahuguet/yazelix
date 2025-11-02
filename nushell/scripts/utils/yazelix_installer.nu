#!/usr/bin/env nu
# Yazelix installation utility - install dependencies like devenv and nushell

# Check if a command is installed
def is_installed [cmd: string] {
    (which $cmd | is-not-empty)
}

# Check if nushell is installed
export def is_nushell_installed [] {
    (is_installed "nu")
}

# Check if devenv is installed
export def is_devenv_installed [] {
    (is_installed "devenv")
}

# Prompt user to install a package
def prompt_install [package: string, description: string, benefit: string, size: string] {
    print ""
    print "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print $"🚀 Install ($package)?"
    print "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print ""
    print $description
    print ""
    print $"Benefit: ($benefit)"
    print $"Size: ($size)"
    print ""
    
    let response = (input $"Install ($package) now? \(y/n\): ")
    
    ($response | str downcase | str trim) == "y"
}

# Install nushell via nix profile
export def install_nushell [] {
    print ""
    print "📦 Installing nushell..."
    print ""
    
    try {
        nix profile install nixpkgs#nushell
        
        print ""
        print "✅ nushell installed successfully!"
        print ""
        print "Nushell is now available in your PATH."
        print ""
        
        true
    } catch {
        print ""
        print "❌ Failed to install nushell."
        print ""
        print "You can install it manually later with:"
        print "  nix profile install nixpkgs#nushell"
        print ""
        
        false
    }
}

# Install devenv via nix profile
export def install_devenv [] {
    print ""
    print "📦 Installing devenv..."
    print ""
    
    try {
        nix profile install nixpkgs#devenv
        
        print ""
        print "✅ devenv installed successfully!"
        print ""
        print "Yazelix will now use devenv for instant shell startup."
        print "The first run will take ~5s to build cache, then ~0.3s every time."
        print ""
        
        true
    } catch {
        print ""
        print "❌ Failed to install devenv."
        print ""
        print "You can install it manually later with:"
        print "  nix profile install nixpkgs#devenv"
        print ""
        
        false
    }
}

# Ensure nushell is installed
export def ensure_nushell_installed [
    --auto-prompt  # If true, prompt automatically when missing
] {
    if (is_nushell_installed) {
        return {
            installed: true
            newly_installed: false
            message: "nushell is already installed"
        }
    }
    
    if $auto_prompt {
        let should_install = (prompt_install 
            "nushell" 
            "Nushell is required for Yazelix to function."
            "Required for Yazelix core functionality"
            "~50MB"
        )
        
        if $should_install {
            let success = (install_nushell)
            return {
                installed: $success
                newly_installed: $success
                message: (if $success { "nushell installed successfully" } else { "nushell installation failed" })
            }
        } else {
            print ""
            print "❌ Nushell is required for Yazelix."
            print "   Install it with: nix profile install nixpkgs#nushell"
            print ""
            return {
                installed: false
                newly_installed: false
                message: "user declined installation"
            }
        }
    }
    
    return {
        installed: false
        newly_installed: false
        message: "nushell not installed, auto-prompt disabled"
    }
}

# Ensure devenv is installed
export def ensure_devenv_installed [
    --auto-prompt  # If true, prompt automatically when missing
] {
    if (is_devenv_installed) {
        return {
            installed: true
            newly_installed: false
            message: "devenv is already installed"
        }
    }
    
    if $auto_prompt {
        let should_install = (prompt_install 
            "devenv" 
            "devenv provides instant shell startup through evaluation caching:\n  • Current speed: ~4-5 seconds\n  • With devenv:   ~0.3 seconds (13x faster!)\n\nThis dramatically improves launch times for:\n  • Desktop entries\n  • Terminal sessions\n  • All Yazelix commands"
            "13x faster shell startup (~4-5s → ~0.3s)"
            "~100MB"
        )
        
        if $should_install {
            let success = (install_devenv)
            return {
                installed: $success
                newly_installed: $success
                message: (if $success { "devenv installed successfully" } else { "devenv installation failed" })
            }
        } else {
            print ""
            print "💡 You can install devenv later for 13x faster performance:"
            print "   nix profile install nixpkgs#devenv"
            print ""
            return {
                installed: false
                newly_installed: false
                message: "user declined installation"
            }
        }
    }
    
    return {
        installed: false
        newly_installed: false
        message: "devenv not installed, auto-prompt disabled"
    }
}

# Check status of all Yazelix dependencies
export def check_dependencies [] {
    print "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print "📦 Yazelix Dependencies"
    print "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print ""
    
    # Check nushell
    if (is_nushell_installed) {
        let version = (nu --version | str trim)
        print $"✅ nushell: ($version)"
    } else {
        print "❌ nushell: NOT INSTALLED (required)"
        print "   Install: nix profile install nixpkgs#nushell"
    }
    
    print ""
    
    # Check devenv
    if (is_devenv_installed) {
        let version = (devenv version | str trim)
        print $"✅ devenv: ($version)"
        print "   Performance: ~0.3s shell startup (13x faster)"
    } else {
        print "⚠️  devenv: NOT INSTALLED (optional, recommended)"
        print "   Performance: ~4-5s shell startup"
        print ""
        print "💡 Install devenv for 13x faster launches:"
        print "   nix profile install nixpkgs#devenv"
        print "   Or run: yzx doctor --fix"
    }
    
    print ""
}
