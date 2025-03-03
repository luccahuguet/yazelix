# Yazelix v6: The POWER of yazi PLUGINS! Lua and nushell, unite! 

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
v4.1 preview (obs: currect v5 flow is better)


![image](https://github.com/user-attachments/assets/46f3f3a8-3c03-47e1-8cbd-cec30f293225)
v5 layout

## Improvements of v6 over v5
- WARNING: After upgrading to latest version of yazelix, kill old yazi instances and terminals to avoid conflicts. For good measure.
- Adds a yazi plugin to make the status bar in the yazi pane look good again, unclutered, and with a cool color
- Adds ghostty config. Author also switched to ghostty as a daily driver, but yazelix should work with any terminal (like wezterm that i used in the past). If it does not work, please open an issue!
- Now whenever you open yazelix's yazi, it knows whether to open yazi full pane (showing parent and preview sections) or yazi in sidebar mode by considering the width available
- And you can seamlessly switch between full-yazi and sidebar-yazi them by pressing `b`! It uses the [toggle-pane](https://github.com/yazi-rs/plugins/tree/main/toggle-pane.yazi) yazi plugin under the hood!
- The default behavior is now, when you open a new tab, it opens full-yazi, and when you open a file it goes to sidebar mode automatically!
- The item above was implemented via 'ya emit-to {yazi_id} {commmand}'. Very cool stuff! 
- Adds yet another plugin called [git](https://github.com/yazi-rs/plugins/tree/main/git.yazi) that shows file changes on the yazi sidebar. Increadibly helpful!
- Reveal-in-yazi command added. Pressing `alt y` in helix will reaveal the file in yazi. See how to set it up [here](<README#Yazelix Custom Keybindings>). LIMITATION: currently it only works for helix instances you opened from yazi (easy: open them only from yazi)
- I recommend making yazelix's yazi config your default (since it's plugin enhanced, and changes layout based on width):

For nushell users, add this to env.nu file (you can run `config env` to open it): 
```
$env.YAZI_CONFIG_HOME = "~/.config/yazelix/yazi"
```

this way if you kill your yazi sidebar you can get it back with just `yazi`, instead of `env YAZI_CONFIG_HOME=~/.config/yazelix/yazi yazi`. Yazelix's yazi knows whether to open in sidebar mode or full mode, comes with the git and toggle-pane plugins
but it's up to you and will work well either way! 

## Compatibility
- Should work with any terminal emulator, but I the ones I use more are wezterm and ghostty
- editor: helix (for now?)
- Take a look at the [table of versions](<README#Table of Versions>)


## Instructions to set it up
1. Make sure [yazi-fm and yazi-cli](https://github.com/sxyazi/yazi), [zellij](https://github.com/zellij-org/zellij), [helix](https://helix-editor.com), [nushell](https://www.nushell.sh/book/installation.html), and [zoxide](https://github.com/ajeetdsouza/zoxide) are installed and in your path.
Tip: if you use [cargo-update](https://github.com/nabijaczleweli/cargo-update), you can later use `cargo install-update -a` to update all your cargo tools at once, and fast! Because it often install binaries directly:
  ```
  cargo install cargo-update
  cargo install-update -i zellij nu yazi-cli yazi-fm zoxide gitui # gitui is optional
  ```
2. Just clone this repo in your `~/.config` dir:
  ```
  git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
  ```
3. Configure WezTerm:  
  ```
  cp ~/.config/yazelix/terminal_configs/wez/.wezterm.lua ~/.wezterm.lua
  ```
  Or Ghostty:
   ```
  cp ~/.config/yazelix/terminal_configs/ghostty/config ~/.config/ghostty/config
  ```
  Or, for other terminal emulators, you have to configure it to run something like
  ```
  nu -c "zellij -l welcome --config-dir ~/.config/yazelix/zellij options --layout-dir ~/.config/yazelix/zellij/layouts"
  ```
  I run it with `nu -c` because nushell loads my environment variables


Notes:
  - Feel free to change the configs and make it yours, this is just a starting point
  - For extra configuration, visit: https://wezfurlong.org/wezterm/config/files.html or https://ghostty.org/docs/config

That's it, and feel free to open issues and PRs 😉

## Why use this project?

- Easy to configure and make it yours
- I daily drive this, and will change it according to my needs, keeping it updated and improving it
- Zero conflict keybindings (i dont like having to lock zellij), very powerful sidebar (learning yazi is a process, but you can do very cool stuff)
- Uses cool yazi plugins out of the box

## Troubleshooting

- If it's not working, try upgrading yazi and zellij to the latest version
- Check the [versions table](<README#Table of Versions>)

## Keybinding remaps

| New Zellij Keybinding | Previous Keybinding | Helix Action that gets liberated!        | Zellij Action remapped      |
|-----------------------|---------------------|------------------------------------------|-----------------------------|
| Ctrl e                | Ctrl o              | jump_backward                            | SwitchToMode "Session"      |
| Ctrl y                | Ctrl s              | save_selection                           | SwitchToMode "Scroll"       |
| Alt w                 | Alt i               | shrink_selection                         | MoveTab "Left"              |
| Alt q                 | Alt o               | expand_selection                         | MoveTab "Right"             |
| Alt m                 | Alt n               | select_next_sibling                      | NewPane                     |
| Alt 2                 | Ctrl b              | move_page_up                             | SwitchToMode "Tmux"         |

If you find a conflict, please open an issue. Keep in mind, though, that compatibility with tmux mode is not a goal of this project.


## Discoverability of keybindings
- zellij: zellij is great at this, works out of the box, you'll visually see all the keybindings in the status-bar
- helix: helix is the same as zellij in that aspect honestly
- yazi: There is only one keybinding to remember: `~` This shows all keybindings and commands (press `alt f` to make the yazi pane fullscreen and view it better)
- nushell: you can run `tutor` on nushell, you can read the https://www.nushell.sh/book/, you can run `help commands | find regex` (if you want to learn about regex for example, but could be anything) 
    - well, I do use `ctrl r` a lot in nushell, it opens a interactive history search
  

## Yazelix Custom Keybindings
- Zellij: Type `alt f` to make your pane fullscreen (and back)
- Yazi: Type `b` to toggle seamlessly between full-yazi and sidebar-yazi (whether it shows parent and preview dirs)
- Helix: Type `alt y` to reaveal the file from helix buffer in yazi. Add this to your helix config:
```toml
[keys.normal]
A-y = ":sh nu ~/.config/yazelix/nushell/reveal_in_yazi.nu \"%{buffer_name}\""
````


## Keybindings tips 
- Zellij: Type `ctrl p` then `r` for a split to the right
- Zellij: Type `ctrl p` then `d` for a split in the "down" direction
- Yazi: Type `z` to use zoxide (fuzzy find to known paths)
- Yazi: Type `Z` to use fzf (fuzzy find to unknown paths)
- Yazi: Type `SPACE` to select files
- Yazi: Type `y` to `yank` and `Y` to `unyank` (cancels the copy)
- Yazi: Type `x` to `cut` and `X` to `uncut` (cancels the cut)
- Yazi: Type `a` to `add` a file (`filename.ext`) or a folder (`foldername/`)


## Tips
- You can add more swap layouts as needed, using the KDL files in `layouts`.
- I recommend using ghostty or wezterm as your terminal
  - they are very extensible and performant
- Special thanks to yazi's, zellij's and helix's contributors/maintainers! 
- Yazi's author graciously contributed some lua code to make yazi's status bar look awesome in the small width of a sidebar

## Im lost, it's too much information

In this case, learn how to use zellij on it's own first. And then optionally yazi. And then re-read the readme.

## Contributing to Yazelix

See here in [contributing](./contributing.md)

## Table of Versions
  - Last tested Frebuary 28th, 2025
  - Should work with older versions, but these are the ones I tested with
  - Should work with other terminal emulators (i've used alacritty in the past)

| Component          | Version                  |
| ------------------ | ------------------------ |
| OS                 | Pop!_OS 24.04            |
| DE                 | COSMIC                   |
| Zellij             | 0.41.2                   |
| Helix (from source)| helix 25.01.1 (0efa8207) |
| Nushell            | 0.102.0                  |
| Zoxide             | 0.9.7                    |
| Yazi               | 25.2.26                  |
| WezTerm            | 20240203-110809-5046fc22 |
| Ghostty            | 1.1.2                    |
| ya (from yazi-cli) | 25.2.26                  |

