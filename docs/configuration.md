# Configuration

`yzx config` opens Nova's Ratconfig interface. It shows packaged defaults,
persists explicit overrides, exposes advanced native files, and identifies
Home Manager-owned configuration as declarative

## Config root

`yzx config` uses the managed config tree under:

```text
~/.config/yazelix/
```

Set `YAZELIX_CONFIG_HOME` to use another source-input root. The FlexNetOS
foundation fixes generated runtime state to:

```text
/run/user/1001/yazelix/profile-runtime/yazelix
```

That volatile directory is created and secured by the profile-owned runtime
service and frontdoors.
Non-foundation packages use `YAZELIX_STATE_DIR` or
`${XDG_RUNTIME_DIR}/yazelix`; they do not fall back to durable home storage.

## Codex configuration and rules

FlexNetOS reviews Codex configuration and durable operating rules in
`agent_configs/codex/`. The profile installs those immutable inputs under
`/home/flexnetos/.nix-profile/share/yazelix/agent_configs/codex/`, the
materializer under `/home/flexnetos/.nix-profile/share/yazelix/nushell/scripts/`,
and the executable
`/home/flexnetos/.nix-profile/bin/yazelix_codex_materialize`. Change the repository input and
rebuild the profile; the installed copy is never an alternate editable owner.

The materializer validates both inputs and completes both mode-`0644` staged
files before replacing either live path. POSIX has no atomic rename for two
independent pathnames, so publication uses a durable recovery journal and
rollback copies rather than claiming a two-path atomic rename. If interrupted
after one replacement, the next invocation restores the exact prior pair before
continuing. The profile-owned `codex` wrapper fixes `CODEX_HOME` to
`~/.nix-profile/runtime/codex` and writes `config.toml` and `RULES.md` there
with exact source hashes and do-not-edit markers.

For `config.toml`, every top-level key or table declared by the reviewed source
belongs to that source and replaces the corresponding live value. Live-only
top-level tables, including Codex-managed `hooks.state`, survive materialization.
Edit declarative preferences in the Yazelix input, never in the generated file.
Auth, sessions, databases, and other non-config runtime state are untouched.

The lexical selector must be exactly
`/home/flexnetos/.nix-profile/bin/codex`; resolving to the same store payload
through another path is not sufficient. An approved cutover archives the prior
generated pair, runs `yazelix_codex_materialize`, and checks
`tests/codex_config_provenance.nu` before the installed runtime starts.

The profile-owned `claude` wrapper applies the same boundary through
`CLAUDE_CONFIG_DIR=/run/user/1001/yazelix/profile-runtime/claude`. Reviewed
`settings.json`, `CLAUDE.md`, and `RTK.md` inputs live in
`agent_configs/claude/`; the profile installs them and
`yazelix_claude_materialize`, then the wrapper publishes exact copies before
the Claude payload runs. The mode-`0600` generation receipt records all three
source hashes. Credentials, sessions, histories, databases, and plugin state
remain untouched. Both agent wrappers reject a competing inherited state owner
before invoking their immutable payload.

The profile selector itself must likewise be a direct, relative Nix generation:
`/home/flexnetos/.nix-profile -> .nix-profile-<generation>-link`. Any alias
through a retired user XDG selector is a second ownership layer and fails the
installed profile contract, even when it resolves to identical store bytes.

## Main settings

The optional root config lives at `~/.config/yazelix/config.toml`. Opening
`yzx config` or starting Nova does not create it. The UI shows packaged defaults
for absent keys, saves only explicit overrides, and removes a key when reset.
Nova rejects unsupported or misspelled paths instead of silently ignoring them,
while custom popup ids remain dynamic within the documented `popups.<id>` fields

| Field | Default | Meaning |
| --- | --- | --- |
| `open.log_level` | `info` | Diagnostics for managed Yazi open requests: `off`, `error`, `info`, `debug` |
| `shell.program` | `nu` | Packaged shell for new panes; only Nushell is supported |
| `editor.command` | `yzx-hx` | Editor used by Yazi opens, Ratconfig text edits, and Git editor flows |
| `welcome.enabled` | `true` | Show the startup welcome splash |
| `welcome.style` | `random` | Startup screen style |
| `welcome.duration_seconds` | `3` | Startup splash duration, 1 to 60 seconds |
| `bar.widgets` | `editor`, `shell`, `term`, `codex_usage`, `cpu`, `ram` | Top bar widgets, left to right |

The Codex quota widget identifies periods from their reported duration and shows
five-hour before weekly when both exist. Unavailable periods are omitted.
Updated windows use a versioned cache so older open sessions cannot reintroduce
incompatible quota periods

`editor.command` accepts one executable name or path, not a shell command with
arguments. Inside Nova, `hx` and `yzx-hx` use packaged managed Helix. Other
editors such as `nvim`, or an absolute host Helix path, skip the managed bridge.
Terminal Git clients receive the same selection through `EDITOR`, `VISUAL`, and
`GIT_EDITOR`

## Popups

The `popups` tab edits popup geometry, the managed agent command, and managed
popup role keys:

| Field | Default | Meaning |
| --- | --- | --- |
| `agent.command` | `auto` | Managed agent popup command. `auto` keeps the built-in provider fallback |
| `agent.args` | `[]` | Arguments for a custom `agent.command` |
| `popup.side_margin` | `1` | Left and right popup margin in terminal cells |
| `popup.vertical_margin` | `0` | Top and bottom popup margin in terminal cells |
| `keybindings.config` | `Alt Shift K` | Config popup trigger |
| `keybindings.agent` | `Alt Shift L` | Agent popup trigger |
| `keybindings.git` | `Alt Shift J` | Git popup trigger |
| `keybindings.menu` | `Alt Shift M` | Menu popup trigger |

