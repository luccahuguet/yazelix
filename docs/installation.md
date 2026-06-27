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
Yazelix supports capable terminal emulators through `yzx enter`. The default `#yazelix` package uses Mars so Yazelix can keep the Rust terminal fork, generated config, cursor behavior, Zellij/Yazi graphics bridge, and agent-driven development workflow aligned. Ghostty is the most tested mature host-terminal path and is a strong choice on macOS. Rio, WezTerm, Kitty, Foot, Ratty, Alacritty, and other emulators work as normal host terminal entrypoints.

Mars uses the temporary Yazelix Zellij fork for Kitty graphics passthrough where that bridge is selected. The Zellij fork is expected to be dropped and archived once upstream Zellij supports the required Kitty graphics path directly enough for Yazelix to return to upstream Zellij.

See [Terminal Emulator Comparison](./terminal_emulators.md) for a detailed breakdown of strengths, gaps, and platform support.

**Mars**
- Default Rust terminal fork for Yazelix
- Provided by the default `yazelix` package runtime and by `yazelix_mars`
- Uses generated Mars config, native cursor integration, `terminal.transparency`, packaged emoji/font profiles, the Yazelix Zellij Kitty graphics bridge, and an agent-driven development focus
- Reference: https://github.com/luccahuguet/mars

**WezTerm**
- Modern, fast, written in Rust
- Configure WezTerm to run `yzx enter`
- Reference: https://wezfurlong.org/wezterm/installation.html

**Ratty**
- GPU-rendered terminal with Kitty graphics support and inline 3D graphics
- Configure Ratty to run `yzx enter`
- Reference: https://github.com/orhun/ratty

**Foot**
- Lightweight Wayland terminal for Linux
- Configure Foot to run `yzx enter`
- Reference: https://codeberg.org/dnkl/foot

**Ghostty**
- Mature, fast terminal with the most tested Yazelix host-terminal path
- Strong macOS recommendation
- Configure Ghostty to run `yzx enter`; run `yzx cursors ghostty setup` for Yazelix cursor shaders
- Download page: https://ghostty.org/download

**Rio**
- Upstream Rust terminal
- Configure Rio to run `yzx enter`
- Reference: https://github.com/raphamorim/rio

**Kitty**
- Fast, feature-rich, GPU-accelerated terminal
- Configure Kitty to run `yzx enter`
- Reference: https://sw.kovidgoyal.net/kitty/binary/

## Quickstart

1. Optional: run the install check:

```bash
curl -fsSL https://raw.githubusercontent.com/luccahuguet/yazelix/main/shells/posix/install_check.sh | sh
```

2. Install Yazelix:

```bash
nix profile add --refresh --accept-flake-config github:luccahuguet/yazelix#yazelix
```

3. Launch Yazelix:

```bash
yzx launch
```

If you prefer to inspect the check before running it:

```bash
curl -fsSLO https://raw.githubusercontent.com/luccahuguet/yazelix/main/shells/posix/install_check.sh
sh install_check.sh
```

The install check is optional. It reports Nix/flakes/platform readiness, explains whether Yazelix's binary cache is trusted, and prints numbered next steps for the current host.

If Nix is already available, the same check is also exposed as a flake app:

```bash
nix run --accept-flake-config github:luccahuguet/yazelix#install_check
```

Use `#yazelix_mars` only when you want to name the explicit Mars package output. Other terminal emulators are supported by running the installed `yzx enter` command from their own startup configuration.

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

### Use the Yazelix Binary Cache

Yazelix publishes selected `x86_64-linux` and `aarch64-darwin` package builds to the public Cachix cache at `https://yazelix.cachix.org`. The cache includes the default Yazelix package plus expensive Yazelix Helix and KGP Zellij runtime packages when CI has published the current revision. The flake advertises this cache through `nixConfig`, so interactive Nix commands can prompt you to accept the substituter and trusted public key. After you accept it, Nix uses the cache automatically for matching store paths. The cache is optional: Nix still builds from source when the cache is unavailable or does not contain the requested output.

For noninteractive installs, pass `--accept-flake-config` to the Nix command that evaluates the Yazelix flake:

```bash
nix profile add --refresh --accept-flake-config github:luccahuguet/yazelix#yazelix
```

For persistent cache setup, use the path that matches your Nix install.

For non-Determinate Nix installs where the legacy `nix-env` command is available, the Cachix CLI can write the root daemon configuration:

```bash
sudo nix run nixpkgs#cachix -- use yazelix --mode root-nixconf
```

Determinate Nix installs can omit `nix-env`, so the Cachix helper above may fail before it edits the daemon configuration. Determinate Nix manages `/etc/nix/nix.conf` and leaves `/etc/nix/nix.custom.conf` for user edits. Add the cache to that custom file instead:

