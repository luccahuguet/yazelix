# Yazelix Home Manager Module

A Home Manager module for [Yazelix](https://github.com/luccahuguet/yazelix) that declaratively manages the package-ready runtime surface alongside `yazelix.toml` and `yazelix_packs.toml`.

## What This Module Does

- **Generates `yazelix.toml` and `yazelix_packs.toml`** from Home Manager options
- **Installs the managed Yazelix runtime** at `~/.local/share/yazelix/runtime/current`
- **Adds `yzx` to the Home Manager profile** through the managed runtime package
- **Installs icons and a desktop entry** that target the managed runtime
- **Keeps the config surface type-safe** with Home Manager validation

## What This Module Does NOT Do

- Does not require or manage a live Yazelix git clone for normal usage
- Does not replace Nix itself; you still need a flake-enabled Nix install
- Does not install a separate host/global Nushell for your everyday shell usage
- Does not auto-enter a Yazelix shell on `home-manager switch`
- Does not manage or require host shell-hook injection for the Home Manager profile `yzx` path

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
- the `yzx` command in your Home Manager profile, typically `~/.nix-profile/bin/yzx`
- `~/.config/yazelix/user_configs/yazelix.toml`
- `~/.config/yazelix/user_configs/yazelix_packs.toml`
- a Home Manager profile desktop entry, typically `~/.nix-profile/share/applications/yazelix.desktop`

Then open a fresh shell and run:

```bash
yzx launch
```

For maintainer workflows, a cloned repo is still useful. Normal Home Manager usage should not depend on treating `~/.config/yazelix` as a live repo checkout.

## Validated Behavior

Manual validation on April 8, 2026 covered both a lived-in account and a throwaway clean-room Home Manager activation.

- Home Manager owns `runtime/current`, the profile-provided `yzx` command, and the generated `user_configs/` TOML files through symlinks into the Home Manager profile.
- The managed `yzx` command resolves through the Home Manager profile, typically `~/.nix-profile/bin/yzx`, rather than through the manual installer's `~/.local/bin/yzx` path.
- The Home Manager desktop entry comes from the Home Manager profile, typically `~/.nix-profile/share/applications/yazelix.desktop`, rather than from `yzx desktop install`.
- Old manual desktop-entry files under `~/.local/share/applications/` can linger after migration; they are not Home Manager-owned and will shadow the Home Manager profile entry until you remove them.
- Host shell hooks are optional for the Home Manager path. Launch through `yzx` or the Home Manager desktop entry; do not expect `home-manager switch` to rewrite `.bashrc` or `~/.config/nushell/config.nu`.

## Example Configuration

See [examples/example.nix](./examples/example.nix) for a comprehensive example showing all available options.

## Migration Guide

### From Manual to Home Manager

1. **Backup your current configuration:**
   ```bash
   cp ~/.config/yazelix/user_configs/yazelix.toml ~/.config/yazelix/user_configs/yazelix.toml.backup
   ```

2. **Configure the Home Manager module** (see example.nix)

3. **Apply with collision backups:**
   ```bash
   home-manager switch -b hm-backup
   ```

Manual installs commonly already own:
- `~/.local/share/yazelix/runtime/current`
- `~/.config/yazelix/user_configs/yazelix.toml`
- `~/.config/yazelix/user_configs/yazelix_packs.toml`

Using `-b hm-backup` lets Home Manager move those remaining unmanaged files aside instead of aborting the switch. It is a takeover aid for `runtime/current` and generated config collisions, not the main launcher story.

4. **Verify the Home Manager-owned surfaces:**
   ```bash
   readlink -f ~/.nix-profile/bin/yzx
   readlink -f ~/.local/share/yazelix/runtime/current
   ls ~/.nix-profile/share/applications/yazelix.desktop
   yzx --version-short
   ```

5. **Launch Yazelix:**
   ```bash
   yzx launch
   ```

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
- Check that `~/.nix-profile/bin/yzx` exists and that your Home Manager profile bin dir is on your `PATH`
- Check that `~/.nix-profile/share/applications/yazelix.desktop` exists if you expect desktop-launcher integration through Home Manager
- Verify Home Manager configuration syntax
- Run `home-manager switch` to apply changes

### Conflicts with an existing manual install
- Existing manual Yazelix files can cause `home-manager switch` to stop with collision errors
- Prefer `home-manager switch -b hm-backup` for the first takeover
- The most common collision paths are `~/.local/share/yazelix/runtime/current` and the generated TOML files under `~/.config/yazelix/user_configs/`
- A leftover manual `~/.local/bin/yzx` does not belong to the Home Manager path anymore; remove it if you want a pure Home Manager-owned launcher surface
- See example.nix to recreate your settings declaratively instead of editing the generated TOML files directly

### Nushell expectations
- Yazelix launchers use the runtime-local Nushell shipped with the managed runtime
- You do **not** need to add `nushell` to `home.packages` just to make Yazelix launch
- If you want Nushell as your normal interactive shell outside Yazelix, install it separately in your own Home Manager config
- Home Manager does **not** rewrite your personal Bash or Nushell startup files for Yazelix; the profile-provided `yzx` command works without those host-shell hooks

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
