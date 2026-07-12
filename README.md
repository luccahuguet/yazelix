# Yazelix Nova — Beta

Yazelix Nova packages a small Yazelix runtime as a Nix flake. During
pre-swap development, the installed command is `yzn` so it can coexist with
public Yazelix v17.

`yzn` opens Mars, starts the Yazelix Zellij fork, and gives you a Yazi-first
workspace with a managed Helix editor bridge, popups for Git/config/agents/menu,
and a compact top bar. The repo keeps the runtime intentionally narrow: one
launcher, one config root, one packaged layout, and focused checks for the
contracts it owns.

## Run

```sh
nix run github:luccahuguet/yazelix-next
```

From a checkout:

```sh
nix run
nix run .#yzn -- help
nix run .#yzn -- config
nix run .#yzn -- doctor
```

Bare `yzn` runs `yzn launch`.

## Commands

| Command | Purpose |
| --- | --- |
| `yzn` | Open Mars and start the managed Yazelix session. |
| `yzn launch [zellij-args...]` | Open Mars first, then start managed Zellij. |
| `yzn enter [zellij-args...]` | Start managed Zellij in the current terminal. |
| `yzn config` | Open the Ratconfig-backed config UI. |
| `yzn menu` | Open the command palette. |
| `yzn doctor` | Check owned runtime setup without launching Mars or Zellij. |
| `yzn status` | Print config/runtime paths and selected settings. |
| `yzn env` | Open the managed shell without launching the UI. |
| `yzn tutor [lesson]` | Print guided Yazelix lessons. |
| `yzn screen [style]` | Show a terminal welcome screen. |
| `yzn reveal <target>` | Reveal a file or directory in the managed Yazi sidebar. |
| `yzn sponsor` | Open the sponsor page, or print its URL. |
| `yzn help` | Print command help. |

Screen styles are `static`, `logo`, `boids`, `boids_predator`,
`boids_schools`, `mandelbrot`, `game_of_life_gliders`,
`game_of_life_oscillators`, `game_of_life_bloom`, and `random`.

Tutor lessons are `workspace`, `discovery`, `troubleshooting`, and
`tool_tutors`. `yzn tutor hx` and `yzn tutor nu` print the native tool tutor
commands.

## Install

```sh
nix profile add --refresh github:luccahuguet/yazelix-next
yzn
```

Local checkout:

```sh
nix profile add --refresh /absolute/path/to/yazelix-next
yzn
```

Update a profile install:

```sh
nix profile upgrade --refresh yazelix-next
```

