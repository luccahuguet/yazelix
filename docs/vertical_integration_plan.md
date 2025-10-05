# Yazelix Vertical Integration Plan: Single Package Implementation

## Overview

Transform Yazelix from a configuration system to a single integrated package while preserving its sophisticated dynamic configuration generation and multi-terminal support.

## Key Finding: Current System is Already NixOS-Compatible

The existing runtime configuration generation is **perfectly compatible** with NixOS principles:
- ✅ Writes to user directories (`~/.config`, `~/.local/share`) not `/nix/store`
- ✅ All dependencies are declared
- ✅ Generation is deterministic given same inputs
- ✅ Follows standard NixOS patterns for user-space applications

## Implementation Strategy

### Phase 1: Core Integration Package

Create a single `yazelix` derivation that bundles all components:

```nix
yazelix-core = pkgs.stdenv.mkDerivation {
  name = "yazelix-core";
  src = ./.;
  
  installPhase = ''
    mkdir -p $out/share/yazelix
    cp -r nushell configs $out/share/yazelix/
    cp -r shells docs $out/share/yazelix/
  '';
};

yazelix = pkgs.writeShellApplication {
  name = "yazelix";
  runtimeInputs = with pkgs; [ 
    # Core tools - guaranteed available
    nushell zellij yazi helix
    # Additional tools based on packs
    starship zoxide fzf
  ];
  
  text = ''
    # Set up clean environment
    export YAZELIX_CORE=${yazelix-core}
    export YAZELIX_DIR=$YAZELIX_CORE/share/yazelix
    
    # Use existing dynamic generation logic
    exec nu $YAZELIX_DIR/nushell/scripts/core/start_yazelix.nu "$@"
  '';
};
```

### Phase 2: Multi-Terminal Support

Support multiple terminal emulators while maintaining the single package approach:

```nix
mkYazelix = { terminal, terminalConfig ? null }: pkgs.writeShellApplication {
  name = "yazelix-${terminal.pname}";
  runtimeInputs = [ terminal nushell zellij yazi helix ];
  
  text = ''
    export YAZELIX_CORE=${yazelix-core}
    export YAZELIX_PREFERRED_TERMINAL="${terminal.pname}"
    ${if terminalConfig != null then 
      "export YAZELIX_TERMINAL_CONFIG=${terminalConfig}" 
      else ""}
    
    exec nu $YAZELIX_CORE/share/yazelix/nushell/scripts/core/start_yazelix.nu "$@"
  '';
};

# Multiple variants
yazelix-ghostty = mkYazelix { terminal = pkgs.ghostty; };
yazelix-kitty = mkYazelix { terminal = pkgs.kitty; };
yazelix-alacritty = mkYazelix { terminal = pkgs.alacritty; };
yazelix-wezterm = mkYazelix { terminal = pkgs.wezterm; };
yazelix-foot = mkYazelix { terminal = pkgs.foot; };

# Default points to ghostty
yazelix = yazelix-ghostty;
```

### Phase 3: Home Manager Integration

Enhanced Home Manager module for the integrated package:

```nix
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.yazelix;
  
  # Select appropriate yazelix variant
  yazelixPackage = 
    if cfg.terminal == "ghostty" then pkgs.yazelix-ghostty
    else if cfg.terminal == "wezterm" then pkgs.yazelix-wezterm
    else if cfg.terminal == "kitty" then pkgs.yazelix-kitty
    else if cfg.terminal == "alacritty" then pkgs.yazelix-alacritty
    else if cfg.terminal == "foot" then pkgs.yazelix-foot
    else pkgs.yazelix; # default
    
in {
  options.programs.yazelix = {
    enable = mkEnableOption "Yazelix terminal environment";
    
    terminal = mkOption {
      type = types.enum [ "ghostty" "wezterm" "kitty" "alacritty" "foot" ];
      default = "ghostty";
      description = "Terminal emulator to bundle with Yazelix";
    };
    
    # All existing options remain the same
    # ... (keep current option structure)
  };

  config = mkIf cfg.enable {
    # Install the appropriate yazelix variant
    home.packages = [ yazelixPackage ];
    
    # Generate user config file (same as current)
    xdg.configFile."yazelix/yazelix.nix" = {
      text = ''
        { pkgs }:
        {
          # Generated from Home Manager options
          recommended_deps = ${if cfg.recommended_deps then "true" else "false"};
          # ... rest of config generation
        }
      '';
    };
    
    # Optional: Shell integration
    programs.bash.initExtra = lib.mkIf cfg.shellIntegration ''
      alias yazelix="${yazelixPackage}/bin/yazelix"
    '';
    
    programs.fish.shellInit = lib.mkIf cfg.shellIntegration ''
      alias yazelix "${yazelixPackage}/bin/yazelix"
    '';
    
    programs.zsh.initExtra = lib.mkIf cfg.shellIntegration ''
      alias yazelix="${yazelixPackage}/bin/yazelix"
    '';
  };
}
```

