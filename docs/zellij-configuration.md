# Zellij Configuration

Yazelix has one semantic owner and two native Zellij inputs:

```text
~/.config/yazelix/config.toml
~/.config/yazelix/zellij/config.kdl
~/.config/yazelix/zellij/plugins.kdl
```

`config.toml` owns Yazelix workspace behavior. `zellij/config.kdl` owns safe native preferences. `zellij/plugins.kdl` owns additive third-party plugins. Generated runtime files under `~/.local/share/yazelix/configs/zellij/` are derived state and must not be edited.

## Native preferences

Put native scalar preferences in `zellij/config.kdl`:

```kdl
mouse_mode false
copy_on_select false
copy_clipboard "primary"
copy_command "wl-copy"
scroll_buffer_size 10000
```

Missing sidecars inherit packaged Zellij behavior. Create only the assignments you want to keep explicit. Changes apply to a fresh Yazelix session.

The final Classic bridge recognizes only the exact generated sidecar bytes shipped by the nested-sidecar releases. A writable regular match is backed up with a timestamp and removed so Nova can inherit its packaged defaults. Any byte difference, comment, custom assignment, plugin declaration, symlink, read-only file, or Home Manager-owned file is preserved whole. `yzx status` and `yzx doctor` report present sidecars as explicit Nova input; remove only the rows you want Nova to inherit.

The sidecar uses a first-token ownership guard. It rejects nodes owned by runtime materialization, including:

- `keybinds`
- `default_shell`, `default_layout`, `layout`, and `layout_dir`
- `plugins` and `load_plugins`
- `support_kitty_keyboard_protocol`
- `env`, `session_name`, and `attach_to_session`
- theme, release-note, force-close, and session-serialization policy

The native sidecar owns `show_startup_tips`, `pane_frames`, `default_mode`, and `ui`. A one-time root-config migration carries the retired semantic values into this file before removing them from `config.toml`.

## Third-party plugins

Put additive plugin declarations in `zellij/plugins.kdl`:

```kdl
plugins {
    room location="https://example.invalid/room.wasm"
}

load_plugins {
    room
}
```

Only `plugins` and `load_plugins` are accepted at the top level. The runtime-owned ids `yzpp` and `yazelix_pane_orchestrator` cannot be redeclared. Yazelix preserves third-party plugin bodies and comments while adding its first-party plugins during materialization.

## Yazelix-owned behavior

The semantic root keeps only popup margins, four managed surface chords, custom popup specs, and the top-bar widget order:

```toml
[popup]
side_margin = 1
vertical_margin = 0

[keybindings]
config = "Alt Shift K"
agent = "Alt Shift L"
git = "Alt Shift J"
menu = "Alt Shift M"

[bar]
widgets = ["editor", "shell", "term", "codex_usage", "cpu", "ram"]

[popups.zenith]
command = "zenith"
keybinding = "Alt Shift I"
keep_alive = true
```

Yazelix owns its generated Classic layout, session policy, fixed integration bindings, status bar, popup plugin, and pane orchestrator. Native startup tips, pane frames, default mode, rounded corners, scrollback, mouse behavior, and other allowed Zellij preferences belong in `zellij/config.kdl`. Use plain `zellij` outside Yazelix for full native keymap or layout ownership

## Importing an existing config

Plain `~/.config/zellij/config.kdl` is an explicit import source, never a runtime fallback:

```bash
yzx import zellij
```

The command validates the source once, rejects guarded nodes such as `keybinds`, and splits safe content into `zellij/config.kdl` and `zellij/plugins.kdl`. Existing destinations are not overwritten unless `--force` is used, in which case timestamped backups are written first.

The retired flat `~/.config/yazelix/zellij.kdl` is accepted only by the startup migration. Migration requires both nested destinations to be absent, writes a timestamped backup, splits the content, omits empty generated-only outputs, and removes the flat input. Coexistence or unsafe content fails before destination writes.

## Home Manager

Install the guarded preference file declaratively with exactly one source:

```nix
programs.yazelix.config.zellij.text = ''
  scroll_buffer_size 10000
  mouse_mode false
'';
```

or:

```nix
programs.yazelix.config.zellij.source = ./zellij_config.kdl;
```

There is no separate Home Manager plugin option. Keep `zellij/plugins.kdl` as a normal Yazelix-owned user file.

## Editing and status

Use `yzx edit zellij` for `zellij/config.kdl` and `yzx edit zellij-plugins` for `zellij/plugins.kdl`. The config UI reports both files through its generic sidecar model.

`yzx status` and `yzx doctor` report the managed inputs and generated output. They may show plain `~/.config/zellij/config.kdl` as an available import source, but never as active Yazelix input.

## Troubleshooting

After editing either sidecar, open a fresh Yazelix window. If launch fails, use the reported path, line, and node to remove the guarded declaration or fix the KDL block structure.

For generated-state problems, run `yzx doctor --fix`, then open a fresh window. Do not edit `~/.local/share/yazelix/configs/zellij/config.kdl` directly.

Zellij's native option reference remains at https://zellij.dev/documentation/
