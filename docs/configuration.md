# Configuration

`yzx config` opens Nova's Ratconfig interface. It shows packaged defaults,
persists explicit overrides, exposes advanced native files, and identifies
Home Manager-owned configuration as declarative. Yazelix maintains reviewed
recommendation sets for Main, Popups, Zellij, and Yazi; other
non-root inventories recommend all of their fields. Overview also includes
explicit, invalid, externally managed, and field-diagnosed settings. All adds
fine tuning and configured custom-popup fields. Normal-mode `a` switches between
Overview and All only when Overview hides at least three fields and one quarter
of the tab. Search spans All without changing the saved view

Packages with the `no-rio` suffix omit the Rio source, tab, native-file action,
and initialization. Their remaining configuration surfaces are unchanged

On a free-form setting, `Enter` starts single-line inline editing and `e` opens
the same staged value in `editor.command`. Inline editing supports Left/Right,
Home/End, Backspace/Delete, Unicode text, and single-line paste; `Ctrl+e` opens
the editor after an inline edit has started. The temporary editor buffer is
labeled with the field path and runs as a blocking child of Ratconfig, outside
the tab's reusable Helix workspace bridge

Boolean editors accept `hjkl`, arrow keys, or `Space` while staging a value.
The yellow `>` marks the staged choice, and `Enter` saves it

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
while custom popup ids remain dynamic within the documented `popups.<id>`
fields. A locally invalid known value remains visible with its raw input and
packaged baseline. Wholly unparseable or structurally unsafe root files open as
an Advanced diagnostic with an exact `config.toml` action

| Field | Default | View | Meaning |
| --- | --- | --- | --- |
| `appearance.mode` | `dark` | Overview | Shared dark/light appearance and Ratconfig palette |
| `open.log_level` | `info` | All | Diagnostics for managed Yazi open requests: `off`, `error`, `info`, `debug` |
| `shell.program` | `nu` | Overview | Packaged shell for new panes: `nu`, `bash`, `zsh`, `fish` |
| `shell.atuin` | `true` | Overview | Use Atuin history and `Ctrl+r` search in new managed shells |
| `editor.command` | `yzx-hx` | Overview | Editor used by Yazi opens, Ratconfig text edits, and Git editor flows |
| `forest.side` | `right` | Overview | Forest placement in managed Helix: `left` or `right` |
| `sidebar.command` | `radar` | Overview | Packaged Radar plugin or one executable for the managed sidebar |
| `sidebar.args` | `[]` | All | Arguments for a custom `sidebar.command` |
| `welcome.enabled` | `true` | Overview | Show the startup welcome splash |
| `welcome.style` | `random` | Overview | Startup animation, including `friends_and_enemies`, `primordial`, and `game_of_life_tumblers`; `yzx anima --help` lists every supported style |
| `welcome.duration_seconds` | `3` | All | Startup splash duration, 1 to 60 seconds |
| `keybindings.sidebar` | `Alt Shift H` | Overview | Hide or show the managed sidebar |
| `keybindings.sidebar_focus` | `Ctrl y` | Overview | Toggle focus between Forest and managed Helix |
| `bar.widgets` | `editor`, `shell`, `term`, `codex_usage`, `cpu`, `ram` | Overview | Top bar widgets, left to right; rightmost survive longest when tabs need space |

Anima owns the animation set and random selection. `static`, `logo`, and
`friends_and_enemies` require explicit selection; `random` excludes them.
The retired `game_of_life_oscillators` and `game_of_life_bloom` names are invalid;
choose `game_of_life_tumblers` or reset `welcome.style` to `random` in Ratconfig.

The Codex quota widget identifies periods from their reported duration and shows
five-hour before weekly when both exist. Unavailable periods are omitted.
Updated windows use a versioned cache so older open sessions cannot reintroduce
incompatible quota periods

