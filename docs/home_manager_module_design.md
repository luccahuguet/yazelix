# Yazelix Home Manager Module Design

> **Note:** The Home Manager module now generates `yazelix.toml`. Much of the design below predates that migration and should be revisited, but it remains useful for historical context.

## Architecture Decision: Configuration-Only Approach

Based on research of existing Home Manager patterns and the safety requirements, we're implementing **Option A: Configuration-Only Module** from the plan.

### Why Configuration-Only?

1. **Maximum Safety**: Never manages files in active git repositories
2. **Zero Conflicts**: Only generates `yazelix.nix` configuration file
3. **Minimal Risk**: User maintains control of the Yazelix repository
4. **Easy Migration**: Simple transition from manual to HM setup

## Module Interface Design

```nix
programs.yazelix = {
  enable = mkEnableOption "Yazelix terminal environment";
  
  # Configuration options (mirrors yazelix_default.nix structure)
  recommended_deps = mkOption {
    type = types.bool;
    default = true;
    description = "Install recommended productivity tools (~350MB)";
  };
  
  yazi_extensions = mkOption {
    type = types.bool;
    default = true;
    description = "Install Yazi file preview extensions (~125MB)";
  };
  
  yazi_media = mkOption {
    type = types.bool;
    default = false;
    description = "Install Yazi media processing tools (~1GB)";
  };
  
  helix_mode = mkOption {
    type = types.enum [ "release" "source" ];
    default = "release";
    description = "Helix build mode: release (nixpkgs) or source (flake)";
  };
  
  default_shell = mkOption {
    type = types.enum [ "nu" "bash" "fish" "zsh" ];
    default = "nu";
    description = "Default shell for Zellij sessions";
  };
  
  extra_shells = mkOption {
    type = types.listOf (types.enum [ "fish" "zsh" ]);
    default = [];
    description = "Additional shells to install beyond nu/bash";
  };
  
  terminals = mkOption {
    type = types.enum [ "wezterm" "ghostty" "kitty" "alacritty" "foot" ];
    default = "ghostty";
    description = "Preferred terminal emulator for launch commands";
  };
  
  editor_config = mkOption {
    type = types.submodule {
      options = {
        set_editor = mkOption {
          type = types.bool;
          default = true;
          description = "Whether to set EDITOR environment variable";
        };
        override_existing = mkOption {
          type = types.bool;
          default = true;
          description = "Whether to override existing EDITOR if already set";
        };
        editor_command = mkOption {
          type = types.str;
          default = "hx";
          description = "Custom editor command (hx, vim, nvim, etc.)";
        };
      };
    };
    default = {};
    description = "Editor configuration options";
  };
  
  debug_mode = mkOption {
    type = types.bool;
    default = false;
    description = "Enable verbose debug logging";
  };
  
  skip_welcome_screen = mkOption {
    type = types.bool;
    default = false;
    description = "Skip the welcome screen on startup";
  };
  
  ascii_art_mode = mkOption {
    type = types.enum [ "static" "animated" ];
    default = "animated";
    description = "ASCII art display mode";
  };
  
  show_macchina_on_welcome = mkOption {
    type = types.bool;
    default = true;
    description = "Show macchina system info on welcome screen";
  };
  
  persistent_sessions = mkOption {
    type = types.bool;
    default = false;
    description = "Enable persistent Zellij sessions";
  };
  
  session_name = mkOption {
    type = types.str;
    default = "yazelix";
    description = "Session name for persistent sessions";
  };
  
  user_packages = mkOption {
    type = types.listOf types.package;
    default = [];
    description = "Additional packages to install in Yazelix environment";
  };
};
```

## Implementation Strategy

### What the Module Does
1. **Generates `yazelix.nix`**: Creates configuration file from Home Manager options
2. **XDG Compliance**: Places config at `~/.config/yazelix/yazelix.nix`
3. **Type Safety**: Validates all configuration options
4. **No File Management**: Never touches Yazelix repository files

### What the Module Does NOT Do
1. **No Repository Management**: User manually clones Yazelix repo
2. **No Package Installation**: Packages installed via `nix develop`
3. **No File Conflicts**: Only manages single config file
4. **No Direct Integration**: User still runs `nix develop ~/.config/yazelix`

### User Workflow
1. User clones Yazelix repository manually: `git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix`
2. User enables module in Home Manager configuration
3. Module generates `~/.config/yazelix/yazelix.nix` from HM options
4. User runs `nix develop ~/.config/yazelix` as usual
5. Yazelix reads generated config and works normally

## Configuration Generation Logic

```nix
xdg.configFile."yazelix/yazelix.nix" = mkIf cfg.enable {
  text = ''
    { pkgs }:
    {
      # Dependency groups
      recommended_deps = ${if cfg.recommended_deps then "true" else "false"};
      yazi_extensions = ${if cfg.yazi_extensions then "true" else "false"};
      yazi_media = ${if cfg.yazi_media then "true" else "false"};
      
      # Helix configuration
      helix_mode = "${cfg.helix_mode}";
      
      # Shell configuration
      default_shell = "${cfg.default_shell}";
      extra_shells = ${builtins.toJSON cfg.extra_shells};
      
      # Terminal configuration
      terminals = "${cfg.terminals}";
      manage_terminals = "${cfg.manage_terminals}";
      
      # Editor configuration
      ${if cfg.editor_config.set_editor then ''
      set_editor = true;
      override_existing = ${if cfg.editor_config.override_existing then "true" else "false"};
      editor_command = "${cfg.editor_config.editor_command}";
      '' else ''
      set_editor = false;
      override_existing = false;
      editor_command = "hx";
      ''}
      
      # Debug and display options
      debug_mode = ${if cfg.debug_mode then "true" else "false"};
      skip_welcome_screen = ${if cfg.skip_welcome_screen then "true" else "false"};
      ascii_art_mode = "${cfg.ascii_art_mode}";
      show_macchina_on_welcome = ${if cfg.show_macchina_on_welcome then "true" else "false"};
      
      # Session configuration
      persistent_sessions = ${if cfg.persistent_sessions then "true" else "false"};
      session_name = "${cfg.session_name}";
      
      # User packages
      user_packages = with pkgs; ${nixPackagesToString cfg.user_packages};
    }
  '';
};
```

## Safety Guarantees

### File Collision Prevention
- Only manages `~/.config/yazelix/yazelix.nix`
- Never touches repository files or other configurations
- Uses Home Manager's built-in collision detection

### Rollback Support
- Configuration changes are atomic via Home Manager
- User can disable module to revert to manual configuration
- Original `yazelix_default.nix` remains untouched

### Version Independence
- Works with any Yazelix version that supports `yazelix.nix`
- No dependency on specific Yazelix repository state
- User controls repository updates independently

## Migration Path

### From Manual to Home Manager
1. User has existing manual Yazelix installation
2. User backs up current `yazelix.nix` (if exists)
3. User enables Home Manager module
4. Module generates new `yazelix.nix` from HM options
5. User can customize HM configuration and rebuild

### From Home Manager to Manual
1. User disables Home Manager module
2. Module-generated `yazelix.nix` is removed
3. User can copy `yazelix_default.nix` to `yazelix.nix` if needed
4. Back to manual configuration workflow

## Testing Strategy

### Unit Tests
- Configuration generation correctness
- Type validation for all options
- Nix expression syntax validation

### Integration Tests
- Fresh Home Manager installation
- Migration from existing manual setup
- Module enable/disable cycles
- Different configuration combinations

### Safety Tests
- File collision detection
- Rollback scenarios
- Invalid configuration handling
