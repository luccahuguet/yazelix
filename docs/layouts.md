# Zellij Layouts

Yazelix includes several swap layouts that automatically organize your workspace based on the number of panes. These layouts are defined in `configs/zellij/layouts/yazelix.swap.kdl`.

## Available Layouts

### `basic` (4 panes exactly)
- **Structure**: Sidebar (20%) + main pane (80%)
- **Best for**: Focused work with file navigation
- **Use cases**: Writing code, editing documents, simple tasks

### `stacked` (5+ panes)
- **Structure**: Sidebar (20%) + stacked panes (80%)
- **Best for**: Multiple files or tools in a stack
- **Use cases**: Comparing files, keeping multiple editors open

### `three_column` (5+ panes)
- **Structure**: Sidebar (20%) + stacked panes (40%) + vertical split (40%)
- **Best for**: Advanced workflows requiring multiple simultaneous views
- **Use cases**: 
  - **Claude Code**: Run AI assistant in the right pane while coding in the center
  - **Code review**: Compare files side-by-side with navigation in sidebar
  - **Development**: Editor in center, terminal/output in right pane

### `bottom_terminal` (5+ panes)
- **Structure**: Sidebar (20%) + horizontal split with stacked panes (70%) + bottom terminal (30%)
- **Best for**: IDE-like experience with persistent terminal access
- **Use cases**:
  - **Quick commands**: Run git, build, test commands without switching panes
  - **lazygit**: Keep git UI always visible in bottom pane
  - **Monitoring**: Watch logs or system status in dedicated terminal
  - **Development**: Code in top, immediate command access below

## Layout Switching

Layouts switch automatically based on pane count:
- **4 panes**: `basic` layout activates
- **5+ panes**: Other layouts become available through Zellij's swap functionality

Use Zellij's built-in keybindings to cycle through available layouts when you have 5+ panes.

## Customization

### Commenting Out Layouts
To disable a layout, comment out its entire block in `yazelix.swap.kdl`:
```kdl
// swap_tiled_layout name="bottom_terminal" {
//     ui min_panes=5 {
//         // ... layout definition
//     }
// }
```

### Adding Custom Layouts
Create new layouts by copying an existing one and modifying:
```kdl
swap_tiled_layout name="my_custom_layout" {
    ui min_panes=6 {
        pane split_direction="vertical" {
            // Your custom layout here
        }
    }
}
```

### Tweaking Existing Layouts
Modify pane sizes, split directions, or minimum pane requirements:
- Change `size "20%"` to adjust pane proportions
- Modify `min_panes=5` to change activation threshold
- Switch `split_direction="vertical"` to `"horizontal"` for different splits

## Tips

- **Use `Alt+f`** to toggle pane fullscreen and hide the layout temporarily
- **Experiment** with different layouts by opening more panes and tools
- **Customize** layouts to match your specific workflow needs
- **Consider pane count** when planning your workspace - layouts adapt automatically