# Keybindings

## Zellij Keybindings

Yazelix uses **selective remapping** - only conflicting Zellij keybindings are changed, preserving muscle memory where possible.

### Core Navigation
| Keybinding                | Action                        |
|--------------------------|-------------------------------|
| Alt+number (1-9)         | Go to tab 1-9                 |
| Alt+w                    | Walk to next tab (focus)      |
| Alt+q                    | Walk to previous tab (focus)  |
| Ctrl+Shift+H             | Move tab left                 |
| Ctrl+Shift+L             | Move tab right                |
| Ctrl+Shift+J             | Move pane down                |
| Ctrl+Shift+K             | Move pane up                  |
| Alt+Shift+F              | Toggle pane fullscreen        |
| Alt+Shift+H              | Toggle the left sidebar       |
| Alt+Shift+J              | Toggle the bottom popup       |
| Alt+Shift+K              | Toggle the top popup          |
| Alt+Shift+L              | Toggle the right Codex agent sidebar |
| Ctrl+Shift+Y             | Toggle editor/right agent focus |

### Zellij Modes (Helix-Compatible)
| Keybinding                | Action                        | Notes |
|--------------------------|-------------------------------|-------|
| **Alt+Shift+M**          | **Yazelix menu**              | opens yzx command palette popup |
| **Alt+Shift+J/K**        | **Bottom/top popup**          | opens, focuses, or closes the configured named popup command |
| **Ctrl+Alt+g**           | **Locked mode**               | ⚠️ Remapped (was Ctrl+g) |
| Ctrl+p                   | Pane mode                     | ✅ Original (no conflict) |
| Ctrl+n                   | Resize mode                   | ✅ Original (no conflict) |
| Ctrl+t                   | Tab mode                      | ✅ Original (no conflict) |
| Ctrl+h                   | Move mode                     | ✅ Original (no conflict) |
| Ctrl+q                   | Quit                          | ✅ Original (no conflict) |
| **Ctrl+Alt+s**           | **Scroll mode**               | ⚠️ Remapped (was Ctrl+s) |
| **Ctrl+Alt+o**           | **Session mode**              | ⚠️ Remapped (was Ctrl+o) |

- **Tab walking**: Alt+w/q walks (focuses) next/previous tab, like browser tab switching.
- **Tab moving**: Ctrl+Shift+H/L moves the current tab left/right.
- **Pane moving**: Ctrl+Shift+J/K moves the current pane down/up.
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
  - `Alt+Shift+H/J/K/L` toggles the left sidebar, bottom popup, top popup, and right Codex agent sidebar
  - `Ctrl+Shift+H/L` moves the current tab left/right
  - `Ctrl+Shift+J/K` moves the current pane down/up
  - `Alt+w/q` walks left/right (focus tabs)
  - `Ctrl+y` toggles focus between the managed left sidebar and editor
  - `Ctrl+Shift+Y` toggles focus between the managed editor and right agent sidebar
  - `Alt+m` opens a new terminal in the current tab workspace root
  - `Alt+r` is the smart reveal key: in the editor it forwards `Alt+r` into the editor, and outside the editor it falls back to the editor/left-sidebar focus flow
  - `Ctrl+Alt+g` locked mode, `Ctrl+Alt+s` scroll mode, `Ctrl+Alt+o` session mode
