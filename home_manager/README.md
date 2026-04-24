# Yazelix Home Manager Module

A Home Manager module for [Yazelix](https://github.com/luccahuguet/yazelix) that declaratively manages the package-ready runtime surface alongside `yazelix.toml`.

## What This Module Does

- **Generates `yazelix.toml`** from Home Manager options
- **Adds `yzx` to the Home Manager profile** through the packaged Yazelix runtime
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

If you want a copyable starting point, begin with [examples/minimal_flake](./examples/minimal_flake)
That example uses a repo-local `path:../../..` Yazelix input so it stays buildable inside this repository; when copying it into your own setup, replace that line with `github:luccahuguet/yazelix`

Add this to your `flake.nix` inputs:

```nix
{
  inputs = {
    # ... your existing inputs
    yazelix-hm = {
      url = "github:luccahuguet/yazelix";
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
It pins the Home Manager module from the top-level Yazelix flake instead of the old `?dir=home_manager` subflake path.

### 2. Configure Yazelix in Your Home Manager Configuration

If you already have your own Home Manager flake, the minimal setup is:

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
- the `yzx` command in your Home Manager profile, typically `~/.nix-profile/bin/yzx`
- `~/.config/yazelix/user_configs/yazelix.toml`
- a Home Manager profile desktop entry, typically `~/.nix-profile/share/applications/yazelix.desktop`

Then open a fresh shell and run:

```bash
yzx launch
```

## Updating a Home Manager-owned Install

For a Home Manager-owned Yazelix install, use:

```bash
yzx update home_manager
```

That command prints the exact `nix flake update yazelix` command it runs in the current flake directory, then prints `home-manager switch` for you to copy and run yourself.

This still matters for `path:` inputs because `flake.lock` pins a snapshot of that local path until you refresh it

Do not mix this with `yzx update upstream` for the same installed Yazelix runtime.

After `home-manager switch`, fresh launches use the profile-owned `yzx` wrapper. Already-open Yazelix windows keep running their current live runtime until you explicitly relaunch them or run `yzx restart`; there is no invisible hot-swap of live sessions.

For maintainer workflows, a cloned repo is still useful. Normal Home Manager usage should not depend on treating `~/.config/yazelix` as a live repo checkout.

## Validated Behavior

Manual validation on April 8, 2026 covered both a lived-in account and a throwaway clean-room Home Manager activation.

- Home Manager owns the profile-provided `yzx` command and the generated `user_configs/` TOML files through symlinks into the Home Manager profile.
- The managed `yzx` command resolves through the Home Manager profile, typically `~/.nix-profile/bin/yzx`, rather than through a legacy user-local wrapper path.
- The active runtime root resolves directly from the packaged Yazelix runtime in the Home Manager profile/store path, not through a manual-install runtime symlink.
- The Home Manager desktop entry comes from the Home Manager profile, typically `~/.nix-profile/share/applications/yazelix.desktop`, rather than from `yzx desktop install`.
- A stale legacy `~/.local/bin/yzx` wrapper can still shadow the profile-owned command on `PATH` after migration; archive it with `yzx home_manager prepare --apply` or remove it manually so `yzx` resolves to the Home Manager profile path.
- Old manual desktop-entry files under `~/.local/share/applications/` can linger after migration; they are not Home Manager-owned and will shadow the Home Manager profile entry until you remove them.
- Host shell hooks are optional for the Home Manager path. Launch through `yzx` or the Home Manager desktop entry; do not expect `home-manager switch` to rewrite `.bashrc` or `~/.config/nushell/config.nu`.

Migration note for older setups:
- Replace `github:luccahuguet/yazelix?dir=home_manager` with `github:luccahuguet/yazelix` in your Home Manager flake inputs.
- Profile installs use `yzx update upstream`; Home Manager installs use `yzx update home_manager`.

## Example Configuration

- Use [examples/minimal_flake](./examples/minimal_flake) for a real minimal flake you can copy and adapt
- Use [examples/example.nix](./examples/example.nix) for a comprehensive option surface example

## Migration Guide

### From Manual to Home Manager

1. **Backup your current configuration:**
   ```bash
   cp ~/.config/yazelix/user_configs/yazelix.toml ~/.config/yazelix/user_configs/yazelix.toml.backup
   ```

2. **Configure the Home Manager module** (see example.nix)

3. **Prepare the existing manual install for takeover:**
   ```bash
   yzx home_manager prepare
   yzx home_manager prepare --apply
   ```

The prepare command archives the common file-based takeover blockers and handoff cleanup paths, and it removes standalone default-profile Yazelix package entries that would collide with Home Manager:
- `~/.config/yazelix/user_configs/yazelix.toml`
- standalone default-profile `yazelix` entries from `nix profile list --json`
- `~/.local/bin/yzx` when it is the legacy Yazelix manual wrapper
- `~/.local/share/applications/com.yazelix.Yazelix.desktop`
- `~/.local/share/icons/hicolor/*/apps/yazelix.png`

4. **Apply the Home Manager configuration:**
   ```bash
   home-manager switch
   ```

If Home Manager still reports an unexpected unmanaged-file collision outside those paths, `home-manager switch -b hm-backup` remains a fallback aid. It is no longer the primary Yazelix migration story.

5. **Verify the Home Manager-owned surfaces:**
   ```bash
   readlink -f ~/.nix-profile/bin/yzx
   ls ~/.nix-profile/share/applications/yazelix.desktop
   yzx --version-short
   ```

6. **Launch Yazelix:**
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

3. **Restore manual config:** recreate `~/.config/yazelix/user_configs/yazelix.toml` from your backup or from the shipped default template in the Yazelix package/repo you install manually.

## Safety Features

- **File collision detection** - Uses Home Manager's built-in collision prevention
- **Atomic changes** - Configuration changes are atomic via Home Manager
- **Easy rollback** - Disable module to revert to manual configuration
- **No repository management requirement** - Normal usage does not depend on a live Yazelix git repository

## Troubleshooting

### Configuration not applied
- Check that `~/.config/yazelix/user_configs/yazelix.toml` was created
- Check that `~/.nix-profile/bin/yzx` exists and that your Home Manager profile bin dir is on your `PATH`
- Check that `~/.nix-profile/share/applications/yazelix.desktop` exists if you expect desktop-launcher integration through Home Manager
- Verify Home Manager configuration syntax
- Run `home-manager switch` to apply changes

### Conflicts with an existing manual install
- Existing manual Yazelix files can cause `home-manager switch` to stop with collision errors
- Prefer `yzx home_manager prepare --apply` before the first takeover
- The most common collision paths are the generated TOML files under `~/.config/yazelix/user_configs/`
- `home-manager switch -b hm-backup` is now the fallback aid if you still hit an unexpected unmanaged-file collision after the prepare step
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
cd /path/to/cloned/yazelix
nix develop
```

Use the repo root environment and your preferred Nix formatting/lint tools as needed.

## Contributing

This module follows Yazelix's configuration structure defined in `yazelix_default.toml`. When adding new options:

1. Add the option to both `yazelix_default.toml` and this module
2. Update the examples and documentation
3. Test with both new and existing Yazelix installations
4. Ensure type safety and proper defaults