## Preserved Features

### Dynamic Configuration Generation
- **Zellij config merging**: Keep `zellij_config_merger.nu` functionality
- **Yazi config merging**: Keep `yazi_config_merger.nu` functionality  
- **Shell initializers**: Continue generating per-shell in `~/.local/share/yazelix/initializers/`
- **Runtime adaptation**: Configs still adapt to system state and user preferences

### User Customization
- **Personal config files**: `~/.config/yazelix/yazelix.nix` still works
- **Plugin system**: Yazi plugins continue to load dynamically
- **Override system**: User configs still override defaults
- **Pack system**: Technology stacks still work via configuration

### Multi-Shell Support
- **Bash, Fish, Zsh, Nushell**: All shells continue to be supported
- **Shell-specific features**: Different initializers per shell maintained
- **Fallback logic**: Graceful handling when shells aren't available

## Simplified Architecture

### What Gets Eliminated
- **Complex terminal detection**: No more fallback chains in `launch_yazelix.nu:25-76`
- **Nix environment bootstrap**: No more `nix_detector.nu` and `nix_env_helper.nu`
- **Shell configuration complexity**: Reduced need for cross-shell compatibility layers
- **Installation complexity**: Single package instead of "install terminal + configure yazelix"

### What Gets Enhanced
- **Startup performance**: No shell detection or Nix bootstrapping overhead
- **Error handling**: Simpler execution path with fewer failure modes
- **User experience**: `nix run yazelix` or `yazelix` just works
- **Branding opportunity**: Custom terminal configurations with Yazelix identity

## Migration Strategy

### Backward Compatibility
Maintain the current configuration-based approach while introducing the integrated package:

```bash
# Current approach (still supported)
nix develop ~/.config/yazelix

# New integrated approach
nix run nixpkgs#yazelix
# or with Home Manager
programs.yazelix.enable = true;
```

### User Migration Path
1. **Existing users**: Continue with current setup, optional migration
2. **New users**: Start with integrated package by default
3. **Documentation**: Clear upgrade guide and comparison
4. **Deprecation timeline**: Gradual transition over multiple versions

## Package Structure

```
yazelix/
├── bin/
│   ├── yazelix              # Main launcher
│   ├── yazelix-ghostty      # Terminal-specific variants
│   ├── yazelix-kitty
│   ├── yazelix-alacritty
│   └── yazelix-wezterm
└── share/yazelix/
    ├── nushell/             # Bundled scripts
    ├── configs/             # Base configurations
    ├── shells/              # Shell integrations
    └── docs/                # Documentation
```

## Benefits Summary

### For Users
- **Simplified installation**: Single package, no setup required
- **Better performance**: Faster startup, no detection overhead
- **Consistent experience**: Terminal optimized for Yazelix workflow
- **Brand identity**: Cohesive Yazelix experience with logo/theming

### For Development
- **Cleaner architecture**: Fewer edge cases and failure modes
- **Easier testing**: Controlled environment with known dependencies
- **Better maintainability**: Less complex bootstrap and detection logic
- **Future-proofing**: Foundation for advanced features like session restoration

### For NixOS Ecosystem
- **Proper packaging**: Standard derivation that follows Nix conventions
- **Flake support**: Easy to include in system configurations
- **Home Manager integration**: Declarative configuration management
- **Reproducible**: Same inputs always produce same Yazelix environment

## Implementation Priority

1. **Phase 1**: Create basic integrated package with Ghostty
2. **Phase 2**: Add multi-terminal support and enhanced Home Manager module
3. **Phase 3**: Migration tooling and documentation
4. **Phase 4**: Advanced features enabled by vertical integration

This plan preserves all of Yazelix's sophisticated features while achieving the benefits of vertical integration and maintaining full NixOS compatibility.
