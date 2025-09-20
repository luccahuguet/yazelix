# Troubleshooting

## Quick Diagnosis

**🔍 Start here:** Run the automated health check to identify common issues:

```bash
yzx doctor                    # Check for problems
yzx doctor --verbose          # Detailed information  
yzx doctor --fix              # Auto-fix safe issues
```

**What it checks:**
- **Helix runtime conflicts** - Detects old `~/.config/helix/runtime` that breaks syntax highlighting
- **Environment variables** - EDITOR, HELIX_RUNTIME, and other critical settings
- **Configuration health** - yazelix.nix validation and shell integration
- **System status** - Log file sizes, file permissions, git repository state

**Auto-fix capabilities:**
- Backup conflicting runtime directories
- Clean oversized log files  
- Create missing configuration files

## Helix Syntax Highlighting Issues

### Missing Syntax Highlighting
If Helix opens files but shows no syntax highlighting, you likely have **conflicting runtime directories** from a previous Helix installation.

**Symptoms:**
- Files open in Helix but appear as plain text
- No language-specific colors or features
- Error messages about missing grammars

**Solution - Clean Conflicting Runtime:**
```bash
# Check for conflicting runtime directory
ls ~/.config/helix/runtime 2>/dev/null && echo "⚠️  Conflicting runtime found"

# Remove old runtime directory (backup first if needed)
mv ~/.config/helix/runtime ~/.config/helix/runtime.backup
```

**Prevention:**
Yazelix manages its own Helix runtime via `HELIX_RUNTIME` environment variable. Old `~/.config/helix/runtime` directories from previous installations can override this and cause conflicts.

## v9.5 Migration Notes

**If upgrading from v9:**
- Terminal configs are now generated dynamically - no manual copying needed
- Home Manager users: `include_terminal` option removed, replaced with `extra_terminals = []`
- New options: `cursor_trail` and `transparency` automatically apply to all terminals

**Terminal config migration:**
```bash
# Old manual approach (no longer needed):
# cp ~/.config/yazelix/configs/terminal_emulators/ghostty/config ~/.config/ghostty/config

# New approach: configs auto-generated when launching yazelix
nu ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
```

## Quick Fixes

### Reset Configuration
```bash
rm ~/.config/yazelix/yazelix.nix
exit         # Exit current session
yzx launch   # Start fresh in new window - regenerates defaults
```

### Restart Fresh
```bash
exit        # Exit current session  
yzx launch  # Start new session in new window
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
