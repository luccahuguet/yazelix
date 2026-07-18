# Runtime Notes

These notes preserve runtime details that are too specific for the README but
matter when changing launch, config, editor, shell, or popup behavior.

## Config UI

`yzx config` validates the optional root config and seeds `cursors.toml` from
the child-owned template when that file is missing. Runtime preparation uses
the same one-time cursor seed.

The UI leaves these sparse sources absent until you save a field:

```text
~/.config/yazelix/config.toml
~/.config/yazelix/mars/config.toml
~/.config/yazelix/zellij/config.kdl
~/.config/yazelix/starship.toml
```

The Helix and advanced native files stay lazy. Opening a file-action row creates
its starter file. Activating either Steel row creates both
`helix/helix.scm` and `helix/init.scm`.

While editing a text field, `Ctrl+e` opens the staged value in the configured
editor environment and returns the edited text to the row. `Enter` saves.

## Zellij Sidecars

`zellij/config.kdl` is a guarded sidecar for scalar preferences such as theme,
pane frames, mouse mode, scrollback size, copy behavior, styled underlines,
startup tips, and `ui.pane_frames.rounded_corners`. Ratconfig lists the 41
themes in the pinned Zellij assets, with a flake check keeping both inventories
aligned. Its virtual `default` choice removes the assignment instead of naming
a synthetic theme.

When `yzx config` runs inside a managed session (`ZELLIJ_SESSION_NAME` or
`YAZELIX_ZELLIJ_SESSION_NAME`, plus `YAZELIX_STATE_DIR`), saving a Zellij tab
field also patches `$YAZELIX_STATE_DIR/zellij/config.kdl` so the running Zellij
watcher can pick up scalars without rewriting integration patches. Fields such
as `theme` and `pane_frames` apply live; `scroll_buffer_size` is session-scoped
and still needs a new session. Quoted custom theme names without KDL escapes
remain accepted; richer string syntax stays preserved but native-file-only.

Ratconfig scopes sidecar diagnostics instead of marking every Zellij field
invalid. Unexposed top-level native leaf nodes are reported as unvalidated,
nonblocking entries on the Advanced tab and kept unchanged across unrelated
saves and resets without interpreting their arguments, properties, or values.
Zellij, not Yazelix, owns whether those nodes are valid.
Invalid known fields affect only their own row and remain repairable. Malformed
or structured native nodes, structural KDL comments or continuations, extra
metadata on managed blocks, and integration-owned nodes remain source-blocking
because the sidecar writer cannot prove those documents safe to update.

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

The packaged agent launcher gives the pane its initial `agent` terminal title,
then replaces itself with `[agent].command`. The default `auto` chooses a
provider once per state directory. On first launch it checks `PATH` in this
order:

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

Any other `agent.command` value is executed directly by the same launcher for
new sessions, so custom commands receive the same initial title. Put argv-style
arguments in `agent.args`, not in `agent.command`.

## Yazi Popup

`Alt Shift Y` asks the pane orchestrator to toggle the packaged `yazi` popup
with the active tab's canonical workspace root as its explicit request cwd.
The popup uses `toggle_close_behavior "hide"`, so the popup plugin preserves
the live Yazi process and navigation state while that requested root still
matches. It remembers the launch root separately from Yazi's changing process
cwd. If the canonical root changes, the next reveal closes the stale process
and launches a fresh one at the new root.

The popup runs the same `yzx-yazi` launcher and layered config as the tiled
sidebar with the private `workspace-popup` role. Packaged Yazi initialization
omits `sidebar-state` and `sidebar-status` for that role, preventing its
`YAZI_ID`, pane id, and cwd from replacing the orchestrator's real sidebar
registration. Popup navigation and ordinary opens retain their existing local
and canonical-workspace semantics.

## Nushell And Starship

When `shell.program = "nu"`, Yazelix does not read normal Nushell config. It
generates runtime Nu files that source packaged Yazelix config first and then
optional user files:

```text
~/.config/yazelix/nu/env.nu
~/.config/yazelix/nu/config.nu
```

