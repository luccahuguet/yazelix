# Home-Manager Integration Guide

This guide explains how to use Yazelix with [Home Manager](https://github.com/nix-community/home-manager), the declarative configuration management system for NixOS and other Nix-based systems.

## Prerequisites

### For NixOS Users
You likely already have Nix and can install Home Manager directly.

### For Non-NixOS Users (Ubuntu, PopOS, etc.)

First, install the Nix package manager:
```bash
# Install Nix (multi-user installation recommended)
curl -L https://nixos.org/nix/install | sh -s -- --daemon

# Restart your shell or source the profile
source /etc/profile.d/nix.sh
```

Then install Home Manager:
```bash
# Add the home-manager channel
nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
nix-channel --update

# Install home-manager
nix-shell '<home-manager>' -A install
```

## Why Home-Manager Integration?

Home-Manager allows you to declaratively manage your entire user environment, including Yazelix. This provides:

- **Reproducible Configuration**: Your Yazelix setup is version-controlled and reproducible across machines
- **Atomic Updates**: Changes are applied atomically, preventing broken states
- **Rollback Support**: Easy rollback to previous configurations if something breaks
- **Integration**: Seamless integration with other home-manager programs

## Architecture Overview

Yazelix now follows XDG Base Directory standards for clean home-manager integration:

```
~/.config/yazelix/           # Static configuration (managed by home-manager)
├── configs/                 # Tool configurations (Yazi, Zellij, etc.)
├── nushell/scripts/         # Static scripts
├── shells/                  # Shell integration scripts
├── yazelix_default.nix      # Configuration template
└── flake.nix               # Nix flake definition

~/.local/share/yazelix/     # Runtime state (never managed by home-manager)
├── logs/                   # Runtime logs
├── initializers/           # Generated shell initializers
└── cache/                  # Generated configurations
```

## Setup Approaches

### Option 1: Released Version (Recommended for Users)
Use the official Yazelix releases through GitHub:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    home-manager.url = "github:nix-community/home-manager";
    yazelix.url = "github:luccahuguet/yazelix";  # Latest release
    # Or pin to specific version: yazelix.url = "github:luccahuguet/yazelix/v7.5";
  };
}
```

### Option 2: Local Development (For Contributors/Customizers)
**⚠️ IMPORTANT**: Never let home-manager manage the yazelix git repository directly!

**❌ WRONG** - This will corrupt your git repository:
```nix
# DON'T DO THIS - Creates circular symlinks!
home.file.".config/yazelix" = {
  source = /path/to/yazelix;
  recursive = true;
};
```

**✅ CORRECT** - Keep git and home-manager separate:

1. **Clone Yazelix manually**:
   ```bash
   git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
   cd ~/.config/yazelix
   nix develop  # Enter development shell
   ```

2. **Import the local module**:
   ```nix
   # flake.nix
   {
     outputs = { nixpkgs, home-manager, ... }: {
       homeConfigurations."username" = home-manager.lib.homeManagerConfiguration {
         modules = [
           # Import the local module directly
           (import /home/username/.config/yazelix/home_manager_module.nix)
           ./home.nix
         ];
       };
     };
   }
   ```

3. **Apply with impure flag**:
   ```bash
   home-manager switch --impure --flake '.#username@hostname'
   ```

## Home-Manager Configuration

Yazelix provides a dedicated home-manager module for the best user experience:

First, add the Yazelix flake to your home-manager inputs:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    home-manager.url = "github:nix-community/home-manager";
    yazelix.url = "github:luccahuguet/yazelix";
  };
  
  outputs = { nixpkgs, home-manager, yazelix, ... }:
    let
      system = "x86_64-linux";  # or your system
    in {
      homeConfigurations."your-username" = home-manager.lib.homeManagerConfiguration {
        pkgs = nixpkgs.legacyPackages.${system};
        modules = [
          yazelix.homeManagerModules.default
          ./home.nix
        ];
      };
    };
}
```

Then configure Yazelix in your `home.nix`:

```nix
# home.nix
{ config, pkgs, ... }:

{
  programs.yazelix = {
    enable = true;
    
    # Core configuration
    helixMode = "source";                # "release" or "source"
    recommendedDeps = true;              # Include extra tools
    yaziExtensions = true;               # File preview extensions
    yaziMedia = true;                    # Media preview support
    
    # Shell configuration
    defaultShell = "nu";                 # "nu", "bash", "fish", "zsh"
    extraShells = [ "bash" "fish" ];     # Additional shells to support
    
    # Terminal emulator
    preferredTerminal = "wezterm";       # "wezterm", "ghostty", "kitty"
    
    # Editor settings
    editorConfig = {
      setEditor = true;                  # Set EDITOR environment variable
      overrideExisting = true;           # Override existing EDITOR
      editorCommand = "hx";              # Editor command
    };
    
    # UI preferences
    asciiArtMode = "animated";           # "static" or "animated"
    skipWelcomeScreen = false;           # Skip welcome screen
    showMacchinaOnWelcome = true;        # Show system info
    
    # Custom packages
    userPackages = with pkgs; [
      # Add your custom packages
    ];
  };
}
```

## Terminal Integration

### WezTerm
```nix
programs.wezterm = {
  enable = true;
  extraConfig = ''
    return {
      color_scheme = 'Abernathy',
      hide_tab_bar_if_only_one_tab = true,
      default_prog = { 'nu', '~/.config/yazelix/nushell/scripts/core/start_yazelix.nu' },
      window_decorations = "NONE",
      window_background_opacity = 0.9,
    }
  '';
};
```

