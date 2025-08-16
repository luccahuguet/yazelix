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
| Ctrl+g                   | Locked mode                   | ✅ Original (no conflict) |
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
  - `Ctrl+Alt+s` scroll mode, `Ctrl+Alt+o` session mode
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
  - `Alt+y`: Focus Helix pane (bidirectional navigation)
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

Yazelix provides two ways to integrate Helix with Yazi:

1. **Sidebar Integration**: Reveal current file in the Yazelix sidebar (requires full workspace)
2. **Native Integration**: Direct file picker within Helix, thanks to [Yazi PR #2461](https://github.com/sxyazi/yazi/pull/2461)

#### Setup

Add the following to your `~/.config/helix/config.toml`:

```toml
[keys.normal]
# Yazelix sidebar integration - reveal current file in Yazi sidebar
A-y = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""


```

#### Usage

| Keybinding | Action |
|------------|--------|
| `Alt+y` | Reveal current file in Yazelix sidebar |

**Workflow**:
1. Press the desired keybinding in Helix normal mode
2. Navigate in Yazi and press `Enter` to select a file
3. The file is revealed in the sidebar

### Benefits

- **Lightweight**: No Zellij required for this workflow
- **Fast**: Direct integration without terminal multiplexer overhead  
- **Familiar**: Uses the same Yazi interface you know from Yazelix
- **Smart**: Automatically starts from the most relevant directory
- **Complementary**: Works alongside the main Yazelix sidebar workflow
- **Conflict-free**: Yazelix has remapped Zellij's scroll mode from `Ctrl+s` to `Ctrl+Alt+s` to avoid conflicts

### Integration Script

The integration is powered by `nushell/scripts/integrations/helix_yazi_picker.nu`, which:
- Determines the best starting directory (current file's directory or working directory)
- Launches Yazi with a chooser file for selection
- Handles path resolution intelligently for both existing and new files

This gives you the best of both worlds - use the native integration for quick file picking, and the full Yazelix environment for comprehensive development workflows!
