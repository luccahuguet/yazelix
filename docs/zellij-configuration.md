# Zellij Configuration

Yazelix uses a layered Zellij configuration system centered on `settings.jsonc` for Yazelix-owned behavior and a managed native sidecar for advanced Zellij settings that Yazelix does not render.

## Quick Start

Use `settings.jsonc` for Yazelix-owned Zellij behavior, including workspace keybindings, popup commands, sidebars, widgets, and layout settings.

For advanced native Zellij settings that Yazelix does not model, edit:

```bash
~/.config/yazelix/zellij.kdl
```

If you already have a native Zellij config without `keybinds` blocks, import it into the managed path first:

```bash
yzx import zellij
```

## How It Works

The merger prefers your **Yazelix-managed Zellij config** when present, then falls back to your native Zellij config, then forcibly layers Yazelix requirements on top:

1. **User config**: `~/.config/yazelix/zellij.kdl` (if it exists). This managed sidecar is for native Zellij settings that Yazelix does not render from `settings.jsonc`. If missing, Yazelix reads `~/.config/zellij/config.kdl` as a read-only fallback. If neither exists, Yazelix falls back to `zellij setup --dump-config`.
2. **Dynamic Yazelix settings**: Generated from `settings.jsonc` (e.g., `zellij.theme`, `zellij.rounded_corners`, and `zellij.pane_frames`) and appended after the user config so they win.
3. **Enforced Yazelix settings**: Always appended last to guarantee required behavior:
   - `support_kitty_keyboard_protocol` set from `settings.jsonc` (default: false)
   - `default_layout` set to Yazelix’s layout file (absolute path)
   - `layout_dir` set to Yazelix’s generated layouts directory

Layouts are copied into `~/.local/share/yazelix/configs/zellij/layouts`, and the merged config is written to `~/.local/share/yazelix/configs/zellij/config.kdl` on every launch. Yazelix also passes an absolute `--default-layout` at launch for extra safety.

`~/.config/yazelix/zellij.kdl` must not contain a `keybinds` block. Yazelix rejects managed `keybinds` blocks, including `keybinds clear-defaults=true`, because they create a second keybinding owner and can bypass managed workspace controls. Use `zellij.keybindings` and `zellij.native_keybindings` in `settings.jsonc` for Yazelix sessions. Use plain `zellij` outside Yazelix if you want full native keybinding ownership.

## Managed Boundaries

Yazelix manages the Zellij behavior needed for its workspace contract:

- semantic workspace keybindings rendered from `zellij.keybindings`
- curated native Zellij conflict policy rendered from `zellij.native_keybindings`
- popup command wiring rendered from `zellij.popup_commands`
- appearance settings rendered from `settings.jsonc`, including `zellij.theme`, `zellij.rounded_corners`, and `zellij.pane_frames`
- built-in layout directory/default layout selection
- bundled plugin wiring for the pane orchestrator, popup plugin, and status bar
- enforced launch settings such as the generated `layout_dir`

Yazelix does not manage these Zellij surfaces:

- arbitrary Zellij native keymaps or mode bindings
- full `keybinds` ownership inside Yazelix sessions
- the user's plain Zellij config at `~/.config/zellij/config.kdl`, except as a read-only fallback or explicit import source
- manual edits to generated runtime files under `~/.local/share/yazelix/configs/zellij/`
- custom Zellij layouts as a supported Yazelix layout family
- third-party plugin behavior beyond preserving native `plugins` and `load_plugins` blocks while adding Yazelix's required plugins

Use plain `zellij` for full native keymap and layout ownership. Use `settings.jsonc` for Yazelix-session behavior and managed appearance. Use `~/.config/yazelix/zellij.kdl` only for native preferences that are safe to merge into Yazelix sessions and are not already rendered by Yazelix.

## Common Customizations

For complete examples and documentation, see the [user config template](../configs/zellij/user/user_config.kdl).

**Themes and UI (`settings.jsonc`):**
```jsonc
{
  "zellij": {
    "theme": "dracula",
    "rounded_corners": true,
    "pane_frames": true
  }
}
```

**Native Zellij preferences (`~/.config/yazelix/zellij.kdl`):**
```kdl
// Disable mouse mode if it interferes with terminal selection
mouse_mode false

// Simplified UI for better compatibility
simplified_ui true
```

**Zjstatus widget tray (`settings.jsonc`):**
```jsonc
{
  "zellij": {
    "widget_tray": [
      "editor",
      "shell",
      "term",
      "cursor",
      "codex_usage",
      "cpu",
      "ram"
    ],
    "tab_label_mode": "full",
    "claude_usage_display": "both",
    "codex_usage_display": "quota",
    "opencode_go_usage_display": "both",
    "opencode_go_usage_periods": ["5h", "week", "month"],
    "claude_usage_periods": ["5h", "week"]
  }
}
```
Comment out any line to hide that widget. Order matters. Restart Yazelix to regenerate layouts.

`tab_label_mode = "full"` keeps the default tab index plus tab name labels. Set it to `"compact"` when a workspace/root widget already shows the project context and tabs should use only index plus fullscreen/sync/floating state indicators.

`editor`, `shell`, and `term` render static labels from the active Yazelix config. `workspace`, `cursor`, and usage widgets read window-local cached facts so separate Yazelix windows keep independent status-bar state. The cursor widget renders mono presets as colored `█ name` and split presets as one-cell vertical or horizontal split glyphs from the launch-scoped Ghostty cursor fact; it shows `none` when Ghostty cursor trails are disabled, `n/a` outside Yazelix-managed Ghostty cursor sessions, and no segment while the cache is missing. CPU and RAM use bundled runtime helper scripts; RAM reads Nushell `sys mem` data instead of scraping the welcome-screen machine summary.