If host `mise` is available on the inherited `PATH`, managed Nu inserts
`mise activate nu` output after packaged `config.nu` and before user
`nu/config.nu`. Missing or failing `mise` is skipped.

Managed Nu always sets `STARSHIP_CONFIG` to a runtime-effective file. That file
starts with Nova's sparse `[character].format = ":: "` marker and recursively
merges optional native overrides from `~/.config/yazelix/starship.toml`.
Top-level `format` stays unset by default, so Starship retains its native `$all`
layout, including directory, Git, environment, and tool modules. Normal
`~/.config/starship.toml` does not affect the managed Nu prompt.

## Helix

`yzx-hx` builds the effective Helix config on each launch from the packaged
default plus optional managed user files under:

```text
~/.config/yazelix/helix/
```

`helix/config.toml` is deep-merged over the packaged TOML default. The generated
effective file is `${YAZELIX_STATE_DIR}/helix/config.toml`. Yazelix reserves
`Alt r` for reveal, so the generated config enforces:

```toml
[keys.normal]
A-r = ':sh yzx reveal "%{buffer_name}"'
```

`helix/languages.toml` is loaded by the managed Helix config dir when present.
`helix/helix.scm` and `helix/init.scm` load through `HELIX_STEEL_CONFIG` once
both files exist. The packaged Steel module provides `:yzx-new-shell`, which
opens a new Yazelix terminal pane at the current file directory or workspace.

## Tab Workspace

The pane orchestrator owns one canonical workspace root per tab. The first
managed open resolves the containing Git worktree, or the opened directory or
file parent outside Git, and publishes it as explicit state. Later ordinary
Yazi opens preserve that root while passing the requested absolute target to
Helix. This includes ignored files, nested repositories, and non-Git
descendants. A managed open also resets a drifted Helix cwd to the canonical
root. After success, only the originating managed Yazi follows the primary
target's directory; the canonical workspace, shell panes, and hidden agent stay
unchanged.

Yazi `Alt z` is the explicit retarget operation. It updates the orchestrator
and managed editor together; an editor failure restores the prior root and its
bootstrap or explicit provenance. Git and agent popup requests carry the
canonical root explicitly. A hidden agent is reused across pane focus and
local navigation changes, and is replaced only after the canonical root
changes.

## Yazi

Managed Yazi accepts native TOML, optional Lua and keymap sidecars, and a
user-owned asset tree:

```text
~/.config/yazelix/yazi/yazi.toml
~/.config/yazelix/yazi/theme.toml
~/.config/yazelix/yazi/package.toml
~/.config/yazelix/yazi/init.lua
~/.config/yazelix/yazi/keymap.toml
~/.config/yazelix/yazi/plugins/*.yazi/
~/.config/yazelix/yazi/flavors/*.yazi/
```

Native TOML tables merge recursively. User scalars and arrays replace packaged
values; only `plugin.prepend_fetchers` uses replace-plus-managed-Git semantics,
which keeps user fetchers while restoring the two sidebar Git fetchers exactly
once. Broken config paths, invalid TOML, and incomplete flavors stop Yazi launch.
The managed edit opener is always restored. Normal `~/.config/yazi` is not read.

Plugin and flavor directories activate materialization independently of
`init.lua` and are linked into the runtime config. Packaged plugin names cannot
be overridden. A user flavor with a packaged name takes precedence over the
packaged copy. `theme.toml` is the native Yazi surface for choosing flavors;
Ratconfig renders its simple values and the sparse `yazi.toml` layer in the
Yazi tab. `package.toml` passes through as opaque `ya pkg` metadata, but Yazelix
never runs the package manager. Create asset directories in this tree or
symlink them from another checkout.

Example managed Yazi plugin layout:

```text
~/.config/yazelix/yazi/plugins/foo.yazi/main.lua
~/.config/yazelix/yazi/flavors/foo.yazi/flavor.toml
~/.config/yazelix/yazi/init.lua
~/.config/yazelix/yazi/theme.toml
```

```lua
require("foo"):setup()
```

```toml
[flavor]
dark = "foo"
light = "foo"
```

Managed Yazi refreshes sidebar git decorations on setup, directory changes, tab
changes, and managed popup close or hide hooks.