`/etc/nix/nix.custom.conf`

Append the Yazelix cache entries:

```bash
yazelix_cache_key='yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mp'
yazelix_cache_key="${yazelix_cache_key}UNzDJ97xC2y7QWYjmGNM="
sudo tee -a /etc/nix/nix.custom.conf >/dev/null <<EOF
extra-substituters = https://yazelix.cachix.org
extra-trusted-public-keys = ${yazelix_cache_key}
EOF
```

Check that the settings are active:

```bash
nix config show | grep -E 'https://yazelix\.cachix\.org|yazelix\.cachix\.org-1:'
```

If the command does not print both the cache URL and a `yazelix.cachix.org-1` key after editing `/etc/nix/nix.custom.conf`, restart the Nix daemon or reboot, then check again.

For NixOS or Home Manager-managed Nix configuration, add the cache to your Nix settings:

```nix
let
  yazelix_cache_key =
    "yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mp"
    + "UNzDJ97xC2y7QWYjmGNM=";
in {
  nix.settings.extra-substituters = [
    "https://yazelix.cachix.org"
  ];

  nix.settings.extra-trusted-public-keys = [
    yazelix_cache_key
  ];
}
```

The `extra-*` settings add Yazelix's cache beside existing caches. For example, a Home Manager user can place those `nix.settings` entries in their Home Manager configuration, then run `home-manager switch`. Standalone Home Manager users should also set `nix.package = pkgs.nix` when Home Manager generates `~/.config/nix/nix.conf`.

Check that representative expensive outputs are present in the public cache:

```bash
helix_out="$(nix eval --raw github:luccahuguet/yazelix#yazelix_helix.outPath)"
kgp_zellij_out="$(nix eval --raw github:luccahuguet/yazelix#yazelix_kgp_zellij.outPath)"
nix path-info --store https://yazelix.cachix.org "$helix_out"
nix path-info --store https://yazelix.cachix.org "$kgp_zellij_out"
```

If `nix path-info` cannot find the output, the requested Yazelix revision has not been published to the cache yet, and Nix will build it from source.

### Step 2: Install Yazelix

Install the Yazelix package exposed by the top-level flake:

```bash
nix profile add --refresh --accept-flake-config github:luccahuguet/yazelix#yazelix
```

The default package is the Mars variant. To install the package-provided Mars variant explicitly:

```bash
nix profile add --refresh --accept-flake-config github:luccahuguet/yazelix#yazelix_mars
```

> If you previously evaluated this flake (for example with `nix run` or `nix flake show`), Nix may have cached an older version. Add `--refresh` to force a fresh fetch:
> ```bash
> nix profile add --refresh --accept-flake-config github:luccahuguet/yazelix#yazelix
> ```

After it finishes:

```bash
yzx launch
```

If you only want to try Yazelix without installing it persistently:

```bash
nix run github:luccahuguet/yazelix#yazelix -- launch
```

Normal usage relies on the package-provided `yzx` entrypoint or the Home Manager module. User configuration lives under `~/.config/yazelix/`.

Host prerequisite contract:
- **Host prerequisite**: Nix with flakes enabled
- **Package-provided**: the Yazelix runtime, including runtime-local `nu`, `zellij`, `yazi`, `helix`, shells, a curated interactive tool surface, and the internal helper closure behind the runtime root
- **User-provided when selected**: a separate host Nushell install for your everyday shell outside Yazelix, or any non-Mars terminal emulator you configure to run `yzx enter`
- **Nushell version ownership**: Yazelix uses the Nushell packaged by the locked `nixpkgs-unstable` input for the runtime and bootstrap path; it does not chase a newer upstream Nushell release until Nixpkgs packages it.

### Step 3: Configure Your Installation (Optional)

If you launch before editing config, Yazelix will auto-create `settings.jsonc` from the shipped defaults. You can edit it anytime afterward:

```bash
hx ~/.config/yazelix/settings.jsonc
```

#### Runtime Surface

The packaged runtime ships a fixed toolset instead of configurable dependency groups. The package includes:
- the core Yazelix stack: `zellij`, `yazi`, `helix`, `nu`, `bash`, `fish`, `zsh`
- Helix Steel authoring helpers: `steel`, `steel-language-server`, `forge`, `cargo-steel-lib`, `repl-connect`
- the default CLI helpers: `fzf`, `zoxide`, `starship`, `lazygit`, `zenith`, `carapace`, `macchina`
- host-managed helper integrations: `mise` and `tombi`
- the default Yazi preview helpers: `p7zip`, `jq`, `fd`, `ripgrep`, `poppler`
- one packaged terminal: Mars by default with the Yazelix Zellij graphics bridge

When you enter `yzx env`, Yazelix exports that curated tool surface to your shell. Runtime-private helpers stay under `libexec/` so host apps launched from Yazelix do not inherit shadowing tools like `dirname` ahead of the system PATH.

