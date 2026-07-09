# Terminal Override Layers

## Summary

Yazelix owns generated terminal config for the packaged Mars path. Ghostty, Kitty, Rio, WezTerm, Foot, Ratty, Alacritty, and other capable terminals are supported through `yzx enter`; their native terminal config remains user-owned.

## Current Behavior

- Yazelix generates Mars config under the runtime state directory
- `terminal.config_mode = "yazelix"` uses Yazelix-managed Mars config
- `terminal.config_mode = "user"` loads the host Mars config path and fails fast if it is missing
- Yazelix does not create `terminal_ghostty.conf` or `terminal_kitty.conf`
- Host terminal preferences for Ghostty, Kitty, Rio, WezTerm, Foot, Ratty, Alacritty, or another emulator belong in that terminal's own config

## Non-goals

- Generic terminal-config merging
- Full ownership handoff of host terminal config files
- Reintroducing generated non-Mars terminal configs
- Creating Yazelix-specific terminal override sidecars for user-owned terminal configs

## Acceptance Cases

1. Yazelix materializes only the generated Mars terminal config
2. Yazelix does not create Ghostty or Kitty override stubs under `~/.config/yazelix`
3. Host terminal setup documentation points users to `yzx enter` instead of generated non-Mars configs

## Verification

- unit tests: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core terminal_materialization`
