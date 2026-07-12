# Changelog

User-visible runtime changes for Yazelix Nova live here.

## Unreleased

- Darwin Package Smoke and Version Gate build the full package, runtime package,
  and Home Manager closure on a real `aarch64-darwin` runner. They also assert
  that Darwin receives no Linux desktop entry; macOS interactive use and the
  Mars GUI remain explicitly unverified.
- The Codex quota widget classifies official limits by their reported window
  duration, keeps a real five-hour limit before the weekly limit, omits
  unavailable periods, and replaces stale quota slots after a successful probe.
  Updated windows use an isolated cache namespace, so older open sessions cannot
  restore incompatible quota slots.
- Installation documentation assigns updates to either the Nix profile or the
  owning Home Manager/nix-darwin configuration and explains that open sessions
  keep their current immutable runtime until relaunched.
- Ratconfig recognizes Home Manager-owned `cursors.toml` as declarative and
  refuses structured mutation with the exact module-option guidance. Empty
  config-root and `PATH` environment variables are treated as unset instead of
  resolving against the current directory. Invalid stale Helix bridge registry
  files are skipped so opening a file can fall back to a new editor pane.
- The flake exposes one fixed Mars-free `runtime` package and app alongside the
  complete default package. Both use `yzn` and the same config/runtime model;
  the runtime variant keeps `enter`, `run`, `env`, config, status, and doctor,
  while `launch` directs users to `enter` or the complete package. On the locked
  x86_64-linux graph it is 1.37 GiB instead of 2.28 GiB and avoids 2,407
  source-build derivations. Home Manager selects it through the existing
  `programs.yazelix.package` option and can own `cursors.toml` through the same
  native `text`/`source` contract.
- `yzn enter` is the supported headless and SSH route for the managed
  Zellij/Yazi/Helix workspace. It requires an interactive host terminal but not
  Mars, a desktop entry, `DISPLAY`, or `WAYLAND_DISPLAY`; terminal-specific
  graphics and clipboard behavior remain host-owned.
- Nova rejects unsupported, misspelled, wrongly shaped, or wrongly typed root
  `config.toml` values before runtime or Ratconfig use while preserving sparse
  inheritance and the documented dynamic `popups.<id>` namespace.
- Ratconfig identifies store-backed Home Manager config as declarative rather
  than merely read-only. Structured saves, resets, and native file actions stop
  before mutation and name the exact `programs.yazelix.config.*` option to edit;
  chmod-only user files remain user-owned.
- First use seeds child-owned `cursors.toml` without overwriting user state.
  The Ratconfig Cursors tab edits the enabled pool, selection, and common effect
  settings while preserving custom definitions; advanced editing opens the full
  file. `yzn launch` passes that exact path to Mars. Mars currently consumes
  selection and basic trail enablement; richer effect, glow, and duration values
  remain available for compatible consumers. The unused Kitty fallback setting
  is removed from the shared schema.
- `Alt Shift F` toggles focused-pane fullscreen, and `Ctrl y` moves directly
  between the managed editor and Yazi sidebar, reopening the sidebar as needed.
- The top-right Zellij corner derives a compact Nova release label from the
  package version: `NOVA DEV`, `NOVA 1β`, or stable major/minor form such as
  `NOVA 1.0`. Exact SemVer remains in `yzn --version` and runtime identity.
- Bare `yzn` prints help instead of launching Mars; sessions start explicitly
  with `launch` or `enter`. `yzn run` executes structured argv in the prepared
  runtime environment, `status --json` exposes a bounded versioned record, and
  `--version` shares one package-owned value with `runtime_identity.json`.
  The root `sponsor` command is removed while its URL remains in help.
- Root `config.toml` is a sparse explicit-override document. Startup and
  `yzn config` inherit packaged values without creating or completing the user
  file; saves write only the selected key and reset removes it. Home Manager
  likewise renders only declared semantic values, with no hidden contract
  metadata.
