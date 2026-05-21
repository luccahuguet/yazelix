# POSIX/XDG Paths

Yazelix separates user-edited config from generated runtime output.

The managed config root resolves in this order:

1. `YAZELIX_CONFIG_DIR`
2. `$XDG_CONFIG_HOME/yazelix`
3. `~/.config/yazelix`

The generated state root resolves in this order:

1. `YAZELIX_STATE_DIR`
2. `$XDG_DATA_HOME/yazelix`
3. `~/.local/share/yazelix`

Yazelix does not use `$XDG_STATE_HOME` for the main generated state root. The state root stays under the data root for the current runtime-materialization contract.

## Key Locations

- User config root, usually `~/.config/yazelix`
  - `settings.jsonc` - canonical user settings file
  - `helix.toml` - managed Helix override surface
  - `zellij.kdl` - managed native Zellij sidecar for settings Yazelix does not render
  - `yazi/` - managed Yazi home containing `yazi.toml`, `keymap.toml`, `init.lua`, `package.toml`, `plugins/`, and `flavors/`
  - `terminal_ghostty.conf`, `terminal_kitty.conf`, `terminal_alacritty.toml`, `terminal_foot.ini` - managed terminal override surfaces
  - `shell_bash.sh`, `shell_zsh.zsh`, `shell_fish.fish`, `shell_nu.nu` - managed shell hook surfaces

- Shared Ghostty cursor config, usually `~/.config/yazelix_ghostty_cursors`
  - `settings.jsonc` - standalone cursor preset config used by Yazelix and `yzc`

- Generated state root, usually `~/.local/share/yazelix`
  - `configs/yazi/` - generated Yazi config used through `YAZI_CONFIG_HOME`
  - `configs/zellij/` - generated Zellij config, layouts, and plugin artifacts
  - `configs/helix/` - generated managed Helix config
  - `configs/terminal_emulators/` - generated terminal config files
  - `initializers/` - generated shell, starship, zoxide, mise, and carapace init scripts
  - `logs/` - runtime setup, launch, and welcome output
  - `profiles/startup/` - startup profiler reports and saved baselines
  - `sessions/` - per-session facts used by runtime integrations
  - `state/rebuild_hash` - generated-state freshness record

- Cache root, usually `~/.cache`
  - Main Yazelix runtime paths do not currently require a top-level Yazelix cache directory
  - Standalone child tools can use XDG cache paths where their own contracts document cache behavior

## Environment Variables

Set by Yazelix entrypoints to wire integrations:

- Installed/runtime-owned launch paths export `YAZELIX_RUNTIME_DIR` to point at the active Yazelix runtime root.
- Packaged and Home Manager entrypoints may export `YAZELIX_CONFIG_DIR` and `YAZELIX_STATE_DIR` to bind Yazelix to owner-provided roots.
- Maintained entrypoints set `IN_YAZELIX_SHELL=true` when executing inside the Yazelix runtime environment.
- `ZELLIJ_DEFAULT_LAYOUT` – chosen layout name (`yzx_side` by default)
- `YAZI_CONFIG_HOME` – `<state-root>/configs/yazi` for consistent Yazi behavior
- `EDITOR` – your configured editor command or Yazelix Helix

Notes:
- If you change `XDG_CONFIG_HOME`, Yazelix looks for config under the new `$XDG_CONFIG_HOME/yazelix` path unless `YAZELIX_CONFIG_DIR` is set.
- Generated files follow `YAZELIX_STATE_DIR` when set, otherwise `$XDG_DATA_HOME/yazelix`.
- The supported `yzx` command normally comes from the install owner, such as a Nix profile or Home Manager.
- A stale legacy `~/.local/bin/yzx` wrapper may still exist on older machines, but it is no longer part of the supported install contract.
