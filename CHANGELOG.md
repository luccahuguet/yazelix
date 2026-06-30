# Changelog

User-visible runtime changes for Yazelix Next live here.

## Unreleased

- `yzn help` prints help, `yzn env` opens the configured managed shell without
  launching the UI with packaged `hx`, `lazygit`, and `git` on PATH, `yzn enter`
  starts the managed Zellij runtime in the current terminal, and `yzn launch`
  opens Mars first. Bare `yzn` defaults to `yzn launch`.
- `yzn config` opens the Ratconfig UI and creates source-backed tabs for
  `config.toml`, `mars/config.toml`, and `zellij/config.kdl` when missing.
  Root `config.toml` keeps Ratconfig contract state, while the Mars and Zellij
  tabs are simple managed render/edit files that apply on new launches. The UI
  refuses to replace a source file whose permissions are read-only.
- The `yzn config` Advanced tab opens managed user `nu/env.nu`,
  `nu/config.nu`, `starship.toml`, `yazi/init.lua`, and `yazi/keymap.toml`
  files in `yzn-hx`, creating tiny starter files only after a row is
  activated.
- The `yzn config` Keys tab lists current packaged keybindings as a read-only
  table with group, key, action, and owner columns, with source paths in
  details.
- `yzn` uses a Rust front door for startup setup and final process handoff:
  `enter` starts managed Zellij, `launch` opens Mars first, `status` prints a
  compact runtime/config summary, `doctor` checks owned setup, and `sponsor`
  opens or prints the GitHub Sponsors URL. Pre-exec failures print a concise
  Yazelix diagnostic with the relevant config path when available.
- `yzn config` ignores unsupported modified terminal keys instead of treating
  them as text.
- `yzn config` restores raw terminal mode if alternate-screen setup fails.
- Yazi opens reuse only a Helix bridge pane in the invoking Zellij tab. `Alt z`
  moves to the zoxide-selected directory, sends it through `yzn-open`, renames
  the tab to the workspace root, and keeps the selected picker directory in
  Helix for Git repos.
- Managed Yazi appends optional user `yazi/init.lua` and `yazi/keymap.toml`
  sidecars after the packaged setup without importing full native Yazi config.
- `config.toml` controls `open.log_level`, `shell.program`, `[popup].size`,
  and `[bar].widgets`; managed popups default to 95% width and height, invalid
  semantic values fail before launch, and `yzn config` shows these root fields
  in the main config tab with bar widgets as an ordered Ratconfig string-list
  picker. Custom bar widget layouts keep the sidebar swap layout paired with the
  generated layout. The empty workspace widget is not selectable.
- `yzn` appends `~/.config/yazelix-next/zellij/config.kdl` as a native Zellij
  sidecar for safe preferences, with a small denylist guardrail for obvious
  ownership lines such as keymaps, shell, layout, plugins, Kitty keyboard
  protocol, environment, and session startup.
- `yzn` uses `~/.config/yazelix-next/mars/config.toml` as a full native Mars
  config override when that file exists, while keeping the Mars launch command
  and managed Zellij runtime owned by `yzn`.
- Managed Nu sets `STARSHIP_CONFIG` to `~/.config/yazelix-next/starship.toml`
  when that file exists, otherwise to an empty config so normal
  `~/.config/starship.toml` does not affect the managed Nu prompt.
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
- `Alt Shift J/K/L/M` toggle LazyGit, config, persistent guarded
  `codex resume`, and menu popups through Yazelix Zellij Popup with Kitty
  keyboard protocol; replacing the agent popup with another managed popup hides
  it instead of killing the Codex process, `yzn menu` prints the same compact
  command/key reference, and `Alt h/l` route through pane orchestrator to skip
  collapsed sidebars and fall back to previous/next tab. When a managed popup is
  visible, `Alt h/l` switches tabs instead of focusing panes behind the popup.

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
