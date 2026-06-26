# Changelog

User-visible runtime changes for Yazelix Next live here.

## Unreleased

- `yzn help` prints help, `yzn enter` starts the managed Zellij runtime in the
  current terminal, and `yzn launch` opens Mars first. Bare `yzn` defaults to
  `yzn launch`.
- `yzn` appends `~/.config/yazelix-next/zellij/config.kdl` as a native Zellij
  sidecar for safe preferences, with a small denylist guardrail for obvious
  ownership lines such as keymaps, shell, layout, plugins, and session startup.
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
- `Alt Shift J` toggles a managed LazyGit popup through the standalone
  Yazelix Zellij Popup plugin, with Kitty keyboard protocol enabled.

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
- `yzn-open` writes bounded rotated diagnostics and honors `YZN_OPEN_LOG`.
- Profile installs include a `Yazelix Next` desktop entry.
