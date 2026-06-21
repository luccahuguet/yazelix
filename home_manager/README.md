# Yazelix Home Manager Module

A Home Manager module for [Yazelix](https://github.com/luccahuguet/yazelix) that manages the package-ready runtime surface while leaving `settings.jsonc` mutable by default

## What This Module Does

- **Leaves `settings.jsonc` mutable by default** so users can edit it directly
- **Can generate `settings.jsonc`** from Home Manager options when `manage_config = true`, including the hidden deterministic ratconfig contract state Yazelix requires
- **Adds `yzx` to the Home Manager profile** through the packaged Yazelix runtime
- **Selects one packaged terminal** with Ghostty by default, Rio, WezTerm, and Kitty as alternates, and Foot/Ratty as Linux options through `terminal`
- **Installs icons and, on Linux, a desktop entry** that target the managed runtime
- **Keeps the config surface type-safe** with Home Manager validation

Config ownership is configurable: set `programs.yazelix.manage_config = true` only if you want Home Manager to generate and own `~/.config/yazelix/settings.jsonc`
Terminal selection is not stored in `settings.jsonc`; choose it with `programs.yazelix.terminal` or with the flake package output you install

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
    terminal = "ghostty"; # Default; use "kitty", "rio", "wezterm", or Linux-only "foot"/"ratty" for alternate packaged terminal paths
    # Customize other options as needed - see example.nix
    # Set manage_config = true if you want Home Manager to own settings.jsonc
  };
}
```

`terminal` controls the packaged terminal Yazelix launches. There is no fallback to another packaged terminal when this option is selected; a missing or mispackaged terminal fails clearly

`extra_terminal_launchers` installs additional Linux desktop entries such as `New Yazelix - Ghostty`, `New Yazelix - Rio`, and `New Yazelix - WezTerm` without changing the active runtime identity. These entries point directly at their terminal variant packages in the Nix store, so their dependencies stay available without adding duplicate `bin/yzx` commands to the Home Manager profile. `yzx launch --term <terminal>` uses these packaged launchers for non-active terminal variants. Do not include the active `terminal` value in this list; the active terminal already gets the profile-owned launcher

When `manage_config = true`, Home Manager can also own user-defined popup surfaces. The default `zenith` popup is expressed through `custom_popups`, so users can edit it or set the list to `[]` to remove it:

```nix
{
  programs.yazelix = {
    enable = true;
    manage_config = true;
    custom_popups = [
      {
        id = "zenith";
        command = [ "zenith" ];
        keybindings = [ "Alt Shift I" ];
        keep_alive = true;
      }
      {
        id = "btop";
        command = [ "btop" ];
        keybindings = [ "Alt Shift Y" ];
        keep_alive = true;
      }
    ];
  };
}
```

Set `keep_alive = true` for monitor TUIs whose process state should survive focused toggle hides. Leave it unset or set it to `false` for popups that should close on focused toggle

To save space by using tools you already manage on your host, set runtime tool sources per tool:

```nix
{
  programs.yazelix = {
    enable = true;
    runtime_tool_sources = {
      lazygit = "host";
      zenith = "host";
      helix = "host";
      steel = "host";
      yazi = "host";
      ripgrep = "host";
      fd = "host";
    };
  };
}
```

Omitted tools stay `bundled`, except `mise` and `tombi`, which default to `host`. Host mode is for leaf tools such as `lazygit`, `zenith`, `helix`, `steel`, `neovim`, `yazi`, `fzf`, `zoxide`, `starship`, `carapace`, `macchina`, `mise`, `tombi`, `git`, `jq`, `fd`, and `ripgrep`. Bootstrap tools such as Nushell, Zellij, the selected terminal, Nix, POSIX utilities, and graphics wrappers remain bundled

Run `yzx doctor` after switching. Doctor reads the runtime manifest and warns when a required host-sourced command is missing from `PATH`; default optional integrations such as `mise` and `tombi` are informational when absent.

Optional helper tools can also be turned off when you want a smaller runtime and do not use that feature:

```nix
{
  programs.yazelix = {
    enable = true;
    runtime_tool_sources = {
      macchina = "off";
      steel = "off";
      p7zip = "off";
      poppler = "off";
      resvg = "off";
    };
    show_macchina_on_welcome = false;
  };
}
```

Coarser Yazelix subsystems use `components`. Disabling `screen` requires skipping the welcome animation and screen saver; disabling `cursors` removes Yazelix cursor shader assets and cursor config ownership from the runtime

```nix
{
  programs.yazelix = {
    enable = true;
    components = {
      cursors = false;
      screen = false;
    };
    skip_welcome_screen = true;
    screen_saver_enabled = false;
  };
}
```

### Lean Runtime Profile

For a smaller advanced Home Manager install, host-source tools you already manage outside Yazelix and disable optional helpers you do not use:

```nix
{
  programs.yazelix = {
    enable = true;
    manage_config = true;

    runtime_tool_sources = {
      helix = "host";
      steel = "off";
      neovim = "host";
      yazi = "host";
      fzf = "host";
      zoxide = "host";
      starship = "host";
      carapace = "host";
      macchina = "off";
      git = "host";
      jq = "host";
      fd = "host";
      ripgrep = "host";
      lazygit = "host";
      zenith = "host";
      p7zip = "off";
      poppler = "off";
      resvg = "off";
    };

    components = {
      cursors = false;
      screen = false;
    };

    show_macchina_on_welcome = false;
    skip_welcome_screen = true;
    screen_saver_enabled = false;

    zellij_widget_tray = [
      "session"
      "editor"
      "shell"
      "term"
      "cpu"
      "ram"
    ];
    agent_usage_programs = [ ];
  };
}
```

This profile keeps the packaged terminal, Nushell, Zellij, Nix, POSIX helpers, and Linux graphics wrappers bundled. It expects host `PATH` to provide every `host` command, and `yzx doctor` reports missing host-sourced commands after `home-manager switch`

Measured on `x86_64-linux` on June 2, 2026, this package-builder shape reduced the default Ghostty package closure from about 3.1 GiB to about 2.2 GiB. It still carries the bundled Linux `nixGLMesa` wrapper closure, so graphics wrapper ownership remains a major remaining Linux storage cost

Feature tradeoffs:

- Host Yazi may not preserve Yazelix's bundled KGP image-preview behavior
- Host Helix may not match the Yazelix Steel fork behavior
- `steel = "off"` removes Steel authoring commands
- `p7zip`, `poppler`, and `resvg` disable archive, PDF, and SVG preview helpers
- `components.screen = false` removes `yzx screen` and requires welcome and screen-saver behavior to stay disabled
- `components.cursors = false` removes Yazelix cursor shader assets and hides cursor fields from the config UI
- `agent_usage_programs = [ ]` is correct only when `claude_usage` and `codex_usage` are removed from `zellij_widget_tray` or intentionally host-provided

See [Package sizes](../docs/package_sizes.md) for the reporter command and current closure measurements

Yazelix's public `x86_64-linux` Cachix cache speeds up package builds and Home Manager switches when CI has published the current revision. The flake advertises the cache through `nixConfig`, so interactive Nix commands can prompt you to accept it. For persistent Home Manager-managed Nix configuration, add the cache explicitly:

```nix
{ pkgs, ... }: {
  nix.package = pkgs.nix;
  nix.settings.substituters = [
    "https://cache.nixos.org/"
    "https://yazelix.cachix.org"
  ];

  nix.settings.trusted-public-keys = [
    "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
    "yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM="
  ];
}
```

Standalone Home Manager needs `nix.package` when it generates `~/.config/nix/nix.conf`

If your Home Manager configuration already defines Nix caches, keep those cache URLs and keys in the same `substituters` and `trusted-public-keys` lists

### 3. Install and Use Yazelix

Run:

```bash
home-manager switch
```

This creates:
- the `yzx` command in your Home Manager profile, typically `~/.nix-profile/bin/yzx`
- `~/.config/yazelix/settings.jsonc`, bootstrapped as a mutable file by default or Home Manager-generated when `manage_config = true`
- on Linux, a Home Manager profile desktop entry such as `~/.nix-profile/share/applications/com.yazelix.Yazelix.Ghostty.desktop`

Then open a fresh shell and run:

```bash
yzx launch
```

## Updating a Home Manager-owned Install

For a Home Manager-owned Yazelix install, use:

```bash
yzx update home_manager
```

That command prints the exact `nix flake update yazelix` command it runs in the current flake directory, then prints `home-manager switch` for you to copy and run yourself

If activation makes your machine throttle, run the printed switch command with per-invocation Nix limits

```bash
NIX_CONFIG=$'max-jobs = 1\ncores = 8\neval-cores = 8' home-manager switch
```

Choose the `cores` and `eval-cores` values as a percentage of your logical CPU count, such as `8` on a 16-thread machine for roughly half the CPU budget
`max-jobs = 1` keeps concurrent derivations from multiplying that budget during activation
Changing the global defaults for every Nix command still requires root-owned Nix configuration

This still matters for `path:` inputs because `flake.lock` pins a snapshot of that local path until you refresh it
If you point Home Manager at a local Yazelix git checkout, prefer `git+file:///absolute/path/to/yazelix` over `path:/absolute/path/to/yazelix` so Nix uses the Git working tree instead of snapshotting the whole directory

