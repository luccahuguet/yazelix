# VS Code and Cursor Integration

This guide shows you how to use Yazelix tools (Nushell, zoxide, starship, lazygit, etc.) directly in your VS Code or Cursor integrated terminal.
Configure your editor to launch the integrated terminal with the full Yazelix environment.

## For Cursor Users

### Method 1: Settings UI

1. **Open Cursor Settings**: Press `Ctrl+,` (or `Cmd+,` on Mac)
2. **Search for**: `terminal integrated profiles`
3. **Look for**: "Terminal › Integrated: Profiles"
4. **Click**: "Edit in settings.json" (small link next to it)

### Method 2: Direct settings.json Edit

1. **Open Command Palette**: `Ctrl+Shift+P`
2. **Type**: `Preferences: Open Settings (JSON)`
3. **Add the following configuration**:

```json
{
  "terminal.integrated.profiles.linux": {
    "yazelix-nushell": {
      "path": "/usr/bin/bash",
      "args": ["-c", "source ~/.nix-profile/etc/profile.d/nix.sh && cd ~/.config/yazelix && nix develop --impure --command nu"],
      "icon": "terminal-bash"
    }
  },
  "terminal.integrated.defaultProfile.linux": "yazelix-nushell"
}
```

**For macOS users**, replace `linux` with `osx`:
```json
{
  "terminal.integrated.profiles.osx": {
    "yazelix-nushell": {
      "path": "/bin/bash",
      "args": ["-c", "source ~/.nix-profile/etc/profile.d/nix.sh && cd ~/.config/yazelix && nix develop --impure --command nu"]
    }
  },
  "terminal.integrated.defaultProfile.osx": "yazelix-nushell"
}
```

You're done!

## How It Works

The configuration:

1. **Uses bash as launcher**: `/usr/bin/bash` (reliable path across systems)
2. **Sources Nix profile**: `source ~/.nix-profile/etc/profile.d/nix.sh` (makes `nix` command available)
3. **Navigates to Yazelix**: `cd ~/.config/yazelix` (required for `nix develop`)
4. **Launches Nix environment**: `nix develop --impure` (loads all Yazelix tools)
5. **Starts Nushell**: `--command nu` (with full Yazelix configuration)

## What You Get

✅ **All Yazelix tools** available instantly, like `z`, `lg`, `mise`, `starship`, `nu`, etc.  

