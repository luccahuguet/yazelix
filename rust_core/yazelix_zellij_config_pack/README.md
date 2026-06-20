# yazelix_zellij_config_pack

Deterministic Zellij config and layout renderer for Yazelix.

The package takes explicit render input and returns rendered `config.kdl` plus built-in layout files. It does not discover Yazelix config paths, generated state roots, live Zellij sessions, Home Manager state, or sibling source checkouts.

The renderer lives in the main Yazelix repository as an internal workspace crate. `yazelix_core` still owns product policy: settings normalization, runtime paths, plugin artifact resolution, generated file writes, doctor/repair, and workspace/session behavior.

## CLI

Render from JSON on stdin:

```bash
yazelix_zellij_config_pack < request.json > output.json
```

Print the renderer schema version:

```bash
yazelix_zellij_config_pack --schema-version
```

## Package

The flake exposes:

- `.#yazelix_zellij_config_pack`
- `.#default`

The package installs:

- `bin/yazelix_zellij_config_pack`
- `share/yazelix_zellij_config_pack/layouts`
- `share/yazelix_zellij_config_pack/config_metadata/zellij_layout_families.toml`
