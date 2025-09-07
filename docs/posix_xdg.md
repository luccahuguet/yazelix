# POSIX/XDG Paths in Yazelix

Yazelix follows the XDG Base Directory Specification and respects these variables:

- `XDG_CONFIG_HOME` (default: `~/.config`)
- `XDG_DATA_HOME`   (default: `~/.local/share`)
- `XDG_STATE_HOME`  (default: `~/.local/state`)
- `XDG_CACHE_HOME`  (default: `~/.cache`)

## Key Locations

- Config (XDG_CONFIG_HOME)
  - `~/.config/yazelix/yazelix.nix` – user config (auto‑created from template on first run)
  - `~/.config/yazelix/yazelix_default.nix` – template with defaults
  - `~/.config/yazelix/nushell/config/config.nu` – Yazelix Nushell config sourced into your shell

- Data (XDG_DATA_HOME)
  - `~/.local/share/yazelix/initializers/` – generated init scripts (nushell, starship, zoxide, carapace)
  - `~/.local/share/yazelix/configs/yazi/` – Yazi config used by integrations (`YAZI_CONFIG_HOME`)
  - `~/.local/share/yazelix/logs/` – shell hook logs and setup output

- State (XDG_STATE_HOME)
  - Reserved for future use (session state, runtime metadata)

- Cache (XDG_CACHE_HOME)
  - Reserved for future use (heavy or reproducible, re‑generable artifacts)

## Environment Variables

Set by the dev shell (flake `shellHook`) to wire integrations:

- `YAZELIX_DIR` – points to `~/.config/yazelix` (root for config and bundled scripts)
- `ZELLIJ_DEFAULT_LAYOUT` – chosen layout name (`yzx_side` or `yzx_no_side`)
- `YAZI_CONFIG_HOME` – `~/.local/share/yazelix/configs/yazi` for consistent Yazi behavior
- `HELIX_RUNTIME` – Helix runtime path matching the selected Helix
- `EDITOR` – your configured editor command or Yazelix Helix

Notes:
- If you change `XDG_CONFIG_HOME`, Yazelix looks for `yazelix.nix` under the new `.../yazelix/` path.
- Generated files follow `XDG_DATA_HOME`.