- **Helix**: See [Helix Custom Keybindings](#helix-custom-keybindings) section below

You can also print these Yazelix-owned bindings and the scoped semantic action ids directly with `yzx keys`.

### Ownership Layers

Use semantic remaps for Yazelix-owned actions and native sidecars for the owning tool's broader keymap.

- Yazelix-owned Zellij actions: `settings.jsonc` under `zellij.keybindings`
- Yazelix curated native Zellij policy: `settings.jsonc` under `zellij.native_keybindings`
- Advanced native Zellij settings without keybinds: `~/.config/yazelix/zellij.kdl`
- Full native Zellij keymap ownership: plain `zellij` outside Yazelix
- Yazelix-owned Yazi integration actions: `settings.jsonc` under `yazi.keybindings`
- Yazi-native bindings: `~/.config/yazelix/yazi/keymap.toml`
- Helix-local bindings for managed Helix sessions: `~/.config/yazelix/helix.toml`
- Terminal-emulator shortcuts: the terminal emulator config

`zellij.keybindings` accepts owner-local action ids such as `bottom_popup`, `top_popup`, `menu`, `toggle_left_sidebar`, `toggle_editor_right_sidebar_focus`, and `open_workspace_terminal`. Shared diagnostics and docs use scoped ids such as `zellij.bottom_popup`. Omitted actions keep defaults, and `[]` disables a Yazelix-owned binding. Yazelix rejects duplicate semantic Zellij keys before launch.

`zellij.popup_commands` sets the command argv for named popup surfaces. Defaults are `bottom_popup = ["lazygit"]`, `top_popup = ["yzx", "config", "ui"]`, and `menu = ["yzx", "menu"]`.

`zellij.native_keybindings` accepts curated native policy ids such as `scroll_mode`, `scroll_mode_unbind`, `move_tab_left`, `move_pane_down`, and `move_tab_left_unbind`. These are Yazelix's shipped conflict-remap and validation defaults for native Zellij commands. Omitted entries keep defaults, and `[]` disables one native policy entry. Managed `~/.config/yazelix/zellij.kdl` rejects `keybinds` blocks so it cannot bypass generated workspace controls.

`yazi.keybindings` accepts owner-local action ids such as `open_directory_as_workspace_pane` and `open_zoxide_in_editor`. Values are alternate generated Yazi bindings such as `<A-p>` and `<A-z>`. Omitted actions keep defaults, and `[]` disables that generated Yazelix-owned Yazi integration binding. Native open-selected keys such as `<Enter>` and `o` remain in `~/.config/yazelix/yazi/keymap.toml`; arbitrary Yazi actions and native multi-key sequences also belong there.

### Sidebar Commands vs Keybindings

The stable sidebar action surface is the semantic keybinding map, not the default keys:

| Action id | Default key | Meaning |
|---------|-------------|---------|
| `toggle_editor_sidebar_focus` | `Ctrl y` | Move focus between the managed editor and managed left sidebar |
| `toggle_editor_right_sidebar_focus` | `Ctrl Shift Y` | Move focus between the managed editor and managed right agent sidebar |
| `toggle_left_sidebar` | `Alt Shift H` | Show or hide the managed left sidebar layout slot |
| `focus_sidebar` | none | Focus the managed sidebar from commands such as `yzx reveal` |

Prefer `zellij.keybindings` for remaps. Native Zellij KDL remains the escape hatch for full keymap ownership.

## Keybinding Tips
- **Zellij**: `Alt+number` for tab, `Alt+w/q` for tab walk, `Ctrl+Shift+H/L` for tab move, `Alt+Shift+H/J/K/L` for directional Yazelix surfaces
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

## Helix Custom Keybindings

Yazelix does not ship a built-in Helix-local Yazi keybinding. If you want an editor-local reveal action, bind `yzx reveal` to any Helix shortcut that fits your setup and does not conflict with your own editor bindings. `Alt+r` is the recommended choice.

The shipped workspace keys are:

| Keybinding | Action |
|------------|--------|
| `Ctrl y` | Toggle focus between the managed editor and left sidebar |
| `Ctrl Shift Y` | Toggle focus between the managed editor and right agent sidebar |
| `Alt Shift H` | Show or hide the left sidebar |
| `Alt r` | Smart reveal key: forwards `Alt+r` into the editor, otherwise falls back to the editor/left-sidebar focus toggle |

If you add a Helix-local reveal binding, treat it as optional editor customization rather than part of the default Yazelix keymap. The recommended split is `Alt+r` for reveal, `Ctrl+y` for editor/left-sidebar focus, `Ctrl Shift Y` for editor/right-sidebar focus, and `Alt Shift H` for sidebar show/hide. Yazelix also binds `Alt+r` at the Zellij layer so it behaves like `Ctrl+y` outside the editor.
