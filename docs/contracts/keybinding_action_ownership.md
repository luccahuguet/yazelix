# Keybinding Ownership Contract

## Semantic Root

`~/.config/yazelix/config.toml` owns exactly four user-configurable managed surface chords:

```toml
[keybindings]
config = "Alt Shift K"
agent = "Alt Shift L"
git = "Alt Shift J"
menu = "Alt Shift M"
```

Each value is one non-empty Zellij chord. Omission inherits the packaged default. Duplicate chords fail validation before launch.

Custom popups own their chord inside the popup definition:

```toml
[popups.zenith]
command = "zenith"
keybinding = "Alt Shift I"
keep_alive = true
```

## Fixed Classic Policy

The final Classic bridge still contains fixed integration actions such as `Ctrl y`, `Ctrl Shift Y`, `Alt Shift H`, `Alt r`, `Alt m`, fullscreen, tab navigation, and pane movement. They are runtime projection details, not fields in the Nova-shaped semantic root.

`Ctrl Shift Y` focuses the Classic right agent pane and retires at the source swap. `Alt Shift L` survives as `keybindings.agent` and opens Nova's managed agent popup after the swap.

## Native Owners

- Yazi-native bindings live in `~/.config/yazelix/yazi/keymap.toml`.
- Helix-local bindings live in `~/.config/yazelix/helix/config.toml`.
- The guarded Yazelix Zellij sidecar rejects `keybinds` blocks.
- Users who need arbitrary Zellij keymaps run plain Zellij with their host-owned config.
- Terminal emulator shortcuts belong to the terminal emulator.

Yazelix does not translate arbitrary native keymaps into semantic root fields and does not keep the retired `zellij.keybindings`, `zellij.native_keybindings`, or `yazi.keybindings` config languages alive.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core action_registry`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core config_normalize`
- `cargo run --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-config-surface-contract`
- Manual final-Classic smoke for `Ctrl y`, `Ctrl Shift Y`, `Alt Shift H`, and the four configurable surface chords
