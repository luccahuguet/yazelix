# Yazelix Bar

`yazelix_bar` is the standalone Zellij/zjstatus bar preset extracted to [yazelix-bar](https://github.com/luccahuguet/yazelix-bar)

Regular Yazelix users do not need to install it separately. The normal Yazelix package already consumes the child crate for tab/status rendering and forwards the standalone package as `#yazelix_bar`

The child package installs:

- `bin/yazelix_bar_generate`
- `share/yazelix_bar/zjstatus.wasm`
- `share/yazelix_bar/yazelix_bar.kdl`
- `share/yazelix_bar/yazelix_bar.template.kdl`
- `share/yazelix_bar/generated/yazelix_bar.kdl`
- `share/yazelix_bar/examples/custom_command_widgets.kdl`
- `share/yazelix_bar/examples/yazelix_runtime_widgets.kdl`

Non-Yazelix users can install the standalone bar directly:

```bash
nix profile install github:luccahuguet/yazelix-bar#yazelix_bar
```

From this repo, the forwarded package remains:

```bash
nix build .#yazelix_bar
```

Use the child README for Zellij layout examples, generator options, and custom command-widget configuration