The package exposes `bin/yzn` and a Linux desktop entry. Package and app outputs
exist for `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, and
`aarch64-darwin`.

On macOS, `help`, `status`, `doctor`, and `enter` are the supported floor after
install. `launch` uses Mars and depends on macOS hardware validation for the full
GUI path.

## Home Manager

```nix
{ inputs, ... }: {
  imports = [ inputs.yazelix-next.homeManagerModules.default ];
  programs.yazelix.enable = true;
}
```

The optional `programs.yazelix.package` setting overrides the installed package.
The module writes no runtime config files unless you configure them.

Example:

```nix
programs.yazelix.config = {
  settings = {
    shell.program = "fish";
    editor.command = "nvim";
    welcome.enabled = false;
  };

  starship.text = ''
    format = ":: "
  '';

  helix.languages.source = ./languages.toml;
  yazi.config.source = ./yazi.toml;
};
```

`settings` renders only the declared values to
`~/.config/yazelix-next/config.toml`; undeclared values inherit packaged Nova
defaults. Native files are `text` or `source` passthroughs. Store-backed files
show as read-only in `yzn config`; edit them in Home Manager.

## Config Root

`yzn config` uses the managed config tree under:

```text
~/.config/yazelix-next/
```

Set `YAZELIX_NEXT_CONFIG_HOME` to use another root. Generated runtime state
defaults to:

```text
${XDG_DATA_HOME:-$HOME/.local/share}/yazelix-next
```

Set `YAZELIX_STATE_DIR` to use another state directory.

## Main Settings

The optional root config lives at `~/.config/yazelix-next/config.toml`. Opening
`yzn config` or starting Nova does not create it. The UI shows packaged defaults
for absent keys, saves only explicit overrides, and removes a key when reset.

| Field | Default | Meaning |
| --- | --- | --- |
| `open.log_level` | `info` | Diagnostics for managed Yazi open requests: `off`, `error`, `info`, `debug`. |
| `shell.program` | `nu` | Packaged shell for new panes: `nu`, `bash`, `zsh`, `fish`. |
| `editor.command` | `yzn-hx` | Editor used by Yazi opens, Ratconfig text edits, and Git editor flows. |
| `welcome.enabled` | `true` | Show the startup welcome splash. |
| `welcome.style` | `random` | Startup screen style. |
| `welcome.duration_seconds` | `3` | Startup splash duration, 1 to 60 seconds. |
| `bar.widgets` | `editor`, `shell`, `term`, `codex_usage`, `cpu`, `ram` | Top bar widgets, left to right. |

`editor.command` accepts one executable name or path, not a shell command with
arguments. `yzn-hx` uses packaged Yazelix Helix. Host editors such as `hx` or
`nvim` run from `PATH` and skip the managed Helix bridge. Terminal Git clients
receive the same selection through `EDITOR`, `VISUAL`, and `GIT_EDITOR`.

## Popups

The `popups` tab edits popup geometry, the managed agent command, and managed
popup role keys:

| Field | Default | Meaning |
| --- | --- | --- |
| `agent.command` | `auto` | Managed agent popup command. `auto` keeps the built-in provider fallback. |
| `agent.args` | `[]` | Arguments for a custom `agent.command`. |
| `popup.side_margin` | `1` | Left and right popup margin in terminal cells. |
| `popup.vertical_margin` | `0` | Top and bottom popup margin in terminal cells. |
| `keybindings.config` | `Alt Shift K` | Config popup trigger. |
| `keybindings.agent` | `Alt Shift L` | Agent popup trigger. |
| `keybindings.git` | `Alt Shift J` | Git popup trigger. |
| `keybindings.menu` | `Alt Shift M` | Menu popup trigger. |

`agent.command` accepts one executable name or path, not a shell command with
arguments. Keep `agent.command = "auto"` to use the built-in `codex resume`,
`grok`, `opencode`, `pi`, `claude --resume` fallback chain.

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
managed popup role keys.

## Native Config Files

| File | Owner | Notes |
| --- | --- | --- |
| `mars/config.toml` | Mars | Sparse overrides for appearance preset, window size, opacity, font, scrollbar, bell, and cursor trail. |
| `zellij/config.kdl` | Zellij sidecar | Safe scalar preferences. Inside a session, saves also patch the active runtime config (many live; some need a new session). Integration-owned nodes are blocked. |
| `zellij/plugins.kdl` | Zellij plugin sidecar | Extra plugin declarations only. Packaged plugin ids cannot be redeclared. |
| `starship.toml` | Starship | Sparse managed Nu prompt overrides; absent values inherit Nova defaults. |
| `helix/config.toml` | Helix | User TOML merged over packaged Yazelix Helix defaults. |
| `helix/languages.toml` | Helix | Managed Helix language config. |
| `helix/helix.scm` | Helix Steel | Loaded with `helix/init.scm` when the pair exists. |
| `helix/init.scm` | Helix Steel | Loaded with `helix/helix.scm` when the pair exists. |
| `nu/env.nu` | Nushell | Loaded after packaged Yazelix env. |
| `nu/config.nu` | Nushell | Loaded after packaged Yazelix config. |
| `yazi/yazi.toml` | Yazi | Native tables merge recursively; user scalars and arrays replace packaged values. |
| `yazi/init.lua` | Yazi | Appended after packaged Yazi init. |
| `yazi/keymap.toml` | Yazi | Appended after packaged Yazi keymap. |
| `yazi/theme.toml` | Yazi | Native theme config, including managed flavor selection. |
| `yazi/package.toml` | Yazi | Opaque package metadata; Yazelix does not run `ya pkg`. |

The managed Yazi merge restores Yazelix's edit opener and its two sidebar Git
fetchers exactly once. Other user fetchers and previewers remain in the merged
native config. Invalid TOML stops before Yazi starts.

Managed `plugins/*.yazi` and `flavors/*.yazi` directories are linked into the
runtime config even without `init.lua`. Packaged names cannot be replaced.
Create the directories directly under the managed Yazi tree or symlink them
there.

For Smart Enter, link `smart-enter.yazi` under `yazi/plugins/`, add
`require("smart-enter"):setup { open_multi = false }` to `yazi/init.lua`, and
bind it in `yazi/keymap.toml`:

```toml
[[mgr.prepend_keymap]]
on = "l"
run = "plugin smart-enter"
```

`l` then enters directories or opens the hovered file through the managed
opener.

Normal host config such as `~/.config/helix`, `~/.config/yazi`, and
`~/.config/starship.toml` does not control the managed runtime unless you route
through these Yazelix-owned files.

Opening `yzn config` does not create `mars/config.toml` or `starship.toml`.
Their tabs show effective Nova defaults, saving writes only the selected
override, and resetting removes that key. Mars merges its sparse file over the
packaged base directly; managed Nu materializes its effective Starship config
under runtime state. Untouched defaults follow upgrades.

Saving `mars.appearance.preset` through `yzn config` switches Mars and the config
UI palette in the same session. Other Mars values apply on the next Mars launch.
Zellij sidecar values update the active managed session config when `yzn config`
runs inside a session; many scalars apply live via Zellij's watcher, and some
still need a new session.

## Editor And File Opens

Managed Yazi opens files through `yzn-open`. With the default
`editor.command = "yzn-hx"`, `yzn-open` reuses a live Helix bridge in the same
Zellij tab or opens packaged Helix in the managed `editor` pane.

Git editing stays in the client terminal. Managed LazyGit overlays only its
file-edit commands and keeps user configuration; it and other terminal Git
clients use `yzn-editor` through the standard editor variables. On return, the
bridge restores the client's transparent Zellij background.

`Alt r` reveals the current Helix buffer in the Yazi sidebar. `yzn reveal
<target>` exposes the same path inside a managed session.

`Alt z` opens a zoxide picker in Yazi, moves to the selected directory, sends it
through `yzn-open`, and renames the tab to the workspace root.

`yzn-open` writes bounded logs under:

```text
${YAZELIX_STATE_DIR}/logs/yzn-open.log
```

## Keybindings

The Ratconfig Keys tab is the packaged key reference. `config.kdl` remains the
runtime source.

| Key | Action |
| --- | --- |
| `Ctrl Alt g` | Toggle locked mode. |
| `Ctrl Alt o` | Open session mode. |
| `Ctrl q` | Quit Yazelix session. |
| `Ctrl p` | Toggle pane mode. |
| `Ctrl t` | Toggle tab mode. |
| `Ctrl n` | Toggle resize mode. |
| `Alt m` | Open a new pane. |
| `Alt h` / `Alt Left` | Move focus left or previous tab. |
| `Alt l` / `Alt Right` | Move focus right or next tab. |
| `Alt 1-9` | Go directly to tab 1-9. |
| `Ctrl Alt h` | Move tab left. |
| `Ctrl Alt j` | Move pane down. |
| `Ctrl Alt k` | Move pane up. |
| `Ctrl Alt l` | Move tab right. |
| `Alt Shift J` | Toggle Git popup. |
| `Alt Shift K` | Toggle config popup. |
| `Alt Shift L` | Hide or show agent popup. |
| `Alt Shift M` | Toggle menu popup. |
| `Alt Shift h` | Toggle the Yazi sidebar. |
| `Alt r` | Reveal editor file in Yazi. |
| `Alt z` | Zoxide jump into the managed editor. |

Move mode is unbound. Managed popup triggers can be remapped through
`keybindings.config`, `keybindings.agent`, `keybindings.git`, and
`keybindings.menu`. Raw Zellij keymaps stay outside the managed sidecar.

## CI

Normal CI runs Linux checks on push, pull request, and manual dispatch.
`Publish Nix Cache` publishes the Linux package cache from `main` and manual
dispatch. `Version Gate` is manual and includes the Linux profile smoke plus the
macOS package smoke. `Darwin Package Smoke` builds
`.#packages.aarch64-darwin.yzn` weekly on Monday when `main` has commits in the
last 7 days, and on manual dispatch always; idle weeks skip the macOS build.
Use Version Gate before version bumps or the main Yazelix swap.

## Development

Use local sibling repositories while hacking runtime inputs:

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
nix run --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix run --override-input yazelixZellijBar ../yazelix-zellij-bar
nix run --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator
```

Useful local checks:

```sh
nix flake check
nix flake show --all-systems
nix build .#yzn --no-link --print-build-logs
nix build .#checks.x86_64-linux.yzn_yazi_materialization --no-link
```

Runtime package changes should also pass a temporary profile install:

```sh
nix profile add --refresh /absolute/path/to/yazelix-next --profile /tmp/yzn-profile
```

Detailed launch/config/editor shell notes live in
[docs/runtime-notes.md](docs/runtime-notes.md).

## LOC Scorecard

Counts **tracked** project files. Excludes Beads state (`.beads/`) and lockfiles
(`*.lock`). New owned sources count automatically once committed.

```sh
git ls-files | grep -Ev '^\.beads/|\.lock$' | xargs wc -l
```

| Language | Lines |
| --- | ---: |
| Ignore (`.gitignore`) | 4 |
| Markdown | 1367 |
| Nix | 998 |
| Shell | 84 |
| YAML | 268 |
| TOML | 249 |
| KDL | 210 |
| Nu | 11 |
| Lua | 247 |
| Rust | 12112 |
| Total | 15550 |
