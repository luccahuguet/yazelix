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

## First Run: Zellij Plugin Permissions

When you first run yazelix, **zjstatus requires you to give it permission:**

Zellij requires plugins to request permissions for different actions and information. These permissions must be granted by you before you start zjstatus. Permissions can be granted by navigating to the zjstatus pane either by keyboard shortcuts or clicking on the pane. Then simply type the letter `y` to approve permissions. This process must be repeated on zjstatus updates, since the file changes.

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
