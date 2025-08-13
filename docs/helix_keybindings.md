# Helix Editor Keybindings Reference

This document catalogs Helix editor keybindings to help avoid conflicts when configuring Yazelix Zellij keybindings.

## Ctrl+ Keybindings by Mode

### Normal Mode
| Key | Action |
|-----|--------|
| `Ctrl+a` | Increment object (number) under cursor |
| `Ctrl+b` | Move page up |
| `Ctrl+c` | Comment/uncomment the selections |
| `Ctrl+d` | Move cursor and page half page down |
| `Ctrl+f` | Move page down |
| `Ctrl+i` | Jump forward on the jumplist |
| `Ctrl+o` | Jump backward on the jumplist |
| `Ctrl+s` | Save the current selection to the jumplist |
| `Ctrl+u` | Move cursor and page half page up |
| `Ctrl+w` | Enter window mode |
| `Ctrl+x` | Decrement object (number) under cursor |

### Insert Mode
| Key | Action |
|-----|--------|
| `Ctrl+d` | Delete next char |
| `Ctrl+h` | Delete previous char |
| `Ctrl+j` | Insert new line |
| `Ctrl+k` | Delete to end of line |
| `Ctrl+r` | Insert a register content |
| `Ctrl+s` | Commit undo checkpoint |
| `Ctrl+u` | Delete to start of line |
| `Ctrl+w` | Delete previous word |
| `Ctrl+x` | Autocomplete |

### Window Mode (accessed via `Ctrl+w`)
| Key | Action |
|-----|--------|
| `Ctrl+h` | Move to left split |
| `Ctrl+j` | Move to split below |
| `Ctrl+k` | Move to split above |
| `Ctrl+l` | Move to right split |
| `Ctrl+v` | Vertical right split |

### Picker Mode
| Key | Action |
|-----|--------|
| `Ctrl+c` | Close picker/prompt |
| `Ctrl+n` | Next entry |
| `Ctrl+p` | Previous entry |
| `Ctrl+s` | Open horizontally |
| `Ctrl+t` | Toggle preview |
| `Ctrl+v` | Open vertically |

### Picker/Prompt Mode (shared)
| Key | Action |
|-----|--------|
| `Ctrl+a` | Move prompt start |
| `Ctrl+b` | Backward a char |
| `Ctrl+c` | Close picker/prompt |
| `Ctrl+d` | Delete next char |
| `Ctrl+e` | Move prompt end |
| `Ctrl+f` | Forward a char |
| `Ctrl+h` | Delete previous char |
| `Ctrl+k` | Delete to end of line |
| `Ctrl+n` | Select next history/Next entry |
| `Ctrl+p` | Select previous history/Previous entry |
| `Ctrl+r` | Insert register content |
| `Ctrl+s` | Open horizontally/Insert a word |
| `Ctrl+u` | Delete to start of line |
| `Ctrl+v` | Open vertically |
| `Ctrl+w` | Delete previous word |

## Alt+ Keybindings

| Key | Action |
|-----|--------|
| `Alt+.` | Repeat last motion |
| `Alt+`` | Switch text to lowercase |
| `Alt+\` | Switch text to uppercase |
| `Alt+u` | Move backward in history |
| `Alt+U` | Move forward in history |
| `Alt+d` | Delete selection without yanking |
| `Alt+c` | Change selection without yanking |
| `Alt+s` | Split selection on newlines |
| `Alt+-` | Merge selections |
| `Alt+_` | Merge consecutive selections |
| `Alt+;` | Collapse selection |
| `Alt+:` | Ensure selections are forward |
| `Alt+,` | Remove primary selection |
| `Alt+(` | Rotate selection contents backward |
| `Alt+)` | Rotate selection contents forward |
| `Alt+x` | Shrink selection to line bounds |
| `Alt+J` | Join selections with inserted space |
| `Alt+K` | Remove selections matching regex |
| `Alt+o`/`Alt+up` | Expand selection to parent syntax node |
| `Alt+i`/`Alt+down` | Shrink syntax tree object selection |
| `Alt+p`/`Alt+left` | Select previous sibling node |
| `Alt+n`/`Alt+right` | Select next sibling node |
| `Alt+a` | Select all sibling nodes |
| `Alt+I`/`Alt+Shift+down` | Select all children nodes |
| `Alt+e` | Move to end of parent node |
| `Alt+b` | Move to start of parent node |
| `Alt+*` | Use current selection as search pattern |

## Nushell Default Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+r` | History search (menu) |
| `Ctrl+x` | Pull more records in history menu |
| `Ctrl+y` | [Available for custom assignment] |
| `Ctrl+z` | Go to previous page in menus |
| `Ctrl+t` | Activate completion menu / next element |
| `Ctrl+o` | Various menu operations |

