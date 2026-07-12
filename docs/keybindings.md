# Keybindings

## Zellij Keybindings

Yazelix uses **selective remapping** - only conflicting Zellij keybindings are changed, preserving muscle memory where possible.

### Core Navigation
The `Alt+Shift+H/J/K/L` layer follows Vim-style spatial placement: `H` is the left sidebar, `J` is the bottom popup, `K` is the top popup, and `L` is the right sidebar.

| Keybinding                | Action                        |
|--------------------------|-------------------------------|
| Alt+number (1-9)         | Go to tab 1-9                 |
| Alt+w                    | Walk to next tab (focus)      |
| Alt+q                    | Walk to previous tab (focus)  |
| Ctrl+Alt+H               | Move tab left                 |
| Ctrl+Alt+L               | Move tab right                |
| Ctrl+Alt+J               | Move pane down                |
| Ctrl+Alt+K               | Move pane up                  |
| Alt+Shift+F              | Toggle pane fullscreen        |
| Alt+Shift+H              | Toggle the left sidebar       |
| Alt+Shift+J              | Toggle the bottom popup       |
| Alt+Shift+K              | Toggle the top popup          |
| Alt+Shift+L              | Toggle the right agent sidebar |
| Alt+Shift+I              | Toggle the keep-alive Zenith process information popup |
| Ctrl+y                   | Toggle editor/left sidebar focus |
| Ctrl+Shift+Y             | Toggle editor/right agent focus |
| Alt+[ / Alt+]            | Previous/next layout family; usually no visible effect with the packaged single family |

### Zellij Modes (Helix-Compatible)
| Keybinding                | Action                        | Notes |
|--------------------------|-------------------------------|-------|
| **Alt+Shift+M**          | **Yazelix menu**              | opens yzx command palette popup |
| **Alt+Shift+J/K**        | **Bottom/top popup**          | opens, focuses, or closes the configured named popup command |
| **Alt+Shift+I**          | **Zenith process information**   | opens, focuses, or hides the bundled process information popup |
| **Ctrl+Alt+g**           | **Locked mode**               | ⚠️ Remapped (was Ctrl+g) |
| **Ctrl+Alt+p**           | **Pane mode**                 | ⚠️ Remapped (was Ctrl+p) |
| **Ctrl+Alt+n**           | **Resize mode**               | ⚠️ Remapped (was Ctrl+n) |
| **Ctrl+Alt+t**           | **Tab mode**                  | ⚠️ Remapped (was Ctrl+t) |
| Ctrl+h                   | Move mode                     | ❌ Removed by default |
| **Ctrl+Alt+q**           | **Quit**                      | ⚠️ Remapped (was Ctrl+q) |
| **Ctrl+Alt+s**           | **Scroll mode**               | ⚠️ Remapped (was Ctrl+s) |
| **Ctrl+Alt+o**           | **Session mode**              | ⚠️ Remapped (was Ctrl+o) |

- **Tab walking**: Alt+w/q walks (focuses) next/previous tab, like browser tab switching.
- **Tab moving**: Ctrl+Alt+H/L moves the current tab left/right.
- **Pane moving**: Ctrl+Alt+J/K moves the current pane down/up.
- **Surface toggles**: Alt+Shift+H/J/K/L maps to left sidebar, bottom popup, top popup, and right sidebar; Alt+Shift+I toggles the keep-alive Zenith process information popup.
- **Direct tab access**: Alt+1 through Alt+9 jumps directly to a tab.

If you find a conflict, please open an issue

