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

## Supported Terminal Emulators
Yazelix provides 5 terminal emulators built-in via Nix - set your `terminals` list in `yazelix.toml`:

**Note**: On macOS, Ghostty uses the native Homebrew version (see below). All other terminals are provided via Nix.

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

Yazelix requires Nix with flakes enabled. We recommend the **Determinate Systems Nix Installer** because it's reliable, fast, and includes modern features out of the box, but any Nix installation with flakes enabled will work.

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

### Optional: Enable Parallel Evaluation (Determinate Nix)

Determinate Nix supports parallel evaluation, which can speed up operations like
`nix search`, `nix flake check`, and `nix eval --json`.

To enable it, add this line to your Determinate config:

`/etc/nix/nix.custom.conf`

```conf
eval-cores = 0
```

Set `eval-cores` to 0 to use all cores, or 1 to disable.

### Step 2: Install Nushell

Nushell is required to run Yazelix (used internally). You can use your preferred shell (bash, fish, zsh, or nushell) for daily work:

```bash
nix profile add nixpkgs#nushell
```

**What this does:**
- Installs Nushell, which Yazelix uses for its internal scripts
- You don't need to learn Nushell - it runs behind the scenes
- Verify installation:
```bash
nu --version
```

**Other platforms**: See https://www.nushell.sh/book/installation.html

### Step 3: Install devenv CLI

