# Yazelix Next

Small start: a Nix flake that installs `yzn`, a conflict-free dev command that
opens Mars with a Yazi-first layout that becomes a sidebar plus stacked work
panes, a bridge-enabled Yazelix Helix editor, reef cursor colors, and the
Yazelix Zellij fork. The top bar uses a rendered Yazelix Zellij Bar tray with
configurable widgets, a `YZN` runtime marker, and bundled `tu` for usage
widgets. `Alt Shift J/K/L/M` default to managed Git, config, agent, and menu
popups and can be remapped with semantic `keybindings.*` role fields. The Git
popup defaults to LazyGit.
Users can add managed custom popups with their own semantic keybinding under
`[popups.<id>]`. Launches show a brief configurable welcome splash, and
`yzn screen` can run the same terminal screen styles directly.

## Run

```sh
nix run
nix run .#yzn
nix run .#yzn -- help
nix run .#yzn -- config
nix run .#yzn -- doctor
nix run .#yzn -- env
nix run .#yzn -- enter
nix run .#yzn -- launch
nix run .#yzn -- menu
nix run .#yzn -- tutor
nix run .#yzn -- screen
nix run .#yzn -- screen static
nix run .#yzn -- status
nix run .#yzn -- sponsor
```

`yzn help` prints help, `yzn config` opens the Ratconfig UI, `yzn
doctor` checks owned runtime setup without launching Mars or Zellij, `yzn
env` opens the configured managed shell without launching the UI, `yzn enter`
starts the managed Zellij runtime inside the current terminal, `yzn launch`
opens Mars first, `yzn menu` opens a live-filter command palette for `config`,
`doctor`, `status`, `screen`, `sponsor`, `launch`, `help`, and `tutor`, and `yzn tutor`
prints guided lessons for the workspace model, discovery, recovery, and native
tool tutors. `yzn screen [style]` shows a Yazelix terminal screen until a key is
pressed; styles are `static`, `logo`, `boids`, `boids_predator`,
`boids_schools`, `mandelbrot`, `game_of_life_gliders`,
`game_of_life_oscillators`, `game_of_life_bloom`, and `random`. `yzn status`
prints a compact runtime/config summary, including editor command, welcome
settings, popup margins, popup keybindings, and selected bar widgets, without
launching Mars or Zellij. `yzn sponsor` opens the GitHub Sponsors page when a host opener is
available, otherwise it prints the URL. Bare `yzn` defaults to `yzn launch`. If
`doctor`, `env`, `enter`, `launch`, or `status` fails before handing control to
a child, `yzn` prints a concise startup diagnostic with the reason and, when
applicable, the config path to check.

## Tutor

`yzn tutor begin` starts the guided Yazelix path. `yzn tutor list` shows the
available lessons: `workspace`, `discovery`, `troubleshooting`, and
`tool_tutors`.

`yzn tutor hx` and `yzn tutor helix` print the packaged Helix tutor command
instead of launching it. `yzn tutor nu` and `yzn tutor nushell` print the
Nushell tutor commands, including the form to run from an existing Nu prompt.

## Install

```sh
nix profile add --refresh /absolute/path/to/yazelix-next
yzn
```

Profile installs include `bin/yzn` and a Linux-style `Yazelix Next` desktop
entry.

Package and app outputs are exposed for `x86_64-linux`, `aarch64-linux`,
`x86_64-darwin`, and `aarch64-darwin`. The macOS floor is package-first:
`yzn help`, `status`, `doctor`, and `enter` are expected to work from a normal
terminal after install. `yzn launch` uses Mars and remains issue-driven
best-effort until macOS hardware validation confirms the full launch path.
Yazelix Next does not ship a macOS app bundle, Ghostty package, Homebrew tap, or
terminal matrix.

Home Manager users can import the narrow module:

```nix
{ inputs, ... }: {
  imports = [ inputs.yazelix-next.homeManagerModules.default ];
  programs.yazelix.enable = true;
}
```

