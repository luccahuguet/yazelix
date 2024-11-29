# Yazelix v5
## Overview

Yazelix integrates yazi, zellij and helix, hence the name, get it?

- Zellij orchestrates everything, with yazi as a sidebar and helix as the editor
- You can open and close the sidebar by switching zellij layouts (press `alt ]` and `alt [`)
  - Or if you only got one pane open, make it fullscreen (`ctrl p + f` or `alt f`)
- Every keybinding from zellij that conflicts with helix is remapped (see them at the bottom)
- When you hit enter on a file/folder in the "sidebar" the following things happen:
  - If helix is already open, in a pane next to the sidebar, it will open that file/folder in a new buffer on that pane (magic)
  - If helix is not open, it will cd into the folder of the file (or the folder itself), and then open it in helix 
  - Note: It is highly recommended that you let the shell script execute in peace, so during these milliseconds don't move around
- This project holds my config files for zellij and yazi, almost like a plugin or something
  - But it's just some config files with a bit of shell scripting!

## Preview

![yazelix_v41_demo](https://github.com/user-attachments/assets/09a452e0-4a62-4e8e-afe6-2c7267f78b11)

## Improvements of v5 over v4.1
- Now when you open a new pane besides the sidebar pane, it will stack by default
- If you want to open a split, you can do so like this:
  - Type `ctrl p` then `r` for a split to the right
  - Type `ctrl p` then `d` for a split in the "down" direction
- New rule: to open a file from yazi into an existing yazelix buffer, the hx plane has to be the topmost

## Instructions to set it up

1. Make sure [yazi](https://github.com/sxyazi/yazi), [zellij](https://github.com/zellij-org/zellij), [helix](https://helix-editor.com), and [nushell](https://www.nushell.sh/book/installation.html) are installed and in your path
2. Just clone this repo in your `~/.config` dir
3. Take a look at the `configure wezterm` step on [https://github.com/luccahuguet/rustifier](https://github.com/luccahuguet/rustifier#installation) to see how to configure yazelix on wezterm  
    - If you use another shell, you have to configure it to run something like `nu -c "zellij -l welcome --config-dir ~/.config/yazelix/zellij options --layout-dir ~/.config/yazelix/zellij/layouts"` on startup  
    - or `zellij -l welcome --config-dir ~/.config/yazelix/zellij options --layout-dir ~/.config/yazelix/zellij/layout` (but you still need `nu` anyways)
    - Another option, if you wish, run this command manually every time you open yazelix  
    - The recommended shell is Wezterm though. More on that in the Notes section below  
4. Optional: Using zoxide enhances the yazelix experience ten-fold, let me tell ya... and it integrates with yazi

That's it, and feel free to open issues and PRs 😉

## Why use this project?

- This project is relatively simple to understand, the inner workings and all. Just a bit of shell scripting magic, but mostly config files
- Easy to configure and make it yours
- I daily drive this, and will change it according to my needs, keeping it updated and improving it
- Zero conflict keybindings, very powerful sidebar (learning yazi is a process, but you can do very cool stuff)

## Troubleshooting

- If it's not working, try upgrading yazi and zellij to the latest version

## Possible Improvements

- Yazelix will only detect helix if it's adjacent to the sidebar. A minor thing.
- When you open a new tab, yazi opens as single pane taking all space
  - But it does not show the parents and preview columns, it only shows the current dir column
  - To address this I would have to reopen yazi with a different config? 

## Keybinding remaps

| New Zellij Keybinding | Previous Keybinding | Helix Action that uses that previous key | Zellij Action remapped    |
|-----------------------|---------------------|------------------------------------------|-----------------------------|
| Ctrl e                | Ctrl o              | jump_backward                            | SwitchToMode "Session"      |
| Ctrl y                | Ctrl s              | save_selection                           | SwitchToMode "Scroll"       |
| Alt w                 | Alt i               | shrink_selection                         | MoveTab "Left"              |
| Alt q                 | Alt o               | expand_selection                         | MoveTab "Right"             |
| Alt m                 | Alt n               | select_next_sibling                      | NewPane                     |
| Alt 2                 | Ctrl b              | move_page_up                             | SwitchToMode "Tmux"         |

If you find a conflict, please open an issue. Keep in mind, though, that compatibility with tmux mode is not a goal of this project.

## Details: Base Layout

The initial layout includes one usable pane (actually 4, counting the tab-bar, status-bar and sidebar):
![image](https://github.com/luccahuguet/zellij/assets/27565287/c8333411-b6f4-4c0e-9ea8-1992859c8749)

- **Tab-bar** at the top
- **Status-bar** at the bottom
- **Yazi pane** (20% width) acting as a sidebar on the left
- **Empty pane** on the right
  
## Discoverability of keybindings
- zellij: zellij is great at this, works out of the box, you'll visually see all the keybindings in the status-bar
- helix: helix is the same honestly
- yazi: There is only one keybinding to remember: `~` This shows all keybindings and commands (press `alt f` to make the yazi pane fullscreen)
- nushell: you can run `tutor` on nushell, you can read the https://www.nushell.sh/book/, you can run `help commands | find regex` (if you want to learn about regex for example, but could be anything) 
    - well, I do use `ctrl r` a lot in nushell, it opens a interactive history search
  

## Keybindings tips 
- Type `alt f` to make your pane fullscreen
- Type `ctrl p` then `r` for a split to the right
- Type `ctrl p` then `d` for a split in the "down" direction
- Yazi: Type `z` to use zoxide (fuzzy find to known paths)
- Yazi: Type `Z` to use fzf (fuzzy find to unknown paths)

## Tips
- You can add more swap layouts as needed, using the KDL files in `layouts`.
- I recommend using wezterm as your terminal
  - because it can be configured to remove its native tabs, very extensible, including its keybindings (haven't found a conflict yet)
  - very performant
- If you test this with nvim and it works, let me know (see the issue [here](https://github.com/luccahuguet/zellij/issues/2))
- Special thanks to yazi's, zellij's and helix's contributors/maintainers! 
- Yazi's author graciously contributed some lua code to make yazi's status bar look awesome in the small width of a sidebar
  - Thanks for that!

## Contributing to Yazelix

See here in [contributing](./contributing.md)

## Similar projects

- [Shelix](https://github.com/webdev23/shelix) 
  - Shelix does intend to maximize the hidden power of Tmux as an IDE, enhance capabilities of the incredibly efficient Helix editor, around an interactive menu that performs IDE related actions 
  - as of 31/06/2024, it has been 4 months since last commit
- [Helix-Wezterm](https://github.com/quantonganh/helix-wezterm): Turning Helix into an IDE with the help of WezTerm and CLI tools
  - as of 31/06/2024, it has been 3 weeks since last commit
- [File tree picker in Helix with Zellij](https://yazi-rs.github.io/docs/tips/#helix-with-zellij) 
  - Yazi can be used as a file picker to browse and open file(s) in your current Helix instance (running in a Zellij session)
