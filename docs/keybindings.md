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
| Alt+Shift+f              | Toggle pane fullscreen        |

### Zellij Modes (Helix-Compatible)
| Keybinding                | Action                        | Notes |
|--------------------------|-------------------------------|-------|
| **Alt+Shift+m**          | **Yazelix menu**              | opens yzx command palette popup |
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
- **Zellij**: Shows all keybindings visually in the status bar—works out of the box
- **Helix**: Similar to Zellij, key bindings are easy to discover
- **Yazi**: Press `~` to see all keybindings and commands (use `Alt Shift f` to fullscreen the pane for a better view)
- **Nushell**:
  - Run `tutor` in Nushell
  - Read the [Nushell Book](https://www.nushell.sh/book/)
  - Use `help commands | find tables` to search, for example, commands related to tables
- **lazygit**: Press `?` to view keybindings
- **Starship**: Customizable prompt; configure in `~/.config/starship.toml` (see [Starship docs](https://starship.rs/config/))

## Yazelix Custom Keybindings
- **Zellij**:
  - `Alt+Shift+f` toggles pane fullscreen
  - `Alt+Shift+H` moves the current tab left
  - `Alt+Shift+L` moves the current tab right
  - `Alt+w/q` walks left/right (focus tabs)
  - `Ctrl+y` toggles focus between the managed sidebar and editor
  - `Alt+y` toggles the sidebar open/closed
  - `Ctrl+Alt+g` locked mode, `Ctrl+Alt+s` scroll mode, `Ctrl+Alt+o` session mode
- **Helix**: See [Helix Custom Keybindings](#helix-custom-keybindings) section below

## Keybinding Tips
- **Zellij**: `Alt+number` for tab, `Alt+w/q` for tab walk, `Alt+Shift+H/L` for tab move
- **Yazi**: 
  - `Z`: Use Zoxide (fuzzy find known paths)
  - `z`: Use fzf (fuzzy find unknown paths)
  - `SPACE`: Select files
  - `y`: Yank (copy); `Y`: Unyank (cancel copy)
  - `x`: Cut; `X`: Uncut (cancel cut)
  - `a`: Add a file (`filename.ext`) or folder (`foldername/`)
  - `Ctrl+y`: Toggle focus between the editor and sidebar
  - `Alt+p`: Open directory in new Zellij pane
- **Nushell**:
  - `Ctrl r`: interactive history search
  - `Ctrl o`: open a temporary buffer
- **lazygit**:
  - `c`: Commit changes
  - `p`: Push commits
  - `P`: Pull changes
  - `s`: Stage/unstage files

## Helix Custom Keybindings

Yazelix does not ship a default Helix-local Yazi keybinding. If you want an editor-local reveal action, bind `reveal_in_yazi.nu` to any Helix shortcut that fits your setup and does not conflict with your own editor bindings.

The shipped workspace keys are:

| Keybinding | Action |
|------------|--------|
| `Ctrl y` | Toggle focus between the managed editor and sidebar |
| `Alt y` | Toggle the sidebar open/closed |

If you add a Helix-local reveal binding, treat it as optional editor customization rather than part of the default Yazelix keymap.