- Opening `yzn config` no longer creates `mars/config.toml`. The Mars tab shows
  all nine curated effective values without persisting them, saves only explicit
  overrides, and removes a key when reset. Mars recursively layers that sparse
  user file over Nova's immutable packaged base, so untouched settings and font
  paths follow upgrades.
- Opening `yzn config` no longer creates `starship.toml` or changes the managed
  prompt. The Starship tab shows Nova defaults without persisting them, saves
  only explicit overrides, and removes an override when reset. Managed Nu
  merges the sparse user file over Nova defaults into runtime state, so
  untouched prompt defaults follow upgrades and ambient
  `~/.config/starship.toml` remains ignored.
- Opening `yzn config` no longer creates `zellij/config.kdl`. The Zellij tab
  shows inherited packaged defaults, saves only explicit scalar overrides, and
  removes an assignment on reset. Resetting the final override removes the
  sidecar, while active-session saves and resets remain best-effort live updates.
- User-visible product surfaces identify the development runtime as Yazelix
  Nova. The command remains `yzn` until the canonical repository swap.
- `yzn config` fully redraws after returning from an external editor instead of
  leaving the editor background until another UI action.
- Optional managed `yazi/yazi.toml` now layers native Yazi tables over the
  packaged config while replacing user scalars and arrays. Yazelix retains its
  edit opener and required sidebar Git fetchers; invalid TOML fails before Yazi
  starts. Managed plugins and flavors activate independently of `init.lua`,
  with opaque `theme.toml` and `package.toml` passthrough. Ratconfig and Home
  Manager expose the managed native files.
- `yzn config` boolean rows use `Space` to stage, `Enter` to save, and `Esc`
  to cancel; normal-mode `Enter` leaves the value unchanged
- Managed LazyGit file edits honor `editor.command` instead of falling back to
  Vim for unknown presets. Direct `yzn-editor` is also exported through
  `EDITOR`, `VISUAL`, and `GIT_EDITOR`; it stays in the client lifecycle with
  the Helix bridge disabled, restores the transparent Zellij background after
  editing, and keeps user LazyGit configuration loaded.
- The managed agent popup command is configurable through root config
  `agent.command` and `agent.args`, exposed in the `yzn config` `popups` tab.
  The default `agent.command = "auto"` keeps the existing provider fallback.
- `yzn config` has a dedicated `popups` tab for popup margins and managed
  config/agent/Git/menu popup keybindings. Root `config.toml` remains the source
  for these fields.
- `yzn config` Zellij tab saves and resets update the active managed session
  config when opened inside a session (`YAZELIX_STATE_DIR` + session env). Many
  scalars apply live via Zellij's config watcher; some still need a new session
  (for example `scroll_buffer_size`). Outside a session, both remain next-session.
- The managed agent popup (`Alt Shift L`, hide keep-alive) now restarts when
  the focused terminal cwd no longer matches the hidden agent pane. Same-cwd
  toggles still reuse the existing agent process.
- Darwin package confidence no longer relies on hosted macOS on every push.
  `Darwin Package Smoke` builds both `aarch64-darwin` package variants weekly
  when `main` moved in the last 7 days (and always on manual dispatch). Routine
  CI stays Linux-only; manual Version Gate still includes the macOS package
  smoke before version bumps or the main Yazelix swap.
- The default `yzn` package no longer pulls Mars' removed SerenityOS emoji
  profile into the Nix closure, avoiding the expensive unused
  `serenityos-emoji-font` / `nanoemoji` build path during declarative installs.
- `yzn config` now declares `mars.appearance.preset` as the Ratconfig UI theme
  source, so changing the Mars dark/light preset through the config UI switches
  the config palette in the same session. Direct file edits while the UI is
  already open are still picked up on the next `yzn config` launch.
- `yzn config` native file rows now show absent optional files as neutral
  `absent` rows instead of warning-colored `missing` rows.
