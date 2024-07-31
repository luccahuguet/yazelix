# Yazelix v4: A true sidebar opens files in a helix buffer! 

### Overview
Yazelix integrates yazi, zellij and helix, hence the name, get it?

- Zellij orchestrates everything, with yazi as a sidebar and helix as the editor
- You can open and close the sidebar by switching zellij layouts (press `alt ]` and `alt [`)
  - Or if you only got one pane open, make it fullscreen (`ctrl p + f` or `alt f` )
- Every keybinding from zellij that conflicts with helix is remapped (see them at the bottom)
- Helix is called when you hit enter on a file in the "sidebar"
  - If helix is already open, in a pane next to the sidebar, it will open in a new buffer on that pane (magic)
- This project holds my config files for zellij and yazi, almost like a plugin or something
- But it's just some config files with a bit of shell scripting!

<br>

### Base Layout
The initial layout includes one usable pane (actually 4, counting the tab-bar, status-bar and sidebar):
![image](https://github.com/luccahuguet/zellij/assets/27565287/c8333411-b6f4-4c0e-9ea8-1992859c8749)

- **Tab-bar** at the top
- **Status-bar** at the bottom
- **Yazi pane** (20% width) acting as a sidebar on the left
- **Empty pane** on the right

<br>

### Swap Layouts Behavior
- Open and close the sidebar by switching layouts (you need at least two panes open to do that). 
- New panes will be appearing to the left, stacked, but this won't change the layout.
- You can add more swap layouts as needed, using the KDL files in `layouts`.

**Two or more panes open with the sidebar open:**
![Sidebar Open](https://github.com/luccahuguet/zellij/assets/27565287/8faf2bc4-7861-467a-8629-b41dc57fbab8)

**Two or more panes open with the sidebar closed:**
![Sidebar Closed](https://github.com/luccahuguet/zellij/assets/27565287/038ce337-dc79-415b-a137-1efcf21b0cf7)

<br>

### Improvements Over v3
- ROUNDED CORNERS
- A wish come true: when you hit enter in a file or folder in yazi, if helix is open in a pane next to yazi, it will open in a helix buffer
  - All it took was some shell scripting magic...
  - it will also change your working dir, so when you press `SPACE f` you open the picker in that folder you're actually in
- The repo was called `zellij` so people could just clone it in their `.config` folder directly, but this just sounded off. 
  - The project's name is yazelix, not zellij, after all.
  - So now you just ...[add instructions]
- New-tab layout has improved. Now new panes are just yazi in a 100% width pane, working sort of like a picker.
  - You just open a file or folder from yazi and it goes to it's proper place as a sidebar to the right
  
<br>

### Instructions to set it up
1. Make sure [yazi](https://github.com/sxyazi/yazi), [zellij](https://github.com/zellij-org/zellij) and [helix](https://helix-editor.com) are installed and in your path
2. Remove (or rename) your old `~/.config/zellij` folder, and just clone this repo in your `~/.config` dir
3. You can open this layout either from `zellij -l welcome` or directly `zellij -l ~/.config/zellij/layouts/yazelix`
  - I just set my terminal config to open zellij on startup, so I never leave zellij (my alacritty files [here](https://github.com/luccahuguet/alacritty-files))

That's it, and feel free to open issues and PRs 😉

<br>

### Why use this project?
- I think one of the main things is just how dead simple this project is. Just a bit shell scripting magic, but mostly config files
- Easy to configure and make it yours
- I daily drive this, and will change it according to my needs, keeping it updated and improving it
- zero conflict keybindings, very powerful sidebar (learning yazi is a process, but you can do a lot)

<br>

### Possible Improvements
- Yazelix will only detect helix if it's adjacent to the sidebar. Would be nice to fix this limitation.
- When you open a new tab, yazi opens as single pane taking all space
  - But it does not show the parents and preview columns (like here TODO), it only show the current dir column
- Rename the repo to yazelix: I did try that but using a custom path to the layout folder just didn't work with `~` or `$HOME` 
  - see some issues [here](https://github.com/zellij-org/zellij/issues/2764) and [here](https://github.com/zellij-org/zellij/issues/3115)
- 

<br>

### Keybinding remaps
| New Zellij Keybinding | Previous Keybinding  | Helix Action that uses that previous key | Zellij Action remaped       |
|-----------------------|----------------------|------------------------------------------|-----------------------------|
| Ctrl e                | Ctrl o               | jump_backward                            | SwitchToMode "Session"      |
| Ctrl y                | Ctrl s               | save_selection                           | SwitchToMode "Scroll"       |
| Alt w                 | Alt i                | shrink_selection                         | MoveTab "Left"              |
| Alt q                 | Alt o                | expand_selection                         | MoveTab "Right"             |
| Alt m                 | Alt n                | select_next_sibling                      | NewPane                     |
| Alt 2                 | Ctrl b               | move_page_up                             | SwitchToMode "Tmux"         |

If you find a conflict, please open an issue. Keep in mind, though, that compatibility with tmux mode is not a goal of this project.

<br>

### Notes
- I recommend running zellij from your shell (`nu -c "zellij -l welcome"` for nushell). 
  - This way you can load your enviroment variables like EDITOR and HELIX_RUNTIME
- I recommend using alacritty as your terminal
  - because it's a "dumb" terminal, it has no panes, no tabs. This means less keybindings conflicts to worry about, less feature overlap
  - very performant
  - but I do want to explore more modern options, so long as they have a "plain mode", like [this](https://raphamorim.io/rio/pt-br/docs/next/navigation#plain)
  - you can check out my alacritty files [here](https://github.com/luccahuguet/alacritty-files) (they include all alacritty themes)
- Use [nushell](https://www.nushell.sh/), it's a great shell, it's fast and beautiful and a proper programming language. Why wouldn't you?
- If you test this with nvim and it works, let me know (see the issue [here](https://github.com/luccahuguet/zellij/issues/2))
- Special thanks to yazi's, zellij's and helix's contributors/maintainers! 

<br>

### Similar projects
- [Shelix](https://github.com/webdev23/shelix): Shelix does intent to maximize the hidden power of Tmux as an IDE, enhance capabilities of the incredibly efficient Helix editor, around an interactive menu that performs IDE related actions
- [Helix-Wezterm](https://github.com/quantonganh/helix-wezterm):Turning Helix into an IDE with the help of WezTerm and CLI tools
- [File tree picker in Helix with Zellij](https://yazi-rs.github.io/docs/tips/#helix-with-zellij): Yazi can be used as a file picker to browse and open file(s) in your current Helix instance (running in a Zellij session)

