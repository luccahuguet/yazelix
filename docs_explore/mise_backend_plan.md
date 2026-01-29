# Mise Backend Implementation Plan

**Status**: Planning Phase
**Priority**: Medium
**Risk Level**: Medium (alternative backend, not replacing Nix)

## Overview

This document outlines the implementation of **mise** as an alternative environment backend for Yazelix. The goal is to provide a lighter-weight, Nix-free option for users who:

- Don't want to install Nix (~2.5GB + devenv ~5GB)
- Don't have admin rights on their machine
- Prefer familiar version management tooling
- Want faster initial setup

**Key principle**: mise is an *alternative* backend, not a replacement. Nix remains the default and most complete option.

---

## Goals

- [ ] Provide mise as an alternative to Nix/devenv for environment management
- [ ] Maintain feature parity where possible (document gaps clearly)
- [ ] Single configuration file (`yazelix.toml`) drives both backends
- [ ] Transparent backend switching via `[environment].backend` option
- [ ] No code duplication - abstract common logic

## Non-Goals

- Replacing Nix as the default backend
- Supporting every tool available in the Nix backend
- Building mise plugins for missing tools
- Terminal emulator management (users provide their own)

---

## Tool Availability Analysis

### Core Tools (Required)

| Tool | mise availability | Backend | Notes |
|------|-------------------|---------|-------|
| helix | aqua:helix-editor/helix | aqua | Full support |
| yazi | ubi:sxyazi/yazi | github/ubi | Full support |
| zellij | ubi:zellij-org/zellij | github/ubi | Full support |
| nushell | ubi:nushell/nushell | github/ubi | Full support |
| fzf | aqua:junegunn/fzf | aqua | Full support |
| zoxide | ubi:ajeetdsouza/zoxide | github/ubi | Full support |
| starship | ubi:starship/starship | github/ubi | Full support |
| bash | System | N/A | Use system bash |
| macchina | ubi:Macchina-CLI/macchina | github/ubi | Full support |
| taplo | cargo:taplo-cli | cargo | Full support |

### Recommended Tools

| Tool | mise availability | Backend | Notes |
|------|-------------------|---------|-------|
| lazygit | aqua:jesseduffield/lazygit | aqua | Full support |
| atuin | ubi:atuinsh/atuin | github/ubi | Full support |
| carapace | ubi:rsteube/carapace-bin | github/ubi | Full support |
| markdown-oxide | cargo:markdown-oxide | cargo | Full support |

### Yazi Extensions

| Tool | mise availability | Backend | Notes |
|------|-------------------|---------|-------|
| fd | aqua:sharkdp/fd | aqua/cargo | Full support |
| ripgrep | aqua:BurntSushi/ripgrep | aqua/cargo | Full support |
| jq | aqua:jqlang/jq | aqua | Full support |
| p7zip | N/A | System | Requires system install |
| poppler | N/A | System | Requires system install |

### Yazi Media

| Tool | mise availability | Backend | Notes |
|------|-------------------|---------|-------|
| ffmpeg | N/A | System | Requires system install |
| imagemagick | N/A | System | Requires system install |

### Terminal Emulators

| Tool | mise availability | Notes |
|------|-------------------|-------|
| ghostty | N/A | User must install separately |
| wezterm | N/A | User must install separately |
| kitty | N/A | User must install separately |
| alacritty | cargo:alacritty | Possible but not recommended |
| foot | N/A | User must install separately |

### Zjstatus (Zellij Plugin)

| Tool | mise availability | Notes |
|------|-------------------|-------|
| zjstatus.wasm | N/A | Must be downloaded from GitHub releases |

---

## Feature Comparison

