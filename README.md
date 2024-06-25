# Yazelix v3: Helix with a File Tree!

### Overview
Yazelix v3 integrates yazi, zellij and helix, hence the name, get it?
- Zellij orchestrates everything, with yazi as a sidebar and helix as the editor
- You can open and close the sidebar by switching zellij layouts (press `alt ]` and `alt [`)
- Every keybinding from zellij that conflicts with helix is remapped (see them at the bottom)
- Helix is called when you hit enter on a file in the "sidebar", opening as a new pane in zellij
  - If helix is called like that, that pane will be closed as well when you quit helix
  - Note: I recommend running zellij from your shell (`nu -c "zellij -l welcome"` for nushell). This way you can load your enviroment variables like EDITOR and HELIX_RUNTIME
- This project holds my config files for zellij and yazi, almost like a plugin or something, but it's just config!

<br>

### Base Layout
The initial layout includes one usable pane (actually 4, counting the tab-bar, status-bar and sidebar):
![image](https://github.com/luccahuguet/zellij/assets/27565287/c8333411-b6f4-4c0e-9ea8-1992859c8749)

- **Tab-bar** at the top
- **Status-bar** at the bottom
- **Yazi pane** (20% width) acting as a sidebar on the left
- **Empty pane** on the right

<br>

### Swap Layout
When you create a second pane (actually the fifth), you transition to the swap layouts:
- Open and close the sidebar by switching layouts.
- The fifth pane, on the left, is where new panes will appear, stacked.
- You can add more swap layouts as needed, using the KDL files in `layouts`.

**Two or more panes open with the sidebar open:**
![Sidebar Open](https://github.com/luccahuguet/zellij/assets/27565287/8faf2bc4-7861-467a-8629-b41dc57fbab8)

**Two or more panes open with the sidebar closed:**
![Sidebar Closed](https://github.com/luccahuguet/zellij/assets/27565287/038ce337-dc79-415b-a137-1efcf21b0cf7)

<br>

### Improvements Over v2
- Before, the yazi config files were in a separate repo, now its all integrated here!  
  - Thanks to Zykino from Zellij's discord for that tip!
- Yazi's maintainer (what an honor!) added a init.lua file that makes the status-bar in yazi look really good in the small width it has
- The project's got a name! Yazelix. It simply had no name before and that was a mistake.
- This one is great: I've remapped 6 keybindings from zellij to avoid conflicts with helix 
  - use `alt m` for new panes and the rest is in zellij's status-bar 
  - this is configured in the `layouts/yazelix.kdl` file, if you want to change something 

<br>

### Instructions to set it up
1. Make sure [yazi](https://github.com/sxyazi/yazi), [zellij](https://github.com/zellij-org/zellij) and [helix](https://helix-editor.com) are installed and in your path
2. Remove (or rename) your old `~/.config/zellij` folder, and just clone this repo in your `~/.config` dir
3. You can open this layout either from `zellij -l welcome` or directly `zellij -l ~/.config/zellij/layouts/yazelix`
  - I just set my terminal config to open zellij on startup, so I never leave zellij (my alacritty files [here](https://github.com/luccahuguet/alacritty-files))

That's it, and feel free to open issues and PRs 😉

<br>

### Why use this project?
- I think one of the main things is just how dead simple to configure this project is. No shell scripting magic
- Easy to configure and make it yours
- I daily drive this, and will change it according to my needs, keeping it updated and improving it
- Even if you don't care about the sidebar, the keybindings may be helpful

<br>

### Possible Improvements
- **More sidebar action:** Sidebar should open and close with only one pane as well
  - This was not working because whenever I close the second (actually fifth) pane, the sidebar and other pane swap
- **Full Yazi pane:** Integrate a full Yazi pane in another swap layout showing parents and preview, not just the current dir
  - this is already implemented, but has a few kinks to iron out.
  - To test, uncomment the yazi_full swap layout and panes, and increase the panes constraints by one
  - basically some panes swap with others when they shouldn't and you have to "walk" through the closed pane, which isn't great
  - Big thanks to zellij's maintainer and other people for helping with this
- **Higher Helix Integration:** Currently, selected files in Yazi open as a new pane in Zellij, running Helix. It would be nice to open them as a split or a buffer inside Helix (though this might be complex to code).
- **Rename the repo to yazelix:** I did try that but using a custom path to the layout folder just didn't work with `~` or `$HOME` (see some issues [here](https://github.com/zellij-org/zellij/issues/2764) and [here](https://github.com/zellij-org/zellij/issues/3115)

<br>

### Keybinding remaps
| New Key Combination | Previous Key Combination | Helix Action that uses that previous key | Zellij Action remaped       |
|---------------------|--------------------------|------------------------------------------|-----------------------------|
| Ctrl + e            | Ctrl + o                 | jump_backward                            | SwitchToMode "Session"      |
| Alt 1               | Ctrl + s                 | save_selection                           | SwitchToMode "Scroll"       |
| Alt w               | Alt + i                  | shrink_selection                         | MoveTab "Left"              |
| Alt q               | Alt + o                  | expand_selection                         | MoveTab "Right"             |
| Alt m               | Alt + n                  | select_next_sibling                      | NewPane                     |
| Alt 2               | Ctrl + b                 | move_page_up                             | SwitchToMode "Tmux"         |

If you find a conflict, please open an issue. Keep in mind, though, that compatibility with tmux mode is not a goal of this project.

<br>

### Notes
- I recommend using alacritty as your terminal
  - because it's a "dumb" terminal, it has no panes, no tabs. This means less keybindings conflicts to worry about, less feature overlap
  - very performant
  - but I do want to explore more modern options, so long as they have a "plain mode", like [this](https://raphamorim.io/rio/pt-br/docs/next/navigation#plain)
  - you can check out my alacritty files [here](https://github.com/luccahuguet/alacritty-files) (they include all alacritty themes)
- Use [nushell](https://www.nushell.sh/), it's a great shell, it's fast and beautiful and a proper programming language. Why wouldn't you?
- If you test this with nvim and it works, let me know (see the issue [here](https://github.com/luccahuguet/zellij/issues/2))
- Special thanks to yazi's and zellij's maintainer (and discord members) for their help with some stuff. Also shoutout to helix's contributors!

<br>

### Similar projects
- [Shelix](https://github.com/webdev23/shelix): Shelix does intent to maximize the hidden power of Tmux as an IDE, enhance capabilities of the incredibly efficient Helix editor, around an interactive menu that performs IDE related actions
- [Helix-Wezterm](https://github.com/quantonganh/helix-wezterm):Turning Helix into an IDE with the help of WezTerm and CLI tools
- [File tree picker in Helix with Zellij](https://yazi-rs.github.io/docs/tips/#helix-with-zellij): Yazi can be used as a file picker to browse and open file(s) in your current Helix instance (running in a Zellij session)

