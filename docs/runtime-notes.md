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

`yzx launch` may create `mars/config.toml` containing only the managed
appearance projection before Mars starts.

The Helix and advanced native files stay lazy. Opening a file-action row creates
its starter file. Activating either Steel row creates both
`helix/helix.scm` and `helix/init.scm`.

While editing a text field, `Ctrl+e` opens the staged value in the configured
editor environment and returns the edited text to the row. `Enter` saves.

## Appearance Projection

Root `appearance.mode` is the managed dark/light authority. Ratconfig uses it
for its own palette. When Mars is included and
`~/.config/yazelix/mars/config.toml` is a writable regular file, a global save
or reset updates only `mars.appearance.preset`; Mars's existing directory
watcher reloads that change. Before `yzx launch`, the runtime performs the same
reconciliation and removes any inherited `MARS_APPEARANCE` that could override
the file.

A store-backed, symlinked, or otherwise read-only Mars file is never replaced.
In that case, launch sets `MARS_APPEARANCE` to the validated root value and the
change applies to the new process. `yzx enter`, SSH use, and no-Mars packages do
not project terminal appearance. Direct edits to the native Mars field may
affect a running window, but the next global save or managed launch restores the
root value.

Yazelix passes only the root dark/light mode to each new managed Zellij
session. Zellij resolves the matching `theme_dark` or `theme_light` member. A
managed session ignores ambient terminal appearance reports so the explicit
root mode remains authoritative. A root appearance save from inside that
session calls `set-dark-theme` or `set-light-theme` against that exact session.
Zellij's host-theme event switches the top bar between its child-owned dark and
light palettes. A bar loaded by a later tab receives the session's current mode
even when no new host-theme transition occurs. Ratconfig keeps the saved root
value when either component projection fails and reports that the next launch
will retry it.

Each managed Yazi launch reads the current root mode before materializing its
config. Yazelix selects the matching native `flavor.dark` or `flavor.light`,
using Yazi Bistro's Bluloco Light default when the light side is absent. An
absent dark side uses Yazi's native preset; Ratconfig exposes that state as the
first dark choice, `default`, without writing it as a flavor name. When a flavor
is selected, both runtime flavor keys receive it so ambient detection cannot
choose the opposite side. The source `theme.toml` is never rewritten, and an
existing Yazi process keeps its current theme until it is reopened.

## Zellij Sidecars

`zellij/config.kdl` is a guarded sidecar for scalar preferences such as paired
dark/light themes, pane frames, mouse mode, scrollback size, copy behavior,
styled underlines, startup tips, and `ui.pane_frames.rounded_corners`.
Ratconfig uses the same 41 identities from the pinned Zellij assets for both
theme fields, with a flake check keeping the inventory aligned. The inherited
pair is `theme_dark "ansi"` and `theme_light "gruvbox-light"`; resetting a
field removes only its sparse override. Custom names found on either member
join the shared pool shown by both pickers.

The former static `theme` field is not part of the managed model. An existing
assignment remains untouched in the user sidecar and is reported as ignored,
while runtime materialization omits it so the complete pair has one clear
authority.

When `yzx config` runs inside a managed session (`ZELLIJ_SESSION_NAME` or
`YAZELIX_ZELLIJ_SESSION_NAME`, plus `YAZELIX_STATE_DIR`), saving a Zellij tab
field also patches `$YAZELIX_STATE_DIR/zellij/config.kdl` so the running Zellij
watcher can pick up scalars without rewriting integration patches. Editing the
active theme-pair member applies live; editing the other member takes effect on
the next appearance change or managed session launch. Ratconfig labels pane
frames, rounded corners, copy-on-select, and clipboard target as `now` when
that active runtime file is addressable. Mouse mode, scrollback size, styled
underlines, and startup tips say `next session`; outside an active managed
session, every Zellij field is next-session. Quoted custom theme names without
KDL escapes remain accepted; richer string syntax stays preserved but
native-file-only.

