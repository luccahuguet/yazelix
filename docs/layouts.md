# Zellij Layouts

Yazelix ships managed-sidebar startup layouts, the historical no-sidebar layout family, one sweep-test layout, and three swap-layout files:

- `configs/zellij/layouts/yzx_side.kdl` for sidebar mode
- `configs/zellij/layouts/yzx_side_closed.kdl` for managed-sidebar mode with the sidebar initially collapsed
- `configs/zellij/layouts/yzx_no_side.kdl` for the historical no-sidebar family
- `configs/zellij/layouts/yzx_side.swap.kdl` for sidebar swap layouts
- `configs/zellij/layouts/yzx_side_closed.swap.kdl` for the collapsed-start sidebar swap layouts
- `configs/zellij/layouts/yzx_no_side.swap.kdl` for no-sidebar swap layouts
- `configs/zellij/layouts/yzx_sweep_test.kdl` for terminal sweep validation

Set the startup mode in `yazelix.toml`:

```toml
initial_sidebar_state = "open"  # Use "closed" to start tabs collapsed
sidebar_width_percent = 20
sidebar_command = "nu"
sidebar_args = ["__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu"]
```

`editor.initial_sidebar_state = "closed"` starts new tabs with the managed sidebar collapsed while keeping `Alt+y`, `Ctrl+y`, and `yzx reveal` available. Legacy configs with `enable_sidebar = false` are treated as a request for the collapsed managed-sidebar startup layout.

`editor.sidebar_width_percent` controls the open Yazi sidebar width as a percentage of the tab. Valid range: `10` to `40`.

`editor.sidebar_command` and `editor.sidebar_args` control the terminal side-surface launched in the managed sidebar slot. The default remains the Yazelix-managed Yazi adapter. When `sidebar_command` changes and `sidebar_args` is left at the default Yazi adapter path, Yazelix renders the custom sidebar command with no inherited args. Set `sidebar_args` explicitly for tools that need them, such as `["status"]` for `lazygit status`. Custom launchers still run inside the pane named `sidebar`; the pane orchestrator keeps owning sidebar identity, focus, and layout state.

## Layout Metadata

The built-in layout family contract lives in `config_metadata/zellij_layout_families.toml`

That file is the machine-readable source for:

- sidebar, collapsed-sidebar startup, and no-sidebar layout ids
- the startup KDL file for each family
- the swap-layout KDL file for each family
- required managed pane names
- required side-surface launcher placeholders
- the swap layout names that Yazelix family-aware controls expect

Run `yzx_repo_validator validate-workspace-session-contract` after changing built-in layout files or layout metadata

## Supported Customization

Yazelix now copies every top-level `.kdl` file in `configs/zellij/layouts/` into the generated runtime layout directory on launch. That means adding a new top-level layout file is supported without updating a hardcoded copy list.

The supported customization paths are:

- Use `editor.sidebar_command` and `editor.sidebar_args` for custom side-surface launchers
- Edit `yzx_side.kdl`, `yzx_side_closed.kdl`, or `yzx_no_side.kdl` to change startup panes
- Edit `yzx_side.swap.kdl`, `yzx_side_closed.swap.kdl`, or `yzx_no_side.swap.kdl` to tweak built-in swap layouts
- Add a new top-level `.kdl` file in `configs/zellij/layouts/` if you also add it to `config_metadata/zellij_layout_families.toml`
- Add custom no-sidebar swap layouts inside `yzx_no_side.swap.kdl`

## Important Boundary

Custom sidebar layout families are not fully first-class yet.

The sidebar-aware controls `Alt+y`, `Alt+[`, and `Alt+]` still understand only the built-in sidebar open/closed families defined by Yazelix. If you add a brand-new sidebar family, Zellij can still parse the layout file, but Yazelix family-aware switching and sidebar toggling will not automatically learn it.

So the current rule is:

- top-level custom layouts: supported
- custom no-sidebar swap layouts: supported
- brand-new sidebar swap families: not yet first-class

## Tips

- Use `Alt+Shift+F` to toggle pane fullscreen temporarily
- Keep custom launch layouts as top-level `.kdl` files under `configs/zellij/layouts/`
- Keep sidebar-family changes inside the built-in families unless you are also updating the pane orchestrator