The optional `programs.yazelix.package` setting overrides the installed
package for local or alternate builds. The module writes no runtime config files
by default.

Home Manager can also own selected Yazelix config files:

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
};
```

`settings` renders `~/.config/yazelix-next/config.toml` with Yazelix defaults
and Ratconfig contract state. Native config files are simple `text` or `source`
passthroughs for Mars, Zellij, Starship, Helix, Yazi, and Nu. Store-backed files
are read-only from `yzn config`; edit them in Home Manager.

## Update

```sh
nix profile upgrade --refresh yazelix-next
```

## Config

`yzn config` opens the Ratconfig UI in the current terminal and creates the
owned config sources when they are missing:

```text
~/.config/yazelix-next/config.toml
~/.config/yazelix-next/mars/config.toml
~/.config/yazelix-next/zellij/config.kdl
~/.config/yazelix-next/starship.toml
```

The `config` tab controls `open.log_level`, which sets the managed
`YZN_OPEN_LOG` level used by managed Yazi open requests. Values are `off`, `error`,
`info`, and `debug`. It also controls `shell.program`, which selects the
packaged shell for new Zellij panes. Values are `nu`, `bash`, `zsh`, and
`fish`. The `editor.command` setting controls managed Yazi opens, Ratconfig
external text edits, Git client editor flows, and the managed session
`EDITOR`/`VISUAL`/`YZN_EDITOR`/`GIT_EDITOR` environment. The default `yzn-hx`
uses packaged Yazelix Helix. Host commands such as `hx` or `nvim` run from
`PATH` without packaged config, plugins, bridge reuse, or reveal parity. The
value is one executable name or path, not a shell command with arguments. The
same tab controls `[welcome].enabled`, `[welcome].style`, and
`[welcome].duration_seconds`, which apply to the startup splash before managed
Zellij starts. Welcome styles are the same fixed `yzn screen` styles; `random`
chooses from the animated styles, excluding `static` and `logo`, and there is
no configurable pool. The same tab edits `[popup].side_margin` and
`[popup].vertical_margin`, the left/right and
top/bottom cell margins for managed popups. The defaults are side `1` and
vertical `0`; higher values inset the popup by exact terminal cells. The
runtime writes these once as `yzpp` popup defaults, and packaged and custom
popup specs inherit them.
The same tab edits `[keybindings].config`, `[keybindings].agent`,
`[keybindings].git`, and `[keybindings].menu`, the semantic key chords for
managed popup roles. They default to `Alt Shift K`, `Alt Shift L`,
`Alt Shift J`, and `Alt Shift M`, remap only those popup role actions, and
reject invalid, duplicate, or conflicting packaged chords before launch.
Custom popups live directly in `config.toml` under `[popups.<id>]`. Each custom
popup has a required `command` and `keybinding`, optional `args`, optional
`title`, and optional `keep_alive = true` to hide/show the pane instead of
closing it. Commands are argv-based; put arguments in `args`, not in `command`.
Titles default to `<id>_popup`, identify existing popup panes, and must be
unique without reusing packaged popup titles. Custom popup keybindings use the
same syntax and collision checks as managed popup role keybindings, and custom
popups inherit `[popup].side_margin` and `[popup].vertical_margin`.
The same tab edits `[bar].widgets`, whose default tray is `editor`, `shell`, `term`,
`codex_usage`, `cpu`, and `ram`; allowed opt-ins are `session`,
`claude_usage`, and `opencode_go_usage`. The shell widget uses the configured
`shell.program` label in compact form, such as `❯nu` or `❯fish`. The `mars`
tab edits the native Mars config for the dark/light appearance preset, window
size, opacity, font, scrollbar, bell, and cursor trail. Low-level Mars
`force-theme`, `[colors]`, and cursor TOML can still live in
`mars/config.toml`, but the config popup does not render those manual fields.
The `zellij` tab edits a
guarded native sidecar that applies to new launches. The `starship` tab edits
real Starship prompt fields in
`starship.toml`: `format`, `right_format`, and `add_newline`. The default left
prompt is colon-colon-space (`:: `). The
`helix` tab opens managed Helix native files in the managed editor: `helix/config.toml`,
`helix/languages.toml`, `helix/helix.scm`, and `helix/init.scm`. These files
are created only when their row is activated, except that either Steel row
creates the `helix.scm`/`init.scm` pair the fork expects. The `keys` tab lists current
packaged bindings in read-only group, key, action, and owner columns, with
source paths in details. The `advanced` tab opens `nu/env.nu`, `nu/config.nu`,
`yazi/init.lua`, `yazi/keymap.toml`, and `zellij/plugins.kdl` in the managed
editor. Advanced files are created only when their row is activated.
While editing a text field, `Ctrl+e` opens the staged value in the config UI's
editor environment and returns the edited text to the row; `Enter` still saves.

Generated runtime state for Zellij, Yazi, Nu, and the Helix bridge defaults to
`${XDG_DATA_HOME:-$HOME/.local/share}/yazelix-next`; set `YAZELIX_STATE_DIR`
to override.

The agent popup bootstraps once. If no provider has been selected yet, it tries
`codex resume`, `grok`, `opencode`, `pi`, then `claude --resume` from `PATH`,
stores the first available provider under `YAZELIX_STATE_DIR`, and launches
that provider on later opens without cascading again. If no provider is
available on first run, the popup exits without selecting a default.

## Welcome Screen

`yzn enter` and `yzn launch` run a bounded welcome splash before Zellij starts.
The default is enabled, random, and 3 seconds:

```toml
[welcome]
enabled = true
style = "random"
duration_seconds = 3
```

Set `enabled = false` to skip the splash, or set `style` to one fixed screen
style such as `static`, `logo`, or `mandelbrot`. The static and logo styles
remain explicit choices for the card-like welcome screens.

## Shell Config

`config.toml` defaults to `shell.program = "nu"`. New Zellij panes and `yzn
env` start a packaged shell dispatcher that reads this value and execs the
matching packaged `nu`, `bash`, `zsh`, or `fish`. The managed shell PATH also
includes packaged `hx`, `lazygit`, and `git`. The selection applies to new
panes, sessions, and non-UI shell entry. Bash, Zsh, and Fish are packaged
binaries with their normal
interactive startup behavior; Yazelix Next only manages extra shell config for
Nu.

## Mars Config

`yzn` uses the packaged Mars config unless this managed native Mars config
exists:

```text
~/.config/yazelix-next/mars/config.toml
```

`yzn config` creates it from the packaged generated Mars config and exposes
native terminal preferences such as `mars.appearance.preset`, window size,
opacity, font size, line height, scrollbar, bell, and cursor trail. Low-level
Mars fields such as `force-theme`, `[colors]`, and `yazelix.cursor` remain
possible in this native file, but the config popup hides those manual rows.
Saving `mars.appearance.preset` through `yzn config` switches Mars and the
config UI palette in the same session; direct file edits while the UI is open
are reflected the next time `yzn config` starts.
Mars appearance belongs to this Mars config; root `config.toml` does not
provide a global appearance mode. Set `YAZELIX_NEXT_CONFIG_HOME` to use a
different Yazelix Next config root. `yzn` still owns the Mars launch command
and the managed Zellij runtime.

## Zellij Config

`yzn` owns Zellij keybindings, layout, plugin/runtime spine, and the managed
default shell dispatcher. Safe native Zellij preferences live in this managed
sidecar:

```text
~/.config/yazelix-next/zellij/config.kdl
```

`yzn config` edits scalar preferences such as pane frames, mouse mode,
scrollback size, copy behavior, styled underlines, startup tips, and
`ui.pane_frames.rounded_corners`. The sidecar is a simple guardrail, not a KDL
merge engine. It is rejected before launch and blocked inside the config UI
when an uncommented line starts with integration-critical ownership such as
`keybinds`, `default_shell`, `default_layout`, `layout`, `plugins`,
`load_plugins`, `support_kitty_keyboard_protocol`, `env`, `session_name`, or
`attach_to_session`.

The packaged config enables Zellij's Kitty keyboard protocol for modified key
chords such as `Alt Shift J/K/L/M`.

Extra Zellij plugins can be declared in a separate managed sidecar:

```text
~/.config/yazelix-next/zellij/plugins.kdl
```

It accepts only `plugins` and `load_plugins` blocks:

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

## Nushell Config

When `shell.program` is `nu`, `yzn` does not read normal Nushell config. It
loads packaged Yazelix Next `nu/env.nu` and `nu/config.nu` first, then optional
user files:

```text
~/.config/yazelix-next/nu/env.nu
~/.config/yazelix-next/nu/config.nu
```

The same `YAZELIX_NEXT_CONFIG_HOME` root applies here.

If host `mise` is available on the inherited `PATH`, managed Nu inserts
`mise activate nu` output after the packaged config and before user
`nu/config.nu`. Yazelix Next does not package or configure `mise`; missing or
failing `mise` is skipped.

## Starship Config

When `shell.program` is `nu`, `yzn-nu` sets `STARSHIP_CONFIG` to this native
Starship config when it exists:

```text
~/.config/yazelix-next/starship.toml
```

Otherwise it uses an empty config, so normal `~/.config/starship.toml` does not
affect the managed Nu prompt. `yzn config` creates and edits this file through
the `starship` tab. `format` defaults to colon-colon-space (`:: `) and
controls the left prompt, `right_format` controls the right prompt, and user
`nu/config.nu` can still override prompt variables for advanced cases.

## Helix Config

`yzn-hx` builds an effective Helix config on each launch from the packaged
Yazelix Next default plus the optional managed user override, so normal
`~/.config/helix` does not affect the managed editor. If `config.toml`,
`languages.toml`, or the Steel file pair exists under this directory, `yzn-hx`
uses the directory as the managed Helix config dir for native Helix lookup:

```text
~/.config/yazelix-next/helix/
```

`helix/config.toml` is a user override fragment merged over the packaged TOML
default when present. The generated effective file is written under
`YAZELIX_STATE_DIR/helix/config.toml`; users should edit the managed source file
through the Helix tab instead. `Alt r` is reserved for Yazelix reveal, so a user
`A-r` override is ignored in the generated config and reported by `yzn doctor`.
`helix/languages.toml` is loaded by the managed Helix config dir when present.
`helix/helix.scm` and `helix/init.scm` are Steel files loaded through
`HELIX_STEEL_CONFIG` from the same managed Helix directory once both files
exist. By default, managed Helix exposes `:yzn-new-shell`, which opens a new
Yazelix terminal pane at the current file directory or workspace. When TOML-only
managed config is active, `yzn-hx` points Steel at an internal state directory
instead so the fork does not create empty Steel files under user config.
Activating either Steel row creates the pair. If only `languages.toml` or the
Steel pair exists, the generated TOML config is still based on the packaged
default.
The packaged config binds `Alt r` to reveal the current buffer in Yazi and
`Ctrl r` to reload Helix config and the current buffer. Other packaged Helix
preferences are normal defaults and can be overridden by the user fragment.

## Editor Opens

Yazi opens files through the packaged `yzn-open` Rust helper. With the default
`editor.command = "yzn-hx"`, `yzn-open` reuses a live Helix bridge in the same
Zellij tab or opens packaged `yzn-hx` in a managed `editor` pane. Host commands
such as `hx` or `nvim` skip the bridge and open in that pane. Missing editor
commands fail before opening a pane with a direct `editor command not found`
diagnostic. The managed Git popup defaults to LazyGit and exports the configured
editor to Git editor environment variables before launching the client. It
closes on toggle so the next open follows the current tab cwd after workspace
retargeting.
Managed Yazi uses scoped Kitty graphics environment for image previews while
preserving the real Zellij session for editor routing.

Inside the packaged Yazi sidebar, `Alt z` opens a zoxide picker, moves Yazi to
the selected directory, sends it through `yzn-open`, and renames the tab to the
workspace root. In Git repositories, Helix keeps the selected picker directory
while editor cwd and tab name use the repo root.

`Alt r` reveals the current Helix buffer in the managed Yazi sidebar. The same
path is available as `yzn reveal <target>` inside a managed session.

Managed Yazi appends optional user Lua from
`~/.config/yazelix-next/yazi/init.lua` and optional user keymap TOML from
`~/.config/yazelix-next/yazi/keymap.toml` after the packaged setup and keymap.
This does not merge `yazi.toml`, themes, or normal `~/.config/yazi`. When the
managed init file exists, plugin directories at
`~/.config/yazelix-next/yazi/plugins/*.yazi` are symlinked into the runtime
config; packaged plugin names cannot be overridden. The config UI's `advanced`
tab can create or open the user init and keymap files.

Example managed Yazi plugin layout:

```text
~/.config/yazelix-next/yazi/plugins/foo.yazi/main.lua
~/.config/yazelix-next/yazi/init.lua
```

```lua
require("foo"):setup()
```

Managed Yazi refreshes sidebar git decorations on setup, directory changes, tab
changes, and managed popup close/hide hooks.

`yzn-open` writes bounded diagnostics to
`${YAZELIX_STATE_DIR}/logs/yzn-open.log` and keeps one rotated
`yzn-open.log.1` file. Managed `yzn` sessions set `YZN_OPEN_LOG` from
`open.log_level` in `config.toml`; the default is `info`.

## Keybindings

`Ctrl p/t/n/q` are the high-frequency Zellij controls. The rest of the native
Zellij layer uses `Ctrl Alt`, leaving most plain `Ctrl` keys available to
Helix, Nushell, Yazi, and terminal programs. The Ratconfig Keys tab is the
human-facing packaged key reference; `config.kdl` remains the runtime binding
source, and flake checks keep the reference backed by packaged bindings.

| Key | Action |
| --- | --- |
| `Ctrl Alt g/s/o` | lock, search, session |
| `Ctrl p/t/n/q` | pane, tab, resize, quit |
| `Ctrl Alt h/j/k/l` | move tab left, move pane down/up, move tab right |
| `Alt 1-9` | go directly to tab 1-9 |
| `Alt h/l` | move focus left/right across visible panes or previous/next tab |
| `Alt m` | new pane in the stacked layout |
| `Alt r` | reveal the current editor file in Yazi |
| `Alt z` | Yazi zoxide jump into the managed editor |
| `Alt Shift J` | toggle the Git popup |
| `Alt Shift K` | toggle the config popup |
| `Alt Shift L` | hide/show the agent popup by default |
| `Alt Shift M` | toggle the menu popup |
| `Alt Shift h` | toggle the managed Yazi sidebar |

Move mode is intentionally unbound. The `keybindings.config`,
`keybindings.agent`, `keybindings.git`, and `keybindings.menu` fields can
move managed popup triggers to valid non-conflicting chords; raw Zellij
`keybinds` remain outside the managed sidecar. Custom `[popups.<id>]` entries
use their own required `keybinding` field and the same collision checks.

## Hack On Mars

```sh
nix run --override-input mars ../mars
nix run --override-input yazelixZellij ../yazelix-zellij
nix run --override-input yazelixHelix ../yazelix-helix
nix run --override-input yazelixZellijPopup ../yazelix-zellij-popup
nix run --override-input yazelixZellijBar ../yazelix-zellij-bar
nix run --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator
```

## LOC Scorecard

Counts owned project files by language with `wc -l`.

```sh
wc -l .gitignore AGENTS.md README.md CHANGELOG.md ARCHITECTURE.md flake.nix home-manager/module.nix packaging/tokenusage.nix packaging/bar-render-request.nix shell/sh/yzn-env-supervisor.sh shell/sh/yzn-helix.sh shell/sh/yzn-shell.sh .github/workflows/ci.yml .github/workflows/version_gate.yml .github/workflows/publish_nix_cache.yml config.toml mars.toml config.kdl layout.kdl layout.swap.kdl nu/config.nu nu/env.nu helix/config.toml yazi/init.lua yazi/keymap.toml yazi/plugins/sidebar-state.yazi/main.lua yazi/plugins/sidebar-status.yazi/main.lua yazi/plugins/zoxide-editor.yazi/main.lua yazi/yazi.toml crates/yzn-config/Cargo.toml crates/yzn-config/src/*.rs crates/yzn-open/Cargo.toml crates/yzn-open/src/bin/yzn-reveal.rs crates/yzn-open/src/bin/yzn-sidebar-refresh.rs crates/yzn-open/src/lib.rs crates/yzn-open/src/main.rs crates/yzn-open/src/sidebar.rs crates/yzn-tutor/Cargo.toml crates/yzn-tutor/src/cli_render.rs crates/yzn-tutor/src/main.rs crates/yzn-tutor/src/tutor_document.rs checks/*.rs runtime/yzn-agent.rs runtime/yzn-menu.rs runtime/yzn-nu.rs runtime/yzn-yazi.rs runtime/yzn/*.rs runtime/yzn-zellij-config.rs
```

| Language | Files | Lines |
| --- | --- | ---: |
| Ignore | `.gitignore` | 4 |
| Markdown | `AGENTS.md`, `README.md`, `CHANGELOG.md`, `ARCHITECTURE.md` | 1262 |
| Nix | `flake.nix`, `home-manager/module.nix`, `packaging/tokenusage.nix`, `packaging/bar-render-request.nix` | 907 |
| Shell | `shell/sh/yzn-env-supervisor.sh`, `shell/sh/yzn-helix.sh`, `shell/sh/yzn-shell.sh` | 82 |
| YAML | `.github/workflows/ci.yml`, `.github/workflows/version_gate.yml`, `.github/workflows/publish_nix_cache.yml` | 171 |
| TOML | `config.toml`, `mars.toml`, `helix/config.toml`, `yazi/yazi.toml`, `yazi/keymap.toml`, `crates/yzn-config/Cargo.toml`, `crates/yzn-open/Cargo.toml`, `crates/yzn-tutor/Cargo.toml` | 236 |
| KDL | `config.kdl`, `layout.kdl`, `layout.swap.kdl` | 210 |
| Nu | `nu/config.nu`, `nu/env.nu` | 11 |
| Lua | `yazi/init.lua`, `yazi/plugins/sidebar-state.yazi/main.lua`, `yazi/plugins/sidebar-status.yazi/main.lua`, `yazi/plugins/zoxide-editor.yazi/main.lua` | 247 |
| Rust | `crates/yzn-config/src/*.rs`, `crates/yzn-open/src/bin/yzn-reveal.rs`, `crates/yzn-open/src/bin/yzn-sidebar-refresh.rs`, `crates/yzn-open/src/lib.rs`, `crates/yzn-open/src/main.rs`, `crates/yzn-open/src/sidebar.rs`, `crates/yzn-tutor/src/cli_render.rs`, `crates/yzn-tutor/src/main.rs`, `crates/yzn-tutor/src/tutor_document.rs`, `checks/*.rs`, `runtime/yzn-agent.rs`, `runtime/yzn-menu.rs`, `runtime/yzn-nu.rs`, `runtime/yzn-yazi.rs`, `runtime/yzn/*.rs`, `runtime/yzn-zellij-config.rs` | 11173 |
| Total | owned project files | 14303 |