`editor.command` accepts one executable name or path, not a shell command with
arguments. In packages that include managed Helix, `hx` and `yzx-hx` select it.
The no-Helix package reports those managed names as unavailable. Other terminal
editors such as `nvim`, or an absolute host Helix path, skip the managed bridge.
Config native-file actions and terminal Git clients run through `yzx-editor`,
which resolves the current `editor.command` for each edit

### Sidebar

Radar occupies Nova's managed sidebar by default. To run a terminal command in
the same slot, set one executable and its argv separately:

```toml
[sidebar]
command = "yzx-yazi"
args = []
```

`yzx-yazi` is Nova's installed managed Yazi wrapper. It uses Yazelix's selected
Yazi, configuration, theme, and opener without requiring a Nix store path or a
separate Yazi installation. Other executables work the same way; for example,
use `command = "btm"` with `args = ["--basic"]`.

The choice applies when you start the next Yazelix session. A custom command
uses the same 32-column side, pane frame, collapsed layout, focus navigation,
popup margins, and `Alt Shift H` toggle. Nova runs it without a shell. If the
command is missing or exits, Zellij shows that failure in the sidebar while the
rest of the session stays available.

`radar` selects the packaged plugin and does not accept `sidebar.args`.
`yzx-yazi` and other custom commands omit Radar's four command shortcuts, cached permission grant, Codex
setup prompt, and doctor integration. User-installed Radar hooks remain under
the user's control.

### Atuin history

`shell.atuin = true` initializes packaged Atuin after the normal user startup
layer of the selected Nushell, Bash, Zsh, or Fish process. Atuin stores captured
commands on the local machine and owns `Ctrl+r`; Up-arrow remains native shell
history, and Atuin AI bindings stay disabled. Atuin owns accounts, sync, AI,
its daemon and pty proxy, and `~/.config/atuin/config.toml`. Nova packages local
capture and search. Review Atuin's privacy filters before capturing commands
from sensitive directories

Set `shell.atuin = false` to disable Nova's managed initialization without
changing either history store. A user-sourced Atuin integration runs regardless
of this setting and remains the sole hook owner. Nova loads managed Atuin after
`~/.config/yazelix/nu/config.nu`, `~/.bashrc`, the effective Zsh `.zshenv` and
`.zshrc`, or Fish's `config.fish`; it does not parse or rewrite those files.
Set `ATUIN_NOBIND` in the matching file to retain Atuin capture without Nova's
managed bindings

To copy existing native history into Atuin, run exactly one matching import:

```sh
# Nushell's default plaintext history
atuin import nu

# Only if you configured Nushell to use SQLite history
atuin import nu-hist-db

# Bash, Zsh, or Fish native history
atuin import bash
atuin import zsh
atuin import fish
```

A second import duplicates imported records. Import leaves native history
intact, but native formats cannot provide metadata absent from their files

## Popups

The `popups` tab edits popup geometry, the managed agent command, and managed
popup role keys:

| Field | Default | View | Meaning |
| --- | --- | --- | --- |
| `agent.command` | `auto` | Overview | Managed agent popup command. `auto` keeps the built-in provider fallback |
| `agent.args` | `[]` | All | Arguments for a custom `agent.command` |
| `popup.side_margin` | `1` | All | Ordinary left and right popup margin in terminal cells |
| `popup.vertical_margin` | `0` | All | Top and bottom popup margin in terminal cells |
| `keybindings.config` | `Alt Shift K` | Overview | Config popup trigger |
| `keybindings.agent` | `Alt Shift L` | Overview | Agent popup trigger |
| `keybindings.git` | `Alt Shift J` | Overview | Git popup trigger |
| `keybindings.menu` | `Alt Shift M` | Overview | Menu popup trigger |
| `keybindings.screen` | `Alt Shift A` | Overview | Random visual popup trigger |

When the sidebar is open, managed popups reserve its framed 32-column rail on
the left while retaining `popup.side_margin` on the right. They resize in place
as the sidebar toggles. When it is collapsed, `popup.side_margin` applies
equally to both horizontal edges.

