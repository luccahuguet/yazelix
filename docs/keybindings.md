# Keybindings

## Zellij Keybindings

Yazelix uses **selective remapping** - only conflicting Zellij keybindings are changed, preserving muscle memory where possible.

### Core Navigation
| Keybinding                | Action                        |
|--------------------------|-------------------------------|
| Alt+number (1-9)         | Go to tab 1-9                 |
| Alt+w                    | Walk to next tab (focus)      |
| Alt+q                    | Walk to previous tab (focus)  |
| Alt+Shift+H              | Move tab left                 |
| Alt+Shift+L              | Move tab right                |
| Alt+Shift+F              | Toggle pane fullscreen        |
| Alt+t                    | Toggle the configured popup program |

### Zellij Modes (Helix-Compatible)
| Keybinding                | Action                        | Notes |
|--------------------------|-------------------------------|-------|
| **Alt+Shift+M**          | **Yazelix menu**              | opens yzx command palette popup |
| **Alt+t**                | **Popup program**             | opens, focuses, or closes the managed popup command |
| **Ctrl+Alt+g**           | **Locked mode**               | ⚠️ Remapped (was Ctrl+g) |
| Ctrl+p                   | Pane mode                     | ✅ Original (no conflict) |
| Ctrl+n                   | Resize mode                   | ✅ Original (no conflict) |
| Ctrl+t                   | Tab mode                      | ✅ Original (no conflict) |
| Ctrl+h                   | Move mode                     | ✅ Original (no conflict) |
| Ctrl+q                   | Quit                          | ✅ Original (no conflict) |
| **Ctrl+Alt+s**           | **Scroll mode**               | ⚠️ Remapped (was Ctrl+s) |
| **Ctrl+Alt+o**           | **Session mode**              | ⚠️ Remapped (was Ctrl+o) |

- **Tab walking**: Alt+w/q walks (focuses) next/previous tab, like browser tab switching.
- **Tab moving**: Alt+Shift+H/L moves the current tab left/right.
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
  - `Alt+t` toggles the configured managed popup program (defaults to `lazygit`)
  - `Alt+Shift+H` moves the current tab left
  - `Alt+Shift+L` moves the current tab right
  - `Alt+w/q` walks left/right (focus tabs)
  - `Ctrl+y` toggles focus between the managed sidebar and editor
  - `Alt+y` toggles the sidebar open/closed
  - `Alt+m` opens a new terminal in the current tab workspace root
  - `Alt+r` is the smart reveal key: in the editor it forwards `Alt+r` into the editor, and outside the editor it falls back to the editor/sidebar focus flow
  - `Ctrl+Alt+g` locked mode, `Ctrl+Alt+s` scroll mode, `Ctrl+Alt+o` session mode
- **Helix**: See [Helix Custom Keybindings](#helix-custom-keybindings) section below

You can also print these Yazelix-owned bindings directly with `yzx keys`.

### Sidebar Commands vs Keybindings

The stable sidebar API is the pane-orchestrator command surface, not the default keys:

| Command | Default key | Meaning |
|---------|-------------|---------|
| `toggle_editor_sidebar_focus` | `Ctrl y` | Move focus between the managed editor and managed sidebar |
| `toggle_sidebar` | `Alt y` | Open or close the managed sidebar layout slot |
| `focus_sidebar` | none | Focus the managed sidebar from commands such as `yzx reveal` |

You can remap the keys in your Zellij override config as long as they still send the same `MessagePlugin` command names to the loaded `yazelix_pane_orchestrator` plugin. The plugin does not know or require Yazelix's default key choices.

## Keybinding Tips
- **Zellij**: `Alt+number` for tab, `Alt+w/q` for tab walk, `Alt+Shift+H/L` for tab move
- **Yazi**: 
  - `Z`: Use Yazi's built-in Zoxide jump and stay inside Yazi
  - `Alt+z`: Use Yazelix's direct-open Zoxide jump to retarget the managed editor and workspace immediately
  - `z`: Use fzf (fuzzy find unknown paths)
  - `SPACE`: Select files
  - `y`: Yank (copy); `Y`: Unyank (cancel copy)
  - `x`: Cut; `X`: Uncut (cancel cut)
  - `a`: Add a file (`filename.ext`) or folder (`foldername/`)
  - `Enter`: Open selected files through Yazelix's configured editor opener
  - `o`: Use Yazi's built-in open action for selected files
  - `O`: Open Yazi's built-in "Open with" menu for more options, including the system file manager flow when available
  - `Ctrl+y`: Toggle focus between the editor and sidebar
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
| `Ctrl y` | Toggle focus between the managed editor and sidebar |
| `Alt y` | Toggle the sidebar open/closed |
| `Alt r` | Smart reveal key: forwards `Alt+r` into the editor, otherwise falls back to the editor/sidebar focus toggle |

If you add a Helix-local reveal binding, treat it as optional editor customization rather than part of the default Yazelix keymap. The recommended split is `Alt+r` for reveal, `Ctrl+y` for editor/sidebar focus, and `Alt+y` for sidebar open/close. Yazelix now also binds `Alt+r` at the Zellij layer so it behaves like `Ctrl+y` outside the editor.
