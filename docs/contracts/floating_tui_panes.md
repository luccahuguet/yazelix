# Floating TUI Pane Contract

## Supported Surfaces

Yazelix provides three popup paths:

- `yzx popup <command ...>` opens one transient command in the current workspace.
- The managed config, Git, agent, and menu surfaces use fixed product commands with user-configurable chords.
- `popups.<id>` declares additional managed popup commands.

The root owns shared cell margins:

```toml
[popup]
side_margin = 1
vertical_margin = 0
```

Margins are non-negative cell counts. They replace the retired percentage geometry fields.

## Managed Surface Chords

```toml
[keybindings]
config = "Alt Shift K"
agent = "Alt Shift L"
git = "Alt Shift J"
menu = "Alt Shift M"
```

The final Classic bridge projects these chords into its existing popup and agent-pane actions. Nova consumes the same meanings directly after the source swap.

## Custom Popups

```toml
[popups.zenith]
command = "zenith"
args = []
title = "Processes"
keybinding = "Alt Shift I"
keep_alive = true
```

- `command` is one executable token.
- `args` is an optional string array.
- `title` is optional display text.
- `keybinding` is one non-empty chord.
- `keep_alive` controls whether hiding preserves the process.
- Reserved built-in ids and titles are rejected.
- Duplicate chords are rejected before launch.

Popup lifecycle, duplicate prevention, KDL rendering, and focus behavior remain owned by the Yazelix popup package and its generated runtime integration. User config owns intent, not generated KDL.

## Failure Behavior

- Popup requests outside Zellij fail clearly.
- Invalid commands, ids, titles, margins, and chords block launch.
- Missing plugin permissions or runtime artifacts produce actionable errors.
- Runtime and Home Manager owners never rewrite ambient terminal or Zellij config.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core config_normalize`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core workspace_commands`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `cargo run --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-workspace-session-contract`