| Feature | Nix Backend | mise Backend |
|---------|-------------|--------------|
| All core tools | ✓ | ✓ |
| Recommended tools | ✓ | ✓ |
| Yazi extensions | ✓ | Partial (p7zip, poppler need system) |
| Yazi media | ✓ | ✗ (ffmpeg, imagemagick need system) |
| Terminal management | ✓ | ✗ (user provides terminal) |
| Terminal wrappers | ✓ | ✗ |
| nixGL support | ✓ | N/A |
| Desktop entry | ✓ | Manual |
| Helix source builds | ✓ | ✗ (release only) |
| Custom packs | ✓ | Limited |
| Reproducibility | Excellent | Good |
| Rollback | Easy | Manual |
| Disk usage | ~9.7GB | ~1GB |
| Admin rights | Yes (Nix install) | No |
| Initial setup time | Slow (first build) | Fast |

---

## Architecture

### Configuration Schema

Add to `yazelix.toml`:

```toml
[environment]
# Backend for managing tool installations
# Options:
# - "nix":  Use devenv/Nix (default, most reproducible, largest disk usage)
# - "mise": Use mise for tool management (lighter, ~90% feature coverage)
backend = "nix"

# mise-specific settings (only used when backend = "mise")
[environment.mise]
# Installation directory for mise-managed tools
# Default: uses mise's default (~/.local/share/mise)
# root = "~/.local/share/mise"

# Use cargo backend for Rust tools (faster updates)
# When false, uses pre-built binaries from GitHub releases
prefer_cargo = false

# Automatically install missing system dependencies
# When true, shows instructions for installing p7zip, poppler, etc.
prompt_system_deps = true
```

### Directory Structure

```
yazelix/
├── nushell/scripts/
│   ├── backends/
│   │   ├── mod.nu              # Backend dispatcher
│   │   ├── nix.nu              # Nix/devenv backend (current behavior)
│   │   ├── mise.nu             # mise backend implementation
│   │   └── common.nu           # Shared utilities
│   ├── core/
│   │   ├── yazelix.nu          # Updated to use backend dispatcher
│   │   ├── start_yazelix.nu    # Updated to support both backends
│   │   └── launch_yazelix.nu   # Updated for mise (no terminal wrappers)
│   └── setup/
│       └── mise_setup.nu       # mise-specific setup logic
├── mise/
│   ├── mise.toml.template      # Template for generating .mise.toml
│   └── tool_mappings.nu        # Package name → mise tool mappings
└── docs/
    └── mise_backend_plan.md    # This document
```

### Generated mise.toml

When `backend = "mise"`, yazelix generates `~/.config/yazelix/.mise.toml`:

```toml
# Auto-generated by Yazelix - do not edit manually
# Regenerate with: yzx env --regenerate

[tools]
# Core tools (always installed)
"ubi:helix-editor/helix" = "latest"
"ubi:sxyazi/yazi" = "latest"
"ubi:zellij-org/zellij" = "latest"
"ubi:nushell/nushell" = "latest"
"aqua:junegunn/fzf" = "latest"
"ubi:ajeetdsouza/zoxide" = "latest"
"ubi:starship/starship" = "latest"
"ubi:Macchina-CLI/macchina" = "latest"
"cargo:taplo-cli" = "latest"

# Recommended tools (if enabled)
"aqua:jesseduffield/lazygit" = "latest"
"ubi:atuinsh/atuin" = "latest"
"ubi:rsteube/carapace-bin" = "latest"
"cargo:markdown-oxide" = "latest"

# Yazi extensions (if enabled)
"aqua:sharkdp/fd" = "latest"
"aqua:BurntSushi/ripgrep" = "latest"
"aqua:jqlang/jq" = "latest"

[env]
YAZELIX_DIR = "~/.config/yazelix"
YAZELIX_BACKEND = "mise"
EDITOR = "hx"
# HELIX_RUNTIME set by mise shim
```

---

## Implementation Phases

### Phase 1: Foundation

- [ ] Add `[environment]` section to `yazelix_default.toml`
- [ ] Add corresponding options to `home_manager/module.nix`
- [ ] Create `nushell/scripts/backends/mod.nu` with dispatcher logic
- [ ] Create `nushell/scripts/backends/common.nu` with shared utilities
- [ ] Move current Nix logic to `nushell/scripts/backends/nix.nu`
- [ ] Update `start_yazelix.nu` to use backend dispatcher
- [ ] Add backend detection in `yazelix.nu` commands

