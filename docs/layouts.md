# Zellij Layouts

Yazelix includes several swap layouts that automatically organize your workspace based on the number of panes. These layouts are defined in `configs/zellij/layouts/yazelix.swap.kdl`.

**Note on Pane Counting**: Zellij internally counts the tab bar and status bar as "panes", but for user clarity, we refer to "user panes" (sidebar, editor, terminal, etc.) since those are what you actually interact with. So when we say "2 user panes", Zellij sees 4 total panes including its UI elements.

## Available Layouts

### `basic` / `stacked` (2+ user panes)
- **Structure**: Sidebar (20%) + main area (80%)
- **Behavior**: 
  - **2 user panes exactly**: Single main pane (`basic` layout)
  - **3+ user panes**: Main area becomes stacked automatically (`stacked` layout)
- **Best for**: General workflow with file navigation
- **Use cases**: Writing code, editing documents, comparing files, keeping multiple editors open
- **Note**: These are essentially the same layout - Zellij automatically stacks panes when you have more than 4

### `three_column` (3+ user panes)
- **Structure**: Sidebar (20%) + stacked panes (40%) + vertical split (40%)
- **Best for**: Advanced workflows requiring multiple simultaneous views
- **Use cases**: 
  - **Claude Code**: Run AI assistant in the right pane while coding in the center
  - **Code review**: Compare files side-by-side with navigation in sidebar
  - **Development**: Editor in center, terminal/output in right pane

### `bottom_terminal` (3+ user panes)
- **Structure**: Sidebar (20%) + horizontal split with stacked panes (70%) + bottom terminal (30%)
- **Best for**: IDE-like experience with persistent terminal access
- **Use cases**:
  - **Quick commands**: Run git, build, test commands without switching panes
  - **lazygit**: Keep git UI always visible in bottom pane
  - **Monitoring**: Watch logs or system status in dedicated terminal
  - **Development**: Code in top, immediate command access below

## Layout Switching

Layouts switch automatically based on user pane count (excluding tab bar and status bar):
- **2 user panes**: `basic` layout activates
- **3+ user panes**: Other layouts become available through Zellij's swap functionality

Use Zellij's built-in keybindings to cycle through available layouts when you have 3+ user panes.

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
- Modify `min_panes=5` to change activation threshold (remember: this includes tab bar + status bar, so 5 = 3 user panes)
- Switch `split_direction="vertical"` to `"horizontal"` for different splits

## Tips

- **Use `Alt+f`** to toggle pane fullscreen and hide the layout temporarily
- **Experiment** with different layouts by opening more panes and tools
- **Customize** layouts to match your specific workflow needs
- **Consider pane count** when planning your workspace - layouts adapt automatically