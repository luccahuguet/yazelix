# Runtime Notes

These notes preserve runtime details that are too specific for the README but
matter when changing launch, config, editor, shell, or popup behavior.

## Config UI

`yzn config` creates the core config sources when they are missing:

```text
~/.config/yazelix-next/config.toml
~/.config/yazelix-next/mars/config.toml
~/.config/yazelix-next/zellij/config.kdl
~/.config/yazelix-next/starship.toml
```

The Helix and advanced native files are lazy. The config UI creates them only
when a row opens the file. Activating either Steel row creates both
`helix/helix.scm` and `helix/init.scm`.

While editing a text field, `Ctrl+e` opens the staged value in the configured
editor environment and returns the edited text to the row. `Enter` saves.

## Zellij Sidecars

`zellij/config.kdl` is a guarded sidecar for scalar preferences such as pane
frames, mouse mode, scrollback size, copy behavior, styled underlines, startup
tips, and `ui.pane_frames.rounded_corners`.

The runtime rejects uncommented top-level ownership nodes in that sidecar:

```text
keybinds
default_shell
default_layout
layout
plugins
load_plugins
support_kitty_keyboard_protocol
env
session_name
attach_to_session
```

Use `zellij/plugins.kdl` for extra plugin declarations. It accepts only
`plugins` and `load_plugins` blocks:

```kdl
plugins {
    my_plugin location="file:/home/me/.config/zellij/plugins/my_plugin.wasm"
}

load_plugins {
    my_plugin
}
```

Plugin ids owned by Yazelix, such as `yzpp` and
`yazelix_pane_orchestrator`, cannot be redeclared. Plugin keybindings are not
managed by this sidecar.

## Agent Popup

The agent popup chooses a provider once per state directory. On first launch it
checks `PATH` in this order:

```text
codex resume
grok
opencode
pi
claude --resume
```

It stores the selected provider at:

```text
${YAZELIX_STATE_DIR}/agent/provider
```

Later launches use that stored provider. If the stored provider is unknown or
missing from `PATH`, the popup prints a diagnostic and tells the user to remove
the provider file so Yazelix can choose again.

## Nushell And Starship

When `shell.program = "nu"`, Yazelix does not read normal Nushell config. It
generates runtime Nu files that source packaged Yazelix config first and then
optional user files:

```text
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

If host `mise` is available on the inherited `PATH`, managed Nu inserts
`mise activate nu` output after packaged `config.nu` and before user
`nu/config.nu`. Missing or failing `mise` is skipped.

Managed Nu sets `STARSHIP_CONFIG` to
`~/.config/yazelix-next/starship.toml` when that file exists. Otherwise it sets
`STARSHIP_CONFIG` to an empty config, so normal `~/.config/starship.toml` does
not affect the managed Nu prompt.

## Helix

`yzn-hx` builds the effective Helix config on each launch from the packaged
default plus optional managed user files under:

```text
~/.config/yazelix-next/helix/
```

`helix/config.toml` is deep-merged over the packaged TOML default. The generated
effective file is `${YAZELIX_STATE_DIR}/helix/config.toml`. Yazelix reserves
`Alt r` for reveal, so the generated config enforces:

```toml
[keys.normal]
A-r = ':sh yzn reveal "%{buffer_name}"'
```

`helix/languages.toml` is loaded by the managed Helix config dir when present.
`helix/helix.scm` and `helix/init.scm` load through `HELIX_STEEL_CONFIG` once
both files exist. The packaged Steel module provides `:yzn-new-shell`, which
opens a new Yazelix terminal pane at the current file directory or workspace.

## Yazi

Managed Yazi appends optional user Lua and keymap TOML after the packaged setup:

```text
~/.config/yazelix-next/yazi/init.lua
~/.config/yazelix-next/yazi/keymap.toml
```

This path does not merge `yazi.toml`, themes, or normal `~/.config/yazi`. When
the managed init file exists, plugin directories under
`~/.config/yazelix-next/yazi/plugins/*.yazi` are linked into the runtime config.
Packaged plugin names cannot be overridden.

Example managed Yazi plugin layout:

```text
~/.config/yazelix-next/yazi/plugins/foo.yazi/main.lua
~/.config/yazelix-next/yazi/init.lua
```

```lua
require("foo"):setup()
```

Managed Yazi refreshes sidebar git decorations on setup, directory changes, tab
changes, and managed popup close or hide hooks.