**Completion criteria**: Existing Nix workflow unchanged, backend abstraction in place

### Phase 2: mise Backend Core

- [ ] Create `nushell/scripts/backends/mise.nu`
- [ ] Implement tool mapping table (yazelix package → mise tool spec)
- [ ] Implement `.mise.toml` generation from `yazelix.toml`
- [ ] Implement `mise install` wrapper with progress output
- [ ] Implement environment activation (`mise activate`)
- [ ] Handle PATH setup for mise-installed tools
- [ ] Download zjstatus.wasm from GitHub releases

**Completion criteria**: `yzx launch` works with `backend = "mise"`

### Phase 3: Environment Setup

- [ ] Implement shell hook generation for mise backend
- [ ] Set up `HELIX_RUNTIME` for mise-installed helix
- [ ] Configure `YAZI_CONFIG_HOME`
- [ ] Handle `EDITOR` and other env vars
- [ ] Implement `yzx env` for mise backend
- [ ] Add mise version pinning support (optional)

**Completion criteria**: Full yazelix environment works with mise backend

### Phase 4: Feature Parity

- [ ] Implement system dependency checking (p7zip, poppler, ffmpeg)
- [ ] Add helpful error messages for missing system deps
- [ ] Support `[packs]` section (where tools are mise-available)
- [ ] Support `user_packages` (limited to mise-available packages)
- [ ] Implement `yzx doctor` checks for mise backend
- [ ] Handle `helix.mode = "source"` (error with helpful message)

**Completion criteria**: Feature parity documentation accurate

### Phase 5: User Experience

- [ ] Add `yzx backend` command to show/switch backend
- [ ] Add `yzx backend migrate` to help users switch
- [ ] Implement `yzx doctor --fix` for mise backend
- [ ] Add first-run experience for mise backend
- [ ] Create mise backend section in installation.md
- [ ] Update troubleshooting.md with mise-specific issues

**Completion criteria**: Smooth onboarding for mise users

### Phase 6: Polish & Documentation

- [ ] Performance testing (startup time comparison)
- [ ] Disk usage measurement and documentation
- [ ] Edge case handling (offline mode, rate limits)
- [ ] Write migration guide (Nix → mise, mise → Nix)
- [ ] Update README with backend options
- [ ] Add mise backend to troubleshooting guide

**Completion criteria**: Production-ready mise backend

---

## Technical Details

### Backend Dispatcher (`backends/mod.nu`)

```nushell
# backends/mod.nu

use ./nix.nu
use ./mise.nu
use ./common.nu

# Detect configured backend from yazelix.toml
export def get_backend [] {
    let config = (common parse_config)
    $config.environment?.backend? | default "nix"
}

# Enter the yazelix environment using configured backend
export def enter_environment [--setup-only: bool] {
    let backend = (get_backend)
    match $backend {
        "nix" => { nix enter_environment --setup-only=$setup_only }
        "mise" => { mise enter_environment --setup-only=$setup_only }
        _ => { error make {msg: $"Unknown backend: ($backend)"} }
    }
}

# Check if environment is available
export def is_environment_available [] {
    let backend = (get_backend)
    match $backend {
        "nix" => { nix is_available }
        "mise" => { mise is_available }
        _ => false
    }
}
```

### mise Backend Implementation (`backends/mise.nu`)

