# Yazelix Installation Guide

This guide provides complete step-by-step instructions for installing and setting up Yazelix.

## What is Nix?

Nix is just a package manager that ensures reproducible, reliable software installations. Think of it like a super-powered version of `apt`, `brew`, or `chocolatey` that:
- Never breaks your system (installs are isolated)
- Allows multiple versions of the same software
- Makes it easy to share exact development environments
- Can completely uninstall without leaving traces

## Why does Yazelix use Nix?

It guarantees that everyone gets the exact same versions of tools (Yazi, Zellij, Helix, etc.) that work perfectly together, regardless of your operating system or existing software. And it's way easier than having to install everything separately and manually.

**Important**: You don't need to learn Nix or Nushell to use Yazelix! Nix just installs the tools and yazelix uses nushell internally, and you can use your preferred shell (bash, fish, zsh, or nushell) for your daily work. You can install nix and nushell once, and forget they ever existed.

## Prerequisites

### Nushell
Required to run yazelix, used internally (but you can use any of our supported shells)

For Nix users, install Nushell with modern profile commands:

```bash
nix profile add nixpkgs#nushell
```

Other platforms and package managers: see official instructions:
https://www.nushell.sh/book/installation.html

### Supported Terminal Emulators
Choose your favorite:

**WezTerm**
- Modern, fast, written in Rust
- Instructions here: https://wezfurlong.org/wezterm/installation.html

