# Runtime Notes

These notes preserve runtime details that are too specific for the README but
matter when changing launch, config, editor, shell, or popup behavior.

## Config UI

`yzx config` validates the optional root config and seeds the complete native
`rio/config.toml` when that path is missing. Runtime preparation uses the same
one-time seed.

Packages with the `no-rio` suffix skip that initialization entirely. Their
`yzx enter` path starts the managed workspace in the current terminal, while
`yzx launch` returns an instruction to use `enter` instead.

The UI leaves these sparse sources absent until you save a field:

```text
~/.config/yazelix/config.toml
~/.config/yazelix/zellij/config.kdl
~/.config/yazelix/starship.toml
```

Legacy `mars/config.toml` and `cursors.toml` files are preserved but ignored.

The Helix and advanced native files stay lazy. Opening a file-action row creates
its starter file. Activating either Steel row creates both
`helix/helix.scm` and `helix/init.scm`.

While editing a text field, `Ctrl+e` opens the staged value in the configured
editor environment and returns the edited text to the row. `Enter` saves.

## Managed Appearance

Root `appearance.mode` is the managed dark/light authority. Ratconfig uses it
for its own palette. When `rio/config.toml` is writable, Nova owns only its
top-level `force-theme` field and projects the root mode there. Rio starts
without a theme override and its native config watcher applies later Ratconfig
saves. A custom complete Rio config must supply its own adaptive pair to
participate; every field other than `force-theme` remains native and
user-owned.

Yazelix passes only the root dark/light mode to each new managed Zellij
session. Zellij resolves the matching `theme_dark` or `theme_light` member. A
managed session ignores ambient terminal appearance reports so the explicit
root mode remains authoritative. A root appearance save from inside that
session calls `set-dark-theme` or `set-light-theme` against that exact session.
Zellij's host-theme event switches the top bar between its child-owned dark and
light palettes. A bar loaded by a later tab receives the session's current mode
even when no new host-theme transition occurs. The coordinated save rolls back
if a live component projection fails.

When Rio's config is read-only or cannot be projected, Nova starts Rio with the
explicit mode and captures that mode for the session. Ratconfig, Zellij, the
bar, and new Yazi opens stay on that side even if the root setting changes. The
saved mode applies everywhere together in the next session.

Each managed Yazi launch reads the active session mode before materializing its
config. Yazelix selects the matching native `flavor.dark` or `flavor.light`,
using Yazi Bistro's Bluloco Light default when the light side is absent. An
absent dark side uses Yazi's native preset; Ratconfig exposes that state as the
first dark choice, `default`, without writing it as a flavor name. When a flavor
is selected, both runtime flavor keys receive it so ambient detection cannot
choose the opposite side. The source `theme.toml` is never rewritten, and an
existing Yazi process keeps its current theme until it is reopened.

## Sessions

`yzx enter` and `yzx launch` pass the managed config and
`--new-session-with-layout` to packaged Zellij before forwarding caller
arguments. Zellij owns session creation, attachment, switching, process
lifetime, and native failure messages. Plain launch creates an independent
session. `--session NAME` requests a fresh named session, while `attach NAME`
requires a live target. Attach and switching preserve the target state without
reapplying the managed layout.

The managed Zellij config sets `default_layout "layout"` and points
`layout_dir` at the directory containing the active `layout.kdl` and
`layout.swap.kdl`. Runtime appearance, shell, or widget projection rewrites
that directory to the materialized state path. Zellij lists the configured
default first, so native type-to-create selects the Yazelix layout before it
creates the named session.

Use full names for supported attach behavior. Native unique-prefix matching,
rename, and structural resurrection remain Zellij surfaces outside the Nova v1
continuity contract. Yazelix adds no registry or second session owner.

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
layout_dir
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

## Sidebar Provider

`sidebar.command = "radar"` selects the packaged Radar plugin. Any other valid
value creates a terminal pane with that executable and `sidebar.args`. The
config helper validates the pair and renders one KDL pane body; the launcher
substitutes it into the base and swap layouts. Both providers therefore use the
same Zellij pane named `sidebar`, including its open and collapsed sizes.

