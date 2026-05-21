# Directional Placement Keymap Decision

## Status

Accepted and implemented as the default Yazelix directional surface map.

## Decision

Use `Alt+Shift+h/j/k/l` as the placement-surface layer:

- `Alt+Shift+h`: toggle or summon `left_sidebar` / `file_tree`
- `Alt+Shift+j`: toggle or summon `bottom_popup` / `git_client`
- `Alt+Shift+k`: toggle or summon `top_popup` / `config_ui`
- `Alt+Shift+l`: toggle or summon `right_sidebar` / `agent`

Use `Ctrl+Shift` for lower-frequency structural movement:

- `Ctrl+Shift+h`: move current tab left
- `Ctrl+Shift+l`: move current tab right
- `Ctrl+Shift+j`: move current pane down
- `Ctrl+Shift+k`: move current pane up

Keep plain `Alt+h/l` on the existing focus/walk layer. Do not use plain
`Alt+h/j/k/l` for placement because those keys already overlap Zellij's normal
pane movement language and Yazelix's current left/right pane walking.

Keep `Alt+Shift+M` as the command palette/menu key. Do not assign
`command_pane` to the directional placement layer.

## Default Status

The `Alt+Shift+h/j/k/l` placement layer is default-on.

The `Ctrl+Shift+h/j/k/l` structural movement layer is default-on for tab and
pane movement. These bindings remain remappable through
`zellij.native_keybindings`.

## Current Conflict Check

Current Yazelix default semantic bindings:

- `Alt+h` / `Alt+Left`: `move_focus_left_or_tab`
- `Alt+l` / `Alt+Right`: `move_focus_right_or_tab`
- `Alt+Shift+J`: `bottom_popup`
- `Alt+Shift+K`: `top_popup`
- `Alt+Shift+M`: `menu`
- `Alt+Shift+C`: `config`
- `Ctrl+y`: `toggle_editor_sidebar_focus`
- `Ctrl+Shift+Y`: `toggle_editor_right_sidebar_focus`
- `Alt+Shift+H`: `toggle_left_sidebar`
- `Alt+Shift+L`: `open_codex_agent_right`
- `popup`: unbound by default and still configurable

Current Yazelix native Zellij policy:

- `Ctrl+Shift+H`: move tab left
- `Ctrl+Shift+L`: move tab right
- `Ctrl+Shift+J`: move pane down
- `Ctrl+Shift+K`: move pane up
- `Alt+Shift+F`: toggle focused pane fullscreen
- `Ctrl+Alt+p`: toggle pane in group
- `Ctrl+Alt+Shift+P`: toggle group marking
- `Ctrl+Alt+g`, `Ctrl+Alt+s`, `Ctrl+Alt+o`: moved mode keys

Implemented changes:

- moved tab movement from `Alt+Shift+H/L` to `Ctrl+Shift+H/L`
- added pane movement on `Ctrl+Shift+J/K`
- moved the default popup-program flow from `Alt+t` to `bottom_popup` on
  `Alt+Shift+J`
- added `top_popup` on `Alt+Shift+K`
- moved sidebar visibility from `Alt+y` to `Alt+Shift+H`
- moved the managed Codex agent sidebar to `Alt+Shift+L`
- freed `Alt+t`

Do not keep legacy aliases by default. Users can still remap semantic actions
through `zellij.keybindings` and native Zellij actions through
`zellij.native_keybindings`.

`Alt+Shift+F` does not conflict with the accepted placement keys and should stay
as fullscreen. `Alt+Shift+M` stays menu/command palette.

## Upstream Zellij Check

The local upstream Zellij default config uses:

- `Alt+i` / `Alt+o` for `MoveTab "Left"` and `MoveTab "Right"`
- `Alt+h/l` for `MoveFocusOrTab`
- `Alt+j/k` for vertical `MoveFocus`
- `Alt+[` / `Alt+]` for swap-layout movement
- `Alt+Shift+p` for `ToggleGroupMarking`

The local upstream Zellij action parser and action enum support the required
native actions:

