# Yazelix v2: Zellij Files

### Overview

Yazelix v2 integrates yazi, zellij and helix in a smooth experience.
- Zellij manages everything, with yazi as a sidebar and helix as the editor
- And helix is called when you select a file in the "sidebar", opening as a new pane in zellij
- You can open and close the sidebar by switching zellij layouts (press `alt ]` and `alt [`)

### Base Layout
The initial layout includes one usable pane, but actually 4 in total:
![Base Layout](https://github.com/luccahuguet/zellij-files/assets/27565287/adc6162c-a1ec-4635-b217-aa7a9ba691c5)
- **Tab-bar** at the top
- **Status-bar** at the bottom
- **Yazi pane** (20% width) acting as a sidebar on the left
- **Empty pane** on the right

### Swap Layout
When you create a second pane (actuall the fifth), you transition to the swap layouts:
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
- **Removes zjstatus Plugin:** The plugin had to be downloaded and configured, while adding nothing really important.
- **Status-bar is Back, baby!:** Life without it isn't easy. The status-bar (help bar) makes the setup much more user-friendly.

## Instructions to set it up
1. Install [Yazi](https://github.com/sxyazi/yazi).
2. Install [Zellij](https://github.com/zellij-org/zellij).
3. Install [Helix](https://helix-editor.com).
4. Place the files from this [yazi repo](https://github.com/luccahuguet/yazi-files) in your `.config/yazi` folder.
5. Place the files from this repo in your `.config/zellij` folder.
6. This layout is named `stack_sidebar` and is pre-configured in my setup.
   - If you haven't copied my config, add `default_layout "stack_sidebar"` to your configuration.
7. Add your full path to `hx` in `scrollback_editor` in your `zellij/config.kdl`.
8. It should also work with Neovim, but I haven't tested it.
9. Feel free to open issues and PRs 😉

## Roadmap
### v3
- [x] **Names the project:** The project is now called Yazelix! (get it?)
- [x] **Better yazi statusbar:** An actual contribution by the creator of yazi!! [Learn More](https://github.com/luccahuguet/yazi-files)
- [x] **More sidebar action:** Sidebar should open and close with only one pane as well
- [x] **Full Yazi pane:** Integrate a full Yazi pane in another swap layout (showing parents and preview, not just the current dir)
- [ ] **Helix-friendly Remap:** I'll definitely add a few remaps to zellij, so that it does not conflict with helix [Learn more](https://zellij.dev/documentation/layouts-with-config)

### Future Enhancements
- **Simplify KDL Files:** Reduce code repetition.
- **Higher Helix Integration:** Currently, selected files in Yazi open as a new pane in Zellij, running Helix. It would be nice to open them as a split or a buffer inside Helix (though this might be complex to code).

## Why use this project?
- I think one of the main things is just how dead simple to configure this project is. No shell scripting magic
- Easy to configure and make it yours
- I daily drive this, and will change it according to my needs, keeping it updated and improving it

## Similar projects
- [Shelix](https://github.com/webdev23/shelix): Shelix does intent to maximize the hidden power of Tmux as an IDE, enhance capabilities of the incredibly efficient Helix editor, around an interactive menu that performs IDE related actions
- [Helix-Wezterm](https://github.com/quantonganh/helix-wezterm):Turning Helix into an IDE with the help of WezTerm and CLI tools
- [File tree picker in Helix with Zellij](https://yazi-rs.github.io/docs/tips/#helix-with-zellij): Yazi can be used as a file picker to browse and open file(s) in your current Helix instance (running in a Zellij session)

## Other Layouts
### `stack_sidebar_zjstatus` (Optional) (does not feature v2 improvements)
This layout offers an advanced tab-bar with more features (e.g., time display, current layout).
- **Setup:** Change the config to `default_layout "stack_sidebar_zjstatus"` and install [zjstatus](https://github.com/dj95/zjstatus), updating the path in your layout KDL file.
- **Use Case:** If you prefer a more powerful tab-bar without a status-bar, this layout is for you.