## Potentially Unbound Ctrl+ Keys in Normal Mode

Based on the Helix documentation, these Ctrl+ combinations appear to be unbound or not explicitly mentioned for normal mode:

- `Ctrl+e` - Not mentioned
- `Ctrl+g` - Not mentioned  
- `Ctrl+l` - Not mentioned (only used in window mode)
- `Ctrl+m` - Not mentioned
- `Ctrl+n` - Not mentioned (only used in picker mode)
- `Ctrl+p` - Not mentioned (only used in picker mode)
- `Ctrl+q` - Not mentioned
- `Ctrl+t` - Not mentioned (only used in picker mode, but conflicts with Nushell)
- `Ctrl+v` - Not mentioned (only used in window/picker modes)
- `Ctrl+y` - Available for custom assignment
- `Ctrl+z` - Not mentioned (but used by Nushell)

## Potentially Unbound Alt+ Keys

These Alt+ combinations don't appear in Helix defaults:
- `Alt+f`, `Alt+g`, `Alt+h`, `Alt+j`, `Alt+k`, `Alt+l`, `Alt+m`, `Alt+q`, `Alt+r`, `Alt+t`, `Alt+v`, `Alt+w`, `Alt+y`, `Alt+z`

## Notes

- This list is based on Helix editor documentation as of January 2025
- Keybindings may vary between Helix versions
- Users can override default keybindings in their Helix configuration
- When choosing Zellij keybindings, prioritize avoiding conflicts with normal mode since that's where most editor interaction happens

## Yazelix Integration Considerations

When configuring Zellij keybindings for Yazelix:

1. **Avoid Normal Mode conflicts**: These are most critical since users spend most time in normal mode
2. **Consider Insert Mode**: Users frequently enter insert mode, so avoid heavy conflicts there
3. **Window/Picker modes**: Less critical but still worth avoiding for clean integration
4. **Shell integration**: Also consider Nushell keybinding conflicts when users are in shell panes

## Recent Changes (January 2025)

**Selective Keybinding Remapping Applied:**
- **Smart approach**: Only remap Zellij keys that actually conflict with Helix
- **Minimal changes**: Preserve original Zellij muscle memory where possible
- Layouts affected: `yzx_side.kdl`, `yzx_no_side.kdl`

**Final Keybinding Scheme:**
```
Ctrl+G = Locked Mode     ✅ No conflict (original Zellij)
Ctrl+P = Pane Mode       ✅ No conflict (original Zellij)  
Ctrl+N = Resize Mode     ✅ No conflict (original Zellij)
Ctrl+Alt+S = Scroll Mode ⚠️  Remapped (Helix conflict: save_selection)
Ctrl+Alt+O = Session Mode ⚠️ Remapped (Helix conflict: jump_backward)
Ctrl+T = Tab Mode        ✅ No conflict (original Zellij)
Ctrl+H = Move Mode       ✅ No conflict (original Zellij)
Ctrl+Q = Quit            ✅ No conflict (original Zellij)
```

This approach minimizes disruption while ensuring zero conflicts with Helix editor.

**Terminal Emulator Conflicts:**

**Ghostty Terminal:**
- `Ctrl+Shift+I` - Inspector/Developer tools
- `Ctrl+Shift+O` - Open configuration

These conflicts affect Yazelix keybindings for:
- Resize mode (Ctrl+Shift+N) - No conflict
- Session mode (Ctrl+Shift+O) - **CONFLICTS with Ghostty**

**Workaround for Ghostty users:**
- Use `Ctrl+Alt` scheme instead of `Ctrl+Shift` when using Ghostty
- Or disable Ghostty's `Ctrl+Shift+O` binding in Ghostty configuration

**WezTerm Terminal:**
- `Ctrl+Alt+H` - Does not work / not recognized

## Recommended Helix Configuration

Add these keybindings to your Helix `config.toml` for improved navigation:

```toml
[keys.normal]
"{" = "goto_prev_paragraph"
"}" = "goto_next_paragraph"
X = "extend_line_up"
# Print the current line's git blame information to the statusline
space.B = ":echo %sh{git blame -L %{cursor_line},+1 %{buffer_name}}"
# Reload config and buffer
A-r = [":config-reload", ":reload"]
# Toggle hidden files in file picker
space.H = ":toggle-option file-picker.hidden"
```

This provides vim-like paragraph navigation using `{` and `}` instead of the default `[p` and `]p`, plus `X` for extending selection upward by line, and `Space+B` for git blame on the current line.