`Alt Shift Y` is the fixed packaged key for the full managed Yazi popup. It is
not a root setting. The popup opens at the active tab's canonical workspace
root and hides on toggle. Ordinary toggles preserve the same live Yazi process
and its navigation even if the tab root later changes. It uses the layered Yazi
configuration and editor opener. `yzx reveal` replaces the popup process and
starts Yazi at the requested target.

`agent.command` accepts one executable name or path, not a shell command with
arguments. Keep `agent.command = "auto"` to use the built-in `codex resume`,
`grok`, `opencode`, `pi`, `claude --resume` fallback chain

All managed action keys accept either a key chord or `false`, share syntax
validation, and use case-insensitive collision checks. An explicit `false`
leaves that action unmapped and consumes no chord; it removes only the shortcut,
not any existing command, menu entry, or popup behavior. Omitting or resetting
the field restores its packaged default.

```toml
[keybindings]
screen = false
```

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
must be unique; `anima`, `screen`, `screen_popup`, `yazi`, and `yazi_popup` are reserved
for packaged surfaces. Custom popup keybindings use the same collision checks
as all managed action keys. Ratconfig passes every leaf actually present under a
configured popup through its generic TOML rows. Those values are explicit, so
they remain visible in Overview too. Optional fields that are not written and popup
ids that do not exist are not invented; open `config.toml` to add them

## Native config files

| File | Owner | Notes |
| --- | --- | --- |
| `rio/config.toml` | Rio + Nova appearance projection | Complete native configuration and referenced adaptive themes seeded once from the package. Ratconfig exposes one exact-file action and does not mirror Rio's schema. Nova reserves only top-level `force-theme` when the file is writable |
| `zellij/config.kdl` | Zellij sidecar | Sparse safe scalar overrides where absent assignments inherit packaged defaults. The Zellij tab keeps themes, pane frames, mouse mode, copy-on-select, and rounded corners in Overview; All adds scrollback size, clipboard target, styled underlines, and startup tips. Search covers the typed fields and the exact native-file action. Safe untyped leaves remain unchanged without a UI row or informational diagnostic per leaf. Advanced diagnostics report only ignored, invalid, structurally unsafe, or integration-owned state; guarded diagnostics name the Yazelix owner. Ratconfig does not claim a complete Zellij schema. A legacy static `theme` assignment remains preserved but omitted from managed runtime. Inside a session, saves and resets also patch the active runtime config. Structural comments or continuations, extra managed-block metadata, other structured native nodes, and integration-owned nodes block unsafe writes |
| `zellij/plugins.kdl` | Zellij plugin sidecar | Extra plugin declarations only. Packaged plugin ids cannot be redeclared |
| `starship.toml` | Starship | Sparse native prompt overrides. Ratconfig consumes the generated schema and default output from packaged Starship 1.26.0. Overview recommends `format`, `right_format`, `add_newline`, and `character.format`; All exposes 832 finite owner fields. Schema-backed strings and booleans are editable. Numeric, structured, union, and dynamic values remain read-only with this exact file action |
| `helix/config.toml` | Helix | Sparse user TOML merged over packaged Nova Helix defaults. Ratconfig renders all packaged and explicit leaves as read-only native rows, recommends eight common or integration-owned values, and opens this exact file for edits. It does not claim a complete Helix schema |
| `helix/languages.toml` | Helix | Dynamic language config. Ratconfig renders entries actually present in the file as read-only rows with this exact file action; it does not invent a finite language registry |
| `helix/helix.scm` | Helix Steel | Loaded with `helix/init.scm` when the pair exists |
| `helix/init.scm` | Helix Steel | Loaded with `helix/helix.scm` when the pair exists |
| `nu/env.nu` | Nushell | Executable source loaded after packaged Yazelix `env.nu` |
| `nu/config.nu` | Nushell | Executable source loaded after packaged Yazelix `config.nu` and any successful host `mise activate nu` output, before the optional managed Atuin default |
| `yazi/yazi.toml` | Yazi | Native tables merge recursively, while user scalars and arrays replace packaged values. Ratconfig joins the pinned official schema to the packaged Yazi preset and the sparse user file. Overview recommends eight manager and preview controls; All exposes 95 finite or preset-observed base rows |
| `yazi/init.lua` | Yazi | Appended after packaged Yazi init |
| `yazi/keymap.toml` | Yazi | Appended after packaged Yazi keymap |
| `yazi/starship.toml` | Yazi Starship | Complete replacement for Nova's packaged compact Starship header config |
| `yazi/theme.toml` | Yazi | Native theme config. Ratconfig joins the paired official schema to Yazi's dark or light preset, exposes all 109 fields, provides separate installed flavor pools, and preserves explicit choices. Yazelix projects the active session side only into generated runtime config |
| `yazi/package.toml` | Yazi | Opaque package metadata that Yazelix does not process with `ya pkg` |