```nushell
# backends/mise.nu

use ../utils/config_parser.nu [parse_yazelix_config]

# Tool mappings: yazelix package name → mise tool spec
const TOOL_MAPPINGS = {
    # Core
    helix: "ubi:helix-editor/helix"
    yazi: "ubi:sxyazi/yazi"
    zellij: "ubi:zellij-org/zellij"
    nushell: "ubi:nushell/nushell"
    fzf: "aqua:junegunn/fzf"
    zoxide: "ubi:ajeetdsouza/zoxide"
    starship: "ubi:starship/starship"
    macchina: "ubi:Macchina-CLI/macchina"
    taplo: "cargo:taplo-cli"

    # Recommended
    lazygit: "aqua:jesseduffield/lazygit"
    atuin: "ubi:atuinsh/atuin"
    carapace: "ubi:rsteube/carapace-bin"
    markdown-oxide: "cargo:markdown-oxide"

    # Yazi extensions
    fd: "aqua:sharkdp/fd"
    ripgrep: "aqua:BurntSushi/ripgrep"
    jq: "aqua:jqlang/jq"
}

# System dependencies that can't be installed via mise
const SYSTEM_DEPS = {
    yazi_extensions: ["p7zip", "poppler"]
    yazi_media: ["ffmpeg", "imagemagick"]
}

# Check if mise is installed
export def is_available [] {
    (which mise | is-not-empty)
}

# Generate .mise.toml from yazelix.toml configuration
export def generate_mise_config [] {
    let config = (parse_yazelix_config)
    let tools = (collect_tools $config)

    # Generate TOML content
    let content = (generate_toml $tools $config)

    # Write to yazelix directory
    let mise_path = $"($env.HOME)/.config/yazelix/.mise.toml"
    $content | save -f $mise_path

    print $"Generated mise config: ($mise_path)"
}

# Collect required tools based on config
def collect_tools [config: record] {
    mut tools = []

    # Always add core tools
    $tools = ($tools | append ["helix" "yazi" "zellij" "nushell" "fzf" "zoxide" "starship" "macchina" "taplo"])

    # Add recommended if enabled
    if ($config.core?.recommended_deps? | default true) {
        $tools = ($tools | append ["lazygit" "atuin" "carapace" "markdown-oxide"])
    }

    # Add yazi extensions if enabled
    if ($config.core?.yazi_extensions? | default true) {
        $tools = ($tools | append ["fd" "ripgrep" "jq"])
    }

    $tools
}

# Generate TOML content
def generate_toml [tools: list<string>, config: record] {
    let tool_lines = ($tools | each {|tool|
        let mise_spec = ($TOOL_MAPPINGS | get -i $tool)
        if ($mise_spec != null) {
            $"\"($mise_spec)\" = \"latest\""
        }
    } | compact | str join "\n")

    $"# Auto-generated by Yazelix - do not edit manually
# Regenerate with: yzx env --regenerate
# Backend: mise

[tools]
($tool_lines)

[env]
YAZELIX_DIR = \"~/.config/yazelix\"
YAZELIX_BACKEND = \"mise\"
IN_YAZELIX_SHELL = \"true\"
"
}

# Install all tools via mise
export def install_tools [] {
    print "Installing tools via mise..."

    cd ~/.config/yazelix
    ^mise install

    # Download zjstatus.wasm
    download_zjstatus
}

# Download zjstatus from GitHub releases
def download_zjstatus [] {
    let zjstatus_dir = $"($env.HOME)/.config/yazelix/configs/zellij"
    let zjstatus_path = $"($zjstatus_dir)/zjstatus.wasm"

    if not ($zjstatus_path | path exists) {
        print "Downloading zjstatus.wasm..."

        # Get latest release URL
        let release_url = "https://github.com/dj95/zjstatus/releases/latest/download/zjstatus.wasm"

        mkdir $zjstatus_dir
        http get $release_url | save $zjstatus_path

        print $"Downloaded zjstatus to: ($zjstatus_path)"
    }
}

# Enter the mise environment
export def enter_environment [--setup-only: bool] {
    # Ensure mise config exists
    if not ($"($env.HOME)/.config/yazelix/.mise.toml" | path exists) {
        generate_mise_config
    }

    # Install tools if needed
    install_tools

    # Check system dependencies
    check_system_deps

    if $setup_only {
        print "mise environment setup complete"
        return
    }

    # Activate mise and set environment
    cd ~/.config/yazelix

    # Set up environment variables
    $env.YAZELIX_DIR = $"($env.HOME)/.config/yazelix"
    $env.YAZELIX_BACKEND = "mise"
    $env.IN_YAZELIX_SHELL = "true"
    $env.YAZI_CONFIG_HOME = $"($env.HOME)/.local/share/yazelix/configs/yazi"

    # Get helix runtime path
    let helix_path = (^mise where helix | str trim)
    $env.HELIX_RUNTIME = $"($helix_path)/lib/runtime"
    $env.EDITOR = $"($helix_path)/bin/hx"
}

# Check for system dependencies and warn user
def check_system_deps [] {
    let config = (parse_yazelix_config)
    mut missing = []

    if ($config.core?.yazi_extensions? | default true) {
        for dep in $SYSTEM_DEPS.yazi_extensions {
            if (which $dep | is-empty) {
                $missing = ($missing | append $dep)
            }
        }
    }

    if ($config.core?.yazi_media? | default false) {
        for dep in $SYSTEM_DEPS.yazi_media {
            if (which $dep | is-empty) {
                $missing = ($missing | append $dep)
            }
        }
    }

    if ($missing | is-not-empty) {
        print $"(ansi yellow)Warning: Some features require system packages:(ansi reset)"
        print $"  Missing: ($missing | str join ', ')"
        print ""
        print "Install with your system package manager:"
        print $"  Ubuntu/Debian: sudo apt install ($missing | str join ' ')"
        print $"  Fedora: sudo dnf install ($missing | str join ' ')"
        print $"  macOS: brew install ($missing | str join ' ')"
        print ""
    }
}
```

