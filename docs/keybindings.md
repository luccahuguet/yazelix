# Keybindings

## Zellij Keybindings
| Keybinding                | Action                        |
|--------------------------|-------------------------------|
| Alt+number (1-9)         | Go to tab 1-9                 |
| Alt+w                    | Walk to next tab (focus)      |
| Alt+q                    | Walk to previous tab (focus)  |
| Alt+Shift+H              | Move tab left                 |
| Alt+Shift+L              | Move tab right                |
| Alt f                    | Toggle pane fullscreen        |
| Ctrl+x                   | Enter scroll mode             |

- **Tab walking**: Alt+w/q walks (focuses) next/previous tab, like browser tab switching.
- **Tab moving**: Alt+Shift+H/L moves the current tab left/right.
- **Direct tab access**: Alt+1 through Alt+9 jumps directly to a tab.

If you find a conflict, please open an issue

## Discoverability of Keybindings
- **Zellij**: Shows all keybindings visually in the status bar—works out of the box
- **Helix**: Similar to Zellij, key bindings are easy to discover
- **Yazi**: Press `~` to see all keybindings and commands (use `Alt f` to fullscreen the pane for a better view)
- **Nushell**:
  - Run `tutor` in Nushell
  - Read the [Nushell Book](https://www.nushell.sh/book/)
  - Use `help commands | find tables` to search, for example, commands related to tables
- **lazygit**: Press `?` to view keybindings
- **Starship**: Customizable prompt; configure in `~/.config/starship.toml` (see [Starship docs](https://starship.rs/config/))

## Yazelix Custom Keybindings
- **Zellij**:
  - `Alt f` toggles pane fullscreen
  - `Alt+Shift+H` moves the current tab left
  - `Alt+Shift+L` moves the current tab right
  - `Ctrl+Alt+H` walks left (focus previous tab)
  - `Ctrl+Alt+L` walks right (focus next tab)
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

# Native Yazi integration - file picker within Helix
# Ctrl+y: Open Yazi file picker from current file's directory
C-y = [
    ':sh rm -f /tmp/yazi-helix-chooser',
    ':insert-output nu ~/.config/yazelix/nushell/scripts/integrations/helix_yazi_picker.nu "%{buffer_name}"',
    ':open %sh{cat /tmp/yazi-helix-chooser}',
    ':redraw'
]

# Ctrl+Shift+y: Open Yazi file picker from current working directory
C-S-y = [
    ':sh rm -f /tmp/yazi-helix-chooser',
    ':insert-output nu ~/.config/yazelix/nushell/scripts/integrations/helix_yazi_picker.nu',
    ':open %sh{cat /tmp/yazi-helix-chooser}',
    ':redraw'
]
```

#### Usage

| Keybinding | Action |
|------------|--------|
| `Alt+y` | Reveal current file in Yazelix sidebar |
| `Ctrl+y` | Open Yazi file picker from current file's directory |
| `Ctrl+Shift+y` | Open Yazi file picker from working directory |

**Workflow**:
1. Press the desired keybinding in Helix normal mode
2. Navigate in Yazi and press `Enter` to select a file
3. The selected file opens automatically in Helix (for `Ctrl+y`/`Ctrl+Shift+y`) or is revealed in sidebar (for `Alt+y`)

### Benefits

- **Lightweight**: No Zellij required for this workflow
- **Fast**: Direct integration without terminal multiplexer overhead  
- **Familiar**: Uses the same Yazi interface you know from Yazelix
- **Smart**: Automatically starts from the most relevant directory
- **Complementary**: Works alongside the main Yazelix sidebar workflow
- **Conflict-free**: Yazelix has remapped Zellij's scroll mode from `Ctrl+y` to `Ctrl+x` to avoid conflicts

### Integration Script

The integration is powered by `nushell/scripts/integrations/helix_yazi_picker.nu`, which:
- Determines the best starting directory (current file's directory or working directory)
- Launches Yazi with a chooser file for selection
- Handles path resolution intelligently for both existing and new files

This gives you the best of both worlds - use the native integration for quick file picking, and the full Yazelix environment for comprehensive development workflows!