Ratconfig scopes sidecar diagnostics instead of marking every Zellij field
invalid. Safe untyped top-level leaves stay unchanged across unrelated saves
and resets without a synthetic field or informational diagnostic for each
leaf. The exact native-file action opens the sidecar for those settings.
Invalid known fields affect only their own row and remain repairable. Advanced
diagnostics report ignored legacy settings, malformed or structured native
nodes, structural KDL comments or continuations, extra metadata on managed
blocks, and integration-owned nodes. Unsafe source state blocks writes, and
each guarded diagnostic names its Yazelix owner.

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

## Popup Lifecycle

Opening or revealing a managed popup shows Zellij's tab-wide floating layer.
Toggling the focused popup off or explicitly closing it hides that layer after
closing or suppressing the managed pane. This returns directly to the tiled
workspace instead of exposing a session manager, terminal, or other floating
pane underneath. Suppressed keep-alive popups and unrelated floating panes
continue running; their explicit keybindings can show them again.

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
cwd. If the canonical root changes, the next ordinary toggle closes the stale
process and launches a fresh one at the new root. `yzx reveal` instead ensures
the existing process is visible and addresses it directly, preserving its
navigation state and leaving the canonical workspace unchanged.

The popup runs the same `yzx-yazi` launcher and layered config as the tiled
sidebar with the private `workspace-popup` role. Packaged Yazi initialization
omits `sidebar-status` for that role. The shared `managed-state` plugin registers
popup and tiled Yazi processes under distinct orchestrator actions, so the
popup's `YAZI_ID`, pane id, and cwd cannot replace the tiled sidebar
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
~/.config/yazelix/yazi/starship.toml
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

Optional `yazi/starship.toml` is a complete replacement for Nova's packaged
compact Starship header config, not a merge layer. Its presence alone activates
materialization. The materializer requires readable TOML and links it into the
effective config as `yazelix_starship.toml`; omission retains the packaged file.
Both the normal sidebar and workspace popup use that same effective path.

Automation can materialize that config without starting Yazi, Zellij, Mars, an
editor, or the full Yazelix runtime:

```sh
effective_config="$(
  yzx yazi-config materialize \
    --user-config-dir "$PWD/yazi" \
    --state-dir "$PWD/state"
)"
YAZI_CONFIG_HOME="$effective_config" yazi --debug
```

`--user-config-dir` names the exact directory that contains `yazi.toml`,
`init.lua`, `plugins/`, and the other native Yazi entries. `--state-dir` names
the isolated state root. Callers must pass both flags; the command does not use
`YAZELIX_CONFIG_HOME` or `YAZELIX_STATE_DIR` as defaults. The selected Yazelix
package supplies the packaged config, including for `no-yazi` variants and
Home Manager installations. Root `appearance.mode` and the packaged Yazi
Bistro light default are supplied internally; the public command takes no
appearance flags. The Home Manager module adds no materializer option.

Successful calls print one absolute effective config directory and a newline
to stdout, with no stderr output. In dark mode, a user directory with no
managed entries returns the package's read-only config directory and leaves the
state path absent. Light mode materializes runtime `theme.toml` even without
user entries so it can select the packaged light default. Other inputs replace
only `<state-dir>/yazi` through a staging directory. Bad command usage exits
64. Materialization failures exit 1; validation failures leave an existing
effective directory intact.

The materializer validates paths, TOML syntax and merge structure, required
integration fields, plugin and flavor shapes and collisions, appearance
projection, source/state overlap, and output construction. Yazi remains
responsible for semantic config checks, Lua and plugin behavior, and external
commands. Run `yazi --debug` with the printed `YAZI_CONFIG_HOME` when automation
needs those checks.

Plugin and flavor directories activate materialization independently of
`init.lua` and are linked into the runtime config. Packaged plugin names cannot
be overridden. A user flavor with a packaged name takes precedence over the
packaged copy. Yazi Bistro supplies the packaged flavor catalog and its
dark/light classification. User-installed, unclassified flavors join both
Ratconfig pools. `theme.toml` remains the native source for explicit choices
and unrelated theme settings; the materializer copies it only when runtime
appearance projection is required. Ratconfig renders its simple values and the
sparse `yazi.toml` layer in the Yazi tab. `package.toml` passes through as
opaque `ya pkg` metadata, but Yazelix never runs the package manager. Create
asset directories in this tree or symlink them from another checkout.

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
