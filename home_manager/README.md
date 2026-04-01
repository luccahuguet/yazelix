# Yazelix Home Manager Module

A Home Manager module for [Yazelix](https://github.com/luccahuguet/yazelix) that declaratively manages the package-ready runtime surface alongside `yazelix.toml` and `yazelix_packs.toml`.

## What This Module Does

- **Generates `yazelix.toml` and `yazelix_packs.toml`** from Home Manager options
- **Installs the managed Yazelix runtime** at `~/.local/share/yazelix/runtime/current`
- **Installs a stable `yzx` shim** at `~/.local/bin/yzx`
- **Installs icons and a desktop entry** that target the managed runtime
- **Keeps the config surface type-safe** with Home Manager validation

## What This Module Does NOT Do

- Does not require or manage a live Yazelix git clone for normal usage
- Does not replace Nix itself; you still need a flake-enabled Nix install
- Does not install a separate host/global Nushell for your everyday shell usage
- Does not auto-enter a Yazelix shell on `home-manager switch`

## Quick Start

### 1. Add the Module to Your Home Manager Configuration

Add this to your `flake.nix` inputs:

```nix
{
  inputs = {
    # ... your existing inputs
    yazelix-hm = {
      url = "github:luccahuguet/yazelix?dir=home_manager";
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

This example assumes you use Home Manager with flakes.
It pins the Home Manager module as a normal flake input instead of referencing a user-specific checkout path.

### 2. Configure Yazelix in Your Home Manager Configuration

See [examples/example.nix](./examples/example.nix) for all options. Minimal setup:

```nix
# ~/.config/home-manager/home.nix (or wherever your HM config is)
{
  programs.yazelix = {
    enable = true;
    # Customize other options as needed - see example.nix
  };
}
```

### 3. Install and Use Yazelix

Run:

```bash
home-manager switch
```

This creates:
- `~/.local/share/yazelix/runtime/current`
- `~/.local/bin/yzx`
- `~/.config/yazelix/user_configs/yazelix.toml`
- `~/.config/yazelix/user_configs/yazelix_packs.toml`
- a desktop entry that targets the managed runtime

Then open a fresh shell and run:

```bash
yzx launch
```

For maintainer workflows, a cloned repo is still useful. Normal Home Manager usage should not depend on treating `~/.config/yazelix` as a live repo checkout.

## Example Configuration

See [examples/example.nix](./examples/example.nix) for a comprehensive example showing all available options.

## Migration Guide

### From Manual to Home Manager

1. **Backup your current configuration:**
   ```bash
   cp ~/.config/yazelix/user_configs/yazelix.toml ~/.config/yazelix/user_configs/yazelix.toml.backup
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

3. **Restore manual config:** `cp ~/.local/share/yazelix/runtime/current/yazelix_default.toml ~/.config/yazelix/user_configs/yazelix.toml`

## Safety Features

- **File collision detection** - Uses Home Manager's built-in collision prevention
- **Atomic changes** - Configuration changes are atomic via Home Manager
- **Easy rollback** - Disable module to revert to manual configuration
- **No repository management requirement** - Normal usage does not depend on a live Yazelix git repository

## Troubleshooting

### Configuration not applied
- Check that `~/.config/yazelix/user_configs/yazelix.toml` was created
- Check that `~/.local/share/yazelix/runtime/current` exists
- Check that `~/.local/bin/yzx` exists and is on your `PATH`
- Verify Home Manager configuration syntax
- Run `home-manager switch` to apply changes

### Conflicts with existing yazelix.toml
- The module will overwrite existing `yazelix.toml`
- Backup your manual configuration before enabling the module
- See example.nix to recreate your settings declaratively

### Nushell expectations
- Yazelix launchers use the runtime-local Nushell shipped with the managed runtime
- You do **not** need to add `nushell` to `home.packages` just to make Yazelix launch
- If you want Nushell as your normal interactive shell outside Yazelix, install it separately in your own Home Manager config

### Module not found
- If you use Home Manager with flakes, check that the `yazelix-hm` input reference is correct in your own `flake.nix`
- Ensure the module is properly imported in your Home Manager configuration

## Development

To work on this module:

```bash
cd /path/to/cloned/yazelix/home_manager
devenv shell  # Provides nixpkgs-fmt, statix, deadnix
```

Format and check the code:
```bash
nixpkgs-fmt *.nix examples/*.nix
statix check .
deadnix .
```

## Contributing

This module follows Yazelix's configuration structure defined in `yazelix_default.toml`. When adding new options:

1. Add the option to both `yazelix_default.toml` and this module
2. Update the examples and documentation
3. Test with both new and existing Yazelix installations
4. Ensure type safety and proper defaults
