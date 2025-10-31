# Zed, VS Code, and Cursor Integration

Use Yazelix tools (Nushell, zoxide, starship, lazygit, etc.) directly from your editor's built-in terminal with dedicated Zed and VS Code/Cursor launchers.

## Zed Integrated Terminal

Zed launches shells directly, so point it at Bash and let `yzx env` take over:

```json
{
  "terminal": {
    "shell": {
      "with_arguments": {
        "program": "bash",
        "args": ["-ic", "yzx env"]
      }
    }
  }
}
```

- `-i` makes Bash interactive, so it sources your `~/.bashrc` (where `yzx` is typically defined).
- `yzx env` hands control to the shell configured in `yazelix.nix` (defaults to Nushell via `nushell/scripts/core/yazelix.nu:214`).
- Prefer to stay in Zed's original shell? Swap the command for `yzx env --no-shell`.

## VS Code and Cursor Integrated Terminal

### Quick Method (Recommended)

1. Open your integrated terminal.
2. Run `yzx env`.
3. Enjoy the full Yazelix toolchain in the shell defined by your configuration.

Need to keep the existing shell? Use `yzx env --no-shell` instead.

### Advanced Method: Automatic Terminal Profile

Configure VS Code or Cursor to launch directly into the Yazelix environment.

#### For Cursor Users

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

Both launchers rely on the same flow:

1. **Bash is the bootstrapper** – either directly (`program: "bash"`) or via `/usr/bin/bash`
2. **Editor startup scripts load `yzx`** – interactive Bash sessions run `~/.bashrc`, which typically defines your Nix profile and the `yzx` command
3. **`yzx env` loads Yazelix tools** – it drops into the shell specified in `yazelix.nix` with all Yazelix binaries and environment variables ready to use (add `--no-shell` to keep the existing shell)

## What You Get

✅ **All Yazelix tools** available instantly, like `z`, `lg`, `mise`, `starship`, `nu`, etc.  
