# Neovim Keybindings for Yazelix

This document describes the recommended Neovim keybindings for full Yazelix integration.

## Essential Keybinding: Reveal in Yazi

The essential keybinding for Yazelix integration should be added to your Neovim config (usually `~/.config/nvim/init.lua`):

```lua
-- Yazelix sidebar integration - reveal current file in Yazi sidebar
vim.keymap.set('n', '<M-y>', function()
  local buffer_path = vim.fn.expand('%:p')
  if buffer_path ~= '' then
    vim.fn.system('nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu "' .. buffer_path .. '"')
  end
end, { desc = 'Reveal in Yazi sidebar' })
```

**Note:** Only works for Neovim instances opened from Yazi.

### For init.vim Users

If you use `init.vim` instead of `init.lua`:

```vim
" Yazelix sidebar integration - reveal current file in Yazi sidebar
nnoremap <M-y> :call system('nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu "' . expand('%:p') . '"')<CR>
```

## Additional Recommended Keybindings

While not required for Yazelix, these keybindings work well with the yazelix workflow:

### File Navigation

```lua
-- Quick file finder (using Telescope or fzf-lua)
vim.keymap.set('n', '<C-p>', ':Telescope find_files<CR>', { desc = 'Find files' })
vim.keymap.set('n', '<C-y>', ':Telescope find_files<CR>', { desc = 'Find files (Helix-style)' })

-- Or with fzf-lua:
-- vim.keymap.set('n', '<C-p>', ':FzfLua files<CR>', { desc = 'Find files' })
```

### Buffer Management

```lua
-- Buffer navigation (similar to Helix buffer management)
vim.keymap.set('n', '<leader>b', ':Telescope buffers<CR>', { desc = 'List buffers' })
```

## Integration Features

With Neovim configured for Yazelix, you get:

- **Alt+y**: Reveal current buffer in Yazi sidebar (jumps focus to Yazi and selects the file)
- **Smart Instance Management**: Opening files from Yazi reuses existing Neovim instance
- **Tab Naming**: Zellij tabs automatically named after your project/directory
- **Yazi Sync**: Yazi directory view stays synchronized with opened files

## Workflow Example

1. Start Yazelix with Neovim: `yzx launch` (with `editor_command = "nvim"` in `yazelix.nix`)
2. Navigate files in Yazi sidebar (left pane)
3. Press `e` on a file to edit in Neovim
4. While editing, press `Alt+y` to reveal the current file in Yazi
5. Navigate to a different file in Yazi and press `e` - it opens in the same Neovim instance

## Troubleshooting

### Alt+y doesn't work

1. **Check if you're in sidebar mode:**
   - Reveal in Yazi only works with `sidebar_enabled = true` (default)
   - Run `echo $YAZELIX_ENABLE_SIDEBAR` - should show "true"

2. **Verify Neovim was opened from Yazi:**
   - Run `echo $YAZI_ID` inside Neovim's terminal
   - If empty, the Neovim instance wasn't started from Yazi

3. **Check the logs:**
   ```bash
   tail ~/.config/yazelix/logs/reveal_in_yazi.log
   ```

### Neovim opens in new instance instead of reusing existing

1. **Check pane detection:**
   ```bash
   tail ~/.config/yazelix/logs/open_neovim.log
   ```

2. **Verify pane naming:**
   - The pane should be named "editor"
   - Check with: `zellij action list-clients`

## Comparison with Helix

| Feature | Helix | Neovim |
|---------|-------|--------|
| Reveal in Yazi (Alt+y) | ✅ | ✅ |
| Same instance opening | ✅ | ✅ |
| Pane detection | ✅ | ✅ |
| Tab naming | ✅ | ✅ |
| Yazi sync | ✅ | ✅ |

Both editors now have full first-class support in Yazelix!
