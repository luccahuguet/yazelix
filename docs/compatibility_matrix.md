# Compatibility Matrix

This page classifies the current Yazelix support surface for terminals,
editors, shells, platforms, and install owners.

Validated on June 22, 2026 by lucca from the current main-repo contracts,
package docs, and terminal comparison matrix.

## Support Levels

| Level | Meaning |
| --- | --- |
| `supported` | First-class documented surface with package or config ownership and current validation evidence |
| `stable alternate` | Supported path that is not the default, but is expected to work for normal users |
| `experimental` | Packaged or documented path that is useful for testing and dogfooding, but not the safest default |
| `issue-driven best-effort` | Expected to be maintained from user reports because maintainers lack direct hardware or regular validation for that surface |
| `best-effort` | Expected to work in ordinary cases, but lacks the same validation depth as supported surfaces |
| `unsupported` | Not a maintained first-class Yazelix surface |

## Terminal Matrix

| Terminal | Linux launch | macOS launch | Cursor trails and shaders | Yazi image previews through Zellij | Managed editor behavior | Zellij web sharing | Install-owner behavior |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Mars | `supported`, packaged runtime | `issue-driven best-effort` | `supported` for Mars native trail cursor and Yazelix split cursor config; Ghostty-compatible shader profile remains terminal-specific | `supported` through the Yazelix Zellij Kitty graphics bridge | `supported`; terminal choice does not weaken managed Helix/Neovim pane routing | `best-effort`; Zellij owns web sharing behavior | `supported` for profile and Home Manager installs |
| Ghostty | `best-effort` host-owned `yzx enter` entrypoint | `best-effort` host-owned `yzx enter` entrypoint | host-owned; standalone cursor tooling can still generate Ghostty-compatible shader assets | host-owned terminal behavior | `supported` at the Yazelix workspace layer after `yzx enter` | `best-effort`; Zellij owns web sharing behavior | user-owned terminal install; Yazelix package owner remains Mars |
| Rio | `best-effort` host-owned `yzx enter` entrypoint | `best-effort` host-owned `yzx enter` entrypoint | host-owned | host-owned terminal behavior | `supported` at the Yazelix workspace layer after `yzx enter` | `best-effort`; Zellij owns web sharing behavior | user-owned terminal install; Yazelix package owner remains Mars |
| WezTerm | `best-effort` host-owned `yzx enter` entrypoint | `best-effort` host-owned `yzx enter` entrypoint | host-owned | host-owned terminal behavior | `supported` at the Yazelix workspace layer after `yzx enter` | `best-effort`; Zellij owns web sharing behavior | user-owned terminal install; Yazelix package owner remains Mars |
| Kitty | `best-effort` host-owned `yzx enter` entrypoint | `best-effort` host-owned `yzx enter` entrypoint | host-owned | host-owned terminal behavior | `supported` at the Yazelix workspace layer after `yzx enter` | `best-effort`; Zellij owns web sharing behavior | user-owned terminal install; Yazelix package owner remains Mars |
| Foot | `best-effort` host-owned `yzx enter` entrypoint | `unsupported` | host-owned | host-owned terminal behavior | `supported` at the Yazelix workspace layer after `yzx enter` | `best-effort`; Zellij owns web sharing behavior | user-owned terminal install; Yazelix package owner remains Mars |
| Ratty | `best-effort` host-owned `yzx enter` entrypoint | `unsupported` | host-owned | host-owned terminal behavior | `supported` at the Yazelix workspace layer after `yzx enter` | `best-effort`; Zellij owns web sharing behavior | user-owned terminal install; Yazelix package owner remains Mars |
| Alacritty | `unsupported` | `unsupported` | `unsupported` | `unsupported` | `best-effort` only if the user runs Yazelix manually inside an unmanaged Alacritty session | `best-effort`; Zellij owns web sharing behavior | `unsupported` as a packaged or Home Manager terminal selection |

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
| Xonsh | `supported`, host-owned | `shell.default_shell = "xonsh"` | Yazelix generates `xonsh/yazelix_init.xsh` and `shell_xonsh.xsh`; the host must provide `xonsh` on `PATH` and source the hook from xonsh rc for native startup integration |
| Other shells | `unsupported` as `default_shell` enum values | not accepted | Use one of the supported enum values, or launch another shell manually inside a pane |

## Platform Matrix

| Platform | Support level | Terminals | Desktop/app launcher notes |
| --- | --- | --- | --- |
| Linux | `supported` | Mars packaged runtime; other terminals are host-owned `yzx enter` entrypoints | Home Manager Linux desktop entry targets the packaged Mars runtime |
| macOS | `supported floor` for package install, `yzx --version-short`, `yzx doctor`, and host-terminal `yzx enter`; Mars is issue-driven until macOS user reports establish stronger evidence | Mars packaged runtime; other terminals are host-owned `yzx enter` entrypoints | `yzx desktop macos_preview install` is experimental, unsigned, unnotarized, and distinct from a supported Dock/Launchpad app |
| Windows | `unsupported` | none | WSL/native Windows support is separate future work |

## Install Owner Matrix

| Owner | Support level | Terminal selection | Update behavior |
| --- | --- | --- | --- |
| Nix profile package | `supported` | Use `#yazelix` or `#yazelix_mars` for the packaged Mars runtime | `yzx update upstream` owns default profile updates |
| Home Manager | `supported` | `programs.yazelix.terminal = "mars"` selects the packaged runtime | `yzx update home_manager` prints the Home Manager update path |
| Manual or host-only launch | `best-effort` | Host terminal choices are user-owned and should run `yzx enter` | Install into a profile or enable Home Manager before relying on Yazelix update ownership |

## Current Boundary Notes

- Yazelix packages Mars and does not fall back to another terminal when Mars is missing or mispackaged
- Non-Mars terminal launchers are host-owned; configure the terminal's startup command to run `yzx enter`
- Terminal choice should not change the managed editor contract: Helix and
  Neovim integrations target the Yazelix-managed editor pane, while
  shell-opened editors remain ordinary panes
- Zellij web sharing is classified as best-effort here because Yazelix does not
  currently add terminal-specific support guarantees beyond running Zellij
- Alacritty is intentionally unsupported as a packaged or Home Manager terminal
  selection

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
