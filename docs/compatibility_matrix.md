# Compatibility Matrix

This page classifies the current Yazelix support surface for terminals,
editors, shells, platforms, and install owners.

Validated on June 7, 2026 by lucca from the current main-repo contracts,
package docs, terminal comparison matrix, and the closed Yazelix Terminal
release gate `yazelix-5br5o.18`.

## Support Levels

| Level | Meaning |
| --- | --- |
| `supported` | First-class documented surface with package or config ownership and current validation evidence |
| `stable alternate` | Supported path that is not the default, but is expected to work for normal users |
| `experimental` | Packaged or documented path that is useful for testing and dogfooding, but not the safest default |
| `best-effort` | Expected to work in ordinary cases, but lacks the same validation depth as supported surfaces |
| `unsupported` | Not a maintained first-class Yazelix surface |

## Terminal Matrix

| Terminal | Linux launch | macOS launch | Cursor trails and shaders | Yazi image previews through Zellij | Managed editor behavior | Zellij web sharing | Install-owner behavior |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Ghostty | `supported`, default packaged runtime | `supported` launch floor through bundled `ghostty-bin`; automatic Ghostty shell integration is not guaranteed | `supported` for Yazelix cursor trails and Ghostty-compatible shaders | `supported` through Yazelix-pinned Zellij/Yazi Kitty graphics passthrough forks | `supported`; terminal choice does not weaken managed Helix/Neovim pane routing | `best-effort`; Zellij owns web sharing behavior | `supported` for profile and Home Manager installs |
| Yazelix Terminal (`yzxterm`) | `experimental` first-party packaged runtime | `best-effort`; no macOS parity claim beyond package availability | `experimental`; Rio trail cursor defaults, `baseline` profile without effects, and `shaders` profile with generated Yazelix cursor shaders | `experimental`; Kitty graphics and stack fixes exist, but visual/font/renderer polish remains active dogfooding | `supported` at the Yazelix workspace layer; terminal rendering/input gaps remain terminal-owned risks | `best-effort`; Zellij owns web sharing behavior | `supported` for selected package/Home Manager runtime; `yzxterm_package` override is maintainer dogfooding only |
| Rio | `stable alternate` packaged upstream runtime | `best-effort` packaged alternate | `supported` for Rio native trail cursor in generated config; no Yazelix cursor shader ABI | `best-effort` through the Yazelix Zellij/Yazi Kitty graphics bridge | `supported` at the Yazelix workspace layer | `best-effort`; Zellij owns web sharing behavior | `supported` for selected package/Home Manager runtime |
| WezTerm | `stable alternate` packaged runtime | `best-effort` packaged alternate | `unsupported` for Yazelix cursor shaders | `best-effort`; broad terminal image support, but not the default preview target | `supported` at the Yazelix workspace layer | `best-effort`; Zellij owns web sharing behavior | `supported` for selected package/Home Manager runtime |
| Kitty | `stable alternate` packaged runtime or host `PATH` terminal | `best-effort` packaged or host `PATH` terminal | `partial`; Kitty has cursor effects, but not the Yazelix Ghostty-compatible shader ABI | `best-effort`; Kitty is the protocol reference and Yazelix has generated config support | `supported` at the Yazelix workspace layer | `best-effort`; Zellij owns web sharing behavior | `supported` for selected package/Home Manager runtime; host `PATH` Kitty is user-owned |
| Foot | `experimental` Linux-only packaged runtime | `unsupported` | `unsupported` for Yazelix cursor shaders | `best-effort`; lightweight terminal path with generated Foot config | `supported` at the Yazelix workspace layer | `best-effort`; Zellij owns web sharing behavior | `supported` on Linux package/Home Manager runtime only |
| Ratty | `experimental` Linux-only packaged runtime | `unsupported` | `unsupported` for Yazelix cursor shaders | `best-effort`; Yazelix can use the Zellij/Yazi Kitty graphics bridge, but does not claim Ratty Graphics Protocol passthrough inside Zellij | `supported` at the Yazelix workspace layer | `best-effort`; Zellij owns web sharing behavior | `supported` on Linux package/Home Manager runtime only |
| Alacritty | `unsupported` | `unsupported` | `unsupported` | `unsupported` | `best-effort` only if the user runs Yazelix manually inside an unmanaged Alacritty session | `best-effort`; Zellij owns web sharing behavior | `unsupported` as a packaged or Home Manager terminal variant |

## Editor Matrix

