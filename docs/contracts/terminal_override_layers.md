# Terminal Override Layers

## Summary

Yazelix does not generate terminal config for the packaged default terminal (Kitty); Kitty's native config is user-owned. Ghostty, Kitty, Rio, WezTerm, Foot, Ratty, Alacritty, and other capable terminals are supported through `yzx enter`; their native terminal config remains user-owned. A generated-config materializer for Mars is retained but dormant for legacy compatibility only — Mars is not supported or packaged, and this materializer is not part of the packaged default path.

## Current Behavior

- Yazelix retains a dormant Mars config materializer under the runtime state directory for legacy compatibility; it is not part of the packaged default (Kitty) path
- `terminal.config_mode = "yazelix"` uses the legacy, dormant Yazelix-managed Mars config path when invoked
- `terminal.config_mode = "user"` loads the host Mars config path and fails fast if it is missing, for the same legacy path
- Yazelix does not create `terminal_ghostty.conf` or `terminal_kitty.conf`
- Host terminal preferences for Ghostty, Kitty, Rio, WezTerm, Foot, Ratty, Alacritty, or another emulator belong in that terminal's own config

## Non-goals

- Generic terminal-config merging
- Full ownership handoff of host terminal config files
- Reintroducing Yazelix-generated config for the packaged default terminal (Kitty) or any terminal beyond the legacy, dormant Mars materializer
- Creating Yazelix-specific terminal override sidecars for user-owned terminal configs

## Acceptance Cases

1. Yazelix does not materialize generated config for the packaged default terminal (Kitty); the only generated terminal config surface is the legacy, dormant Mars materializer
2. Yazelix does not create Ghostty or Kitty override stubs under `~/.config/yazelix`
3. Host terminal setup documentation points users to `yzx enter` instead of expecting Yazelix-generated config for their terminal

## Verification

- unit tests: `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core terminal_materialization`