**Ghostty**
- Modern, fast, written in Zig, newer
- Instructions here: https://ghostty.org/download
- **Note**: Due to a [Zellij/Yazi/Ghostty interaction](https://github.com/zellij-org/zellij/issues/2814#issuecomment-2965117327), image previews in Yazi may not display properly, for now. If this is a problem for you, use WezTerm instead

**Kitty**
- Fast, feature-rich, GPU-accelerated terminal
- Instructions here: https://sw.kovidgoyal.net/kitty/binary/

**Alacritty**
- Fast, GPU-accelerated terminal written in Rust
- Instructions here: https://github.com/alacritty/alacritty/blob/master/INSTALL.md

## Step-by-Step Installation

### Step 1: Install Nix Package Manager

We use the **Determinate Systems Nix Installer** - it's reliable, fast, and includes modern features out of the box:

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**What this does:**
- Installs Nix with flakes: just follow the instructions
- Sets up proper file permissions and system integration
- Provides a reliable uninstaller if you ever want to remove Nix
- Verify installation:
```bash
nix --version
```

### Step 2: Download Yazelix

Clone the Yazelix repository to your system:
```bash
git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
```

### Step 3: Configure Your Installation (Optional)

**Before installing dependencies**, create and customize your configuration to control what gets downloaded (else, yazelix will create a config for you based on yazelix_default.nix):

```bash
# Create your personal config from the template
cp ~/.config/yazelix/yazelix_default.nix ~/.config/yazelix/yazelix.nix

# Edit the configuration to suit your needs
# Use your preferred editor (hx, vim, etc.)
hx ~/.config/yazelix/yazelix.nix
```

#### Dependency Groups & Size Estimates

| Group | Size | Default | Description |
|-------|------|---------|-------------|
| **✅ Essential Tools** | ~225MB | Always included | Core Yazelix functionality |
| **🔧 Recommended Tools** | ~350MB | Enabled | Productivity enhancers |
| **🗂️ Yazi Extensions** | ~125MB | Enabled | File preview & archive support |
| **🎬 Yazi Media** | ~1GB | Disabled | Heavy media processing |

#### Installation Options
- **Minimal install**: ~225MB (essential only)
- **Standard install**: ~700MB (default config)
- **Full install**: ~1.7GB (all groups enabled)

📋 For detailed package breakdowns and configuration strategies, see **[Package Sizes Documentation](./package_sizes.md)**

#### Configuration Options
- **Custom shells**: Set `default_shell` to your preference (`"nu"`, `"bash"`, `"fish"`, `"zsh"`)
- **Terminal preference**: Set `preferred_terminal` (`"ghostty"`, `"wezterm"`, `"kitty"`, `"alacritty"`)
- **Editor choice**: Configure your editor (see [Editor Configuration](./editor_configuration.md))

### Step 4: Install Fonts (Required for Kitty and Alacritty)

If you're using Kitty or Alacritty, install Nerd Fonts for proper icon display using modern Nix commands:

**Option A: Using nix profile (recommended - modern replacement for nix-env):**
```bash
nix profile add nixpkgs#nerd-fonts.fira-code nixpkgs#nerd-fonts.symbols-only
```

**Option B: Using Home Manager (if you use Home Manager for system configuration):**
Add to your Home Manager configuration:
```nix
home.packages = with pkgs; [
  nerd-fonts.fira-code
  nerd-fonts.symbols-only
];
```

**Fallback: Legacy nix-env (if modern methods don't work):**
```bash
nix-env -iA nixpkgs.nerd-fonts.fira-code nixpkgs.nerd-fonts.symbols-only
```

**Note**: WezTerm and Ghostty have better font fallback and don't require this step.

### Step 5: Set Up Yazelix to Auto-Launch in Your Terminal

#### Option A: Automatic Launch (Recommended for most users)

Copy the appropriate terminal config to automatically start Yazelix:

**For WezTerm:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/wezterm/.wezterm.lua ~/.wezterm.lua
```

**For Ghostty:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/ghostty/config ~/.config/ghostty/config
```

**For Kitty:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/kitty/kitty.conf ~/.config/kitty/kitty.conf
```

**For Alacritty:**
```bash
cp ~/.config/yazelix/configs/terminal_emulators/alacritty/alacritty.toml ~/.config/alacritty/alacritty.toml
```

**Result**: Every time you open your terminal, it will automatically launch Yazelix. You won't need to run any commands.

#### Option B: Manual Launch (For users who don't want to modify terminal configs)

If you prefer to keep your existing terminal configuration unchanged, just run Yazelix once and it will automatically set up the `yzx` command for you:

```bash
nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu
```

This will automatically configure your shell and then you can use:
- `yzx launch` (opens Yazelix in a new terminal window)  
- `yzx start` (starts Yazelix in current terminal)
- `yzx help` (see all available commands)

#### Optional: Desktop Application Entry

To make Yazelix searchable from your desktop environment (GNOME, KDE, etc.), copy the desktop entry:

```bash
cp ~/.config/yazelix/assets/desktop/com.yazelix.Yazelix.desktop ~/.local/share/applications/
```

Run this command from within your yazelix terminal session. After this, you can search for "Yazelix" in your application launcher and launch it directly.

For better icon quality, see [desktop_icon_setup.md](./desktop_icon_setup.md).

### Step 6: Using Yazelix

**Option A users**: Simply open your terminal! Yazelix will automatically launch with the full environment.  
**Option B users**: Use `yzx launch` or `yzx start` to launch Yazelix when needed.

**First Run**: The first time you launch Yazelix, it will install all dependencies (Zellij, Yazi, Helix, etc.). This may take several minutes, but subsequent launches will be instant.

#### Quick Start Tips
- Use `alt hjkl` to switch between Zellij panes and tabs
- Press `Enter` in Yazi to open files in your configured editor
- Use `yzx help` to see all available management commands
- Use `Alt+Shift+f` to toggle fullscreen on the current pane

### Step 7: Configure Helix Integration (Optional but Recommended)

To enable full Helix-Yazi integration, add the basic Yazelix keybinding to your Helix config (usually `~/.config/helix/config.toml`):

```toml
[keys.normal]
# Yazelix sidebar integration - reveal current file in Yazi sidebar
A-y = ":sh nu ~/.config/yazelix/nushell/scripts/integrations/reveal_in_yazi.nu \"%{buffer_name}\""
```

**Note:** Only works for Helix instances opened from Yazi.

For additional recommended Helix keybindings that enhance your editing experience with Yazelix, see [Helix Keybindings Configuration](./helix_keybindings.md).

## Alternative Installation Methods

### CLI-Only Mode
Use Yazelix tools without starting the full interface (no sidebar, no Zellij):
```bash
yzx env
```
This loads all tools (helix, yazi, lazygit, etc.) into your current shell with Yazelix environment variables set.

If you prefer a raw Nix shell:
```bash
nix develop --impure ~/.config/yazelix
```

### Home Manager Integration
Yazelix includes optional Home Manager support for declarative configuration management. See [home_manager/README.md](../home_manager/README.md) for setup instructions.

## What Gets Installed

### Essential Tools (~225MB)
- **Core functionality**: [Yazi](https://github.com/sxyazi/yazi) (file manager), [Zellij](https://github.com/zellij-org/zellij) (terminal multiplexer), [Helix](https://helix-editor.com) (editor)
- **Shells**: bash/nushell, plus your preferred shell
- **Navigation**: [fzf](https://github.com/junegunn/fzf), [zoxide](https://github.com/ajeetdsouza/zoxide), [Starship](https://starship.rs)

### Recommended Tools (~350MB, enabled by default)
- [lazygit](https://github.com/jesseduffield/lazygit) (or `lg`)
- [mise](https://github.com/jdx/mise)
- [cargo-update](https://github.com/nabijaczleweli/cargo-update)
- [ouch](https://github.com/ouch-org/ouch)
- [atuin](https://github.com/atuinsh/atuin) (shell history manager)

### Yazi Extensions (~125MB, enabled by default)
- `p7zip`, `jq`, `poppler`, `fd`, `ripgrep` (for archives, search, document previews)

### Yazi Media Extensions (~1GB, disabled by default)
- `ffmpeg`, `imagemagick` (for media previews)

### Environment Setup
- Proper paths, variables, and shell configurations

## Post-Installation

### Version Check
Check installed tool versions: `nu nushell/scripts/utils/version_info.nu`

### Health Check
Run diagnostics: `yzx doctor` - Automated health checks and fixes

### Customization
If you followed step 3, you already have your `~/.config/yazelix/yazelix.nix` config file ready! You can modify it anytime and restart Yazelix to apply changes. See [yazelix_default.nix](../yazelix_default.nix) for all available options and their descriptions.

For complete customization options, see the [Customization Guide](./customization.md).

## Troubleshooting

🔍 **Quick diagnosis:** `yzx doctor` - Automated health checks and fixes

📖 **[Complete Troubleshooting Guide](./troubleshooting.md)** - Comprehensive solutions for common issues

## Notes

- The `--impure` flag in `nix develop` allows access to the HOME environment variable, necessary for config paths
- Tweak configs to make them yours; this is just a starting point!
- For extra configuration, see: [WezTerm Docs](https://wezfurlong.org/wezterm/config/files.html)
- Add more swap layouts as needed using the KDL files in `configs/zellij/layouts`
- Use `lazygit`, it's great
