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

**Important**: You don't need to learn Nix or Nushell to use Yazelix. Nix with flakes is the only real host prerequisite. The normal product surface is the `yazelix` package or the top-level Home Manager module.

## Supported Terminal Emulators
Yazelix provides a Ghostty runtime variant by default and a WezTerm runtime variant for users who want the WezTerm/image-compatible path from the package. Kitty, Alacritty, and Foot remain supported terminal choices, but you provide those binaries yourself and then list them in `terminals` in `yazelix.toml`.

See [Terminal Emulator Comparison](./terminal_emulators.md) for a detailed breakdown of strengths, gaps, and platform support.

**WezTerm**
- Modern, fast, written in Rust
- Provided by the `yazelix_wezterm` package/runtime variant, or supported as a PATH-provided alternative terminal
- Reference: https://wezfurlong.org/wezterm/installation.html

**Ghostty** (Default)
- Modern, fast, written in Zig, newer
- **Linux and macOS**: Provided by the default `yazelix` / `yazelix_ghostty` package runtime
- Download page: https://ghostty.org/download
- **Note**: Due to a [Zellij/Yazi/Ghostty interaction](https://github.com/zellij-org/zellij/issues/2814#issuecomment-2965117327), image previews in Yazi may not display properly, for now. If this is a problem for you, use WezTerm instead

**Kitty**
- Fast, feature-rich, GPU-accelerated terminal
- Supported as a PATH-provided alternative terminal
- Reference: https://sw.kovidgoyal.net/kitty/binary/

**Alacritty**
- Fast, GPU-accelerated terminal written in Rust
- Supported as a PATH-provided alternative terminal
- Reference: https://github.com/alacritty/alacritty/blob/master/INSTALL.md

**Foot**
- Fast, simple, written in C
- Supported as a PATH-provided Linux-only alternative terminal
- Reference: https://codeberg.org/dnkl/foot/src/branch/master/INSTALL.md

## Quickstart

If you already have Nix with flakes enabled, the canonical install flow is:

```bash
nix profile add github:luccahuguet/yazelix#yazelix
yzx launch
```

Use `#yazelix_wezterm` instead if you want the package-provided WezTerm runtime variant.

One-off use without installing also works:

```bash
nix run github:luccahuguet/yazelix#yazelix -- launch
```

Declarative users can install through the top-level Home Manager module instead of `nix profile add`.

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

Install the Yazelix package exposed by the top-level flake:

```bash
nix profile add github:luccahuguet/yazelix#yazelix
```

The default package is the Ghostty variant. To install the package-provided WezTerm variant instead:

```bash
nix profile add github:luccahuguet/yazelix#yazelix_wezterm
```

> If you previously evaluated this flake (for example with `nix run` or `nix flake show`), Nix may have cached an older version. Add `--refresh` to force a fresh fetch:
> ```bash
> nix profile add --refresh github:luccahuguet/yazelix#yazelix
> ```

After it finishes:

```bash
yzx launch
```

If you only want to try Yazelix without installing it persistently:

```bash
nix run github:luccahuguet/yazelix#yazelix -- launch
```

Normal usage relies on the package-provided `yzx` entrypoint or the Home Manager module. User configuration lives under `~/.config/yazelix/user_configs/`.

Host prerequisite contract:
- **Host prerequisite**: Nix with flakes enabled
- **Package-provided**: the Yazelix runtime, including runtime-local `nu`, `zellij`, `yazi`, `helix`, shells, a curated interactive tool surface, and the internal helper closure behind the runtime root
- **Not package-provided**: a separate host Nushell install for your everyday shell outside Yazelix, or PATH-provided alternative terminals outside the selected Ghostty/WezTerm runtime variant
- **Nushell version ownership**: Yazelix uses the Nushell packaged by the locked `nixpkgs` input for the runtime and bootstrap path. The maintainer update workflow records that as `PINNED_NUSHELL_VERSION`; it does not chase a newer upstream Nushell release until Nixpkgs packages it.

### Step 3: Configure Your Installation (Optional)

If you launch before editing config, Yazelix will auto-create `user_configs/yazelix.toml` from the shipped default. You can edit it anytime afterward:

```bash
hx ~/.config/yazelix/user_configs/yazelix.toml
```

#### Runtime Surface

The trimmed v15 packaged runtime ships a fixed toolset instead of configurable dependency groups. The package includes:
- the core Yazelix stack: `zellij`, `yazi`, `helix`, `nu`, `bash`, `fish`, `zsh`
- the default CLI helpers: `fzf`, `zoxide`, `starship`, `lazygit`, `mise`, `carapace`, `macchina`
- the default Yazi preview helpers: `p7zip`, `jq`, `fd`, `ripgrep`, `poppler`
- one packaged terminal variant: Ghostty by default, or WezTerm through `#yazelix_wezterm` / `programs.yazelix.runtime_variant = "wezterm"`

When you enter `yzx env`, Yazelix exports that curated tool surface to your shell. Runtime-private helpers stay under `libexec/` so host apps launched from Yazelix do not inherit shadowing tools like `dirname` ahead of the system PATH.

What it does not ship anymore:
- a runtime-local `devenv` binary
- dynamic packs or `user_packages`
- non-selected terminal binaries; install Kitty, Alacritty, or Foot yourself if you choose them
- heavyweight media helpers such as `ffmpeg` or ImageMagick

#### Configuration Options
- **Custom shells**: Set `default_shell` to your preference (`"nu"`, `"bash"`, `"fish"`, `"zsh"`)
- **Terminal preference**: Set `terminals` (`["ghostty", "wezterm", "kitty", "alacritty", "foot"]`, ordered)
- **Terminal launch**: Ghostty is the built-in default; the WezTerm package variant provides WezTerm instead; other configured terminals are launched from `PATH` in the order you configure
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

For most users, a profile or Home Manager install is enough. After installing `#yazelix`, open a fresh shell if your profile `bin` dir is not already on your `PATH`, then use:

```bash
yzx launch
```

Useful launch variants:
- `yzx launch` opens Yazelix in a new terminal window
- `yzx enter` starts Yazelix in the current terminal
- `yzx help` shows the command surface

**First run note**: the first launch can take a bit longer while Yazelix generates managed runtime state under `~/.local/share/yazelix`. Later launches are usually faster because that generated state already exists. Launch does not rewrite your host shell dotfiles.

If you want to use Nushell as your normal host shell outside Yazelix, install it separately in the way you prefer. Yazelix no longer requires that extra host `nu` install just to bootstrap or launch the installed runtime.

#### Optional: Desktop/Application Launcher Integration

##### Linux (GNOME, KDE, etc.)

To make Yazelix searchable from your desktop environment, generate the user-local desktop entry:

```bash
yzx desktop install
```

After this, you can search for "Yazelix" in your application launcher and launch it directly.
`yzx desktop install` points the desktop entry at the active Yazelix runtime launcher, and `yzx desktop uninstall` removes that user-local desktop integration again.
For Home Manager installs, do not run `yzx desktop install`; the Home Manager module owns the profile desktop entry. Use `yzx desktop uninstall` only to remove a stale user-local entry that shadows the Home Manager launcher.

For better icon quality, see [desktop_icon_setup.md](./desktop_icon_setup.md).

**System Keybind for Launching Yazelix:**

To bind a system keyboard shortcut (in GNOME, KDE, Hyprland, etc.), use the `yzx` command from your profile or Home Manager PATH:

```bash
yzx desktop launch
```

This launches the same command surface used by the generated desktop entry.

##### macOS (Experimental Launcher Preview)

The supported macOS launch path remains `yzx launch` from a terminal after installing the package via `nix profile add` or Home Manager.

Community testers can opt into an experimental package-first app bundle preview:

```bash
yzx desktop macos_preview install
```

This creates `~/Applications/Yazelix Preview.app`. The preview app calls `desktop launch` through the active profile-owned `yzx` wrapper, so default-profile installs resolve through `~/.nix-profile/bin/yzx` or `/etc/profiles/per-user/$USER/bin/yzx`, and Home Manager installs resolve through the Home Manager profile wrapper when it exists. It does not assume a repo clone or a checked-out runtime path.

If the package-owned launcher is missing or no longer executable, the app shows an actionable failure and asks you to reinstall Yazelix and rerun `yzx desktop macos_preview install`. If startup itself fails, run `yzx doctor --verbose` from Terminal and include that output when reporting feedback.

Remove the preview app with:

```bash
yzx desktop macos_preview uninstall
```

This launcher preview is unsigned, unnotarized, and not maintainer-validated on macOS hardware. It is a community feedback path, not a supported Spotlight/Launchpad/Dock contract. See the [macOS support floor spec](./specs/macos_support_floor.md).

The current production stance is intentionally `unsigned preview`: Yazelix owns the app-bundle metadata, install/uninstall path, and failure messages, but signed or notarized distribution is gated until the release workflow can defend Developer ID signing, notarization, stapling, and macOS hardware smoke tests. See the [macOS launcher productization spec](./specs/macos_launcher_productization.md).

## Maintainer / Clone-Based Flow

Normal users should prefer the `#yazelix` package or the Home Manager module.

If you are doing maintainer work or explicitly want to run from a cloned repo, that still works, and the clone can live anywhere:

```bash
git clone https://github.com/luccahuguet/yazelix ~/src/yazelix
nix run ~/src/yazelix#yazelix -- launch
```

That is now the advanced/maintainer path, not the primary install story.

### Step 8: Using Yazelix

**Option A users**: Simply open your terminal! Yazelix will automatically launch with the full environment.
**Option B users**: Use `yzx launch` or `yzx enter` to launch Yazelix when needed.

**First Run**: The first time you launch Yazelix, it will install all dependencies (Zellij, Yazi, Helix, etc.). This may take several minutes, but subsequent launches will be instant.

**Zellij Plugin Permissions**: Yazelix pre-seeds bundled Zellij plugin permissions for its managed zjstatus and pane-orchestrator plugin paths before launch

If Zellij still shows a plugin permission prompt, answer **yes**. This can happen after manually deleting `~/.cache/zellij/permissions.kdl`, revoking permissions, or using a Zellij/plugin state Yazelix cannot safely infer.

`Alt+y` and `Ctrl+y` require the Yazelix pane-orchestrator plugin permissions. `Alt+m` opens a new terminal in the current tab workspace root.

If the top status bar looks transparent or broken, see [troubleshooting](troubleshooting.md#first-run-zellij-plugin-permissions-is-the-top-bar-looking-funnyweirdbroken) for the manual recovery path. See the [zjstatus permissions documentation](https://github.com/dj95/zjstatus/wiki/2-%E2%80%90-Permissions) for more details.

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
This loads the curated Yazelix tool surface into your configured shell with Yazelix environment variables set. Add `--no-shell` to keep using your current shell instead.

If you want the Yazelix tool PATH without switching into your configured shell:
```bash
yzx env --no-shell
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

### Yazi Media Helpers
Yazelix does not ship `ffmpeg` or ImageMagick in the runtime variants. Install them outside Yazelix if you want heavy media previews.

### Environment Setup
- Proper paths, variables, and shell configurations

## Post-Installation

### Version Check
Check installed tool versions: `yzx status --versions`

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
