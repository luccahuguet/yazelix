# Configuration

`yzx config` opens Nova's Ratconfig interface. It shows packaged defaults,
persists explicit overrides, exposes advanced native files, and identifies
Home Manager-owned configuration as declarative. Yazelix classifies every
non-root inventory field as Core until that inventory receives its own review.
For Main and Popups, Core contains the ordinary product controls and All adds
diagnostics, fine tuning, and every configured custom-popup field.
Normal-mode `a` switches between Core and All when the current tab has an
All-only field; search always spans All without changing the saved view, and
explicit or invalid values remain in Core

On a free-form setting, `Enter` starts single-line inline editing and `e` opens
the same staged value in `editor.command`. Inline editing supports Left/Right,
Home/End, Backspace/Delete, Unicode text, and single-line paste; `Ctrl+e` opens
the editor after an inline edit has started. The temporary editor buffer is
labeled with the field path and runs as a blocking child of Ratconfig, outside
the tab's reusable Helix workspace bridge

## Config root

`yzx config` uses the managed config tree under:

```text
~/.config/yazelix/
```

Set `YAZELIX_CONFIG_HOME` to use another root. Generated runtime state
defaults to:

```text
${XDG_DATA_HOME:-$HOME/.local/share}/yazelix
```

Set `YAZELIX_STATE_DIR` to use another state directory. The managed config
directory must not be the generated `state/yazi` subtree or live below it

## Main settings

The optional root config lives at `~/.config/yazelix/config.toml`. Opening
`yzx config` or starting Nova does not create it. The UI shows packaged defaults
for absent keys, saves only explicit overrides, and removes a key when reset.
Nova rejects unsupported or misspelled paths instead of silently ignoring them,
while custom popup ids remain dynamic within the documented `popups.<id>` fields

| Field | Default | View | Meaning |
| --- | --- | --- | --- |
| `open.log_level` | `info` | All | Diagnostics for managed Yazi open requests: `off`, `error`, `info`, `debug` |
| `shell.program` | `nu` | Core | Packaged shell for new panes: `nu`, `bash`, `zsh`, `fish` |
| `editor.command` | `yzx-hx` | Core | Editor used by Yazi opens, Ratconfig text edits, and Git editor flows |
| `welcome.enabled` | `true` | Core | Show the startup welcome splash |
| `welcome.style` | `random` | Core | Startup screen style |
| `welcome.duration_seconds` | `3` | All | Startup splash duration, 1 to 60 seconds |
| `keybindings.sidebar` | `Alt Shift H` | Core | Hide or show the managed Yazi sidebar |
| `keybindings.sidebar_focus` | `Ctrl y` | Core | Toggle focus between the editor and managed Yazi sidebar |
| `bar.widgets` | `editor`, `shell`, `term`, `codex_usage`, `cpu`, `ram` | Core | Top bar widgets, left to right |

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

| Field | Default | View | Meaning |
| --- | --- | --- | --- |
| `agent.command` | `auto` | Core | Managed agent popup command. `auto` keeps the built-in provider fallback |
| `agent.args` | `[]` | All | Arguments for a custom `agent.command` |
| `popup.side_margin` | `1` | All | Left and right popup margin in terminal cells |
| `popup.vertical_margin` | `0` | All | Top and bottom popup margin in terminal cells |
| `keybindings.config` | `Alt Shift K` | Core | Config popup trigger |
| `keybindings.agent` | `Alt Shift L` | Core | Agent popup trigger |
| `keybindings.git` | `Alt Shift J` | Core | Git popup trigger |
| `keybindings.menu` | `Alt Shift M` | Core | Menu popup trigger |

`agent.command` accepts one executable name or path, not a shell command with
arguments. Keep `agent.command = "auto"` to use the built-in `codex resume`,
`grok`, `opencode`, `pi`, `claude --resume` fallback chain

All managed action keys share syntax validation and case-insensitive collision checks.
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
must be unique. Custom popup keybindings use the same collision checks as all
managed action keys. Ratconfig passes every leaf actually present under a
configured popup through its generic TOML rows. Those values are explicit, so
they remain visible in Core too. Optional fields that are not written and popup
ids that do not exist are not invented; open `config.toml` to add them

