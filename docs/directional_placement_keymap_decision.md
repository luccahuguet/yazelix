# Directional Placement Keymap Decision

## Status

Accepted for the next role-placement implementation, with a manual terminal
gate before release.

This is a planning decision, not a live keybinding contract. Current Yazelix
defaults stay unchanged until the role-placement implementation updates the
generated keybinding metadata and docs.

## Decision

Use `Alt+Shift+h/j/k/l` as the placement-surface layer:

- `Alt+Shift+h`: toggle or summon `left_sidebar` / `file_tree`
- `Alt+Shift+j`: toggle or summon `bottom_popup` / `git_client`
- `Alt+Shift+k`: toggle or summon `top_popup` / `config_ui`
- `Alt+Shift+l`: toggle or summon `right_sidebar` / `agent`

Use `Ctrl+Alt` for lower-frequency structural movement:

- `Ctrl+Alt+h`: move current tab left
- `Ctrl+Alt+l`: move current tab right
- `Ctrl+Alt+j`: move current pane down
- `Ctrl+Alt+k`: move current pane up

Keep plain `Alt+h/l` on the existing focus/walk layer. Do not use plain
`Alt+h/j/k/l` for placement because those keys already overlap Zellij's normal
pane movement language and Yazelix's current left/right pane walking.

Keep `Alt+Shift+M` as the command palette/menu key. Do not assign
`command_pane` to the directional placement layer.

## Default Status

The `Alt+Shift+h/j/k/l` placement layer should become default-on when the
role-placement implementation lands.

The `Ctrl+Alt+h/j/k/l` structural movement layer is accepted as the replacement
target for tab and pane movement, but it needs a manual terminal gate in the
first-class Ghostty and WezTerm variants before release. If that gate fails for
either first-class terminal, the movement layer should remain remappable or
opt-in rather than blocking the placement layer.

## Current Conflict Check

Current Yazelix default semantic bindings:

- `Alt+h` / `Alt+Left`: `move_focus_left_or_tab`
- `Alt+l` / `Alt+Right`: `move_focus_right_or_tab`
- `Alt+t`: `popup`
- `Alt+Shift+M`: `menu`
- `Alt+Shift+C`: `config`
- `Ctrl+y`: `toggle_editor_sidebar_focus`
- `Alt+y`: `toggle_sidebar`

Current Yazelix native Zellij policy:

- `Alt+Shift+H`: move tab left
- `Alt+Shift+L`: move tab right
- `Alt+Shift+F`: toggle focused pane fullscreen
- `Ctrl+Alt+p`: toggle pane in group
- `Ctrl+Alt+Shift+P`: toggle group marking
- `Ctrl+Alt+g`, `Ctrl+Alt+s`, `Ctrl+Alt+o`: moved mode keys

Required changes:

- move tab movement from `Alt+Shift+H/L` to `Ctrl+Alt+h/l`
- move config UI from `Alt+Shift+C` to `Alt+Shift+k` when config UI becomes
  the `top_popup` role
- move git popup behavior from `Alt+t` to `Alt+Shift+j` when `git_client`
  becomes the `bottom_popup` role
- move sidebar visibility from `Alt+y` to `Alt+Shift+h` when `file_tree`
  becomes the `left_sidebar` role

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
`Ctrl+Alt+h/l` keeps Yazelix's curated tab-move layer instead of reviving the
upstream keys.

## Terminal Behavior

`Alt+Shift+h/j/k/l` is the safer default placement layer. It avoids control-key
ASCII ambiguity and has prior runtime feedback as the more reliable layer while
`zellij.support_kitty_keyboard_protocol = false`.

`Ctrl+Alt+h/j/k/l` is lower confidence because legacy terminal encodings can
collapse Ctrl-letter chords to control bytes, and Alt can be represented as an
escape prefix. This is acceptable for lower-frequency movement only if manual
validation passes in first-class terminals.

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

## Ctrl+Shift+y

Do not ship `Ctrl+Shift+y` as an unconditional default for right-sidebar focus.

Without the Kitty keyboard protocol, Ctrl+Shift-letter chords may be
indistinguishable from Ctrl-letter chords in terminal input. Since Yazelix's
current default is `zellij.support_kitty_keyboard_protocol = false`, a default
`Ctrl+Shift+y` would risk colliding with the existing `Ctrl+y` left
sidebar/editor focus action.

The right-sidebar focus action should remain semantic and remappable. A default
may be added only after the first-class terminal matrix proves it is distinct
from `Ctrl+y`, or if the key is gated behind a profile that enables the needed
keyboard protocol support.

## AltGr And International Layouts

`Ctrl+Alt` is commonly equivalent to AltGr on international keyboard layouts.
That is another reason to keep `Ctrl+Alt+h/j/k/l` on lower-frequency structural
movement instead of core placement.

Placement visibility stays on `Alt+Shift` because it is more likely to be
usable on non-US layouts. All accepted actions must remain remappable through
semantic config so users with AltGr conflicts can choose local keys.

## Follow-On Implementation Notes

When this decision is implemented:

- add semantic placement actions instead of overloading old sidebar/popup names
- add native movement actions for `Ctrl+Alt+h/l/j/k`
- remove old default `Alt+Shift+H/L`, `Alt+Shift+C`, `Alt+t`, and `Alt+y`
  bindings unless the maintainer explicitly asks for compatibility aliases
- update `yzx keys`, README/docs keybinding surfaces, config UI descriptions,
  and Home Manager defaults together
- manually test Ghostty and WezTerm with
  `zellij.support_kitty_keyboard_protocol = false` and `true`
- for WezTerm plus kitty-keyboard mode, test with `enable_kitty_keyboard = true`

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