`agent.command` accepts one executable name or path, not a shell command with
arguments. Keep `agent.command = "auto"` to use the built-in `codex resume`,
`grok`, `opencode`, `pi`, `claude --resume` fallback chain

Custom popups live in root config under `[popups.<id>]`:

```toml
[popups.btm]
command = "btm"
args = ["--basic"]
title = "btm_popup"
keybinding = "Alt Shift B"
keep_alive = true
```

Commands are argv-based. Put arguments in `args`, not in `command`. Popup titles
must be unique. Custom popup keybindings use the same collision checks as the
managed popup role keys

## Native config files

| File | Owner | Notes |
| --- | --- | --- |
| `cursors.toml` | Yazelix Cursors | Shared cursor pool, selection, and effects. The child-owned template seeds it once, and Ratconfig preserves custom definitions |
| `mars/config.toml` | Mars | Sparse overrides for appearance preset, window size, opacity, font, scrollbar, and bell |
| `zellij/config.kdl` | Zellij sidecar | Sparse safe scalar overrides where absent assignments inherit packaged defaults. Inside a session, saves and resets also patch the active runtime config (many apply live, while some need a new session). Integration-owned nodes are blocked |
| `zellij/plugins.kdl` | Zellij plugin sidecar | Extra plugin declarations only. Packaged plugin ids cannot be redeclared |
| `starship.toml` | Starship | Sparse managed Nu prompt overrides where absent values inherit Nova defaults |
| `helix/config.toml` | Helix | Sparse user TOML merged over packaged Yazelix Helix defaults, with explicit creation starting from only an ownership comment |
| `helix/languages.toml` | Helix | Managed Helix language config |
| `helix/helix.scm` | Helix Steel | Loaded with `helix/init.scm` when the pair exists |
| `helix/init.scm` | Helix Steel | Loaded with `helix/helix.scm` when the pair exists |
| `nu/env.nu` | Nushell | Loaded after packaged Yazelix env |
| `nu/config.nu` | Nushell | Loaded after packaged Yazelix config |
| `yazi/yazi.toml` | Yazi | Native tables merge recursively, while user scalars and arrays replace packaged values |
| `yazi/init.lua` | Yazi | Appended after packaged Yazi init |
| `yazi/keymap.toml` | Yazi | Appended after packaged Yazi keymap |
| `yazi/theme.toml` | Yazi | Native theme config, including managed flavor selection |
| `yazi/package.toml` | Yazi | Opaque package metadata that Yazelix does not process with `ya pkg` |

The managed Yazi merge restores Yazelix's edit opener and its two sidebar Git
fetchers exactly once. Other user fetchers and previewers remain in the merged
native config. Invalid TOML stops before Yazi starts

Managed `plugins/*.yazi` and `flavors/*.yazi` directories are linked into the
runtime config even without `init.lua`. Packaged names cannot be replaced.
Create the directories directly under the managed Yazi tree or symlink them
there

For Smart Enter, link `smart-enter.yazi` under `yazi/plugins/`, add
`require("smart-enter"):setup { open_multi = false }` to `yazi/init.lua`, and
bind it in `yazi/keymap.toml`:

```toml
[[mgr.prepend_keymap]]
on = "l"
run = "plugin smart-enter"
```

`l` then enters directories or opens the hovered file through the managed
opener

Normal host config such as `~/.config/helix`, `~/.config/yazi`, and
`~/.config/starship.toml` does not control the managed runtime unless you route
through these Yazelix-owned files

Opening `yzx config` does not create `mars/config.toml`, `starship.toml`, or
`zellij/config.kdl`. Their tabs show effective Nova defaults, saving writes only
the selected override, and resetting removes that key. Mars and Zellij layer
their sparse files over packaged configuration directly, while managed Nu
materializes its effective Starship config under runtime state. Untouched
defaults follow upgrades

The first config or runtime use seeds `cursors.toml` without replacing an
existing file. Its Cursors tab edits the enabled pool, selection, and common
effect settings, while the full-file row opens custom cursor definitions.
`yzx launch` passes this exact file to Mars. Mars currently consumes cursor
selection and basic trail enablement, while the richer trail/mode effects, glow,
and duration remain available to compatible consumers such as a future Ghostty
integration

Saving `mars.appearance.preset` through `yzx config` switches Mars and the config
UI palette in the same session. Other Mars values apply on the next Mars launch.
Zellij sidecar saves and resets update the active managed session config when
`yzx config` runs inside a session, and many scalars apply live via Zellij's
watcher, while some still need a new session

## Editor and file opens

Managed Yazi opens files through `yzx-open`. With the default
`editor.command = "yzx-hx"`, `yzx-open` reuses a live Helix bridge in the same
Zellij tab or opens packaged Helix in the managed `editor` pane. Typing `hx`
inside Nova invokes this same managed Helix wrapper

Git editing stays in the client terminal. Managed LazyGit overlays only its
file-edit commands and keeps user configuration, while it and other terminal
Git clients use `yzx-editor` through the standard editor variables. On return,
the bridge restores the client's transparent Zellij background

`Alt r` reveals the current Helix buffer in the Yazi sidebar. `yzx reveal
<target>` exposes the same path inside a managed session

`Alt z` opens a zoxide picker in Yazi, moves to the selected directory, sends it
through `yzx-open`, and renames the tab to the workspace root

`yzx-open` writes bounded logs under:

```text
${YAZELIX_STATE_DIR}/logs/yzx-open.log
```

Implementation-level config layering and sidecar contracts live in
[Runtime Notes](runtime-notes.md)