### HELIX_RUNTIME Handling

For mise-installed helix, the runtime is bundled with the release:

```nushell
# Get helix runtime path for mise installation
def get_helix_runtime [] {
    let helix_path = (^mise where helix err> /dev/null | str trim)

    if ($helix_path | is-empty) {
        error make {msg: "Helix not installed via mise"}
    }

    # Helix releases include runtime in lib/runtime or runtime/
    let runtime_paths = [
        $"($helix_path)/lib/runtime"
        $"($helix_path)/runtime"
        $"($helix_path)/share/helix/runtime"
    ]

    for path in $runtime_paths {
        if ($path | path exists) {
            return $path
        }
    }

    error make {msg: $"Could not find helix runtime in ($helix_path)"}
}
```

---

## User Experience

### First Run (mise backend)

```
$ yzx launch

Yazelix - mise backend
======================

Checking mise installation... OK
Generating tool configuration...
Installing tools via mise:
  ✓ helix (25.01)
  ✓ yazi (0.4.2)
  ✓ zellij (0.42.0)
  ✓ nushell (0.102.0)
  ✓ fzf (0.57.0)
  ✓ zoxide (0.9.6)
  ✓ starship (1.22.0)
  ✓ lazygit (0.45.0)
  ...

Downloading zjstatus.wasm... OK

⚠ Some optional features require system packages:
  Missing: p7zip, poppler

  Install with: sudo apt install p7zip poppler-utils

Launching Yazelix...
```

### Backend Switching

```
$ yzx backend
Current backend: nix

$ yzx backend set mise
Switching to mise backend...

Note: The following features are not available with mise:
  - Terminal emulator management (provide your own terminal)
  - Helix source builds (release versions only)
  - Yazi media previews require system ffmpeg/imagemagick

Proceed? [y/N] y

Generating mise configuration...
Installing tools...
Done! Restart yazelix to use mise backend.

$ yzx backend set nix
Switching to nix backend...
Done! Run 'yzx launch' to rebuild nix environment.
```

### Error Messages

```
# When helix.mode = "source" with mise backend
$ yzx launch
Error: Helix source builds are not supported with mise backend.

The mise backend installs pre-built helix releases from GitHub.
To build helix from source, switch to the nix backend:

  [environment]
  backend = "nix"

  [helix]
  mode = "source"

Or use the release version with mise:

  [helix]
  mode = "release"  # This is the default
```

