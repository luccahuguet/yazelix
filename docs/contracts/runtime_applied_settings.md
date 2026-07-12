# Runtime-Applied Settings Contract

## Summary

The sparse semantic root contains launch-time configuration. Saving a supported field persists explicit user intent; a fresh Yazelix window consumes the new value. Home Manager-owned files remain declarative and read-only from runtime editors.

Native Mars, Zellij, Helix, Yazi, Nushell, Starship, and cursor files keep their own ownership and activation rules. They are not semantic root fields.

## Apply Modes

| Mode | Meaning |
| --- | --- |
| `tab_session_restart` | A fresh Yazelix window consumes the saved value |
| `shell_terminal_restart` | New shells and terminal processes consume the saved value |
| `package_home_manager_activation` | The Home Manager source must be changed and activated |

Retired generated-runtime and pane-refresh modes remain historical snapshot metadata only. The live apply vocabulary does not parse or execute them.

## Current Semantic Inventory

| Settings | Apply mode | Runtime effect |
| --- | --- | --- |
| `open.log_level` | `tab_session_restart` | Managed Yazi-open diagnostics |
| `shell.program` | `shell_terminal_restart` | Packaged shell for new panes |
| `editor.command` | `tab_session_restart` | Editor executable for managed opens |
| `agent.command`, `agent.args` | `tab_session_restart` | Managed agent executable and argv |
| `welcome.enabled`, `welcome.style`, `welcome.duration_seconds` | `tab_session_restart` | Startup welcome behavior |
| `popup.side_margin`, `popup.vertical_margin` | `tab_session_restart` | Managed popup margins |
| `keybindings.config`, `keybindings.agent`, `keybindings.git`, `keybindings.menu` | `tab_session_restart` | Chords for the four managed surfaces |
| `bar.widgets` | `tab_session_restart` | Top-bar widget order |
| `popups.<id>` | `tab_session_restart` | User-defined popup command, argv, title, chord, and keep-alive policy |

The final Classic bridge projects these meanings into its existing runtime internals. That projection is temporary and is deleted with Classic during the source swap; it does not make the retired Classic field names part of the supported root.

## Save And Apply Semantics

1. Validate the sparse root before writing it.
2. Preserve omitted fields as omitted so packaged defaults remain live.
3. Report that saved semantic values apply in a fresh Yazelix window.
4. Refuse runtime edits when Home Manager owns `config.toml`; the user changes the declarative source and runs `home-manager switch`.
5. Never mutate native files or generated runtime state as a semantic save side effect.
6. Keep visible errors for invalid fields, read-only ownership, and failed writes.

## Native Files

- Mars overlays sparse `~/.config/yazelix/mars/config.toml` values in the next Mars window.
- Zellij preferences live in `~/.config/yazelix/zellij/config.kdl`; third-party plugin declarations live in `zellij/plugins.kdl`.
- Cursor settings live in the child-owned `~/.config/yazelix/cursors.toml` contract.
- Helix, Yazi, Nushell, and Starship use their native files under `~/.config/yazelix/`.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core config_ui`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core config_apply`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core config_normalize`
- `cargo run --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-config-surface-contract`