What it does not ship anymore:
- a runtime-local `devenv` binary
- dynamic packs or `user_packages`
- non-Mars terminal binaries; those terminals stay user-installed and run Yazelix with `yzx enter`
- heavyweight media helpers such as `ffmpeg` or ImageMagick

#### Configuration Options
- **Custom shells**: Set `default_shell` to your preference (`"nu"`, `"bash"`, `"fish"`, `"zsh"`, `"xonsh"`); xonsh must be installed on the host and available on `PATH`
- **Host xonsh hooks**: Yazelix generates xonsh initializers, but xonsh remains host-installed and native xonsh startup must source `~/.config/yazelix/shell_xonsh.xsh`
- **Terminal package**: choose `#yazelix` or `#yazelix_mars`; Home Manager uses `programs.yazelix.terminal = "mars"`
- **Terminal launch**: Mars is the Yazelix-owned terminal for Rust stack compatibility and agent-driven workflows; other terminal emulators should start Yazelix with `yzx enter`
- **Editor choice**: Configure your editor (see [Editor Configuration](./editor_configuration.md))

### Step 4: Install Fonts (Required for Kitty)

If you're using Kitty, install Nerd Fonts for proper icon display using modern Nix commands:

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
For Home Manager installs on Linux, do not run `yzx desktop install`; the Home Manager module owns the profile desktop entries. Use `yzx desktop uninstall` only to remove a stale user-local entry that shadows the Home Manager launcher.

For better icon quality, see [desktop_icon_setup.md](./desktop_icon_setup.md).

**System Keybind for Launching Yazelix:**

To bind a system keyboard shortcut (in GNOME, KDE, Hyprland, etc.), use the `yzx` command from your profile or Home Manager PATH:

```bash
yzx desktop launch
```

This launches the same command surface used by the generated desktop entry.

##### macOS (Experimental Launcher Preview)

The supported macOS package path remains `yzx launch` from a terminal after installing the package via `nix profile add` or Home Manager. Mars is the packaged terminal path, while Ghostty is the strongest mature host-terminal recommendation on macOS. WezTerm, Kitty, and other capable macOS terminals run Yazelix with `yzx enter`. The Home Manager module does not emit Linux `xdg.desktopEntries` on macOS.

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

This launcher preview is unsigned, unnotarized, and not maintainer-validated on macOS hardware. It is a community feedback path, not a supported Spotlight/Launchpad/Dock contract. See the [macOS support floor contract](./contracts/macos_support_floor.md).

The current production stance is intentionally `unsigned preview`: Yazelix owns the app-bundle metadata, install/uninstall path, and failure messages, but signed or notarized distribution is gated until the release workflow can defend Developer ID signing, notarization, stapling, and macOS hardware smoke tests.

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

`Alt+Shift+H/J/K/L`, `Ctrl+y`, and `Ctrl+Shift+Y` require the Yazelix pane-orchestrator plugin permissions. `Alt+m` opens a new terminal in the current tab workspace root.