- The root `[appearance].mode` setting was removed before release. Mars
  appearance stays Mars-owned: `yzn config` exposes the native
  `mars.appearance.preset` dark/light selector in `mars/config.toml`, while
  low-level Mars `force-theme` and `[colors]` remain native TOML only. Shared
  cursor configuration lives in `cursors.toml`.
- Flake package and app outputs are exposed for `x86_64-darwin` and
  `aarch64-darwin` in addition to the Linux systems. The macOS floor is
  package-first: `yzn help`, `status`, `doctor`, and `enter` are the supported
  floor, while Mars-backed `launch` remains issue-driven best-effort until
  trusted macOS hardware validation confirms it.
- `yzn tutor` prints guided Yazelix lessons for workspace flow, discovery,
  troubleshooting, and native tool tutors. `yzn tutor hx`/`helix` and
  `yzn tutor nu`/`nushell` print the packaged Helix and Nushell tutor commands
  instead of launching them.
- `homeManagerModules.default` exposes a narrow Home Manager surface:
  `programs.yazelix.enable = true` installs the selected package,
  `programs.yazelix.package` supports local or alternate package overrides, and
  optional `programs.yazelix.config` owns selected runtime config files. No
  runtime config files are generated by default. `settings` renders only
  declared sparse root values; Cursors, Mars, Zellij, Starship, Helix, Yazi, and
  Nu native files are text/source passthroughs.
- `yzn help` prints help, `yzn env` opens the configured managed shell without
  launching the UI with packaged `hx`, `lazygit`, and `git` on PATH, `yzn enter`
  starts the managed Zellij runtime in the current terminal, and `yzn launch`
  opens Mars first. Bare `yzn` prints help.
- `yzn config` opens source-backed Ratconfig tabs for root, Mars, Zellij, and
  Starship configuration. Root and Starship values are sparse overrides; Mars
  and Zellij are managed native files; and the Starship tab edits
  `format`, `right_format`, and `add_newline` as sparse overrides. The managed
  Starship left prompt defaults to colon-colon-space (`:: `). The UI
  can open staged text edits in the config UI's editor environment before
  saving and refuses to replace a source file whose permissions are read-only.
- Root `config.toml` supports `[editor].command`, defaulting to `yzn-hx`.
  Inside Nova, `hx` and `yzn-hx` resolve to packaged managed Helix; other editor
  commands such as `nvim`, or absolute host paths, bypass the Helix bridge and
  stay user-owned. `yzn status` and `yzn doctor` report the configured and
  effective editor, Ratconfig external text edits and the managed Git popup use
  the same editor, and missing editor commands fail with a direct diagnostic
  before opening a pane. The Git popup defaults to LazyGit.
- The `yzn config` Helix tab opens managed `helix/config.toml`,
  `helix/languages.toml`, `helix/helix.scm`, and `helix/init.scm` in `yzn-hx`.
  Files are created only when activated; either Steel row creates the pair. A
  new `helix/config.toml` is a comment-only sparse override. `yzn-hx` generates
  effective config under `YAZELIX_STATE_DIR` by merging it over packaged
  defaults and restoring reserved `Alt r` to Yazelix reveal. Packaged Helix
  binds `Ctrl r` to reload its config and current buffer; that binding remains
  user-overridable. `yzn doctor` warns when an override sets reserved `Alt r`.
- Managed Helix loads the packaged `:yzn-new-shell` Steel command by default.
  It opens a new Yazelix terminal pane at the current Helix file directory or
  workspace. User-managed Steel files still replace the packaged Steel module
  when both `helix.scm` and `init.scm` exist.
- The `yzn config` Advanced tab opens managed user `nu/env.nu`,
  `nu/config.nu`, `yazi/init.lua`, `yazi/keymap.toml`, and
  `zellij/plugins.kdl` files in `yzn-hx`, creating tiny starter files only
  after a row is activated.
- Managed sessions export packaged `ya` and Zellij helper paths so popup
  hide/close hooks can refresh Yazi sidebar git decorations even when those
  commands are not on the user's shell `PATH`.
