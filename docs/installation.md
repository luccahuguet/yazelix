# Yazelix Installation Guide

This guide provides the canonical install flow for Yazelix.

## What is Nix?

Nix is just a package manager that ensures reproducible, reliable software installations. Think of it like a super-powered version of `apt`, `brew`, or `chocolatey` that:
- Never breaks your system (installs are isolated)
- Allows multiple versions of the same software
- Makes it easy to share exact development environments
- Can completely uninstall without leaving traces

## Why does Yazelix use Nix?

It guarantees that everyone gets the exact same versions of tools (Yazi, Zellij, Helix, etc.) that work perfectly together, regardless of your operating system or existing software. And it's way easier than having to install everything separately and manually.

**Important**: You don't need to learn Nix or Nushell to use Yazelix. Nix with flakes is the only real host prerequisite. The installer will materialize the Yazelix runtime, ship runtime-local `devenv` and `nu`, and install the stable `yzx` command for you.

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

## Quickstart

If you already have Nix with flakes enabled, the canonical install flow is:

```bash
nix run github:luccahuguet/yazelix#install
yzx launch
```

What this does:
- materializes the Yazelix runtime under `~/.local/share/yazelix/runtime/current`
- ships runtime-local `devenv` and `nu` used by the installed `yzx` and POSIX launchers
- seeds `~/.config/yazelix/user_configs/` if missing
- installs the stable `yzx` command into `~/.local/bin/yzx`

You do **not** need to clone the repo for normal installation.

What this does **not** do:
- install Nix itself for you
- install a separate global/host Nushell just for your normal shell usage
- auto-install the desktop entry; that stays an explicit follow-up via `yzx desktop install`

## Step-by-Step Installation

### Step 1: Install Nix Package Manager (~2.5GB)

Yazelix requires Nix with flakes enabled. We recommend the **Determinate Systems Nix Installer** because it's reliable, fast, and includes modern features out of the box, and it is the path we exercise most. Other flake-enabled Nix installs are expected to work, but are not yet equally verified.

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

### Step 2: Install Yazelix

Run the installer app exposed by the top-level Yazelix flake:

```bash
nix run github:luccahuguet/yazelix#install
```

After it finishes:

```bash
yzx launch
```

Normal usage relies on the installed runtime and `yzx` entrypoints. User configuration lives under `~/.config/yazelix/user_configs/`.

Bootstrap-tool contract:
- **Host prerequisite**: Nix with flakes enabled
- **Installer-managed**: the persistent Yazelix runtime, including runtime-local `devenv` and `nu`, plus the stable installed `yzx` launcher
- **Not installer-managed**: a separate host Nushell install for your everyday shell outside Yazelix

### Step 3: Configure Your Installation (Optional)

If you skipped customization before the installer, it will auto-create `user_configs/yazelix.toml` and `user_configs/yazelix_packs.toml` from the shipped defaults. You can edit them anytime afterward:

```bash
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

**Note**: WezTerm and Ghostty have better font fallback and don't require this step.

### Step 5: Launch and Shell Integration

For most users, the installer is enough. After `nix run ...#install`, open a fresh shell if `~/.local/bin` is not already on your `PATH`, then use:

```bash
yzx launch
```

Useful launch variants:
- `yzx launch` opens Yazelix in a new terminal window
- `yzx enter` starts Yazelix in the current terminal
- `yzx help` shows the command surface

**First run note**: the first launch may take several minutes while Yazelix downloads and installs its environment. Subsequent launches are much faster because `devenv` caching is reused.

If you want to use Nushell as your normal host shell outside Yazelix, install it separately in the way you prefer. Yazelix no longer requires that extra host `nu` install just to bootstrap or launch the installed runtime.

#### Optional: Desktop/Application Launcher Integration

##### Linux (GNOME, KDE, etc.)

To make Yazelix searchable from your desktop environment, generate the user-local desktop entry:

```bash
yzx desktop install
```

After this, you can search for "Yazelix" in your application launcher and launch it directly.

For better icon quality, see [desktop_icon_setup.md](./desktop_icon_setup.md).

**System Keybind for Launching Yazelix:**

To bind a system keyboard shortcut (in GNOME, KDE, Hyprland, etc.), use the stable installed Yazelix launcher:

```bash
~/.local/bin/yzx desktop launch
```

This launches the same command surface used by the generated desktop entry.

## Maintainer / Clone-Based Flow

Normal users should prefer `nix run github:luccahuguet/yazelix#install`.

If you are doing maintainer work or explicitly want to run from a cloned repo, that still works, and the clone can live anywhere:

```bash
git clone https://github.com/luccahuguet/yazelix ~/src/yazelix
nu ~/src/yazelix/nushell/scripts/core/start_yazelix.nu --setup-only
```

That is now the advanced/maintainer path, not the primary install story.

##### macOS (Spotlight, Launchpad, Dock)

To integrate Yazelix with macOS launchers:

```bash
# Copy the app bundle to Applications
cp -r ~/src/yazelix/assets/macos/Yazelix.app /Applications/

# Optional: Create high-quality icon
nu ~/src/yazelix/assets/macos/create_icns.nu
```

After installation, you can:
- Search for "Yazelix" in Spotlight (Cmd+Space)
- Find it in Launchpad
- Add it to your Dock
- Set up global keyboard shortcuts in System Settings → Keyboard → Keyboard Shortcuts

For detailed macOS setup and troubleshooting, see [assets/macos/README.md](../assets/macos/README.md).

### Step 8: Using Yazelix

**Option A users**: Simply open your terminal! Yazelix will automatically launch with the full environment.
**Option B users**: Use `yzx launch` or `yzx enter` to launch Yazelix when needed.

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
Yazelix includes optional Home Manager support for declarative configuration management through the top-level flake's `homeManagerModules.default` output. See [home_manager/README.md](../home_manager/README.md) for setup instructions.

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
