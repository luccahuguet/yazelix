# Cargo-Based Installation for Yazelix v6.4

This guide provides instructions for installing Yazelix v6.4 using `cargo`, a straightforward Rust-based setup that supports any terminal emulator, including WezTerm and Ghostty. For the recommended Nix-based installation, see the [main README](../README.md).

## Steps

1. Install the required dependencies using `cargo` (you may prefer other methods, e.g., system package managers, for some dependencies):
   ```bash
   cargo install cargo-update cargo-binstall # this first line makes the installation waaaaay faster
   cargo install-update -i yazi-fm yazi-cli zellij nu
   cargo install-update -a # will update everything whenever you want
   ```
   Optionally, install additional tools to enhance Yazelix:
   ```bash
   cargo install-update -i zoxide lazygit starship
   ```
   Install optional Yazi-enhancing dependencies (e.g., for media previews, search, archives) using your system package manager:

   <details>
   <summary>Ubuntu/Pop!_OS</summary>

   ```bash
   sudo apt install ffmpeg p7zip-full jq poppler-utils fd-find ripgrep imagemagick
   ```
   </details>

   <details>
   <summary>Fedora</summary>

   ```bash
   sudo dnf install ffmpeg-free p7zip jq poppler-utils fd-find ripgrep ImageMagick
   ```
   </details>

   <details>
   <summary>Arch Linux</summary>

   ```bash
   sudo pacman -S ffmpeg p7zip jq poppler fd ripgrep imagemagick
   ```
   </details>

2. Clone this repo into your `~/.config` directory:
   ```bash
   git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
   ```

3. Configure your terminal emulator:
   - For WezTerm:
     ```bash
     cp ~/.config/yazelix/terminal_configs/wez/.wezterm.lua ~/.wezterm.lua
     ```
   - For Ghostty:
     ```bash
     cp ~/.config/yazelix/terminal_configs/ghostty/config ~/.config/ghostty/config
     ```
   - For other emulators, configure to run something like:
     ```bash
     "nu -c 'zellij --config-dir ~/.config/yazelix/zellij attach --create yazelix_ghostty options --default-layout yazelix'"
     ```

4. (Optional) Make Yazelix’s Yazi config your default:
   - For Nushell users, add to `~/.config/nushell/env.nu` (edit with `config env`):
     ```nushell
     $env.YAZI_CONFIG_HOME = "~/.config/yazelix/yazi"
     ```

## Notes
- The required dependencies (`yazi-fm`, `yazi-cli`, `zellij`, `nu`, `fzf`, `cargo-update`, `cargo-binstall`) are essential for Yazelix's core functionality.
- Optional tools (`zoxide`, `lazygit`, `starship`) and dependencies (`ffmpeg`, `p7zip`, `jq`, `poppler`, `fd`, `ripgrep`, `imagemagick`) enhance Yazi with features like smart navigation, Git integration, a customizable prompt, and media/archive support.
- For extra terminal emulator configuration, see: [WezTerm Docs](https://wezfurlong.org/wezterm/config/files.html) or [Ghostty Docs](https://ghostty.org/docs/config).
- Run `~/.config/yazelix/start-yazelix.sh` to launch Yazelix in Zellij.