## Native config files

| File | Owner | Notes |
| --- | --- | --- |
| `cursors.toml` | Yazelix Cursors | Shared cursor pool, selection, and effects. The child-owned template seeds it once, and Ratconfig preserves custom definitions |
| `mars/config.toml` | Mars | Sparse overrides for appearance preset, window size, opacity, font, scrollbar, and bell |
| `zellij/config.kdl` | Zellij sidecar | Sparse safe scalar overrides where absent assignments inherit packaged defaults. Ratconfig offers Default plus the 41 themes embedded by the pinned Zellij package; Default removes the override, while quoted custom theme names without KDL escapes remain accepted. Unexposed top-level leaf nodes are preserved as unvalidated Advanced diagnostics without interpreting their values. Inside a session, saves and resets also patch the active runtime config. Structural comments or continuations, extra managed-block metadata, other structured native nodes, and integration-owned nodes block unsafe writes |
| `zellij/plugins.kdl` | Zellij plugin sidecar | Extra plugin declarations only. Packaged plugin ids cannot be redeclared |
| `starship.toml` | Starship | Sparse native prompt overrides. Ratconfig curates only `character.format`; absent layout fields retain Starship defaults |
| `helix/config.toml` | Helix | Sparse user TOML merged over packaged Yazelix Helix defaults, with explicit creation starting from only an ownership comment |
| `helix/languages.toml` | Helix | Managed Helix language config |
| `helix/helix.scm` | Helix Steel | Loaded with `helix/init.scm` when the pair exists |
| `helix/init.scm` | Helix Steel | Loaded with `helix/helix.scm` when the pair exists |
| `nu/env.nu` | Nushell | Loaded after packaged Yazelix env |
| `nu/config.nu` | Nushell | Loaded after packaged Yazelix config |
| `yazi/yazi.toml` | Yazi | Native tables merge recursively, while user scalars and arrays replace packaged values. Ratconfig renders safe existing values in its Yazi tab |
| `yazi/init.lua` | Yazi | Appended after packaged Yazi init |
| `yazi/keymap.toml` | Yazi | Appended after packaged Yazi keymap |
| `yazi/theme.toml` | Yazi | Native theme config. Ratconfig renders safe existing values and provides installed dark/light flavor pickers |
| `yazi/package.toml` | Yazi | Opaque package metadata that Yazelix does not process with `ya pkg` |

The managed Yazi merge restores Yazelix's edit opener and its two sidebar Git
fetchers exactly once. Other user fetchers and previewers remain in the merged
native config. Invalid TOML, a broken input, or an incomplete flavor stops launch

Managed `plugins/*.yazi` and `flavors/*.yazi` directories are linked into the
runtime config even without `init.lua`. A user-managed `starship.yazi` replaces
the packaged Starship plugin as one complete directory and must contain
`main.lua`; plugin directories are never recursively merged. Nova still
initializes and refreshes Starship, so a replacement must preserve that plugin
API. Other packaged plugin names remain protected because they own managed
layout, navigation, or sidebar behavior. A user flavor with a packaged name
takes precedence, so `ya` can own an explicitly installed version. Create the
directories directly under the managed Yazi tree or symlink them there

Managed files and asset directories may be symlinked from another checkout, but
their resolved targets must stay outside the generated `state/yazi` runtime

Ratconfig's Yazi tab reads the sparse user `yazi.toml` against Nova's packaged
layer and reads native `theme.toml`. Strings, booleans, integers, finite floats,
and non-empty string arrays with safe dotted paths are editable; complex tables,
empty or complex arrays, non-finite floats, and quoted paths remain
read-only rows with compact previews of their complete values. On a writable
source, press `e` on a structured `yazi.toml` or `theme.toml` row to open that
exact file; read-only sources retain their ownership guidance. The file actions
in the same tab also open those files plus `keymap.toml`, `package.toml`, and
`init.lua` for complete native editing. A setting added through the file action
appears in Ratconfig after the editor closes. Structured saves apply on the next
managed Yazi launch or sidebar reopen

### Yazi flavors

