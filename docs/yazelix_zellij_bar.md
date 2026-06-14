# Yazelix Zellij Bar

`yazelix_zellij_bar` is the standalone Zellij bar plugin package extracted to [yazelix-zellij-bar](https://github.com/luccahuguet/yazelix-zellij-bar).

Regular Yazelix users do not need to install it separately. The normal Yazelix package consumes the child package for integrated zjstatus plugin-block rendering from the child runtime KDL template and forwards the standalone package as `#yazelix_zellij_bar`.

The child package installs:

- `bin/yazelix_zellij_bar_widget`
- `share/yazelix_zellij_bar/zjstatus.wasm`
- `share/yazelix_zellij_bar/yazelix_zellij_bar.kdl`
- `share/yazelix_zellij_bar/yazelix_zellij_bar.template.kdl`
- `share/yazelix_zellij_bar/yazelix_runtime_bar.template.kdl`
- `share/yazelix_zellij_bar/examples/custom_command_widgets.kdl`
- `share/yazelix_zellij_bar/examples/standalone_zellij_layout.kdl`
- `share/yazelix_zellij_bar/examples/yazelix_runtime_widgets.kdl`

The standalone child repo installs `zjstatus.wasm` from its pinned `zjstatus` flake input. This main repo makes that child input follow Yazelix's own `zjstatus` pin when forwarding `#yazelix_zellij_bar`, and the integrated Yazelix runtime consumes the same pinned upstream package artifact for its managed plugin path.

Non-Yazelix users can install the standalone bar directly:

```bash
nix profile install github:luccahuguet/yazelix-zellij-bar#yazelix_zellij_bar
```

From this repo, the forwarded package remains:

```bash
nix build .#yazelix_zellij_bar
```

Use the child README for Zellij layout examples and custom command-widget configuration. KDL is the customization surface for standalone users. Integrated Yazelix calls `yazelix_zellij_bar_widget render-yazelix-runtime --json <config>` to render the child-owned runtime KDL template into a complete zjstatus plugin block for generated layouts. The integrated template replaces zjstatus `{tabs}` with a child-owned `command_yazelix_tabs` widget that reads the launch-scoped status-bar cache once per second, so activity labels animate without making the pane orchestrator repaint tab names on every frame.

The standalone non-workspace widget commands are intentionally short:

```bash
yazelix_zellij_bar_widget codex
yazelix_zellij_bar_widget claude
yazelix_zellij_bar_widget opencode_go
yazelix_zellij_bar_widget cpu
yazelix_zellij_bar_widget ram
```

Workspace and integrated animated tabs remain Yazelix-only because they depend on pane-orchestrator session facts. The other widgets run without `yzx`, `yzx_control`, Nushell, Yazelix runtime cache paths, or Yazelix session state.
