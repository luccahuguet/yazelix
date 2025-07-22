# Home-Manager Integration Guide

This guide explains how to use Yazelix with [Home Manager](https://github.com/nix-community/home-manager), the declarative configuration management system for NixOS and other Nix-based systems.

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

**Important**: The home-manager module only handles shell integration and state directories. You manage the Yazelix installation itself separately (`git clone`, `nix develop`) to avoid git conflicts and circular symlinks.

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