# Zed, VS Code, and Cursor Integration

Use Yazelix tools (Nushell, zoxide, starship, lazygit, etc.) directly from your editor's built-in terminal. Two configurations are supported:

- **Optional profile**: add a named Yazelix terminal without replacing your default
- **Default terminal**: make Yazelix your editor's default integrated terminal

## Quick Method: One-Off Yazelix Terminal

The simplest approach is the same in every editor:

1. Open your editor's integrated terminal.
2. Run `yzx env`.
3. Work in the Yazelix environment for that terminal only.

If you want to keep the editor's current shell instead of switching into the shell configured in `settings.jsonc`, use:

```bash
yzx env --no-shell
```

This is the lowest-friction option and leaves your editor's normal terminal behavior unchanged.

## VS Code and Cursor

### Optional: Named Yazelix Terminal Profile

VS Code supports named integrated terminal profiles through `terminal.integrated.profiles.<platform>`. Cursor uses the same `settings.json` model in practice.

To add an optional Yazelix terminal without replacing your normal default terminal:

- add a new named profile
- do **not** set `terminal.integrated.defaultProfile.<platform>`

#### Linux

Add this to `settings.json`:

```json
{
  "terminal.integrated.profiles.linux": {
    "Yazelix Env": {
      "path": "/usr/bin/bash",
      "args": ["-ic", "yzx env"],
      "icon": "terminal-bash",
      "overrideName": true
    }
  }
}
```

#### macOS

Add this to `settings.json`:

```json
{
  "terminal.integrated.profiles.osx": {
    "Yazelix Env": {
      "path": "/bin/bash",
      "args": ["-ic", "yzx env"],
      "icon": "terminal-bash",
      "overrideName": true
    }
  }
}
```

#### How to Open It

After adding the profile:

1. Open the terminal panel.
2. Use the terminal dropdown next to **+**.
3. Choose **Yazelix Env**.

That opens a Yazelix-powered terminal only when you explicitly pick it. Your existing default terminal stays unchanged.

### Default: Make Yazelix the Default Terminal

If you want every new integrated terminal to start in the Yazelix environment, set the Yazelix profile as the default.

#### Linux

```json
{
  "terminal.integrated.profiles.linux": {
    "Yazelix Env": {
      "path": "/usr/bin/bash",
      "args": ["-ic", "yzx env"],
      "icon": "terminal-bash",
      "overrideName": true
    }
  },
  "terminal.integrated.defaultProfile.linux": "Yazelix Env"
}
```

#### macOS

```json
{
  "terminal.integrated.profiles.osx": {
    "Yazelix Env": {
      "path": "/bin/bash",
      "args": ["-ic", "yzx env"],
      "icon": "terminal-bash",
      "overrideName": true
    }
  },
  "terminal.integrated.defaultProfile.osx": "Yazelix Env"
}
```

Setting `defaultProfile` means every new VS Code or Cursor terminal opens directly into the Yazelix environment. You can still open a non-Yazelix terminal from the dropdown if needed.

### Notes

- `bash -ic` starts an interactive Bash shell so your normal shell init can expose `yzx`.
- `yzx env` then loads the Yazelix environment and switches into the shell configured in `settings.jsonc`.
- Prefer to stay in Bash/Zsh/Fish/Nushell instead of switching shells? Change the profile command to `yzx env --no-shell`.
- If `yzx` is not available in your editor terminal yet, make sure your normal shell startup files expose `~/.local/bin` on `PATH`.

## Zed

### Optional: Task-Based Yazelix Terminal

Zed does support terminal shell configuration with:

```json
{
  "terminal": {
    "shell": {
      "with_arguments": {
        "program": "/bin/bash",
        "args": ["--login"]
      }
    }
  }
}
```

But that setting is global for Zed's built-in terminal. If you point it at `yzx env`, you are effectively changing the default shell for all Zed terminals.

So if your goal is:

- keep Zed's normal default terminal
- add a separate optional Yazelix terminal entry

the better fit is a **task**, not `terminal.shell`.

#### Global Zed Task

Create or edit `~/.config/zed/tasks.json` and add:

```json
[
  {
    "label": "Yazelix Env",
    "command": "yzx",
    "args": ["env"],
    "use_new_terminal": true,
    "allow_concurrent_runs": true,
    "reveal": "always",
    "hide": "never"
  }
]
```

#### Project-Local Zed Task

If you want this only for one project, put the same task in:

- `.zed/tasks.json`

inside that project.

#### How to Open It

1. Open the command palette.
2. Run `task: spawn`.
3. Choose `Yazelix Env`.

Zed will launch that task in its integrated terminal, giving you an optional Yazelix terminal without changing the default shell used by normal Zed terminals.

### Default: Make Yazelix the Zed Shell

If you want every Zed terminal to start in the Yazelix environment, set the global shell configuration:

```json
{
  "terminal": {
    "shell": {
      "with_arguments": {
        "program": "/bin/bash",
        "args": ["--login", "-ic", "yzx env"]
      }
    }
  }
}
```

This makes Yazelix the default for all Zed terminals. Use `yzx env --no-shell` instead if you prefer to keep your normal shell after loading the Yazelix environment.

### Zed Notes

- Zed tasks run in a login shell, so they typically see the same `PATH` setup as your normal shell startup files.
- If you want to keep the current shell instead of switching into the shell configured by Yazelix, use:
  - `"args": ["env", "--no-shell"]`
- If `yzx` is not available yet, ensure `~/.local/bin` is on your shell `PATH`.

## What You Get

**Optional profile mode:**
- ✅ All Yazelix tools available when you explicitly open the Yazelix terminal
- ✅ Your editor's normal default terminal remains unchanged
- ✅ A clean way to use `z`, `lg`, `mise`, `starship`, `nu`, and the rest of the Yazelix environment only when you want it

**Default terminal mode:**
- ✅ Every new editor terminal starts in the Yazelix environment automatically
- ✅ No need to manually select the Yazelix profile each time
- ✅ You can still open a non-Yazelix terminal from the dropdown if needed
