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
- **Configuration health** - yazelix.toml validation and shell integration
- **System status** - Log file sizes, file permissions, git repository state

**Auto-fix capabilities:**
- Backup conflicting runtime directories
- Clean oversized log files
- Create missing configuration files

## Configuration File Migration

**Yazelix now uses `yazelix.toml` and `devenv.nix` instead of the old `yazelix.nix` and `flake.nix`.**

If you have an older Yazelix setup:
- Configuration is now in `~/.config/yazelix/yazelix.toml` (not `yazelix.nix`)
- Development environment is defined in `devenv.nix` (not `flake.nix`)
- The default template is `yazelix_default.toml`

**Migration steps:**
1. It's recommended that you go through the [Installation Guide](installation.md) to properly install devenv
2. Your `yazelix.toml` will be auto-created from `yazelix_default.toml` on yazelix startup if not found
3. Copy any custom settings from your old `yazelix.nix` to the new `yazelix.toml` format

## First Run: Zellij Plugin Permissions (is the top bar looking funny/weird/broken?)

When you first run yazelix, **zjstatus requires you to give it permission:**

Zellij requires plugins to request permissions for different actions and information. These permissions must be granted by you before you start zjstatus. Permissions can be granted by navigating to the zjstatus pane either by keyboard shortcuts (alt h/j/k/l) or clicking on the (top) pane. Then simply type the letter `y` to approve permissions. This process must be repeated on zjstatus updates, since the file changes.

See the [zjstatus permissions documentation](https://github.com/dj95/zjstatus/wiki/2-%E2%80%90-Permissions) for more details.

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

## Quick Fixes

### Reset Configuration
```bash
rm ~/.config/yazelix/yazelix.toml
exit         # Exit current session
yzx launch   # Start fresh in new window - regenerates defaults
```

### Restart Fresh
```bash
exit        # Exit current session  
yzx launch  # Start new session in new window
```

## Desktop Launcher Issues

### "bind: command not found" or Garbled Output

If launching Yazelix from your desktop environment (application menu, keyboard shortcut) shows errors like `bash: bind: command not found` or garbled escape sequences:

**Cause:** Your bash profile files (`.bashrc`, `.bash_profile`) contain interactive-only commands (like `bind` for readline) that fail when bash runs without a TTY.

**Solution:** Update your desktop entry to the latest version:
```bash
cp ~/.config/yazelix/assets/desktop/com.yazelix.Yazelix.desktop ~/.local/share/applications/
```

The updated launcher uses POSIX `sh` with explicit Nix paths, bypassing bash profile issues entirely.

### Desktop Launcher Doesn't Start

If clicking Yazelix in your application menu does nothing:

1. **Check if `nu` is installed:** Run `nu --version` in a terminal
2. **Re-copy the desktop entry:** The launcher may be outdated
   ```bash
   cp ~/.config/yazelix/assets/desktop/com.yazelix.Yazelix.desktop ~/.local/share/applications/
   ```
3. **Verify Nix paths:** Ensure `~/.nix-profile/bin` or `~/.local/state/nix/profile/bin` exists

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

## Getting Help

1. Check logs: `~/.config/yazelix/logs/`
2. Test with defaults: delete `yazelix.toml`
3. Report issues
