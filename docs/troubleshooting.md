# Troubleshooting

## Quick Fixes

### Reset Configuration
```bash
rm ~/.config/yazelix/yazelix.nix
exit && yazelix  # Regenerates defaults
```

### Delete Swap Files
```bash
rm ~/.config/yazelix/.*.swp
```

### Restart Fresh
```bash
exit && yazelix
```

## Editor Issues

### File Opening Broken
```bash
echo $EDITOR                    # Should show path
tail ~/.config/yazelix/logs/open_editor.log
```

### Runtime Errors
```bash
echo $HELIX_RUNTIME
ls $HELIX_RUNTIME               # Should show grammars/ themes/
```

### Wrong Editor Used
Check `editor_command` in `yazelix.nix`:
- `null` = yazelix's Helix
- `"hx"` = system Helix (needs `helix_runtime_path`)
- `"vim"` = other editor

## Performance Issues

### Slow Startup
```bash
time $EDITOR --version
```

### Large Log Files
Log files auto-trim but you can manually clean:
```bash
rm ~/.config/yazelix/logs/*.log
```

## Common Problems

### "Command not found"
- Check `which yazelix`
- Ensure Nix environment is loaded

### "Permission denied"
- Check file permissions in `~/.config/yazelix/`
- Ensure not running as root

### Git Conflicts
```bash
cd ~/.config/yazelix
git status                      # Check for conflicts
git stash                       # Save local changes
git pull                        # Update
git stash pop                   # Restore changes
```

## Getting Help

1. Check logs: `~/.config/yazelix/logs/`
2. Test with defaults: delete `yazelix.nix`
3. Report issues with:
   - OS and version
   - Yazelix version
   - Error messages
   - Configuration file content