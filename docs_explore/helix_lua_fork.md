# Helix Lua Fork (Proposed)

This note captures a clean path to use the Lua fork of Helix
(`https://github.com/gj1118/helix`) with Yazelix today, and a minimal design
for a future `lua_fork` mode.

## Recommended Today: Custom Helix Build

Use a locally built fork and point Yazelix at that binary + runtime. This
keeps all existing Yazelix integrations working and avoids fork-specific
changes in Yazelix.

1. Build the fork locally (example):
   ```bash
   git clone https://github.com/gj1118/helix
   cd helix
   cargo build --release
   ```

2. Point Yazelix at the fork in `~/.config/yazelix/yazelix.toml`:
   ```toml
   [editor]
   command = "/path/to/helix/target/release/hx"

   [helix]
   runtime_path = "/path/to/helix/runtime"
   ```

3. Restart Yazelix to pick up the new binary.

## Future: `lua_fork` Mode (Design Sketch)

Goal: add a dedicated `helix.mode = "lua_fork"` that selects the fork via a
separate Nix input, without changing the default Helix behavior.

Minimal changes:
- Add a new input in `devenv.yaml`:
  `helix_lua_fork: github:gj1118/helix`
- Extend `helix.mode` in `devenv.nix` to accept `lua_fork` and select
  `inputs.helix_lua_fork` when set.
- Update config validation and docs to include the new enum value:
  - `yazelix_default.toml`
  - `nushell/scripts/utils/config_schema.nu`
  - `home_manager/module.nix`
  - `docs/editor_configuration.md`

This keeps the fork opt-in and avoids taking a hard dependency for users who
prefer upstream Helix.