Nova captures the provider when it creates a session. It passes custom
arguments as argv without a shell. The pane orchestrator remains
provider-neutral and addresses the pane by name. With a custom provider, Nova
omits Radar command bindings and permission seeding, sets
`YZX_RADAR_ENABLED=false` for child launchers, and skips Radar's Codex doctor
path.

## Popup Lifecycle

Opening or revealing a managed popup shows Zellij's tab-wide floating layer.
Toggling the focused popup off or explicitly closing it hides that layer after
closing or suppressing the managed pane. This returns directly to the tiled
workspace instead of exposing a session manager, terminal, or other floating
pane underneath. Suppressed keep-alive popups and unrelated floating panes
continue running; their explicit keybindings can show them again.

## Popup Geometry

The popup plugin observes the tiled pane named `sidebar`. While that pane is
wider than its collapsed divider, managed popups use a shared 33-cell left
margin so the framed 32-column sidebar rail remains visible. A pane update
reflows any visible managed popup without restarting it. When the rail is
collapsed, the configured side margin applies symmetrically again.

For a sidebar toggle, the pane orchestrator asks Nova Zellij to select the
exact named tiled swap layout underneath the visible floating layer. Zellij
applies it in one render, so the popup is never hidden, restarted, or
refocused.

## Agent Popup

Radar owns agent activity. The top bar does not track execution state; its tabs
use Zellij's native names, selection, bells, and layout indicators. The
orchestrator's v2 session response retains an empty `extensions.ai_pane_activity`
list for existing consumers, but no activity tracker or broadcast feeds it.
Codex activity status and prompt labels reach Radar through enabled, trusted
Radar hooks. Nova does not query Codex for activity or provide an equivalent
hookless fallback. Without those hooks, ordinary pane and command information
remains available, but Codex activity reporting is unavailable.

The packaged agent launcher gives the pane its initial `agent popup` terminal title,
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

With Radar selected, the agent launcher runs `zj-radar setup codex --check`
before the first interactive Codex launch in a Nova state directory. A healthy or intentionally
disabled installation is remembered without mutation. Missing hooks produce one
`Enable Codex activity in Radar? [Y/n]` prompt when stdin and stderr are
terminals, preceded by the hook requirement and `yzx radar-setup` recovery path.
Either answer creates
`agent/radar-codex-setup-offered`; yes runs marker-owned setup and no starts Codex
without enabling activity reporting. Closing the prompt input or entering an
unrecognized answer leaves no marker. Redirected input or prompt output skips
the prompt and marker.
Later launches leave removed or disabled hooks alone. Setup failures warn without
blocking Codex. `yzx doctor` replays Radar's read-only Codex diagnosis and keeps
missing integration warning-only. It omits the trust reminder until hooks exist
and reports startup failures once through the shared diagnostic. Its default
report groups useful health checks and colors statuses on a TTY.
`yzx doctor --verbose` prints Radar's raw report and individual Classic residue
entries. `yzx status` remains
the owner of paths and settings. Trust remains owned by Codex's `/hooks` UI.
`yzx radar-setup` and its `Alt Shift M` entry invoke the packaged
`zj-radar setup codex claude opencode` with inherited terminal input/output and
without `--yes`. Radar owns sequential detection, existing-state handling,
consent and installation. Its prompts default to no (`[y/N]`); noninteractive
setup skips writes. Codex and OpenCode detection also considers existing config,
including `CODEX_HOME` and `XDG_CONFIG_HOME`. Setup does not consult Nova's offer
marker, so a remembered decline never prevents explicit setup. Child output and
exit status are preserved, and unsupported Nova arguments fail before setup.
Start a fresh agent session afterward and review Codex trust in `/hooks`.
The automatic Codex offer never runs setup for other agents. Claude Code and
OpenCode setup stays explicit; Grok and Pi have no Radar adapter.

Any other `agent.command` value is executed directly by the same launcher for
new sessions, so custom commands receive the same initial title. Put argv-style
arguments in `agent.args`, not in `agent.command`.

## Yazi Picker and Popup

Every new managed tab starts with the configured sidebar provider and one
focused tiled `yazi_picker`. A successful file, directory, or `Alt z` workspace
choice owns the tab's initial retarget, creates the managed editor, and only
then closes that exact picker by id. There is no hidden starter shell or
prestarted editor whose cwd can become stale.

