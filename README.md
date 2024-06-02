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
- You can add more swap layouts as needed, in the kdl files in `layouts`.

**Two or more panes open with the sidebar open:**
![Sidebar Open](https://github.com/luccahuguet/yazi-files/assets/27565287/557eecbf-6eeb-48f9-8de4-252f78bda4fd)

**Two or more panes open with the sidebar closed:**
![Sidebar Closed](https://github.com/luccahuguet/zellij-files/assets/27565287/4f63de6e-4df7-452f-9877-90461071b673)

## Improvements Over v1
- **Sidebar Control:** Now you can open and close the sidebar.
- **Simplified Dependencies:** No more nushell dependency. Nushell is a beautiful table-centric cross-platform shell written in rust but the way I used it was an ugly hack.
- **Simpler Layout Files:** The KDL files are more streamlined.
- **Removes zjstatus plugin:** The plugin had to be downloaded and configured, while adding nothing important
- **status-bar is back, baby!:** Life without it ain't easy. The status-bar (help bar) makes this much more user friendly

## Instructions
- Install [yazi](https://github.com/sxyazi/yazi)
- Install [zellij](https://github.com/zellij-org/zellij)
- Install [helix](https://helix-editor.com)
- Place this repo files in your `.config/zellij` folder
- This layout is named `stack_sidebar`. It is pre-configured in my setup. 
- If you haven't copied my config, add `default_layout "stack_sidebar"` to your configuration.
- remember to add your full path to `hx` in `scrollback_editor` in your `zellij/config.kdl`
- It should also work with neovim, but I never tested
- Feel free to open issues and PRs

## Roadmap
### Future Enhancements
- **Simplify KDL Files:** Reduce code repetition.
- **Flexible Sidebar Control:** Enable sidebar toggling with just four panes.
- **Third Swap Layout:** Integrate a full Yazi pane (showing parents and preview). [Learn more](https://github.com/luccahuguet/yazi-files)
  - This requires Yazi to accept config as an argument.
- **Higher Helix Integration:** Currently the selected files in yazi will open as a new pane in zellij, running helix (which is how I use helix, I never used the splits that helix has out-of-the-box), but it would be nice to open it as a split or a buffer inside helix (sounds scary to code this)

## Other Layouts
### `stack_sidebar_zjstatus` (Optional)
This layout offers an advanced tab-bar with more features (e.g., time display, current layout). 
- **Setup:** Change the config to `default_layout "stack_sidebar_zjstatus"` and install [zjstatus](https://github.com/dj95/zjstatus), updating the path in your layout KDL file.
- **Use Case:** If you prefer a more powerful tab-bar without a status-bar, this layout is for you.

