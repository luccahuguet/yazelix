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
- **Environment variables** - EDITOR and other critical session settings
- **Configuration health** - yazelix.toml validation and shell integration
- **System status** - Log file sizes, file permissions, git repository state

**Auto-fix capabilities:**
- Backup conflicting runtime directories
- Clean oversized log files
- Create missing configuration files

## Configuration File Migration

**Yazelix now uses `yazelix.toml` and the packaged `yazelix` runtime instead of the old `yazelix.nix` flow.**

If you have an older Yazelix setup:
- Configuration is now in `~/.config/yazelix/user_configs/yazelix.toml` (not `yazelix.nix`)
- The normal runtime entry path is the packaged `yazelix` flake output
- The top-level flake now exposes the package-first product surface: `nix run github:luccahuguet/yazelix#yazelix -- launch`
- The default template is `yazelix_default.toml`

**Migration steps:**
1. It's recommended that you go through the [Installation Guide](installation.md) and install the packaged `yazelix` runtime cleanly
2. Your `user_configs/yazelix.toml` will be auto-created from `yazelix_default.toml` on yazelix startup if not found
3. Copy any custom settings from your old `yazelix.nix` to the new `user_configs/yazelix.toml` format

## First Run: Zellij Plugin Permissions (is the top bar looking funny/weird/broken?)

When you first run yazelix, **you need to grant permissions for two separate Zellij plugins:**

- **zjstatus**: its permission prompt can look like an "invisible pane" at the very top where the status bar should be. Navigate to that top bar area either by keyboard shortcuts (`alt h/j/k/l`) or by clicking it, then press `y`.
- **Yazelix pane-orchestrator plugin**: Yazelix should also open a popup asking for permission for its own orchestrator plugin. You need to answer **yes** to that popup too.

`Alt+y` and `Ctrl+y` require the Yazelix pane-orchestrator plugin permissions. `Alt+m` opens a new terminal in the current tab workspace root.

The `zjstatus` permission step must be repeated on `zjstatus` updates, since the file changes.

See the [zjstatus permissions documentation](https://github.com/dj95/zjstatus/wiki/2-%E2%80%90-Permissions) for more details.

### Pane-Orchestrator Rebuild / Reload Limbo

If you rebuild the pane-orchestrator plugin while Yazelix is already open, avoid reloading it in place inside the live session. That can leave Zellij in a broken permission state where the permission popup is unusable and future Yazelix launches open blank tabs.

**Safer maintainer flow:**
```bash
yzx dev build_pane_orchestrator --sync
yzx restart
```

**If you are already stuck with blank tabs or a broken permission popup:**
```bash
zellij delete-all-sessions -f -y
yzx enter
```

Yazelix now keeps the pane-orchestrator on a stable runtime plugin path so previously granted permissions can be preserved across rebuilds, but a clean restart is still safer than reloading the plugin inside the active session.

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
Yazelix aligns Helix with the selected runtime automatically. Old `~/.config/helix/runtime` directories from previous installations can still override that and cause conflicts.

## Quick Fixes

### Reset Configuration
```bash
rm ~/.config/yazelix/user_configs/yazelix.toml
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

**Solution:** Reinstall the generated desktop entry:
```bash
yzx desktop install
```

The updated launcher uses POSIX `sh` with explicit Nix paths, bypassing bash profile issues entirely.

### Desktop Launcher Doesn't Start

If clicking Yazelix in your application menu does nothing:

1. **Reinstall the desktop entry for manual/profile installs:** The launcher may be outdated or still point at an older runtime path
   ```bash
   yzx desktop install
   ```
2. **For Home Manager installs:** reapply your Home Manager configuration instead of running `yzx desktop install`; Home Manager owns the profile desktop entry
3. **Verify your package/profile path:** Ensure the package or Home Manager profile that provides `yzx` is still present
4. **If a stale user-local entry shadows Home Manager:** remove it with `yzx desktop uninstall`

### `yzx update upstream` Still Tries `#install`

If `yzx update upstream` still tries the removed `github:luccahuguet/yazelix#install` path, your shell is almost certainly resolving `yzx` through a stale legacy `~/.local/bin/yzx` wrapper instead of the current profile-owned command

Check what your shell is using:

```bash
type yzx
which yzx
readlink -f "$(which yzx)"
```

If `type yzx` reports a shell function that points at an older `/nix/store/...-yazelix/bin/yzx`, your host shell is still sourcing a stale legacy Yazelix block and shadowing the current profile command.

If `which yzx` points at `~/.local/bin/yzx` while your real install is owned by Home Manager or a Nix profile:

- For Home Manager migration, run `yzx home_manager prepare --apply`, then `home-manager switch`
- For a plain Nix profile install, remove the stale `~/.local/bin/yzx` wrapper and keep the profile-owned `yzx`

After cleanup, open a fresh shell and verify `type yzx` no longer reports a shell function and `which yzx` resolves to the current owner path

## Editor Issues

### File Opening Broken
```bash
echo $EDITOR                    # Should show path
tail ~/.config/yazelix/logs/open_editor.log
```

### Runtime Errors
```bash
hx --health | head -n 8
```

## Getting Help

1. Check logs: `~/.config/yazelix/logs/`
2. Test with defaults: delete `yazelix.toml`
3. Report issues