Nova packages Catppuccin Latte, Frappé, Macchiato, Mocha, and Dracula from the
official `yazi-rs/flavors` repository. Press `8` in Ratconfig and choose the
dark and light flavors. Ratconfig writes only the corresponding native
`theme.toml` keys, and reset returns that mode to Yazi's default theme

Install community flavors or an explicitly user-managed version into writable
managed config with Yazi's package manager:

```sh
config_home="${YAZELIX_CONFIG_HOME:-${XDG_CONFIG_HOME:-$HOME/.config}/yazelix}"
mkdir -p "$config_home/yazi"
YAZI_CONFIG_HOME="$config_home/yazi" \
  yzx run ya pkg add yazi-rs/flavors:catppuccin-mocha
```

Select it through Ratconfig or in `$config_home/yazi/theme.toml`:

```toml
[flavor]
dark = "catppuccin-mocha"
light = "catppuccin-mocha"
```

`ya` owns `package.toml` and the installed flavor directory. Yazelix uses its
packaged, version-matched `ya` for `yzx run ya`, projects those native files at
Yazi launch, and never installs or upgrades packages automatically. Compatible
user-installed flavors appear in the Ratconfig picker automatically

Home Manager can select a packaged flavor without installing another source:

```nix
programs.yazelix.config.yazi.theme.text = ''
  [flavor]
  dark = "catppuccin-mocha"
  light = "catppuccin-mocha"
'';
```

For a flavor Nova does not package, pin its repository as a non-flake input:

```nix
inputs.my-yazi-flavor = {
  url = "github:owner/flavor-repository";
  flake = false;
};
```

Link the repository under a native `.yazi` directory and select its package
name through Yazelix's `theme.toml` passthrough:

```nix
programs.yazelix.config.yazi.theme.text = ''
  [flavor]
  dark = "my-flavor"
  light = "my-flavor"
'';

xdg.configFile."yazelix/yazi/flavors/my-flavor.yazi".source =
  inputs.my-yazi-flavor.outPath;
```

Home Manager owns that store-backed flavor. Update its flake input rather than
running `ya pkg` against the read-only managed directory.

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
`zellij/config.kdl`. Saving writes only the selected override, and resetting
removes that key. The Starship tab curates only `character.format`, whose Nova
default is `:: `. Managed Nu materializes that sparse marker override under
runtime state without setting top-level `format`, so Starship retains its native
`$all` layout. Mars and Zellij layer their sparse files over packaged
configuration directly. Untouched defaults follow upgrades

Ratconfig's Zellij theme picker lists the identities declared by the pinned
Zellij package rather than maintaining theme definitions. Choosing `default`
removes the sparse `theme` assignment. A custom theme name written directly in
`zellij/config.kdl` remains valid and visible; the picker itself offers the
packaged set.

The first config or runtime use seeds `cursors.toml` without replacing an
existing file. Its Cursors tab edits the enabled pool, selection, and common
effect settings, while the full-file row opens custom cursor definitions.
`yzx launch` passes this exact file to Mars. Mars currently consumes cursor
selection and basic trail enablement, while the richer trail/mode effects, glow,
and duration remain available to compatible consumers such as a future Ghostty
integration

Saving `mars.appearance.preset` through `yzx config` switches Mars and the config
UI palette in the same session. Opacity, font size, line height, scrollbar, and
bell changes also apply to open Mars windows. Width and height apply to newly
created Mars windows. Zellij sidecar saves and resets update the active managed
session config when `yzx config` runs inside a session, and many scalars apply
live via Zellij's watcher, while some still need a new session

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

`Alt z` opens a zoxide picker in Yazi, moves to the selected directory, and
explicitly retargets the tab workspace and managed editor through `yzx-open`.
Ordinary Yazi opens keep the existing tab workspace. After the editor accepts
the request, the originating managed sidebar follows the primary file's parent
or the opened directory; failed and non-sidebar opens do not move it

`yzx-open` writes bounded logs under:

```text
${YAZELIX_STATE_DIR}/logs/yzx-open.log
```

Implementation-level config layering and sidecar contracts live in
[Runtime Notes](runtime-notes.md)
