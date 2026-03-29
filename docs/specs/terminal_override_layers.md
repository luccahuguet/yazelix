# Terminal Override Layers

## Summary

Yazelix should keep owning launch and integration-critical terminal behavior while giving users a clean, Yazelix-specific override layer for terminal-local preferences. The initial supported terminals are Ghostty, Kitty, and Alacritty.

## Why

The real customization need is not full terminal-config ownership. It is a safe way to inject themes, fonts, opacity, padding, cursor style, and similar preferences without making Yazelix startup behavior depend on ambient terminal config files.

## Scope

- Ghostty, Kitty, and Alacritty terminal override layering
- Yazelix-managed base terminal configs
- Yazelix-specific user override files under `~/.config/yazelix/user_configs/terminal/`
- launcher ownership of startup behavior where needed so user override files can stay broad

## Behavior

- Yazelix generates its own managed base config for supported terminals.
- Yazelix also supports a per-terminal user override file under `~/.config/yazelix/user_configs/terminal/`.
- User override files are automatically picked up when Yazelix launches in the managed-config path.
- `terminal.config_mode = "yazelix"` keeps using the managed config plus the Yazelix-specific override layer.
- `terminal.config_mode = "user"` switches to the terminal's real native config path and fails fast if that file does not exist.
- Yazelix does not read the terminal's normal default config by default for this override feature.
- Startup behavior remains Yazelix-owned at the launcher layer, even when the terminal config file itself comes from the user path.

## Non-goals

- Generic terminal-config merging
- Full ownership handoff to user configs
- Using the terminal's normal config location as the default override source
- Solving WezTerm or Foot in this first slice

## Acceptance Cases

1. When Yazelix generates Ghostty, Kitty, or Alacritty configs, it also supports a Yazelix-specific user override file for that terminal.
2. When a user adds harmless terminal-native settings to the override file, Yazelix picks them up automatically in the managed-config path.
3. Launch-critical startup behavior for Ghostty, Kitty, and Alacritty stays Yazelix-owned instead of being overridden through the terminal override file.
4. The override-layer implementation is simpler than a full terminal ownership split.

## Verification

- unit tests: `nushell/scripts/dev/test_yzx_generated_configs.nu`
- integration tests: relocated runtime smoke in `nushell/scripts/dev/test_yzx_core_commands.nu`
- manual verification: edit each override file and confirm the launched terminal reflects the preference change

## Traceability

- Bead: `yazelix-tee`
- Defended by: `nushell/scripts/dev/test_yzx_generated_configs.nu`

## Open Questions

- Whether Yazelix should warn on obviously dangerous keys inside override files
