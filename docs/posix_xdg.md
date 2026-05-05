# POSIX/XDG Paths in Yazelix

Yazelix follows the XDG Base Directory Specification and respects these variables:

- `XDG_CONFIG_HOME` (default: `~/.config`)
- `XDG_DATA_HOME`   (default: `~/.local/share`)
- `XDG_STATE_HOME`  (default: `~/.local/state`)
- `XDG_CACHE_HOME`  (default: `~/.cache`)

## Key Locations

- Config (XDG_CONFIG_HOME)
  - `~/.config/yazelix/settings.jsonc` – canonical user settings file (auto-created from shipped defaults on first run)
  - `~/.config/yazelix/tombi.toml` – managed Tombi support file for formatting Yazelix TOML sidecars
  - `~/.config/yazelix/nushell/config/config.nu` – Yazelix Nushell config sourced into your shell

- Data (XDG_DATA_HOME)
  - `~/.local/share/yazelix/initializers/` – generated init scripts (nushell, starship, zoxide, carapace)
  - `~/.local/share/yazelix/configs/yazi/` – Yazi config used by integrations (`YAZI_CONFIG_HOME`)
  - `~/.local/share/yazelix/configs/zellij/` – generated Zellij config and layouts
  - `~/.local/share/yazelix/logs/` – runtime setup and welcome output

- State (XDG_STATE_HOME)
  - `~/.local/share/yazelix/state/rebuild_hash` – generated-state freshness record used by the trimmed refresh path

- Cache (XDG_CACHE_HOME)
  - Reserved for future use (heavy or reproducible, re‑generable artifacts)

## Environment Variables

Set by Yazelix entrypoints to wire integrations:

- Installed/runtime-owned launch paths export `YAZELIX_RUNTIME_DIR` to point at the active Yazelix runtime root.
- Maintained entrypoints set `IN_YAZELIX_SHELL=true` when executing inside the Yazelix runtime environment.
- `ZELLIJ_DEFAULT_LAYOUT` – chosen layout name (`yzx_side` by default)
- `YAZI_CONFIG_HOME` – `~/.local/share/yazelix/configs/yazi` for consistent Yazi behavior
- `EDITOR` – your configured editor command or Yazelix Helix

Notes:
- If you change `XDG_CONFIG_HOME`, Yazelix looks for config directly under the new `.../yazelix/` path.
- Generated files follow `XDG_DATA_HOME`.
- The supported `yzx` command normally comes from the install owner, such as a Nix profile or Home Manager.
- A stale legacy `~/.local/bin/yzx` wrapper may still exist on older machines, but it is no longer part of the supported install contract.