Do not mix this with `yzx update upstream` for the same installed Yazelix runtime.

After `home-manager switch`, fresh launches use the profile-owned `yzx` wrapper. Already-open Yazelix windows keep running their current live runtime until you explicitly relaunch them or run `yzx restart`; there is no invisible hot-swap of live sessions.

For maintainer workflows, a cloned repo is still useful. Normal Home Manager usage should not depend on treating `~/.config/yazelix` as a live repo checkout.

## Validated Behavior

Manual validation on April 8, 2026 covered both a lived-in account and a throwaway clean-room Home Manager activation.

- By default, Home Manager owns the package/runtime integration while Yazelix bootstraps the main `settings.jsonc` as a mutable file
- Set `programs.yazelix.manage_config = true` only if you want Home Manager to own generated Yazelix settings through a symlink into the Home Manager profile
- The managed `yzx` command resolves through the Home Manager profile, typically `~/.nix-profile/bin/yzx`, rather than through a legacy user-local wrapper path.
- The active runtime root resolves directly from the packaged Yazelix runtime in the Home Manager profile/store path, not through a manual-install runtime symlink.
- On Linux, the Home Manager desktop entries come from the Home Manager profile, including the active Yazelix entry
- Optional `extra_terminal_launchers` entries also live in the Home Manager profile and point at their terminal variant store packages instead of installing extra profile `yzx` commands
- A stale legacy `~/.local/bin/yzx` wrapper can still shadow the profile-owned command on `PATH` after migration; archive it with `yzx home_manager prepare --apply` or remove it manually so `yzx` resolves to the Home Manager profile path.
- Old manual desktop-entry files under `~/.local/share/applications/` can linger after migration; they are not Home Manager-owned and will shadow the Home Manager profile entry until you remove them.
- Host shell hooks are optional for the Home Manager path. Launch through `yzx` or, on Linux, the Home Manager desktop entry; do not expect `home-manager switch` to rewrite `.bashrc` or `~/.config/nushell/config.nu`.

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
   cp ~/.config/yazelix/settings.jsonc ~/.config/yazelix/settings.jsonc.backup
   ```

2. **Configure the Home Manager module** (see example.nix)

3. **Prepare the existing manual install for takeover:**
   ```bash
   yzx home_manager prepare
   yzx home_manager prepare --apply
   ```

The prepare command archives the common file-based takeover blockers and handoff cleanup paths, and it removes standalone default-profile Yazelix package entries that would collide with Home Manager:
- `~/.config/yazelix/settings.jsonc`
- standalone default-profile `yazelix` entries from `nix profile list --json`
- `~/.local/bin/yzx` when it is the legacy Yazelix manual wrapper
- `~/.local/share/applications/com.yazelix.Yazelix.desktop`
- `~/.local/share/applications/yazelix.desktop`
- `~/.local/share/icons/hicolor/*/apps/yazelix.png`

Old mutable `yazelix.toml`, `cursors.toml`, and `user_configs/` files are stale config inputs, not Home Manager takeover artifacts. If Yazelix reports them, move them aside manually or run `yzx reset config --yes`

4. **Apply the Home Manager configuration:**
   ```bash
   home-manager switch
   ```

If Home Manager still reports an unexpected unmanaged-file collision outside those paths, `home-manager switch -b hm-backup` remains a fallback aid. It is no longer the primary Yazelix migration story.

5. **Verify the Home Manager-owned surfaces:**
   ```bash
   readlink -f ~/.nix-profile/bin/yzx
   ls ~/.nix-profile/share/applications/com.yazelix.Yazelix.Ghostty.desktop
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

3. **Restore manual config:** recreate `~/.config/yazelix/settings.jsonc` from your backup or by running `yzx reset config` in the Yazelix package/repo you install manually.

## Safety Features

- **File collision detection** - Uses Home Manager's built-in collision prevention
- **Atomic changes** - Configuration changes are atomic via Home Manager
- **Easy rollback** - Disable module to revert to manual configuration
- **No repository management requirement** - Normal usage does not depend on a live Yazelix git repository

## Troubleshooting

### Configuration not applied
- Check that `~/.config/yazelix/settings.jsonc` was created
- By default, that file should be a normal writable file, not a Home Manager store symlink
- Check that `~/.nix-profile/bin/yzx` exists and that your Home Manager profile bin dir is on your `PATH`
- On Linux, check that `~/.nix-profile/share/applications/com.yazelix.Yazelix.Ghostty.desktop` exists if you expect Ghostty desktop-launcher integration through Home Manager
- Verify Home Manager configuration syntax
- Run `home-manager switch` to apply changes

### Conflicts with an existing manual install
- Existing manual Yazelix files can cause `home-manager switch` to stop with collision errors
- Prefer `yzx home_manager prepare --apply` before the first takeover
- The most common collision paths are generated Yazelix settings files under `~/.config/yazelix/`
- By default, Home Manager will not take over the main `settings.jsonc` file
- If you set `programs.yazelix.manage_config = true`, Home Manager owns that file through a profile symlink
- `home-manager switch -b hm-backup` is now the fallback aid if you still hit an unexpected unmanaged-file collision after the prepare step
- See example.nix to recreate your settings declaratively instead of editing the generated settings file directly

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

This module follows Yazelix's configuration structure defined by `config_metadata/main_config_contract.toml`. When adding new options:

1. Add the option to `settings_default.jsonc`, `config_metadata/main_config_contract.toml`, and this module
2. Update the examples and documentation
3. Test with both new and existing Yazelix installations
4. Ensure type safety and proper defaults

Cursor presets and effects live in `~/.config/yazelix_cursors/settings.jsonc`. Set `programs.yazelix.manage_cursor_config = true` only when you want Home Manager to own that cursor registry.
