# Zellij Layouts

Yazelix ships one managed sidebar family from the in-tree `rust_core/yazelix_zellij_config_pack` crate:

- `yzx_side.kdl` for sidebar mode
- `yzx_side.swap.kdl` for sidebar swap layouts

Set the file-open behavior in `settings.jsonc`:

```jsonc
{
  "editor": {
    "hide_sidebar_on_file_open": false
  },
  "workspace": {
    "left_sidebar": {
      "command": "yzx",
      "args": ["sidebar", "yazi"],
      "width_percent": 20
    },
    "right_sidebar": {
      "command": "yzx",
      "args": ["agent"],
      "width_percent": 40
    }
  }
}
```

`editor.hide_sidebar_on_file_open = true` hides the managed sidebar after opening a file from Yazi while keeping new tabs on the normal managed-sidebar startup layout. `Alt+Shift+H`, `Ctrl+y`, `Ctrl+Shift+Y`, and `yzx reveal` remain available because the managed side panes still exist.

The directional surface layer follows HJKL placement: `Alt+Shift+H` toggles the left sidebar, `Alt+Shift+J` toggles the bottom popup, `Alt+Shift+K` toggles the top popup, and `Alt+Shift+L` toggles the right agent sidebar.

`workspace.left_sidebar.width_percent` controls the open left sidebar width as a percentage of the tab. With the default launcher, that sidebar is the Yazi file tree. Valid range: `1` to `48`.

`workspace.right_sidebar.width_percent` controls the open right sidebar width as a percentage of the tab. The default right sidebar launches `yzx agent`, which starts host-installed `codex` when it is on `PATH` and otherwise opens a normal shell with setup guidance. Valid range: `1` to `48`.

`workspace.left_sidebar.command` / `args` and `workspace.right_sidebar.command` / `args` control the terminal side surfaces launched in the managed sidebar slots. Set `args` explicitly for tools that need them, such as `["status"]` for `lazygit status`. The right sidebar can run another agent or any non-agent terminal command. Custom launchers still run inside managed panes named `sidebar` and `agent`; the pane orchestrator keeps owning sidebar identity, focus, and layout state.

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

- Use `workspace.left_sidebar.*` and `workspace.right_sidebar.*` for custom side-surface launchers
- Edit `rust_core/yazelix_zellij_config_pack/layouts/yzx_side.kdl` to change startup panes
- Edit `rust_core/yazelix_zellij_config_pack/layouts/yzx_side.swap.kdl` to tweak built-in swap layouts

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