```
# When mise is not installed
$ yzx launch
Error: mise is not installed.

Yazelix is configured to use the mise backend, but mise was not found.

Install mise:
  curl https://mise.run | sh

Or switch to the nix backend in yazelix.toml:

  [environment]
  backend = "nix"
```

---

## Migration Guide

### Nix → mise

1. Install mise: `curl https://mise.run | sh`
2. Edit `yazelix.toml`:
   ```toml
   [environment]
   backend = "mise"
   ```
3. Run `yzx launch` - tools will be installed via mise
4. Install system dependencies if needed (p7zip, poppler, etc.)

### mise → Nix

1. Edit `yazelix.toml`:
   ```toml
   [environment]
   backend = "nix"
   ```
2. Run `yzx launch` - devenv will build the environment
3. Optionally uninstall mise tools: `mise uninstall --all`

---

## Limitations Documentation

### Tools Not Available via mise

These tools require system installation when using mise backend:

| Tool | Purpose | How to Install |
|------|---------|----------------|
| p7zip | Archive extraction (Yazi) | `apt install p7zip` / `brew install p7zip` |
| poppler | PDF preview (Yazi) | `apt install poppler-utils` / `brew install poppler` |
| ffmpeg | Video preview (Yazi) | `apt install ffmpeg` / `brew install ffmpeg` |
| imagemagick | Image processing (Yazi) | `apt install imagemagick` / `brew install imagemagick` |
| zjstatus | Zellij status bar | Auto-downloaded from GitHub releases |

### Features Not Supported

| Feature | Reason | Workaround |
|---------|--------|------------|
| Terminal management | mise doesn't package GUI apps | Install terminal separately |
| Terminal wrappers | Depends on Nix wrappers | Use terminal directly |
| nixGL support | Nix-specific | N/A (use native drivers) |
| Helix source builds | No build system | Use release builds |
| Desktop entry | Nix-generated | Create manually |
| Language packs | Limited coverage | Use mise's language support |

---

## Testing Plan

### Unit Tests

- [ ] Tool mapping completeness
- [ ] TOML generation correctness
- [ ] System dependency detection
- [ ] Error message formatting

### Integration Tests

- [ ] Fresh mise install + yazelix setup
- [ ] Backend switching (nix ↔ mise)
- [ ] Tool version updates
- [ ] Environment variable propagation
- [ ] Shell hook generation

### Platform Tests

- [ ] Linux x86_64
- [ ] Linux aarch64
- [ ] macOS x86_64
- [ ] macOS aarch64 (Apple Silicon)

### Edge Cases

- [ ] mise not installed
- [ ] Partial tool installation failure
- [ ] Network errors during download
- [ ] GitHub rate limiting
- [ ] Conflicting tool versions

---

## Success Criteria

- [ ] `yzx launch` works with `backend = "mise"`
- [ ] All core tools available and functional
- [ ] Clear error messages for unsupported features
- [ ] Documentation complete and accurate
- [ ] Startup time < 2 seconds (after initial install)
- [ ] Disk usage < 1.5GB for full install
- [ ] No admin rights required
- [ ] Works on all supported platforms

---

## Progress Tracking

### Current Status

- **Phase 1:** Not Started
- **Phase 2:** Not Started
- **Phase 3:** Not Started
- **Phase 4:** Not Started
- **Phase 5:** Not Started
- **Phase 6:** Not Started

### Open Questions

1. Should we support version pinning in yazelix.toml for mise tools?
2. How to handle mise trust prompts for `.mise.toml`?
3. Should we auto-activate mise in shell hooks or require explicit `yzx env`?
4. How to handle tools with different names (e.g., `nushell` binary is `nu`)?

---

## References

- [mise documentation](https://mise.jdx.dev/)
- [mise backends](https://mise.jdx.dev/dev-tools/backends/)
- [mise registry](https://mise.jdx.dev/registry.html)
- [ubi backend](https://mise.jdx.dev/dev-tools/backends/ubi.html)
- [aqua registry](https://github.com/aquaproj/aqua-registry)

---

**Last Updated**: 2025-01-26
**Author**: Claude (with Lucca)