### Ghostty
```nix
programs.ghostty = {
  enable = true;
  settings = {
    command = ["nu" "~/.config/yazelix/nushell/scripts/core/start_yazelix.nu"];
    window-decoration = false;
    background-opacity = 0.9;
  };
};
```

## Migration from Standard Installation

If you're migrating from a standard Yazelix installation:

### 1. Backup Your Configuration
```bash
cp ~/.config/yazelix/yazelix.nix ~/yazelix-backup.nix
```

### 2. Clean Old Installation
```bash
# Remove old runtime files (safe to delete)
rm -rf ~/.config/yazelix/logs
rm -rf ~/.config/yazelix/*/initializers
```

### 3. Apply Home-Manager Configuration
```bash
home-manager switch
```

### 4. Verify State Directory
```bash
ls ~/.local/share/yazelix/  # Should exist and be writable
```

## Environment Detection

Yazelix automatically detects your environment:

- **Home-Manager**: Uses read-only config approach, no auto-creation of `yazelix.nix`
- **Standard**: Auto-creates `yazelix.nix` from template, full write access
- **Read-Only**: Warns about potential issues, provides guidance

## Troubleshooting

### Circular Symlink Error / Git Repository Corruption
**Symptoms:**
- Git shows all files as `typechange` 
- Error: "Too many levels of symbolic links"
- Repository appears corrupted after `home-manager switch`

**Cause:** Home-manager tried to manage the yazelix directory directly, creating circular symlinks.

**Fix:**
```bash
cd ~/.config/yazelix
git reset --hard HEAD  # Restore all files
rm -f *.backup        # Clean up backup files
```

**Prevention:** Never use `home.file.".config/yazelix" = { source = ...; }` in your home-manager config!

### Read-Only Configuration Warning
If you see this warning:
```
⚠️ WARNING: Read-only configuration directory detected!
```

**Solutions:**
1. Use the home-manager module approach above
2. Ensure proper permissions on `~/.config/yazelix/`
3. Check if your system uses a different configuration management system

### State Directory Issues
If initializers aren't being generated:
```bash
# Check state directory permissions
ls -la ~/.local/share/yazelix/
mkdir -p ~/.local/share/yazelix/{logs,initializers,cache}
```

### Shell Integration Issues
If shell aliases aren't working:
```bash
# Check shell configuration
yzx config_status
```

## Advanced Configuration

### Custom Shell Initializers
You can add custom shell initialization by extending the configuration:

```nix
programs.yazelix = {
  enable = true;
  # ... other configuration
  
  customShellInit = {
    nushell = ''
      # Custom nushell configuration
      $env.MY_CUSTOM_VAR = "value"
    '';
    bash = ''
      # Custom bash configuration
      export MY_CUSTOM_VAR="value"
    '';
  };
};
```

### Development Workflow
For Yazelix development with home-manager:

```nix
programs.yazelix = {
  enable = true;
  # Point to local development version
  source = /path/to/local/yazelix;
  debugMode = true;
};
```

## Best Practices

1. **Version Pinning**: Use specific commit hashes for reproducibility
2. **Modular Configuration**: Split large configurations into multiple files
3. **Backup**: Keep your configuration in version control
4. **Testing**: Test configuration changes in a VM or separate user account
5. **Gradual Migration**: Migrate one tool at a time if coming from imperative setup

## Quick Start for PopOS/Ubuntu Users

Here's the complete setup process for PopOS users:

### 1. Install Nix and Home Manager
```bash
# Install Nix
curl -L https://nixos.org/nix/install | sh -s -- --daemon

# Reload your shell
exec $SHELL

# Install Home Manager
nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
nix-channel --update
nix-shell '<home-manager>' -A install
```

### 2. Initialize Home Manager Configuration
```bash
# Create home-manager directory
mkdir -p ~/.config/home-manager

# Create flake.nix
cat > ~/.config/home-manager/flake.nix << 'EOF'
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    home-manager.url = "github:nix-community/home-manager";
    yazelix.url = "github:luccahuguet/yazelix";
  };

  outputs = { nixpkgs, home-manager, yazelix, ... }: {
    homeConfigurations."$(whoami)" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [
        yazelix.homeManagerModules.default
        ./home.nix
      ];
    };
  };
}
EOF

# Create home.nix
cat > ~/.config/home-manager/home.nix << 'EOF'
{ config, pkgs, ... }: {
  home.username = "$(whoami)";
  home.homeDirectory = "/home/$(whoami)";
  home.stateVersion = "23.11";

  programs.yazelix = {
    enable = true;
    helixMode = "release";
    recommendedDeps = true;
    yaziExtensions = true;
    yaziMedia = true;
    defaultShell = "nu";
    preferredTerminal = "wezterm";
  };
}
EOF
```

### 3. Apply Configuration
```bash
cd ~/.config/home-manager
home-manager switch --flake '.#$(whoami)'
```

### 4. Start Using Yazelix
```bash
# The yzx command should now be available
source ~/.bashrc  # Or restart your terminal
yzx launch
```

## Getting Help

- Check the [main README](../README.md) for general usage
- Use `yzx help` for command reference
- Enable debug mode for troubleshooting: `debugMode = true`
- Check logs in `~/.local/share/yazelix/logs/`

## Contributing

If you find issues with home-manager integration:

1. Check existing [issues](https://github.com/luccahuguet/yazelix/issues)
2. Provide your home-manager configuration
3. Include relevant log files from `~/.local/share/yazelix/logs/` 