The Nu files remain executable Advanced actions, not finite Ratconfig schemas.
`shell.program` and `shell.atuin` stay in main, and Starship stays under its own
owner.

The packaged Helix fork identifies its version but publishes no stable
machine-readable catalog of configuration paths, defaults, constraints, or
safe writable shapes. Ratconfig therefore uses the packaged Yazelix default as
a baseline document and joins it to the sparse native user document. Overview
contains theme, auto-format, bufferline, cursorline, insert cursor shape, hidden
file visibility, soft wrap, and the reserved reveal binding, plus any explicit
value that needs attention. All and search contain the other packaged or
explicit rows. Ratconfig records explicit user values as intent without
claiming an effective value; Helix validates them only at launch. Invalid TOML
produces one source diagnostic and retains the packaged rows and exact repair
action.

Yazelix always resolves `keys.normal.A-r` to its reveal command, even when the
user document contains another value; Ratconfig shows both that explicit intent
and the integration-owned effective value. `helix.scm` and `init.scm` stay a
paired Steel source action rather than inferred settings.

The managed Yazi merge restores Yazelix's edit opener and its two managed Git
fetchers exactly once. Other user fetchers and previewers remain in the merged
native config. Invalid TOML, a broken input, or an incomplete flavor stops launch

Managed `plugins/*.yazi` and `flavors/*.yazi` directories are linked into the
runtime config even without `init.lua`. A user-managed `starship.yazi` replaces
the packaged Starship plugin as one complete directory and must contain
`main.lua`; plugin directories are never recursively merged. Nova still
initializes and refreshes Starship, so a replacement must preserve that plugin
API. Other packaged plugin names remain protected because they own managed
layout or navigation behavior. A user flavor with a packaged name
takes precedence, so `ya` can own an explicitly installed version. Create the
directories directly under the managed Yazi tree or symlink them there

Managed files and asset directories may be symlinked from another checkout, but
their resolved targets must stay outside the generated `state/yazi` runtime

The optional `yazi/starship.toml` file replaces the packaged compact Starship
config without replacing `starship.yazi` itself. It is syntax-validated as TOML
and projected to the existing managed Yazi runtime path.
Home Manager exposes it through the same native `text`/`source` contract:

```nix
let
  prompt = ./starship.toml;
in {
  programs.yazelix.config = {
    starship.source = prompt;
    yazi.starship.source = prompt;
  };
}
```

Use another source for a dedicated compact header. Omitting `yazi.starship`
keeps Nova's packaged header even when the managed shell uses `starship`.

Ratconfig's Yazi tab reads the sparse user `yazi.toml` against Nova's packaged
layer and reads native `theme.toml` against the dark or light Yazi preset
selected by the active session appearance. The version-paired official schemas add
known settings absent from both documents, so All exposes 205 base settings and
search spans the complete finite catalog. Overview recommends manager layout,
sorting, line mode, visibility, preview wrapping, and the two flavor choices;
explicit or invalid advanced settings remain visible there too

