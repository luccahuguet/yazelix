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
Yazelix provides 5 terminal emulators built-in via Nix - simply set your `preferred_terminal` in `yazelix.toml`:

**Note**: All terminals except Ghostty on macOS are automatically installed by Yazelix. Ghostty on macOS requires Homebrew installation (see below).

See [Terminal Emulator Comparison](./terminal_emulators.md) for a detailed breakdown of strengths, gaps, and platform support.

**WezTerm**
- Modern, fast, written in Rust
- Provided by Yazelix via Nix (no installation needed)
- Reference: https://wezfurlong.org/wezterm/installation.html

**Ghostty** (Default)
- Modern, fast, written in Zig, newer
- **Linux**: Provided by Yazelix via Nix (no installation needed)
- **macOS**: Install via Homebrew: `brew install --cask ghostty`
  - Nix package doesn't support macOS due to app bundle limitations
  - Yazelix will auto-detect Homebrew installation
- Download page: https://ghostty.org/download
- **Note**: Due to a [Zellij/Yazi/Ghostty interaction](https://github.com/zellij-org/zellij/issues/2814#issuecomment-2965117327), image previews in Yazi may not display properly, for now. If this is a problem for you, use WezTerm instead

**Kitty**
- Fast, feature-rich, GPU-accelerated terminal
- Provided by Yazelix via Nix (no installation needed)
- Reference: https://sw.kovidgoyal.net/kitty/binary/

**Alacritty**
- Fast, GPU-accelerated terminal written in Rust
- Provided by Yazelix via Nix (no installation needed)
- Reference: https://github.com/alacritty/alacritty/blob/master/INSTALL.md

**Foot**
- Fast, simple, written in C
- Provided by Yazelix via Nix (no installation needed)
- Reference: https://codeberg.org/dnkl/foot/src/branch/master/INSTALL.md

## Step-by-Step Installation

### Step 1: Install Nix Package Manager (~2.5GB)

We use the **Determinate Systems Nix Installer** - it's reliable, fast, and includes modern features out of the box:

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**What this does:**
- Installs Nix with flakes (~2.5GB including dependencies)
- Sets up proper file permissions and system integration
- Provides a reliable uninstaller if you ever want to remove Nix
- Verify installation:
```bash
nix --version
```

### Step 2: Install devenv CLI

Yazelix runs on the [`devenv`](https://devenv.sh) development environment. Install the CLI once so you can launch the Yazelix shell quickly:

```bash
nix profile install github:cachix/devenv/latest
```

**What this does:**
- Installs the latest `devenv` CLI into your user profile (~5GB with all dependencies)
- Provides the `devenv shell` command that Yazelix uses for fast, cached launches
- Verify installation:
```bash
devenv --version
```

### Step 3: Clone Yazelix

Clone the Yazelix repository to your system:
```bash
git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
```

### Step 4: Configure Your Installation (Optional)

**Before installing dependencies**, create and customize your configuration to control what gets downloaded (else, Yazelix will create a config for you based on `yazelix_default.toml`):

```bash
# Create your personal config from the template
cp ~/.config/yazelix/yazelix_default.toml ~/.config/yazelix/yazelix.toml

# Edit the configuration to suit your needs
# Use your preferred editor (hx, vim, etc.)
hx ~/.config/yazelix/yazelix.toml
```

#### Dependency Groups & Size Estimates

| Group | Size | Default | Description |
|-------|------|---------|-------------|
| **✅ Essential Tools** | ~1.7GB | Always included | Core Yazelix functionality (Yazi, Zellij, Helix, shells, built-in Ghostty, etc.) |
| **🔧 Recommended Tools** | ~350MB | Enabled | Productivity enhancers (lazygit, atuin, etc.) |
| **🗂️ Yazi Extensions** | ~125MB | Enabled | File preview & archive support |
| **🎬 Yazi Media** | ~1GB | Disabled | Heavy media processing |

#### Installation Options

**Note**: All installations require Nix (~2.5GB) as a prerequisite.

- **Minimal install**: Nix (~2.5GB) + devenv (~5GB) + essential tools (~1.7GB) = **~9.2GB total**
- **Standard install**: Nix (~2.5GB) + devenv (~5GB) + default config (~2.2GB) = **~9.7GB total**
- **Full install**: Nix (~2.5GB) + devenv (~5GB) + all groups (~3.2GB) = **~10.7GB total**

📋 For detailed package breakdowns and configuration strategies, see **[Package Sizes Documentation](./package_sizes.md)**

#### Configuration Options
- **Custom shells**: Set `default_shell` to your preference (`"nu"`, `"bash"`, `"fish"`, `"zsh"`)
- **Terminal preference**: Set `preferred_terminal` (`"ghostty"`, `"wezterm"`, `"kitty"`, `"alacritty"`, `"foot"`)
- **Editor choice**: Configure your editor (see [Editor Configuration](./editor_configuration.md))

### Step 5: Install Fonts (Required for Kitty and Alacritty)

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

**Note**: WezTerm and Ghostty have better font fallback and don't require this step.

### Step 6: Set Up Yazelix to Auto-Launch in Your Terminal

**Note for v9 (or higher) users:** Terminal configs are now auto-generated - no more manual copying! See [troubleshooting](./troubleshooting.md#v95-migration-notes) for migration details.

#### Option A: Automatic Launch (Recommended for most users)

Use the yazelix launcher to automatically configure your terminal and start yazelix:

```bash
nu ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu
```

**What this does**:
- Automatically generates optimized terminal configs based on your yazelix settings
- Configures cursor trails and transparency from your `yazelix.toml` preferences
- Launches your preferred terminal with yazelix pre-loaded
- No manual copying required - everything is handled dynamically

**Result**: Your terminal opens with yazelix ready to use, with your personalized settings applied.

#### Option B: Manual Launch (For users who don't want to modify terminal configs)

If you prefer to keep your existing terminal configuration unchanged, just run Yazelix once and it will automatically set up the `yzx` command for you:

```bash
nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu
```

This will automatically configure your shell and then you can use:
- `yzx launch` (opens Yazelix in a new terminal window in current directory)
- `yzx launch --here` (starts Yazelix in current terminal)
- `yzx help` (see all available commands)

#### Optional: Desktop Application Entry

To make Yazelix searchable from your desktop environment (GNOME, KDE, etc.), copy the desktop entry:

```bash
cp ~/.config/yazelix/assets/desktop/com.yazelix.Yazelix.desktop ~/.local/share/applications/
```

Run this command from within your yazelix terminal session. After this, you can search for "Yazelix" in your application launcher and launch it directly.

For better icon quality, see [desktop_icon_setup.md](./desktop_icon_setup.md).

##### System Keybind for Launching Yazelix

To bind a system keyboard shortcut (e.g., in GNOME, KDE, Hyprland, etc.) to launch Yazelix, use this command:

```bash
bash -l -c "$HOME/.config/yazelix/nushell/scripts/core/desktop_launcher.nu"
```

The `-l` flag ensures your shell profile is loaded, making Nix packages (like `nu` and `devenv`) available in the PATH. This works reliably across different desktop environments and window managers.

### Step 7: Using Yazelix

**Option A users**: Simply open your terminal! Yazelix will automatically launch with the full environment.
**Option B users**: Use `yzx launch` or `yzx launch --here` to launch Yazelix when needed.

**First Run**: The first time you launch Yazelix, it will install all dependencies (Zellij, Yazi, Helix, etc.). This may take several minutes, but subsequent launches will be instant.

**Zellij Plugin Permissions**: When you first run yazelix, **zjstatus requires you to give it permission**. Navigate to the zjstatus pane (either by keyboard shortcuts or clicking on the pane) and type the letter `y` to approve permissions. This process must be repeated on zjstatus updates, since the file changes. See the [zjstatus permissions documentation](https://github.com/dj95/zjstatus/wiki/2-%E2%80%90-Permissions) for more details.

#### Quick Start Tips
- Use `alt hjkl` to switch between Zellij panes and tabs
- Press `Enter` in Yazi to open files in your configured editor
- Use `yzx help` to see all available management commands
- Use `Alt+Shift+f` to toggle fullscreen on the current pane

### Step 8: Configure Helix Integration (Optional but Recommended)

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
This loads all tools (helix, yazi, lazygit, etc.) into your configured shell with Yazelix environment variables set. Add `--no-shell` to keep using your current shell instead.

If you prefer a raw environment shell:
```bash
devenv shell
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
If you followed step 4, you already have your `~/.config/yazelix/yazelix.toml` config file ready! You can modify it anytime and restart Yazelix to apply changes. See [yazelix_default.toml](../yazelix_default.toml) for all available options and their descriptions.

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
