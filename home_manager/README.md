# Yazelix Home Manager Module

> **Legacy Note:** The current Home Manager module still generates the legacy `yazelix.nix` configuration file. Migration to `yazelix.toml` is planned but not yet implemented.

A **configuration-only** Home Manager module for [Yazelix](https://github.com/luccahuguet/yazelix) that provides declarative configuration management while preserving the existing workflow.

## What This Module Does

- **Generates `yazelix.nix`** from Home Manager options
- **Type-safe configuration** with validation
- **Preserves existing workflow** - you still `git clone` and use `devenv shell`
- **Zero file conflicts** - only manages configuration file
- **Easy migration** - simple enable/disable in Home Manager

## What This Module Does NOT Do

- Does not manage the Yazelix repository (you still clone it manually)
- Does not install packages directly (packages installed via `devenv shell`)
- Does not modify terminal configurations
- Does not replace existing Yazelix functionality

## Quick Start

### 1. Add the Module to Your Home Manager Configuration

Add this to your `flake.nix` inputs:

```nix
{
  inputs = {
    # ... your existing inputs
    yazelix-hm = {
      url = "path:/home/user/.config/yazelix/home_manager";  # Adjust path as needed
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  
  outputs = { home-manager, yazelix-hm, ... }: {
    homeConfigurations.your-user = home-manager.lib.homeManagerConfiguration {
      modules = [
        yazelix-hm.homeManagerModules.default
        # ... your other modules
      ];
    };
  };
}
```

### 2. Configure Yazelix in Your Home Manager Configuration

See [examples/example.nix](./examples/example.nix) for all options. Minimal setup:

```nix
# ~/.config/home-manager/home.nix (or wherever your HM config is)
{
  # REQUIRED: Add nushell to your packages for terminal emulator compatibility
  home.packages = with pkgs; [
    nushell  # Required for Yazelix terminal startup
  ];

  programs.yazelix = {
    enable = true;
    # Customize other options as needed - see example.nix
  };
}
```

### 3. Install and Use Yazelix

Follow the [main Yazelix installation guide](https://github.com/luccahuguet/yazelix#installation) to clone the repository and set up the `yzx` command. Then run `home-manager switch` to apply your configuration.

## Example Configuration

See [examples/example.nix](./examples/example.nix) for a comprehensive example showing all available options.

## Migration Guide

### From Manual to Home Manager

1. **Backup your current configuration:**
   ```bash
   cp ~/.config/yazelix/yazelix.nix ~/.config/yazelix/yazelix.nix.backup
   ```

2. **Configure the Home Manager module** (see example.nix)

3. **Apply:** `home-manager switch`

### From Home Manager back to Manual

1. **Disable the module:**
   ```nix
   programs.yazelix.enable = false;
   ```

2. **Apply the change:**
   ```bash
   home-manager switch
   ```

3. **Restore manual config:** `cp ~/.config/yazelix/yazelix_default.nix ~/.config/yazelix/yazelix.nix`

## Safety Features

- **File collision detection** - Uses Home Manager's built-in collision prevention
- **Atomic changes** - Configuration changes are atomic via Home Manager
- **Easy rollback** - Disable module to revert to manual configuration
- **No repository management** - Never touches the Yazelix git repository

## Troubleshooting

### Configuration not applied
- Check that `~/.config/yazelix/yazelix.nix` was created
- Verify Home Manager configuration syntax
- Run `home-manager switch` to apply changes

### Conflicts with existing yazelix.nix
- The module will overwrite existing `yazelix.nix`
- Backup your manual configuration before enabling the module
- See example.nix to recreate your settings declaratively

### Nushell not found error
- Terminal emulators need `nushell` available in PATH to launch Yazelix
- Add `nushell` to your `home.packages` in Home Manager
- This is required even though Yazelix installs its own nushell via Nix

### Module not found
- Check that the flake path is correct in your `flake.nix`
- Ensure the module is properly imported in your Home Manager configuration

## Development

To work on this module:

```bash
cd ~/.config/yazelix/home_manager
devenv shell  # Provides nixpkgs-fmt, statix, deadnix
```

Format and check the code:
```bash
nixpkgs-fmt *.nix examples/*.nix
statix check .
deadnix .
```

## Contributing

This module follows Yazelix's configuration structure defined in `yazelix_default.nix`. When adding new options:

1. Add the option to both `yazelix_default.nix` and this module
2. Update the examples and documentation
3. Test with both new and existing Yazelix installations
4. Ensure type safety and proper defaults
