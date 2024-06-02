# Yazi-Helix File Tree v2: Now with a Closeable Sidebar

### Base Layout
The initial layout includes four panes:
![Base Layout](https://github.com/luccahuguet/zellij-files/assets/27565287/adc6162c-a1ec-4635-b217-aa7a9ba691c5)
- **Tab-bar** at the top
- **Status-bar** at the bottom
- **Yazi pane** (20% width) acting as a sidebar on the left
- **Empty pane** on the right

### Swap Layout
When you create a fifth pane, you transition from the base layout:
- Open and close the sidebar by switching layouts.
- The fifth pane, on the left, is where new panes will appear, stacked.
- You can add more swap layouts as needed, using the KDL files in `layouts`.

**Two or more panes open with the sidebar open:**
![Sidebar Open](https://github.com/luccahuguet/yazi-files/assets/27565287/557eecbf-6eeb-48f9-8de4-252f78bda4fd)

**Two or more panes open with the sidebar closed:**
![Sidebar Closed](https://github.com/luccahuguet/zellij-files/assets/27565287/4f63de6e-4df7-452f-9877-90461071b673)

## Improvements Over v1
- **Sidebar Control:** Now you can open and close the sidebar.
- **Simplified Dependencies:** No more nushell dependency. Nushell is a beautiful table-centric cross-platform shell written in Rust, but the way I used it was an ugly hack.
- **Simpler Layout Files:** The KDL files are more streamlined.
- **Removes zjstatus Plugin:** The plugin had to be downloaded and configured, while adding nothing important.
- **Status-bar is Back:** Life without it isn't easy. The status-bar (help bar) makes this much more user-friendly.

## Instructions
1. Install [Yazi](https://github.com/sxyazi/yazi).
2. Install [Zellij](https://github.com/zellij-org/zellij).
3. Install [Helix](https://helix-editor.com).
4. Place the files from this repo in your `.config/zellij` folder.
5. This layout is named `stack_sidebar` and is pre-configured in my setup.
   - If you haven't copied my config, add `default_layout "stack_sidebar"` to your configuration.
6. Add your full path to `hx` in `scrollback_editor` in your `zellij/config.kdl`.
7. It should also work with Neovim, but I haven't tested it.
8. Feel free to open issues and PRs 😉

## Roadmap
### Future Enhancements
- **Simplify KDL Files:** Reduce code repetition.
- **Flexible Sidebar Control:** Enable sidebar toggling with just four panes.
- **Full Yazi pane:** Integrate a full Yazi pane in another swap layout (showing parents and preview, not just the current dir). [Learn more](https://github.com/luccahuguet/yazi-files)
  - This requires Yazi to accept config as an argument.
- **Higher Helix Integration:** Currently, selected files in Yazi open as a new pane in Zellij, running Helix. It would be nice to open them as a split or a buffer inside Helix (though this might be complex to code).

## Other Layouts
### `stack_sidebar_zjstatus` (Optional)
This layout offers an advanced tab-bar with more features (e.g., time display, current layout).
- **Setup:** Change the config to `default_layout "stack_sidebar_zjstatus"` and install [zjstatus](https://github.com/dj95/zjstatus), updating the path in your layout KDL file.
- **Use Case:** If you prefer a more powerful tab-bar without a status-bar, this layout is for you.
