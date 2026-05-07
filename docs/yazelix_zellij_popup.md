# Yazelix Zellij Popup

`yazelix_zellij_popup` is a standalone Zellij plugin for toggling one managed floating command pane from plain Zellij.

## Package

```bash
nix build .#yazelix_zellij_popup
```

The package installs:

- `share/yazelix_zellij_popup/yazelix_zellij_popup.wasm`
- `share/yazelix_zellij_popup/examples/gitui.kdl`
- `share/yazelix_zellij_popup/examples/gitui.template.kdl`
- `share/doc/yazelix_zellij_popup/README.md`

`gitui.kdl` points at the package-local plugin wasm. `gitui.template.kdl` keeps the wasm placeholder for packagers or users who want to substitute a different path.

## Zellij Example

Add the packaged example blocks to your Zellij config, then change the command, pane title, geometry, or keybinding as needed:

```kdl
plugins {
    yazelix_zellij_popup location="file:/nix/store/.../share/yazelix_zellij_popup/yazelix_zellij_popup.wasm"
}

load_plugins {
    yazelix_zellij_popup
}

keybinds {
    normal {
        bind "Alt t" {
            MessagePlugin "yazelix_zellij_popup" {
                name "transient_popup"
                payload "{\"action\":\"toggle\",\"spec\":{\"id\":\"gitui\",\"pane_title\":\"gitui_popup\",\"command_marker\":\"gitui\",\"command\":[\"gitui\"],\"cwd\":\".\",\"width_percent\":90,\"height_percent\":85},\"args\":[]}"
            }
        }
    }
}
```

The plugin supports `toggle`, `open`, `focus`, and `close` actions. `toggle` opens the command when missing, focuses it when present, and closes it when the managed floating pane is already focused.

## Permissions

Zellij prompts for plugin permissions when the plugin first loads. The standalone popup plugin requests:

- `ReadApplicationState`
- `ChangeApplicationState`
- `OpenTerminalsOrPlugins`
- `RunCommands`
- `ReadCliPipes`

Those permissions are the supported minimum for reading active panes, opening a command pane, focusing or closing the managed pane, and receiving `MessagePlugin` pipe requests.

## Request Contract

The pipe name is `transient_popup`. Payloads are strict JSON:

```json
{
  "action": "toggle",
  "spec": {
    "id": "gitui",
    "pane_title": "gitui_popup",
    "command_marker": "gitui",
    "command": ["gitui"],
    "cwd": ".",
    "width_percent": 90,
    "height_percent": 85
  },
  "args": []
}
```

The command is argv, not a shell string. Width and height must be integers from `1` through `100`. Blank ids, pane titles, command paths, and command markers are rejected.

## Yazelix Boundary

This package is the plain-Zellij popup surface. Full Yazelix keeps using its integrated pane orchestrator for `yzx popup`, command palette, config popup, workspace cwd, generated keybindings, Home Manager integration, and sidebar refresh behavior.

## Verification

```bash
nix build .#yazelix_zellij_popup
cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml transient_pane_contract
```