Schema booleans, finite string choices, and unconstrained strings with a safe
sparse TOML path are editable. Numeric, structured, reference-defined, dynamic,
and inline-table-blocked values remain read-only with the exact native-file
action. Existing dynamic opener entries remain visible without pretending
arbitrary opener names form a finite schema. `keymap.toml`, `package.toml`, and
`init.lua` remain honest file actions rather than synthetic scalar inventories.
A setting added through a file action appears after the editor closes. Saved
native values apply on the next managed Yazi launch

### Yazi flavors

[Yazi Bistro](https://github.com/Yazelix/yazi-bistro) supplies 22 complete,
pinned flavors with provenance and license metadata: 17 dark and 5 light.
Press `8` in Ratconfig to choose from the corresponding packaged pool.
User-installed flavors without a Bistro classification appear in both pools.
Ratconfig writes only the selected native `theme.toml` key.

The active session appearance selects which side a new managed Yazi uses. An explicit
`flavor.dark` or `flavor.light` wins for that mode. Ratconfig lists `default`
first in the dark pool; selecting it removes `flavor.dark` and uses Yazi's
native preset. Resetting the light field inherits Bluloco Light. At launch,
Yazelix writes both flavor keys in generated runtime `theme.toml` when an
explicit flavor is selected. It never changes the user or Home Manager source
file, never restarts an existing Yazi process, and preserves unrelated native
theme settings.

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
user-installed flavors appear in both Ratconfig pools automatically

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

Opening `yzx config` seeds `rio/config.toml` once, but does not create
`starship.toml` or `zellij/config.kdl`. Saving writes only the selected override,
and resetting removes that key. New managed Yazi
processes read the saved root mode and project its selected flavor into runtime
state without editing the native theme file. The Starship tab gets its complete
All inventory and native baselines from the exact packaged Starship owner
artifacts. Its four Overview recommendations cover the global prompt layouts
and Nova's `character.format = ":: "` marker. Managed Nu materializes sparse
overrides under runtime state without setting top-level `format`, so Starship
retains its native `$all` layout. Zellij layers its sparse file over packaged
configuration directly. Untouched defaults follow upgrades

Rio owns the full schema and validation for `rio/config.toml`. When that path
does not already exist, Yazelix copies the packaged config and any missing
`nova-dark`/`nova-light` themes; an existing theme file is never replaced.
Upgrades leave those user-owned files alone. New configs enable `window.blur`
with `window.opacity = 0.88`; blur depends on compositor support.
Ratconfig's Rio tab exposes eight native controls: blur, opacity, font family and
size, line height, cursor trail, audio bell, and quit confirmation. Rio's versioned
`--config-editor` inventory and native parser own the field defaults and validation
(RIO-CONFIG-UI-001). Candidate TOML is checked before writing; theme and font
availability remain Rio launch checks. Comments and unrelated values survive edits.
Reset removes a key and restores Rio's native default, which can differ from
Nova's initial seed; for example Rio's native blur default is off. Platform-specific
window overrides take precedence over the global blur and opacity controls.
Read-only and Home Manager files retain their existing ownership protections.
Controls indicate next-Rio-launch application; some values also reload live.
The complete native-file action remains available for other settings and repairs.
Rio-free packages expose neither the controls nor a Rio dependency. Legacy
`mars/config.toml` and `cursors.toml` files remain byte-for-byte untouched and
are ignored by Nova

Yazi's compact Starship header mirrors the default contextual module coverage.
Directory and Git retain compact text; every other decoration renders only its
symbol, so values such as cloud profiles and regions stay out of the header

Ratconfig's Zellij Dark theme and Light theme pickers list the identities
declared by the pinned Zellij package rather than maintaining theme definitions.
They inherit `ansi` and `gruvbox-light` respectively, and resetting either
removes only that sparse override. Custom names written to either field remain
valid and join the shared picker pool alongside the packaged set. The old
static `theme` field is no longer a managed setting. Yazelix preserves an
existing assignment in the user sidecar, reports that it is ignored, and leaves
it out of materialized runtime configuration. Root `appearance.mode` selects a
member of the pair when Yazelix starts Zellij. A live-capable save from inside
a managed session calls Zellij's native action for that session, and the top
bar follows the resulting mode event with its internal dark or light palette.

When the current managed session has a writable Rio config, saving root
`appearance.mode` switches the config UI and calls the matching native Zellij
action. Nova also updates only Rio's top-level `force-theme`; Rio's native
config watcher reloads that setting, the top bar follows Zellij's mode event,
Ratconfig repaints, and new Yazi opens use the selected side. A failure rolls
the coordinated save back instead of leaving a mixed appearance. Every other
Rio setting remains native and user-owned. A custom complete Rio file must
supply its own adaptive pair.

With a read-only Rio config, including a store-backed Home Manager source, Nova
keeps the session's captured appearance across Ratconfig, Rio, Zellij, the bar,
and new Yazi opens. The saved root mode applies to all of them together in the
next session; Rio receives the launch-time override because `force-theme`
cannot be projected. Zellij sidecar saves and resets still update the active
managed session when `yzx config` runs inside it. Pane frames, rounded corners,
copy-on-select, and clipboard target apply via the Zellij watcher; mouse mode,
scrollback size, styled underlines, and startup tips need a new session.

## Editor and file opens

Managed Yazi opens files through `yzx-open`. With the default
`editor.command = "yzx-hx"`, `yzx-open` reuses a live Helix bridge in the same
Zellij tab or opens packaged Helix in the managed `editor` pane when the
selected package includes it. The no-Helix package requires another terminal
editor command. In a managed-Helix package, typing `hx` invokes the same wrapper

Git editing stays in the client terminal. Managed LazyGit overlays only its
file-edit commands and keeps user configuration, while it and other terminal
Git clients use `yzx-editor` through the standard editor variables. On return,
the bridge restores the client's transparent Zellij background

`Alt r` starts the active tab's persistent Yazi popup at the current Helix
buffer. `yzx reveal <target>` exposes the same behavior inside a managed session
without changing the tab's canonical workspace. The target may be absolute,
relative to the command's current directory, or use an exact leading `~` or
`~/` resolved from `HOME`. Editor-to-Yazi reveal replaces an existing popup
process; ordinary popup toggles preserve its live navigation state.

In the managed Yazi popup, `Alt r` hides the popup, preserves its navigation
state, and returns to the underlying pane without opening the hovered item.
`Enter` remains the explicit file-open action. Helix and Yazi bind `Alt r`
locally, and Zellij does not replay it across the focus change.

Yazelix does not modify external editor configuration. Neovim users can opt
into the same `Alt r` behavior in their own config:

```lua
vim.keymap.set("n", "<M-r>", function()
  local path = vim.api.nvim_buf_get_name(0)
  if path ~= "" then vim.fn.jobstart({ "yzx", "reveal", path }) end
end, { desc = "Reveal buffer in Yazelix Yazi popup" })
```

Terminal Emacs users can bind the same command:

```elisp
(defun yazelix-reveal-buffer ()
  "Reveal the current buffer in the Yazelix Yazi popup."
  (interactive)
  (if buffer-file-name
      (start-process "yzx-reveal" nil "yzx" "reveal" buffer-file-name)
    (user-error "Current buffer does not visit a file")))

(global-set-key (kbd "M-r") #'yazelix-reveal-buffer)
```

These bindings expect the editor process to inherit the managed Yazelix
session environment. The Emacs example replaces the default `M-r` binding.
Choose another key to retain `move-to-window-line-top-bottom`.

`Alt z` opens a zoxide picker in Yazi, moves to the selected directory, and
explicitly retargets the tab workspace and managed editor through `yzx-open`.
Ordinary Yazi opens keep the existing tab workspace.

`yzx-open` writes bounded logs under:

```text
${YAZELIX_STATE_DIR}/logs/yzx-open.log
```

Implementation-level config layering and sidecar contracts live in
[Runtime Notes](runtime-notes.md)
