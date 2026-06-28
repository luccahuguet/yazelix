# Changelog

User-visible runtime changes for Yazelix Next live here.

## Unreleased

- `yzn help` prints help, `yzn enter` starts the managed Zellij runtime in the
  current terminal, and `yzn launch` opens Mars first. Bare `yzn` defaults to
  `yzn launch`.
- `yzn config` opens the Ratconfig UI and creates source-backed tabs for
  `config.toml`, `mars/config.toml`, and `zellij/config.kdl` when missing.
  Root `config.toml` keeps Ratconfig contract state, while the Mars and Zellij
  tabs are simple managed render/edit files that apply on new launches. The UI
  refuses to replace a source file whose permissions are read-only.
- `yzn config` ignores unsupported modified terminal keys instead of treating
  them as text.
- `yzn config` restores raw terminal mode if alternate-screen setup fails.
- Yazi `Alt z` moves the sidebar to the zoxide-selected directory before
  sending it through `yzn-open`.
- `open.log_level` in `config.toml` controls the managed `YZN_OPEN_LOG`
  level used by Yazi-to-Helix opens.
- `yzn` appends `~/.config/yazelix-next/zellij/config.kdl` as a native Zellij
  sidecar for safe preferences, with a small denylist guardrail for obvious
  ownership lines such as keymaps, shell, layout, plugins, Kitty keyboard
  protocol, environment, and session startup.
- `yzn` uses `~/.config/yazelix-next/mars/config.toml` as a full native Mars
  config override when that file exists, while keeping the Mars launch command
  and managed Zellij runtime owned by `yzn`.
- `yzn` sets `STARSHIP_CONFIG` to `~/.config/yazelix-next/starship.toml` when
  that file exists, otherwise to an empty config so normal
  `~/.config/starship.toml` does not affect the managed prompt.
- Nushell delegates the right prompt to Starship, so `right_format` in
  `~/.config/yazelix-next/starship.toml` is honored.
- The top bar uses the standalone Yazelix Zellij Bar package while the bottom
  native Zellij status bar still owns key hints.
- `yzn` renders the top bar through the Yazelix Zellij Bar widget command with
  no `NORMAL` mode segment, native tab labels at the left edge, the Yazelix
  home marker, and editor, shell, terminal, Codex, CPU, RAM, and version
  widgets.
- The Codex usage widget shows quota/reset windows without token totals, uses
  the bundled `tu` helper and a yzn-owned status cache path, and avoids stale
  generic bar cache state.
- Tab-mode new tabs use the packaged Yazelix sidebar layout and the same
  Yazelix home marker instead of a bare `Tab #N` pane.
- The Yazelix Zellij fork focuses plugin permission prompts as they appear,
  uses a full-viewport prompt for tiny layout panes, and drains concurrent
  startup permission prompts one at a time before restoring pane focus.
- `yzn` uses an isolated Zellij plugin-permission cache and pre-seeds its
  packaged Yazelix Bar and Popup plugin permissions so desktop launches do not
  depend on hidden plugin permission prompts.
- `Alt Shift K` toggles the config popup and `Alt Shift J` toggles a managed
  LazyGit popup through the standalone Yazelix Zellij Popup plugin, with Kitty
  keyboard protocol enabled.
- `Alt Shift L` toggles a guarded `codex resume` popup that checks for `codex`
  on `PATH` before launching it, `Alt Shift M` toggles a menu popup, and
  `yzn menu` prints the same compact command/key reference.

## 2026-06-25

- `yzn` installs a Nix/Lix-compatible flake runtime that opens Mars with the
  Yazelix Zellij fork.
- Mars uses the packaged Yazelix Next visual config, reef cursor colors,
  JetBrains Mono, and no window bar.
- Zellij starts with a Yazi sidebar and stacked work panes. `Alt Shift h`
  toggles the sidebar layout.
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
