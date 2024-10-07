# Yazelix v4.1

### Overview
Yazelix integrates yazi, zellij and helix, hence the name, get it?
- Zellij orchestrates everything, with yazi as a sidebar and helix as the editor
- You can open and close the sidebar by switching zellij layouts (press `alt ]` and `alt [`)
  - Or if you only got one pane open, make it fullscreen (`ctrl p + f` or `alt f` )
- Every keybinding from zellij that conflicts with helix is remapped (see them at the bottom)
- Helix is called when you hit enter on a file in the "sidebar"
  - If helix is already open, in a pane next to the sidebar, it will open in a new buffer on that pane (magic)
  - Note: It is highly recommended that you let the shell script execute in peace, so during these miliseconds don´t move around
- This project holds my config files for zellij and yazi, almost like a plugin or something
  - But it's just some config files with a bit of shell scripting!

<br>

### Preview

![yazelix_v41_demo](https://github.com/user-attachments/assets/09a452e0-4a62-4e8e-afe6-2c7267f78b11)


### Improvements of the v4.1 over v4
- the open_file script is now written in nushell, with some modifications:
  - now working with files with spaces in the filename
  - now more sensitive to detecting hx on the next pane... previously it would sometimes not detect helix and thus open hx in a new pane instead of just opening the file in a new buffer the way it should)
  - now it cds into the folder of the file being opened, if you clicked on a file, or into the folder itself, if you clicked on a folder 
  - I do prefer quite a lot being able to write nushell instead of bash
- nushell is now a dependency (technically not an improvement, but feels like one, for me)


### Improvements of the v4 over v3
- ROUNDED CORNERS
- A wish come true: when you hit enter in a file or folder in yazi, if helix is open in a pane next to yazi, it will open in a helix buffer
  - All it took was some shell scripting magic...
  - it will also change your working dir, so when you press `SPACE f` you open the picker in that folder you're actually in
- New-tab layout has improved. Now new panes are just yazi in a 100% width pane, working sort of like a picker.
  - You just open a file or folder from yazi and it goes to it's proper place as a sidebar to the right
- Added a dedicate kb to make panes full screen `alt f`
- The repo was previously called `zellij` so people could just clone it in their `.config` folder directly, but this just sounded off. 
  - The project's name is yazelix, not zellij, after all. So now the repo name is yazelix the way god intended
  - Take a look at the new instructions to set it up just below!
  
<br>

### Instructions to set it up
1. Make sure [yazi](https://github.com/sxyazi/yazi), [zellij](https://github.com/zellij-org/zellij), [helix](https://helix-editor.com), and [nushell](https://www.nushell.sh/book/installation.html) are installed and in your path
2. Just clone this repo in your `~/.config` dir
3. Take a look at the `configure wezterm` step on https://github.com/luccahuguet/rustifier to see how it configure yazelix on wezterm
4. Optional: Using zoxide enhances the yazelix experience ten-fold, let me tell ya... and it integrates with yazi

That's it, and feel free to open issues and PRs 😉

<br>

### Why use this project?
- This project is relatively simple to understand, the inner workings and all. Just a bit shell scripting magic, but mostly config files
- Easy to configure and make it yours
- I daily drive this, and will change it according to my needs, keeping it updated and improving it
- Zero conflict keybindings, very powerful sidebar (learning yazi is a process, but you can do very cool stuff)

<br>

### Possible Improvements
- Yazelix will only detect helix if it's adjacent to the sidebar. A minor thing.
- When you open a new tab, yazi opens as single pane taking all space
  - But it does not show the parents and preview columns, it only shows the current dir column
  - To address this I would have to reopen yazi with a different config? 
- The opening of files in a helix buffer implementation works but feels like a workaround. But it does not matter much. Helix will get a plugin system and then a file tree plugin probably between the beginning and middle of 2025 anyaways.. 

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

### Details: Base Layout
The initial layout includes one usable pane (actually 4, counting the tab-bar, status-bar and sidebar):
![image](https://github.com/luccahuguet/zellij/assets/27565287/c8333411-b6f4-4c0e-9ea8-1992859c8749)

- **Tab-bar** at the top
- **Status-bar** at the bottom
- **Yazi pane** (20% width) acting as a sidebar on the left
- **Empty pane** on the right

<br>

### Notes
- You can add more swap layouts as needed, using the KDL files in `layouts`.
- I recommend using wezterm as your terminal
  - because it can be configured to remove its native tabs, very extensible, including its keybindings (haven't found a conflict yet)
  - very performant
- Use [nushell](https://www.nushell.sh/), it's a great shell, it's fast and beautiful and a proper programming language. Why wouldn't you?
- If you test this with nvim and it works, let me know (see the issue [here](https://github.com/luccahuguet/zellij/issues/2))
- Special thanks to yazi's, zellij's and helix's contributors/maintainers! 
- Yazi's author graciously contributed some lua code to make yazi's status bar look awesome in the small width of a sidebar
  - Thanks for that!

<br>

### Similar projects
- [Shelix](https://github.com/webdev23/shelix) 
  - Shelix does intend to maximize the hidden power of Tmux as an IDE, enhance capabilities of the incredibly efficient Helix editor, around an interactive menu that performs IDE related actions 
  - as of 31/06/2024, it has been 4 months since last commit
- [Helix-Wezterm](https://github.com/quantonganh/helix-wezterm):Turning Helix into an IDE with the help of WezTerm and CLI tools
  - as of 31/06/2024, it has been 3 weeks since last commit
- [File tree picker in Helix with Zellij](https://yazi-rs.github.io/docs/tips/#helix-with-zellij) 
  - Yazi can be used as a file picker to browse and open file(s) in your current Helix instance (running in a Zellij session)