## Discoverability of Keybindings
- **Yazelix**: Run `yzx keys` for Yazelix-owned bindings; use `yzx keys yazi`, `yzx keys hx`, or `yzx keys nu` for tool-specific discoverability hints
- **Zellij**: Shows all keybindings visually in the status bar—works out of the box
- **Helix**: Similar to Zellij, key bindings are easy to discover
- **Yazi**: Press `~` to see all keybindings and commands (use `Alt Shift F` to fullscreen the pane for a better view)
- **Nushell**:
  - Run `tutor` in Nushell
  - Read the [Nushell Book](https://www.nushell.sh/book/)
  - Use `help commands | find tables` to search, for example, commands related to tables
- **lazygit**: Press `?` to view keybindings
- **Starship**: Customizable prompt; configure in `~/.config/starship.toml` (see [Starship docs](https://starship.rs/config/))

## Yazelix Custom Keybindings
- **Zellij**:
  - `Alt+Shift+F` toggles pane fullscreen
  - `Alt+Shift+H/J/K/L` toggles the left sidebar, bottom popup, top popup, and right agent sidebar
  - `Alt+Shift+I` toggles the keep-alive Zenith process information popup
  - `Alt+[` / `Alt+]` selects the previous/next layout family; with the packaged single managed sidebar family, those keys usually keep the visible layout unchanged
  - `Ctrl+Alt+H/L` moves the current tab left/right
  - `Ctrl+Alt+J/K` moves the current pane down/up
  - `Alt+w/q` walks left/right (focus tabs)
  - `Ctrl+y` toggles focus between the managed left sidebar and editor
  - `Ctrl+Shift+Y` toggles focus between the managed editor and right agent sidebar
  - `Alt+m` opens a new terminal in the current tab workspace root
  - `Alt+r` is the smart reveal key: in the editor it forwards `Alt+r` into the editor, and outside the editor it falls back to the editor/left-sidebar focus flow
  - `Ctrl+Alt+g` locked mode, `Ctrl+Alt+s` scroll mode, `Ctrl+Alt+o` session mode
- **Helix**: See [Helix Custom Keybindings](#helix-custom-keybindings) section below

You can also print these Yazelix-owned bindings and the scoped semantic action ids directly with `yzx keys`.

### Ownership Layers

The sparse root exposes only the four chords that cross directly into Nova:

```toml
[keybindings]
config = "Alt Shift K"
agent = "Alt Shift L"
git = "Alt Shift J"
menu = "Alt Shift M"
```

Custom popup commands live under `popups.<id>` with their own `keybinding`. Fixed Classic workspace actions such as `Ctrl y`, `Ctrl Shift Y`, `Alt Shift H`, `Alt r`, and `Alt m` are runtime policy rather than semantic root fields. `Ctrl Shift Y` remains available in the final Classic bridge but retires when the right agent sidebar becomes Nova's agent popup

- Native Zellij preferences: `~/.config/yazelix/zellij/config.kdl`
- Full native Zellij keymap ownership: plain `zellij` outside Yazelix
- Yazi-native bindings: `~/.config/yazelix/yazi/keymap.toml`
- Helix-local bindings: `~/.config/yazelix/helix/config.toml`
- Terminal-emulator shortcuts: the terminal emulator config

Managed `zellij/config.kdl` rejects `keybinds` blocks so it cannot bypass generated workspace controls. Use plain Zellij and `~/.config/zellij/config.kdl` when full native keymap ownership matters

### Fixed Classic Workspace Bindings

| Binding | Meaning |
|---------|---------|
| `Ctrl y` | Move focus between the managed editor and managed left sidebar |
| `Ctrl Shift Y` | Move focus between the editor and Classic right agent sidebar |
| `Alt Shift H` | Show or hide the managed left sidebar layout slot |
| `Alt r` | Reveal the editor path or return focus to the editor/sidebar pair |

## Keybinding Tips
- **Zellij**: `Alt+number` for tab, `Alt+w/q` for tab walk, `Ctrl+Alt+H/L` for tab move, `Ctrl+Alt+J/K` for pane move, `Alt+Shift+H/J/K/L` for directional Yazelix surfaces
- **Yazi**: 
  - `Z`: Use Yazi's built-in Zoxide jump and stay inside Yazi
  - `Alt+z`: Use Yazelix's direct-open Zoxide jump to retarget the managed editor and workspace immediately
  - `z`: Use fzf (fuzzy find unknown paths)
  - `SPACE`: Select files
  - `y`: Yank (copy); `Y`: Unyank (cancel copy)
  - `x`: Cut; `X`: Uncut (cancel cut)
  - `a`: Add a file (`filename.ext`) or folder (`foldername/`)
  - `Enter`: Yazi-native `open` key; editable files route through Yazelix's managed editor opener
  - `o`: Yazi-native `open` key for selected files
  - `O`: Open Yazi's built-in "Open with" menu for more options, including the system file manager flow when available
  - `Ctrl+y`: Toggle focus between the editor and left sidebar
  - `Ctrl+Shift+Y`: Toggle focus between the editor and right agent sidebar
  - `Alt+p`: Open the selected directory in a new Zellij pane and make it the tab workspace root
- **Nushell**:
  - `Ctrl r`: interactive history search
  - `Ctrl o`: open a temporary buffer
- **lazygit**:
  - `c`: Commit changes
  - `p`: Push commits
  - `P`: Pull changes
  - `s`: Stage/unstage files

## Helix Managed Keybindings

Yazelix-managed Helix sessions ship curated Helix-local defaults through `~/.config/yazelix/helix/config.toml`. The default editor-local reveal action is `Alt+r`, which runs `yzx reveal` for the current buffer.

The shipped workspace keys are:

| Keybinding | Action |
|------------|--------|
| `Ctrl y` | Toggle focus between the managed editor and left sidebar |
| `Ctrl Shift Y` | Toggle focus between the managed editor and right agent sidebar |
| `Alt Shift H` | Show or hide the left sidebar |
| `Alt r` | Smart reveal key: forwards `Alt+r` into the editor, otherwise falls back to the editor/left-sidebar focus toggle |

The recommended split is `Alt+r` for reveal inside Helix, `Ctrl+y` for editor/left-sidebar focus, `Ctrl Shift Y` for editor/right-sidebar focus, and `Alt Shift H` for sidebar show/hide. Yazelix also binds `Alt+r` at the Zellij layer so it behaves like `Ctrl+y` outside the editor.
