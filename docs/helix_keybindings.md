# Helix Keybindings Configuration for Yazelix

This guide covers recommended Helix keybindings that enhance your editing experience when using Yazelix.

## Basic Yazelix Integration

The essential keybinding for Yazelix integration should be added to your Helix config (usually `~/.config/helix/config.toml`):

```toml
[keys.normal]
# Yazelix sidebar integration - reveal current file in Yazi sidebar
A-y = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""
```

**Note:** Only works for Helix instances opened from Yazi.

## Additional Recommended Keybindings

Add these keybindings to your Helix config for an enhanced editing experience:

```toml
[keys.normal]
# Navigation and movement
"{" = "goto_prev_paragraph"
"}" = "goto_next_paragraph"
g.e = "goto_file_end"
ret = ["move_line_down", "goto_first_nonwhitespace"]
A-ret = ["move_line_up", "goto_first_nonwhitespace"]

# Selection and editing
X = "extend_line_up"
C-k = [
  "extend_to_line_bounds",
  "delete_selection",
  "move_line_up",
  "paste_before",
]
C-j = ["extend_to_line_bounds", "delete_selection", "paste_after"]

# System integration
C-y = ":yank-diagnostic"
A-r = [":config-reload", ":reload"]

# Git integration
A-g.b = ":sh git blame -L %{cursor_line},+1 %{buffer_name}"
A-g.s = ":sh git status --porcelain"
A-g.l = ":sh git log --oneline -10 %{buffer_name}"

# Execute selections in shells
tab.x = ":sh $YAZELIX_DEFAULT_SHELL -c '%{selection}'"
tab.b = ":sh bash -c '%{selection}'"
tab.B = ":sh bash -c 'source ~/.bashrc && %{selection}'"
tab.n = ":sh nu -c '%{selection}'"
tab.N = ":sh nu -c 'source ~/.config/nushell/config.nu; %{selection}'"

# File picker toggles
tab.h = ":toggle-option file-picker.hidden"
tab.i = ":toggle-option file-picker.git-ignore"

# Configuration shortcuts
tab.l = ":o ~/.config/helix/languages.toml"
tab.c = ":config-open"
```

## Keybinding Categories

### Navigation and Movement
- `{` / `}`: Navigate between paragraphs
- `g.e`: Go to end of file
- `ret` / `A-ret`: Move lines up/down and go to first non-whitespace

### Selection and Editing
- `X`: Extend selection line up
- `C-k`: Cut current line and paste above
- `C-j`: Cut current line and paste below

### System Integration
- `C-y`: Yank diagnostic messages
- `A-r`: Reload configuration and current file
- `A-y`: **Yazelix integration** - Reveal current file in Yazi sidebar

### Git Integration
- `A-g.b`: Show git blame for current line
- `A-g.s`: Show git status (porcelain format)
- `A-g.l`: Show recent git log for current file

### Shell Execution
Execute selected text in different shells:
- `tab.x`: Execute in your default Yazelix shell
- `tab.b`: Execute in bash
- `tab.B`: Execute in bash with sourced bashrc
- `tab.n`: Execute in Nushell
- `tab.N`: Execute in Nushell with config

### File Picker Toggles
- `tab.h`: Toggle hidden files in file picker
- `tab.i`: Toggle git-ignore filtering in file picker

### Configuration Shortcuts
- `tab.l`: Open Helix languages.toml
- `tab.c`: Open main Helix configuration

## Usage Tips

1. **Shell Execution**: Select any text and use the `tab.x` bindings to execute it in your preferred shell. Great for testing code snippets or running commands.

2. **Git Integration**: The git keybindings help you stay in context while coding. `A-g.b` is particularly useful for understanding code history.

3. **File Picker Toggles**: Use `tab.h` and `tab.i` to quickly adjust what files are visible when using Helix's file picker.

4. **Yazelix Integration**: The `A-y` keybinding creates a seamless workflow between Helix and Yazi - you can quickly reveal any file you're editing in the sidebar.

## Customization

Feel free to modify these keybindings to match your workflow. The key principles are:
- Use `Alt` (A-) for Yazelix-specific integrations
- Use `tab.` for utility functions that don't interfere with normal editing
- Keep git operations grouped under `A-g.`

For more Helix configuration options, see the [Helix documentation](https://docs.helix-editor.com/configuration.html).

## Complete Keybindings Reference

For a complete list of all Yazelix keybindings across all tools, see [keybindings.md](./keybindings.md).