Yazelix runs on the [`devenv`](https://devenv.sh) development environment. Install the CLI once so you can launch the Yazelix shell quickly. Prefer the Yazelix-pinned revision so the standalone CLI matches the environment Yazelix was checked against:

```bash
nix profile install github:cachix/devenv/cfd12842b061f9df79e18375d93d72e41f1fbbdf#devenv
```

**What this does:**
- Installs the Yazelix-pinned `devenv` CLI into your user profile (~5GB with all dependencies)
- Provides the `devenv shell` command that Yazelix uses for fast, cached launches
- Verify installation:
```bash
devenv --version
```

### Step 4: Install the Yazelix Runtime

Yazelix needs its shipped runtime assets available somewhere on disk. A source checkout still works, but it is only one possible runtime layout.

For maintainer/source-checkout installs, clone the repository to your system:
```bash
git clone https://github.com/luccahuguet/yazelix ~/.config/yazelix
```

Normal usage should rely on the installed Yazelix runtime and `yzx` entrypoints. User configuration still lives under `~/.config/yazelix/user_configs/`, but the shipped runtime assets do not need to come from a live git checkout forever.

### Step 5: Configure Your Installation (Optional)

**Before installing dependencies**, create and customize your configuration to control what gets downloaded (else, Yazelix will create a config for you based on `yazelix_default.toml`):

```bash
# Create your personal config from the template
mkdir -p ~/.config/yazelix/user_configs
cp ~/.config/yazelix/yazelix_default.toml ~/.config/yazelix/user_configs/yazelix.toml

# Edit the configuration to suit your needs
# Use your preferred editor (hx, vim, etc.)
hx ~/.config/yazelix/user_configs/yazelix.toml
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
- **Terminal preference**: Set `terminals` (`["ghostty", "wezterm", "kitty", "alacritty", "foot"]`, ordered)
- **Managed terminals**: Set `manage_terminals = true` to install via Nix, or false to use system-installed terminals only
- **Editor choice**: Configure your editor (see [Editor Configuration](./editor_configuration.md))

### Step 6: Install Fonts (Required for Kitty and Alacritty)

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

### Step 7: Set Up Yazelix to Auto-Launch in Your Terminal

#### Option A: Automatic Launch (Recommended for most users)

For the **first launch**:

- source-checkout installs can run the repo script directly
- package-ready installs should run the shipped `start_yazelix.sh` from the installed runtime root instead

Source-checkout example:

```bash
nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu --setup-only
```

**What this does**:
- Bootstraps the devenv environment and installs all Yazelix packages (~2.2GB)
- Sets up shell hooks (makes the `yzx` command available)
- Installs a stable `~/.local/bin/yzx` wrapper for editor integrations and host-launched tools
- Does NOT launch the UI (avoids terminal compatibility issues)

**First run note**: The first launch will take several minutes to download and install all dependencies. Subsequent launches will be instant thanks to devenv's caching.

**After setup completes**:
1. Restart your shell (or source your shell config)
2. Use `yzx launch` to start Yazelix in a new terminal window:
```bash
yzx launch  # Opens in new terminal in current directory
```

#### Option B: Manual Launch (For users who don't want to modify terminal configs)

If you prefer to keep your existing terminal configuration unchanged, just run Yazelix once and it will automatically set up the `yzx` command for you.

Source-checkout example:

```bash
nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu
```

This will automatically configure your shell and then you can use:
- `yzx launch` (opens Yazelix in a new terminal window in current directory)
- `yzx launch --here` (starts Yazelix in current terminal)
- `yzx help` (see all available commands)

#### Optional: Desktop/Application Launcher Integration

##### Linux (GNOME, KDE, etc.)

To make Yazelix searchable from your desktop environment, generate the user-local desktop entry:

```bash
yzx desktop install
```

After this, you can search for "Yazelix" in your application launcher and launch it directly.

For better icon quality, see [desktop_icon_setup.md](./desktop_icon_setup.md).

**System Keybind for Launching Yazelix:**

To bind a system keyboard shortcut (in GNOME, KDE, Hyprland, etc.), use the installed Yazelix desktop launcher from your runtime:

```bash
~/.config/yazelix/shells/posix/desktop_launcher.sh
```

This launches the same POSIX entrypoint used by the generated desktop entry. In package-ready installs, the same launcher should come from the installed runtime rather than a source checkout.

##### macOS (Spotlight, Launchpad, Dock)

To integrate Yazelix with macOS launchers:

```bash
# Copy the app bundle to Applications
cp -r ~/.config/yazelix/assets/macos/Yazelix.app /Applications/

# Optional: Create high-quality icon
nu ~/.config/yazelix/assets/macos/create_icns.nu
```

After installation, you can:
- Search for "Yazelix" in Spotlight (Cmd+Space)
- Find it in Launchpad
- Add it to your Dock
- Set up global keyboard shortcuts in System Settings → Keyboard → Keyboard Shortcuts

For detailed macOS setup and troubleshooting, see [assets/macos/README.md](../assets/macos/README.md).

### Step 8: Using Yazelix

**Option A users**: Simply open your terminal! Yazelix will automatically launch with the full environment.
**Option B users**: Use `yzx launch` or `yzx launch --here` to launch Yazelix when needed.

**First Run**: The first time you launch Yazelix, it will install all dependencies (Zellij, Yazi, Helix, etc.). This may take several minutes, but subsequent launches will be instant.

**Zellij Plugin Permissions**: When you first run yazelix, you need to grant permissions for **both** Zellij plugins:

- **zjstatus**: this can look like an "invisible pane" at the very top where the status bar should be. Focus that top bar area and press `y`.
- **Yazelix pane-orchestrator plugin**: Yazelix should also open a popup asking for permission for its own orchestrator plugin. You need to answer **yes** to that popup too.

`Alt+y` and `Ctrl+y` require the Yazelix pane-orchestrator plugin permissions. `Alt+m` opens a new terminal in the current tab workspace root.

The `zjstatus` permission step must be repeated on `zjstatus` updates, since the file changes. See the [zjstatus permissions documentation](https://github.com/dj95/zjstatus/wiki/2-%E2%80%90-Permissions) for more details.

If you are maintaining Yazelix and rebuild the pane-orchestrator plugin, prefer `yzx restart` after `yzx dev build_pane_orchestrator --sync` instead of reloading the plugin inside the current session.

#### Quick Start Tips
- Use `alt hjkl` to switch between Zellij panes and tabs
- Press `Enter` in Yazi to open files in your configured editor
- Use `yzx help` to see all available management commands
- Use `Alt+Shift+F` to toggle fullscreen on the current pane

### Step 9: Configure Helix Integration (Optional)

If you want a Helix-local reveal action, bind `yzx reveal` to any editor-local shortcut that fits your setup. Yazelix recommends `Alt+r` for this; `Ctrl+y` and `Alt+y` are reserved for workspace navigation in Zellij.

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
If you followed step 5, you already have your `~/.config/yazelix/user_configs/yazelix.toml` config file ready. You can modify it anytime and restart Yazelix to apply changes. See [yazelix_default.toml](../yazelix_default.toml) for all available options and their descriptions.

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
