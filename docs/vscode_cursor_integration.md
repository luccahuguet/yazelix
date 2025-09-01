# VS Code and Cursor Integration

This guide shows you how to use Yazelix tools (Nushell, zoxide, starship, lazygit, etc.) directly in your VS Code or Cursor integrated terminal.

## Quick Method (Recommended): Using `yzx env`

The easiest way to get Yazelix tools in your VS Code/Cursor terminal:

1. **Open integrated terminal** in VS Code/Cursor
2. **Run**: `yzx env` 
3. **Done!** All Yazelix tools are now available

This method works instantly and doesn't require any configuration changes.

## Advanced Method: Automatic Terminal Profile

Configure your editor to launch the integrated terminal with the full Yazelix environment automatically.

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
    "yazelix-env": {
      "path": "/usr/bin/bash",
      "args": ["-c", "source ~/.nix-profile/etc/profile.d/nix.sh && yzx env"],
      "icon": "terminal-bash"
    }
  },
  "terminal.integrated.defaultProfile.linux": "yazelix-env"
}
```

**For macOS users**, replace `linux` with `osx`:
```json
{
  "terminal.integrated.profiles.osx": {
    "yazelix-env": {
      "path": "/bin/bash",
      "args": ["-c", "source ~/.nix-profile/etc/profile.d/nix.sh && yzx env"]
    }
  },
  "terminal.integrated.defaultProfile.osx": "yazelix-env"
}
```

You're done!

## How It Works

The advanced configuration:

1. **Uses bash as launcher**: `/usr/bin/bash` (reliable path across systems)
2. **Sources Nix profile**: `source ~/.nix-profile/etc/profile.d/nix.sh` (makes `yzx` command available)
3. **Runs yzx env**: Loads all Yazelix tools without the UI interface

## What You Get

✅ **All Yazelix tools** available instantly, like `z`, `lg`, `mise`, `starship`, `nu`, etc.  