`Alt Shift Y` asks the pane orchestrator to toggle the packaged `yazi` popup
with the active tab's canonical workspace root as its explicit request cwd.
The popup uses `toggle_close_behavior "hide"` and
`preserve_on_cwd_change true`, so ordinary toggles only hide or show the same
live Yazi process. Yazi remains the sole owner of its navigation state even if
its process cwd or the canonical tab root changes. `yzx reveal` sends one
configured `replace` request to the popup owner with the absolute target as a
launch argument. Yazi reveals that target during normal startup, so reveal does
not wait for or address a partially started process. This deliberately resets
the popup's navigation state while leaving the canonical workspace unchanged.
The managed `PATH` resolves `yzx` to the exact package that launched the
session before consulting the inherited host path, so mixed installed channels
cannot change the behavior of Helix's reveal command.
Managed Yazi owns `Alt r` locally and passes no hovered path. The popup role
sends one explicit `hide` to the popup owner, preserving the live Yazi process
and navigation state and returning to the underlying pane. It does not open the
hovered item or change editor cwd or the canonical workspace; `Enter` remains
the explicit file-open action. Helix owns its
reveal binding locally as well. Zellij does not intercept or reroute the chord,
so hiding the popup cannot replay it as a Helix reveal.

The popup runs the `yzx-yazi` launcher and layered config with the private
`workspace-popup` role. Popup navigation and ordinary opens retain their local
and canonical-workspace semantics.

## Shell History, Nushell, And Starship

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

Managed Nu enables the pinned native clipboard commands. Pipe any Nu value into
`clip copy` (for example, `pwd | clip copy`) and read it with `clip paste`.
The managed `clc` and `clp` aliases provide shorter copy and paste forms.
Nushell owns platform clipboard access and errors; Nova does not translate
remote or headless sessions to a client clipboard.

`shell.atuin = true` sources Nova's build-generated Atuin integration after the
selected shell's user startup layer. `yzx-nu` composes Nushell after user
`nu/config.nu`; `yzx-shell` uses `~/.bashrc`, the effective Zsh `.zshenv` and
`.zshrc`, or Fish's `config.fish`. An existing shell-specific Atuin search
function skips the managed source. Nova uses the locked packaged binary, binds
contextual history search to `Ctrl+r`, leaves Up-arrow with native history, and
disables Atuin AI bindings. `ATUIN_NOBIND` suppresses managed bindings while
retaining lifecycle hooks. A managed init failure reaches stderr without
preventing shell startup. `shell.atuin = false` omits Nova's hooks and bindings
without changing either history store. See
[Configuration](configuration.md#atuin-history) for explicit one-time imports
and the native-config ownership boundary.

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

`keybindings.sidebar_focus` adds, remaps, or disables Nova's generated
`:forest-open` binding and Forest's matching focus toggle. A user-owned Helix
binding at the old key is preserved when Forest moves or is disabled. Managed
Helix prepends the exact packaged Forest, notify, and glyph modules to
`STEEL_SEARCH_PATHS`, opens Forest with the `forest.side` choice (`right` by
default), and still loads user Steel initialization last. When managed Helix
starts on one directory, Forest stays visible but unfocused so Helix's native
directory picker owns input.

`helix/languages.toml` is loaded by the managed Helix config dir when present.
`helix/helix.scm` and `helix/init.scm` load through a private
`HELIX_STEEL_CONFIG` overlay once both files exist. The overlay keeps the user
command module and init while retaining the packaged bridge startup. Without a
user Steel pair, the packaged module provides `:yzx-new-shell`, which opens a
new Yazelix terminal pane at the current file directory or workspace.

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
which keeps user fetchers while restoring the two managed Git fetchers exactly
once. Broken config paths, invalid TOML, and incomplete flavors stop Yazi launch.
The managed edit opener is always restored. Normal `~/.config/yazi` is not read.

Optional `yazi/starship.toml` is a complete replacement for Nova's packaged
compact Starship header config, not a merge layer. Its presence alone activates
materialization. The materializer requires readable TOML and links it into the
effective config as `yazelix_starship.toml`; omission retains the packaged file.
The workspace popup uses that same effective path.

Automation can materialize that config without starting Yazi, Zellij, Rio, an
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
Home Manager installations. The active session appearance and the packaged
Yazi Bistro light default are supplied internally; the public command takes no
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
