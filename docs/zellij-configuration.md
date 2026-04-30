# Zellij Configuration

Yazelix uses a three-layer Zellij configuration system centered on its managed user config surface.

## Quick Start

Edit your Yazelix-managed Zellij config:

```bash
~/.config/yazelix/user_configs/zellij/config.kdl
```

If you already have a native Zellij config, import it into the managed path first:

```bash
yzx import zellij
```

## How It Works

The merger now prefers your **Yazelix-managed Zellij config** when present, then falls back to your native Zellij config, then forcibly layers Yazelix requirements on top:

1. **User config**: `~/.config/yazelix/user_configs/zellij/config.kdl` (if it exists). If missing, Yazelix reads `~/.config/zellij/config.kdl` as a read-only fallback. If neither exists, Yazelix falls back to `zellij setup --dump-config`.
2. **Dynamic Yazelix settings**: Generated from `yazelix.toml` (e.g., rounded corners) and appended after the user config so they win.
3. **Enforced Yazelix settings**: Always appended last to guarantee required behavior:
   - `pane_frames false` (needed for `zjstatus`)
   - `support_kitty_keyboard_protocol` set from `yazelix.toml` (default: false)
   - `on_force_close` set to `quit` so closed windows do not leave detached Yazelix sessions behind
   - `default_layout` set to Yazelix’s layout file (absolute path)
   - `layout_dir` set to Yazelix’s generated layouts directory

Layouts are copied into `~/.local/share/yazelix/configs/zellij/layouts`, and the merged config is written to `~/.local/share/yazelix/configs/zellij/config.kdl` on every launch. Yazelix also passes `--pane-frames false` and an absolute `--default-layout` at launch for extra safety.

## Common Customizations

For complete examples and documentation, see the [user config template](../configs/zellij/user/user_config.kdl).

**Themes and UI:**
```kdl
theme "dracula"
// theme "nord"
// theme "tokyo-night"

// Disable mouse mode if it interferes with terminal selection
mouse_mode false

// Simplified UI for better compatibility
simplified_ui true
```

**Zjstatus widget tray (yazelix.toml):**
```toml
[zellij]
widget_tray = [
  "editor",  # Active editor
  "shell",   # Active shell
  "term",    # Terminal emulator
  # "workspace", # Workspace root
  # "ai_activity", # AI pane activity
  # "token_budget", # AI token budget
  # "claude_usage", # Grouped Claude usage
  # "codex_usage", # Grouped Codex usage
  # "opencode_usage", # Grouped OpenCode usage
  "cpu",     # CPU usage
  "ram",     # RAM usage
]

agent_usage_display = "both" # "both", "tokens", or "money"
claude_usage_periods = ["day", "month"]
codex_usage_periods = ["day", "month"]
opencode_usage_periods = ["day"]
```
Comment out any line to hide that widget. Order matters. Restart Yazelix to regenerate layouts.

The grouped usage widgets render compactly, for example `[codex d 124M $6.69 | mon 1.58B $98]`. The usage widgets are inert until their matching usage binary is available in the Yazelix runtime. Claude and Codex widgets use `tu` from tokenusage. OpenCode widgets use `ccusage-opencode`. Standalone flake users can install `.#yazelix_agent_tools`; Home Manager users can set `programs.yazelix.agent_usage_programs = [ "tokenusage" "ccusage-opencode" ]`.

**Idle screen saver (yazelix.toml):**
```toml
[zellij]
screen_saver_enabled = false
screen_saver_idle_seconds = 300
screen_saver_style = "random"
```
When enabled, the pane orchestrator opens `yzx screen` after the configured idle threshold. The screen uses the same renderer and styles as the manual `yzx screen` command.

**Session behavior:**
```kdl
// Show startup tips (Yazelix disables by default)
show_startup_tips true

// Copy/paste settings
copy_on_select false
copy_clipboard "primary"
scroll_buffer_size 50000
```

## Best Practices

**For UI settings**, add them to your managed Zellij config:
```kdl
ui {
    pane_frames {
        rounded_corners true  # Yazelix may override via yazelix.toml
    }
}
```

**For keybindings**, use your managed Zellij override config. Layout files define panes and swap layouts; keybindings are policy:
```kdl
keybinds {
    shared_except "locked" {
        bind "Alt Shift Y" {
            MessagePlugin "yazelix_pane_orchestrator" {
                name "toggle_sidebar"
            }
        }
    }
}
```

Sidebar commands such as `toggle_sidebar`, `toggle_editor_sidebar_focus`, and `focus_sidebar` are the stable pane-orchestrator contract. The default keys (`Alt+y`, `Ctrl+y`) are just Yazelix's shipped policy.

**Simple settings** (like `theme`, `copy_command`) work perfectly - your value always wins.

**For the sidebar launcher**, prefer Yazelix config instead of editing layout templates:
```toml
[editor]
sidebar_command = "nu"
sidebar_args = ["__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu"]
```

The default launches the managed Yazi file-tree adapter. You can point the same managed sidebar slot at another terminal side surface; if `sidebar_args` remains at the default Yazi adapter path, Yazelix renders the custom command with no inherited args. The pane remains named `sidebar` so the pane orchestrator keeps one owner for focus and layout state.

## Current Yazelix Defaults

- Default layout: `yzx_side` or `yzx_side_closed` for the managed-sidebar startup surface
- Copy command: `wl-copy` (Wayland clipboard)
- Scrollback editor: `hx` (Helix)
- Session serialization: enabled for Zellij's own session state
- `on_force_close`: `quit`
- Startup tips: disabled

## Troubleshooting

**Config not updating?**
- Restart Yazelix or open a fresh Yazelix window so the managed Zellij config is regenerated
- If the managed runtime config or plugin permissions still look stale after an update, run `yzx doctor --fix` and restart Yazelix

**Settings not working as expected?**
- Check `~/.local/share/yazelix/configs/zellij/config.kdl` for duplicate sections
- Look for your setting - it should appear last to take effect
- For nested blocks (ui, keybinds), you may need to override the entire section

**KDL syntax errors?**
- Check your managed Zellij config syntax against examples in the template
- Zellij will show parsing errors on startup if KDL is invalid

**Migrating from a native Zellij config?**
- Preferred explicit path: run `yzx import zellij` to copy `~/.config/zellij/config.kdl` into `~/.config/yazelix/user_configs/zellij/config.kdl`
- If you already have a managed override and want to replace it, use `yzx import zellij --force` so Yazelix writes a backup first
- If the managed file is missing, Yazelix can still read `~/.config/zellij/config.kdl` as the base config for that launch, but it will not move or delete it
- If both files exist, Yazelix keeps using the managed `user_configs` copy and leaves the native Zellij config alone for plain `zellij` launches
- If you want changes from `~/.config/zellij/config.kdl` to become the managed Yazelix config, run `yzx import zellij` explicitly

For complete Zellij configuration options: https://zellij.dev/documentation/
