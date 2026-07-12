# Directional Placement Keymap Decision

## Status

Accepted for the final Classic bridge and the Nova-shaped semantic root

The final Classic runtime still implements sidebar and popup placement internally. The source swap changes the right agent surface from a sidebar to a popup without changing its configurable `Alt Shift L` default

## Directional Layer

The default placement layer is:

| Binding | Final Classic action | Nova action after the source swap |
| --- | --- | --- |
| `Alt Shift H` | Toggle the left file sidebar | Toggle the left file sidebar |
| `Alt Shift J` | Toggle the bottom Git popup | Open the managed Git popup |
| `Alt Shift K` | Toggle the top config popup | Open the config popup |
| `Alt Shift L` | Toggle the right agent sidebar | Open the managed agent popup |

`Alt Shift M` remains the command palette and `Alt Shift F` remains focused-pane fullscreen

The final Classic structural movement layer stays fixed at `Ctrl Alt H/L` for tab movement and `Ctrl Alt J/K` for pane movement. `Alt h/l` remains the focus-and-tab walking layer

## Config Ownership

Only the four chords that cross the source swap are semantic root fields:

```toml
[keybindings]
config = "Alt Shift K"
agent = "Alt Shift L"
git = "Alt Shift J"
menu = "Alt Shift M"
```

Custom popup definitions own their chord under `popups.<id>.keybinding`

Fixed Classic integration bindings such as `Alt Shift H`, `Alt Shift F`, `Ctrl y`, `Ctrl Shift Y`, `Alt r`, and structural movement are runtime projection details, not additional root configuration languages. Managed `zellij/config.kdl` rejects `keybinds` blocks; users who need complete native keymap ownership should run plain Zellij outside Yazelix

## Focus Bindings

- `Ctrl y` moves focus between the managed editor and left sidebar and survives the source swap
- `Ctrl Shift Y` moves focus between the editor and the Classic right agent sidebar and retires with that sidebar
- `Alt Shift L` is the durable one-stroke agent action

## Terminal Input

The generated final Classic Zellij config enables the Kitty keyboard protocol so modified-letter chords remain distinct. Host terminals must support the protocol correctly; terminal-native settings remain terminal concerns

`Ctrl Alt` may overlap AltGr on international layouts. The four durable placement chords use `Alt Shift`, while complete native remapping remains outside the managed Yazelix Zellij surface

## Evidence

- `config_metadata/main_config_contract.toml`
- `docs/contracts/keybinding_action_ownership.md`
- `rust_core/yazelix_core/src/action_registry.rs`
- `rust_core/yazelix_core/src/classic_nova_root_translation.rs`
- `rust_core/yazelix_zellij_config_pack/src/lib.rs`
