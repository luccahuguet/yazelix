# Zellij Layouts

Yazelix ships one managed sidebar family from the in-tree `rust_core/yazelix_zellij_config_pack` crate:

- `yzx_side.kdl` for sidebar mode
- `yzx_side.swap.kdl` for sidebar swap layouts

Configure the managed agent command in `config.toml`:

```toml
[agent]
command = "auto"
args = []
```

The Classic bridge keeps the packaged left Yazi sidebar and current right agent pane fixed. `agent.command = "auto"` uses provider discovery; set one executable and its argument list to use a different agent command. Sidebar command, width, and file-open hiding fields are retired rather than carried into the Nova root

The directional surface layer follows HJKL placement: `Alt+Shift+H` toggles the left sidebar, `Alt+Shift+J` toggles the bottom popup, `Alt+Shift+K` toggles the top popup, and `Alt+Shift+L` toggles the right agent sidebar.

The final Classic runtime still implements `Alt+Shift+L` through its existing right agent pane. The source swap changes that surface to Nova's managed agent popup. `Ctrl+Shift+Y` is therefore a Classic-only focus shortcut and is not part of the Nova root contract

## Layout Ownership

The built-in layout templates live in the in-tree `rust_core/yazelix_zellij_config_pack` crate. Main Yazelix consumes the pure renderer and validates generated layout freshness through the crate's bundled template names.

The in-tree config-pack crate is the machine-readable source for:

- sidebar startup layout ids
- the startup KDL file for each family
- the swap-layout KDL file for each family
- required managed pane names
- required side-surface launcher placeholders
- the swap layout names that Yazelix family-aware controls expect

After changing built-in layouts, run `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_zellij_config_pack` and `yzx_repo_validator validate-workspace-session-contract`

## Supported Customization

The supported customization paths are:

- Use `agent.command` and `agent.args` for the managed agent command
- Use native `~/.config/yazelix/zellij/config.kdl` for supported Zellij preferences
- Edit the in-tree config-pack layouts only when maintaining the packaged Classic layout

Yazelix does not currently expose a second declarative layout-profile language. Keep complex custom layout work in KDL so the generated runtime, Zellij, and the workspace contract share the same source files.

## Important Boundary

Custom sidebar layout families are not fully first-class yet.

The sidebar-aware controls understand the packaged built-in sidebar family defined by Yazelix. If you add a brand-new sidebar family, Zellij can still parse the layout file, but Yazelix family-aware switching and sidebar toggling will not automatically learn it.

`Alt+[` and `Alt+]` are reserved for previous/next layout-family cycling. Because the packaged runtime ships one managed sidebar family, pressing those bindings usually keeps the visible layout unchanged.

So the current rule is:

- top-level custom layouts: supported
- brand-new sidebar swap families: not yet first-class

## Tips

- Use `Alt+Shift+F` to toggle pane fullscreen temporarily
- Keep custom launch layouts in the in-tree config-pack crate when they are meant to ship with Yazelix
- Keep sidebar-family changes inside the built-in families unless you are also updating the pane orchestrator
