# Keybindings

## Zellij Keybindings
| New Zellij Keybinding | Previous Keybinding | Helix Action that conflicted before | Zellij Action Remapped     |
|-----------------------|---------------------|-------------------------------------|----------------------------|
| Ctrl e                | Ctrl o              | jump_backward                       | SwitchToMode "Session"     |
| Ctrl y                | Ctrl s              | save_selection                      | SwitchToMode "Scroll"      |
| Alt w                 | Alt i               | shrink_selection                    | MoveTab "Left"             |
| Alt q                 | Alt o               | expand_selection                    | MoveTab "Right"            |
| Alt m                 | Alt n               | select_next_sibling                 | NewPane                    |
| Alt 2                 | Ctrl b              | move_page_up                        | SwitchToMode "Tmux"        |

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
- **Zellij**: `Alt f` toggles pane fullscreen
- **Helix**: `Alt y` reveals the file from the Helix buffer in Yazi, add this to your Helix config:
  ```toml
  [keys.normal]
  A-y = ":sh nu -c \"use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; reveal_in_yazi '%{buffer_name}'\""
  ```
  - **Limitation**: Only works for Helix instances opened from Yazi
  - **Requirement**: Build Helix from source until the next release includes command expansions

## Keybinding Tips
- **Zellij**: `Ctrl p` then `r` for a split to the right; `Ctrl p` then `d` for a downward split
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