If the top status bar looks transparent or broken, see [troubleshooting](troubleshooting.md#first-run-zellij-plugin-permissions-is-the-top-bar-looking-funnyweirdbroken) for the manual recovery path. See the [zjstatus permissions documentation](https://github.com/dj95/zjstatus/wiki/2-%E2%80%90-Permissions) for more details.

If you are maintaining Yazelix and test a newly packaged pane-orchestrator plugin, prefer `yzx restart` after the local override runtime build instead of reloading the plugin inside the current session.

#### Quick Start Tips
- Use `Alt+Shift+H/J/K/L` for the left sidebar, bottom popup, top popup, and right sidebar
- Press `Enter` in Yazi to open files in your configured editor
- Use `yzx help` to see all available management commands
- Use `Alt+Shift+F` to toggle fullscreen on the current pane

### Step 9: Configure Helix Integration (Optional)

Yazelix-managed Helix sessions ship a curated Helix-local config with `Alt+r` bound to reveal the current buffer in Yazi. `Ctrl+y`, `Ctrl+Shift+Y`, and `Alt+Shift+H` remain reserved for workspace navigation in Zellij.

To override or remove the managed Helix defaults, edit `~/.config/yazelix/helix/config.toml`. For the default keybinding list, see [Helix Keybindings Configuration](./helix_keybindings.md).

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

### Granular Nix Customization

The default flake packages stay batteries-included. Yazelix does not expose a package matrix for every possible storage-saving combination; use Home Manager or `lib.${system}.mkYazelix` when you want specific tools to come from your host `PATH`.

Users who prefer another terminal emulator should keep that terminal in its native configuration flow and configure its startup command to run `yzx enter`.

Home Manager is the recommended granular path:

```nix
{
  programs.yazelix = {
    enable = true;
    runtime_tool_sources = {
      lazygit = "host";
      zenith = "host";
      helix = "host";
      yazi = "host";
      ripgrep = "host";
      fd = "host";
    };
  };
}
```

For a validated advanced storage-saving profile, see the [Home Manager lean runtime profile](../home_manager/README.md#lean-runtime-profile). It keeps the packaged terminal and bootstrap runtime bundled while host-sourcing large leaf tools and disabling optional preview/helper components.

Advanced flake users can build the same shape directly:

```nix
let
  system = "x86_64-linux";
  pkgs = import nixpkgs { inherit system; };
in
inputs.yazelix.lib.${system}.mkYazelix {
  inherit pkgs;
  runtimeToolSources = {
    lazygit = "host";
    zenith = "host";
    helix = "host";
  };
}
```

Package-set users can also use the default overlay:

```nix
{
  nixpkgs.overlays = [
    inputs.yazelix.overlays.default
  ];
}
```

`host` mode removes that tool from the Yazelix runtime export and lets the inherited `PATH` provide the required command. Run `yzx doctor` after switching; it reports missing host-sourced commands from the runtime manifest, warning for required commands and treating default optional integrations such as `mise` and `tombi` as informational when absent.

`off` mode is supported for the first optional helper slice: `steel`, `macchina`, `p7zip`, `poppler`, and `resvg`. Disabled helpers are omitted from the runtime package/export and reported by `yzx doctor` as intentional disablement. If `macchina = "off"`, set `show_macchina_on_welcome = false`.

Home Manager and `mkYazelix` also accept component toggles for `cursors` and `screen`. `components.cursors = false` removes Yazelix cursor shader assets and the default cursor sidecar from the runtime tree; Mars config generation skips Yazelix cursor shaders and the config UI hides cursor fields. `components.screen = false` requires `skip_welcome_screen = true` and `screen_saver_enabled = false`; `yzx screen` then fails with a disabled-component error instead of looking for missing screen assets.

## What Gets Installed

### Essential Tools (~225MB)
- **Core functionality**: [Yazi](https://github.com/sxyazi/yazi) (file manager), [Zellij](https://github.com/zellij-org/zellij) (terminal multiplexer), [Helix](https://helix-editor.com) (editor)
- **Shells**: bash/nushell, plus your preferred shell
- **Navigation**: [fzf](https://github.com/junegunn/fzf), [zoxide](https://github.com/ajeetdsouza/zoxide), [Starship](https://starship.rs)

### Recommended Tools (enabled by default)
- [lazygit](https://github.com/jesseduffield/lazygit) (or `lg`)
- [Zenith](https://github.com/bvaisvil/zenith)
- [cargo-update](https://github.com/nabijaczleweli/cargo-update)
- [ouch](https://github.com/ouch-org/ouch)
- [atuin](https://github.com/atuinsh/atuin) (shell history manager)

### Host-Managed Helper Integrations
- [mise](https://github.com/jdx/mise)
- [tombi](https://tombi-toml.github.io/tombi/)

Yazelix configures or uses these integrations when the command is available from the host `PATH`; they are not bundled by default.

### Yazi Extensions (~125MB, enabled by default)
- `p7zip`, `jq`, `poppler`, `fd`, `ripgrep` (for archives, search, document previews)

### Helix Steel Authoring Tools (~117MB, enabled by default)
- `steel`, `steel-language-server`, `forge`, `cargo-steel-lib`, `repl-connect` from the bundled Steel package

### Yazi Media Helpers
Yazelix does not ship `ffmpeg` or ImageMagick in the packaged runtime. Install them outside Yazelix if you want heavy media previews.

### Environment Setup
- Proper paths, variables, and shell configurations

## Post-Installation

### Version Check
Check installed tool versions: `yzx status --versions`

### Health Check
Run diagnostics: `yzx doctor` - Automated health checks and fixes

### Customization
If you followed step 5, you already have your `~/.config/yazelix/settings.jsonc` config file ready. You can modify it anytime and restart Yazelix to apply changes. Main options live in that file; cursor presets live in `~/.config/yazelix_cursors/settings.jsonc`.

For complete customization options, see the [Customization Guide](./customization.md).

## Troubleshooting

🔍 **Quick diagnosis:** `yzx doctor` - Automated health checks and fixes

📖 **[Complete Troubleshooting Guide](./troubleshooting.md)** - Comprehensive solutions for common issues

## Notes

- The `--impure` flag in `nix develop` allows access to the HOME environment variable, necessary for config paths
- Tweak configs to make them yours; this is just a starting point!
- For host-terminal configuration, see your terminal emulator's startup-command documentation
- Add more swap layouts by changing `rust_core/yazelix_zellij_config_pack/layouts`
- Use `lazygit`, it's great