- The `yzn config` tab bar labels include monochrome Nerd Font icons for
  faster visual scanning in Mars/Ghostty-style terminals, and the root
  `config.toml` tab is labeled `main`.
- The `yzn config` Keys tab lists current packaged keybindings as a read-only
  table with group, key, action, and owner columns, with source paths in
  details. The table includes packaged `Ctrl Alt h/j/k/l` movement bindings,
  and flake checks keep the human-facing key reference backed by `config.kdl`.
- `yzn menu` opens a packaged `fzf` live-filter command palette and uses the
  same command descriptions as `yzn help` for its curated command list.
- `yzn` uses a Rust front door for startup setup and final process handoff:
  `enter` starts managed Zellij, `launch` opens Mars first, `status` prints a
  compact runtime/config summary, `doctor` checks owned setup, and `sponsor`
  opens or prints the GitHub Sponsors URL. Pre-exec failures print a concise
  Yazelix diagnostic with the relevant config path when available.
- `yzn screen [style]` shows a Yazelix terminal screen directly. Startup runs a
  bounded welcome splash before managed Zellij, controlled by
  `[welcome].enabled`, `[welcome].style`, and `[welcome].duration_seconds`.
  The fixed style set includes the card-like `static` and `logo` screens plus
  imported screen animations and `random`; `random` skips `static` and `logo`,
  and configurable pools are intentionally not exposed.
- `yzn config` ignores unsupported modified terminal keys instead of treating
  them as text.
- `yzn config` restores raw terminal mode if alternate-screen setup fails.
- Yazi opens reuse only a Helix bridge pane in the invoking Zellij tab. `Alt z`
  moves to the zoxide-selected directory, sends it through `yzn-open`, renames
  the tab to the workspace root, and keeps the selected picker directory in
  Helix for Git repos.
- `Alt 1` through `Alt 9` jump directly to tabs 1 through 9.
- Plain `Alt i` and `Alt o` no longer inherit Zellij's default tab-move
  bindings; Yazelix tab movement stays on `Ctrl Alt h/l`.
- `Alt r` reveals the current Helix buffer in the managed Yazi sidebar, and
  `yzn reveal <target>` exposes the same active-tab reveal path inside a
  managed session. New managed editor panes use the pane-orchestrator's
  canonical `editor` title so smart reveal forwards into Helix before falling
  back to sidebar focus. The reveal helper treats empty successful focus pipe
  replies as success.
- Managed Yazi appends optional user `yazi/init.lua` and `yazi/keymap.toml`
  sidecars after the packaged setup without importing full native Yazi config.
- `config.toml` controls `open.log_level`, `shell.program`,
  `[popup].side_margin`, `[popup].vertical_margin`,
  `keybindings.config`/`agent`/`git`/`menu`, semantic `[popups.<id>]`
  custom popups, and `[bar].widgets`; managed
  popups default to one left/right margin cell and zero top/bottom margin
  cells, popup role triggers default to `Alt Shift J/K/L/M`, invalid,
  duplicate, or conflicting semantic values fail before launch, custom popups
  require argv-based `command` plus a managed `keybinding`, accept optional
  `args`, `title`, and `keep_alive`, reject title collisions with custom or
  packaged popups, inherit global popup margins and sidebar refresh hooks
  through the plugin-level `yzpp` default block, render title-backed command
  markers for stable pane identity, and `yzn config` shows scalar root fields in the main config
  tab with bar widgets as an ordered Ratconfig string-list picker. Custom bar
  widget layouts keep
  the sidebar swap layout paired with the generated layout. The empty
  workspace widget is not selectable. The shell bar widget follows
  `shell.program` in compact form, such as `❯nu` or `❯fish`.
- `yzn` appends `~/.config/yazelix-next/zellij/config.kdl` as a native Zellij
  sidecar for safe preferences, with a small denylist guardrail for obvious
  ownership lines such as keymaps, shell, layout, plugins, Kitty keyboard
  protocol, environment, and session startup.