The Codex usage widget includes quota-window position and official quota percentages by default, for example `[codex 2h20m/5h 49% · 4d5h/7d 80%]`; with `codex_usage_display = "both"` it also shows token totals as `[codex 2h20m/5h 138M 49% · 4d5h/7d 1.34B 80%]`. The Claude usage widget combines local token totals with official quota percentages, for example `[claude 5h|15.5M|75% wk|66.6M|65%]`. The OpenCode Go widget reads OpenCode's local SQLite database directly and renders the compact 5h/week/month shape with the `go` label. Claude and Codex widgets use `tu` from tokenusage. Standalone flake users can install `.#yazelix_agent_tools`; Home Manager users can set `programs.yazelix.agent_usage_programs = [ "tokenusage" ]`.

**Idle screen saver (`settings.jsonc`):**
```jsonc
{
  "zellij": {
    "screen_saver_enabled": false,
    "screen_saver_idle_seconds": 300,
    "screen_saver_style": "random"
  }
}
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
        hide_session_name true
    }
}
```

**For keybindings**, use `settings.jsonc` semantic keys for Yazelix-owned Zellij action remaps instead of copying a Zellij `keybinds` block:
```jsonc
"zellij": {
  "popup_commands": {
    "bottom_popup": ["lazygit"],
    "top_popup": ["yzx", "config", "ui"],
    "menu": ["yzx", "menu"]
  },
  "keybindings": {
    "bottom_popup": ["Alt Shift J"],
    "top_popup": ["Alt Shift K"],
    "menu": ["Alt Shift M"],
    "toggle_editor_right_sidebar_focus": ["Ctrl Shift Y"],
    "toggle_left_sidebar": ["Alt Shift H"],
    "move_focus_left_or_tab": ["Alt h", "Alt Left"],
    "move_focus_right_or_tab": ["Alt l", "Alt Right"]
  }
}
```

Supported owner-local action ids are `open_workspace_terminal`, `popup`, `bottom_popup`, `top_popup`, `menu`, `config`, `move_focus_left_or_tab`, `move_focus_right_or_tab`, `toggle_editor_sidebar_focus`, `toggle_editor_right_sidebar_focus`, `toggle_left_sidebar`, `open_codex_agent_right`, `smart_reveal`, `previous_family`, and `next_family`. `yzx keys` shows the matching scoped ids, such as `zellij.popup`. Omitted actions keep their defaults. Set an action to `[]` to disable Yazelix's generated binding for that action. Yazelix rejects duplicate keys across this semantic map before launch.

Use `zellij.popup_commands` to change the command argv behind the named popup surfaces. The built-in defaults are bottom popup `lazygit`, top popup `yzx config ui`, and menu `yzx menu`.

For Yazelix's curated native Zellij key policy, use `zellij.native_keybindings` in `settings.jsonc`. This covers shipped remaps such as `scroll_mode` / `scroll_mode_unbind`, `session_mode` / `session_mode_unbind`, tab movement, tab jumps, pane grouping, and related Zellij-native conflict cleanup. Omitted entries keep defaults; set an entry to `[]` to disable that one bind or unbind.

Managed `~/.config/yazelix/zellij.kdl` rejects all `keybinds` blocks. A read-only native fallback from `~/.config/zellij/config.kdl` can still contain native keybinds, but Yazelix strips `clear-defaults=true` semantics in that fallback path and appends its generated integration keybindings so popup/menu/sidebar focus behavior keeps working.

**Simple native settings** that Yazelix does not render from `settings.jsonc`, such as `copy_command`, are safe in `~/.config/yazelix/zellij.kdl`.

**For the sidebar launcher**, prefer Yazelix config instead of editing layout templates:
```toml
[editor]
sidebar_command = "nu"
sidebar_args = ["__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu"]
```

The default launches the managed Yazi file-tree adapter. You can point the same managed sidebar slot at another terminal side surface; if `sidebar_args` remains at the default Yazi adapter path, Yazelix renders the custom command with no inherited args. The pane remains named `sidebar` so the pane orchestrator keeps one owner for focus and layout state.

## Current Yazelix Defaults

- Default layout: `yzx_side` for the managed-sidebar startup surface
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
- For nested blocks such as `ui`, you may need to override the entire section

**KDL syntax errors?**
- Check your managed Zellij config syntax against examples in the template
- Zellij will show parsing errors on startup if KDL is invalid

**Migrating from a native Zellij config?**
- Preferred explicit path: run `yzx import zellij` to copy `~/.config/zellij/config.kdl` into `~/.config/yazelix/zellij.kdl` when the source has no `keybinds` blocks
- If you already have a managed override and want to replace it, use `yzx import zellij --force` so Yazelix writes a backup first
- `yzx import zellij` rejects native files that contain `keybinds` blocks; move Yazelix-session remaps to `settings.jsonc` first
- If the managed file is missing, Yazelix can still read `~/.config/zellij/config.kdl` as the base config for that launch, but it will not move or delete it, and `clear-defaults=true` there does not disable Yazelix integration keybindings
- If both files exist, Yazelix keeps using the managed `zellij.kdl` copy and leaves the native Zellij config alone for plain `zellij` launches
- If you want changes from `~/.config/zellij/config.kdl` to become the managed Yazelix config, run `yzx import zellij` explicitly

For complete Zellij configuration options: https://zellij.dev/documentation/