- `MoveTab "Left"`
- `MoveTab "Right"`
- `MovePane "Down"`
- `MovePane "Up"`

Yazelix already unbinds upstream `Alt+i` and `Alt+o`, so moving tab movement to
`Ctrl+Shift+h/l` keeps Yazelix's curated tab-move layer instead of reviving the
upstream keys.

## Terminal Behavior

`Alt+Shift+h/j/k/l` is the safer default placement layer. It avoids control-key
ASCII ambiguity and has prior runtime feedback as the more reliable layer while
`zellij.support_kitty_keyboard_protocol = false`.

`Ctrl+Shift+h/j/k/l` is accepted for structural movement because it preserves
the same hjkl direction language without consuming the `Alt+Shift` placement
surface.

Ghostty supports the Kitty keyboard protocol. No extra Ghostty setting is
required by this decision, but the implementation should test both
`zellij.support_kitty_keyboard_protocol = false` and `true`.

WezTerm requires `enable_kitty_keyboard = true` before it honors application
requests to use the Kitty keyboard protocol. If Yazelix documents or recommends
`zellij.support_kitty_keyboard_protocol = true` for the movement layer in
WezTerm, the matching WezTerm config must be documented too:

```lua
config.enable_kitty_keyboard = true
```

Do not enable Zellij's kitty-keyboard setting globally just to make this keymap
work until the normal Yazelix terminal matrix proves it is stable.

## Ctrl+Shift+Y

Ship `Ctrl+Shift+Y` as the default trial binding for right-sidebar focus.

Without the Kitty keyboard protocol, Ctrl+Shift-letter chords may be
indistinguishable from Ctrl-letter chords in terminal input. Since Yazelix's
current default is `zellij.support_kitty_keyboard_protocol = false`, this may
collide with the existing `Ctrl+y` left sidebar/editor focus action in some
terminal paths.

The right-sidebar focus action remains semantic and remappable through
`zellij.keybindings.toggle_editor_right_sidebar_focus`. If the default aliases
to `Ctrl+y` in a user's terminal, they can remap or disable it without changing
the underlying pane-orchestrator command.

## AltGr And International Layouts

`Ctrl+Alt` is commonly equivalent to AltGr on international keyboard layouts.
That is why the accepted structural movement layer uses `Ctrl+Shift` instead of
`Ctrl+Alt`.

Placement visibility stays on `Alt+Shift` because it is more likely to be
usable on non-US layouts. All accepted actions must remain remappable through
semantic config so users with AltGr conflicts can choose local keys.

## Implementation Notes

- `bottom_popup` and `top_popup` are semantic names over distinct configured
  entries in `zellij.popup_commands` and shared popup geometry for now. They
  are excellent remap defaults, not separate placement engines yet.
- Old default `Alt+Shift+H/L`, `Alt+t`, and `Alt+y` bindings are not kept as
  compatibility aliases.
- `yzx keys`, README/docs keybinding surfaces, config UI descriptions, and
  Home Manager defaults should stay aligned with the action registry.
- Manual terminal testing should still cover Ghostty and WezTerm with
  `zellij.support_kitty_keyboard_protocol = false` and `true`.
- For WezTerm plus kitty-keyboard mode, test with `enable_kitty_keyboard = true`.

## Evidence Checked

- `config_metadata/main_config_contract.toml`
- `configs/zellij/yazelix_overrides.kdl`
- `rust_core/yazelix_core/src/action_registry.rs`
- `rust_core/yazelix_core/src/keys_commands.rs`
- `/home/lucca/pjs/open_source/yazelix_related/zellij/example/default.kdl`
- `/home/lucca/pjs/open_source/yazelix_related/zellij/zellij-utils/src/input/actions.rs`
- `/home/lucca/pjs/open_source/yazelix_related/zellij/zellij-utils/src/kdl/mod.rs`
- WezTerm official `enable_kitty_keyboard` and keyboard-encoding docs
- Ghostty official feature docs for Kitty keyboard protocol support