- `yzn` reads `~/.config/yazelix-next/zellij/plugins.kdl` as a plugin-only
  Zellij sidecar. It accepts only `plugins` and `load_plugins` blocks, injects
  them into the managed Zellij config, allows comments and quoted string values
  in plugin bodies, and rejects Yazelix-owned plugin ids.
- Managed Nu inserts host `mise activate nu` output after packaged Nu config
  and before optional user Nu config when `mise` is available on the inherited
  `PATH`; missing or failing `mise` is skipped.
- Nushell delegates the right prompt to Starship, so `right_format` in
  `~/.config/yazelix-next/starship.toml` is honored.
- Generated runtime state defaults to `${XDG_DATA_HOME:-$HOME/.local/share}/yazelix-next`,
  with non-empty `YAZELIX_STATE_DIR` still taking precedence.
- The top bar uses standalone Yazelix Zellij Bar with no `NORMAL` segment,
  native tab labels, the Yazelix home marker, selected widgets, a `YZN` runtime
  marker, bundled `tu` Codex quota/reset data, and a yzn-owned cache path; the
  bottom native status bar still owns key hints, and Tab-mode new tabs use the
  packaged sidebar layout/home marker with a home-scoped Yazi cwd.
- The Yazelix Zellij fork focuses plugin permission prompts as they appear,
  uses a full-viewport prompt for tiny layout panes, and drains concurrent
  startup permission prompts one at a time before restoring pane focus.
- `yzn` uses an isolated Zellij plugin-permission cache and pre-seeds packaged
  Bar, Popup, and pane-orchestrator permissions so desktop launches do not
  depend on hidden plugin permission prompts.
- `Alt Shift J/K/L/M` toggle Git, config, agent, and menu popups through
  Yazelix Zellij Popup with Kitty keyboard protocol.
  `keybindings.config`/`agent`/`git`/`menu` can remap those semantic actions
  without exposing raw Zellij keymaps. The Git popup closes on toggle so the
  next LazyGit open follows the current tab cwd after workspace retargeting,
  and popup close/hide hooks refresh managed Yazi git decorations. The
  agent popup bootstraps once from `codex resume`, `grok`, `opencode`, `pi`,
  then `claude --resume`, persists the first available provider under
  `YAZELIX_STATE_DIR`, and does not cascade again after a provider is selected.
  If no provider is available on first run, it leaves the popup pane empty.
  Replacing the agent popup with another managed popup hides it instead of
  killing the agent process, `yzn menu` opens a live-filter command palette for
  `config`, `doctor`, `status`, `screen`, `sponsor`, `launch`, `help`, and `tutor`, and
  `Alt h/l` route through pane orchestrator to skip collapsed sidebars and fall
  back to previous/next tab. When a managed popup is visible, `Alt h/l`
  switches tabs instead of focusing panes behind the popup.

## 2026-06-25

- `yzn` installs a Nix/Lix-compatible flake runtime that opens Mars with the
  Yazelix Zellij fork.
- Mars uses the packaged Yazelix Next visual config, reef cursor colors,
  JetBrains Mono, and no window bar.
- Zellij starts with a Yazi sidebar and stacked work panes. `Alt Shift h`
  toggles the managed sidebar.
- The Zellij status bar groups first-class key hints into `Ctrl`, `Ctrl Alt`,
  and `Alt` clusters.
- `Ctrl p/t/n/q`, `Ctrl Alt g/s/o`, `Ctrl Alt h/j/k/l`, and `Alt m` define the
  current Zellij keymap.
- Nushell loads packaged Starship, Carapace, and Zoxide setup before optional
  user files under `~/.config/yazelix-next/nu`.
- Yazi opens files and directories through `yzn-open`, reusing a live Yazelix
  Helix bridge inside the current `yzn` window when possible.
- Yazi `Alt z` opens a zoxide picker and sends the selected directory through
  `yzn-open`.
- `yzn-open` writes bounded rotated diagnostics and honors `YZN_OPEN_LOG`.
- Profile installs include a `Yazelix Next` desktop entry.