| Editor choice | Support level | Managed editor pane | Yazi open/reveal integration | Notes |
| --- | --- | --- | --- | --- |
| Bundled Yazelix Helix | `supported`, default | `supported` | `supported`; `Alt+r` reveal is managed through Yazelix Helix config | Uses the `luccahuguet/yazelix-helix` fork with the `--config-dir` support Yazelix needs |
| Yazelix-compatible external Helix fork | `supported` for fork development | `supported` when `helix.external` points to matching binary and runtime | `supported` when the fork keeps Yazelix-compatible config/runtime behavior | Vanilla/upstream Helix is not a supported `helix.external` target |
| Neovim | `supported` | `supported` | `supported` with a user/editor-local reveal binding such as `yzx reveal` | Yazelix targets the managed Neovim pane deterministically |
| Vim, Kakoune, Nano, Emacs, and other terminal editors | `best-effort` | Basic pane launch only | Limited; no first-party same-instance or reveal contract | Configure with `editor.command`; Emacs has a separate low-priority future compatibility bead |
| Editors started manually from a shell pane | ordinary pane, not managed | `unsupported` as managed editor panes | `unsupported` for managed Yazi routing | Yazelix does not auto-adopt shell-opened editors as the managed `editor` pane |

## Shell Matrix

| Shell | Support level | Config value | Notes |
| --- | --- | --- | --- |
| Nushell | `supported`, default | `shell.default_shell = "nu"` | Default session shell; Yazelix owns a small runtime Nushell UX surface |
| Bash | `supported` | `shell.default_shell = "bash"` | Supported generated initializer path |
| Fish | `supported` | `shell.default_shell = "fish"` | Supported generated initializer path |
| Zsh | `supported` | `shell.default_shell = "zsh"` | Supported generated initializer path |
| Other shells | `unsupported` as `default_shell` enum values | not accepted | Use one of the supported enum values, or launch another shell manually inside a pane |

## Platform Matrix

| Platform | Support level | Terminals | Desktop/app launcher notes |
| --- | --- | --- | --- |
| Linux | `supported` | Ghostty default; yzxterm, Rio, WezTerm, Kitty, Foot, and Ratty variants | Home Manager Linux desktop entries and `extra_terminal_launchers` are supported |
| macOS | `supported floor` for package install, `yzx --version-short`, `yzx doctor`, and Ghostty `yzx launch`; other paths are `best-effort` | Ghostty is the intended first-party terminal; WezTerm and Kitty are best-effort alternates; Foot and Ratty are unsupported | `yzx desktop macos_preview install` is experimental, unsigned, unnotarized, and distinct from a supported Dock/Launchpad app |
| Windows | `unsupported` | none | WSL/native Windows support is separate future work |

## Install Owner Matrix

| Owner | Support level | Terminal selection | Update behavior |
| --- | --- | --- | --- |
| Nix profile package | `supported` | Choose one flake output such as `#yazelix`, `#yazelix_ghostty`, `#yzxterm`, `#yazelix_rio`, `#yazelix_wezterm`, `#yazelix_kitty`, `#yazelix_foot`, or `#yazelix_ratty` | `yzx update upstream` owns default profile updates |
| Home Manager | `supported` | `programs.yazelix.terminal` selects one active packaged runtime; `extra_terminal_launchers` adds Linux desktop launchers and `yzx launch --term` targets without changing active runtime identity | `yzx update home_manager` prints the Home Manager update path |
| Manual or host-only launch | `best-effort` | Host `PATH` terminal choices are user-owned unless selected through a package/Home Manager variant | Install into a profile or enable Home Manager before relying on Yazelix update ownership |

## Current Boundary Notes

- Yazelix selects one packaged terminal runtime at a time and does not fall
  back to another terminal when the selected variant is missing or mispackaged
- `extra_terminal_launchers` creates additional Linux desktop launch surfaces;
  it does not install duplicate profile `yzx` commands and does not change the
  active runtime identity
- Terminal choice should not change the managed editor contract: Helix and
  Neovim integrations target the Yazelix-managed editor pane, while
  shell-opened editors remain ordinary panes
- Zellij web sharing is classified as best-effort here because Yazelix does not
  currently add terminal-specific support guarantees beyond running Zellij
- Alacritty is intentionally unsupported as a current packaged or Home Manager
  terminal variant

## Source Evidence

- [Terminal emulators](./terminal_emulators.md)
- [Installation](./installation.md)
- [Editor configuration](./editor_configuration.md)
- [macOS support floor](./contracts/macos_support_floor.md)
- [Current trimmed runtime contract](./contracts/v15_trimmed_runtime_contract.md)
- [Terminal launch contract](./contracts/terminal_launch_contract.md)
- [Active runtime identity contract](./contracts/active_runtime_identity.md)
- [Shell-opened editors contract](./contracts/shell_opened_editors.md)
- [Home Manager README](../home_manager/README.md)
