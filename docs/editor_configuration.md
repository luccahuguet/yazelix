# Editor Configuration

Yazelix provides smart editor configuration to avoid conflicts with existing installations while maintaining full integration features.

## Quick Start

**Most users should use the default:**
```nix
editor_command = null;  # Uses yazelix's Helix - no conflicts, full features
```

**If you have specific needs:**
- Existing Helix setup → [Using Your Existing Helix](#using-your-existing-helix)
- Prefer other editors → [Using Other Editors](#using-other-editors)  
- Runtime conflicts → See [Troubleshooting](#troubleshooting)

## How It Works

Yazelix sets your configured editor as the `EDITOR` environment variable throughout the system. The editor choice affects:
- **File opening behavior** from Yazi file manager
- **Integration features** (reveal in sidebar, open in same instance, etc.)
- **Zellij pane management** and tab naming
- **Shell commands** that respect `$EDITOR`

## Configuration Options

### Default (Recommended): Yazelix's Helix

```nix
# In yazelix.nix:
editor_command = null;           # Use yazelix's Nix-provided Helix
helix_runtime_path = null;       # Use matching runtime automatically
```

**Benefits:**
- ✅ **No conflicts** with existing Helix installations
- ✅ **Always works** - binary and runtime are perfectly matched
- ✅ **Full integration** - All yazelix features work (reveal in sidebar, open in same instance, etc.)
- ✅ **Zero configuration** - Works out of the box

**How it works:**
- Yazelix uses its own Nix-provided Helix binary (`/nix/store/.../bin/hx`)
- Runtime is automatically set to the matching version (`/nix/store/.../share/helix/runtime`)
- No interference with your system's Helix installation

### Using Your Existing Helix

```nix
# In yazelix.nix:
editor_command = "hx";                           # Use system Helix from PATH
helix_runtime_path = "/home/user/helix/runtime"; # MUST match your Helix version
```

**Benefits:**
- ✅ **Full integration** - All yazelix features work if runtime matches
- ✅ **Use your custom build** - Great for Helix developers

**Requirements:**
- ⚠️ **Requires matching runtime** - Version mismatch causes startup errors
- ⚠️ **Manual configuration** - You must specify the correct runtime path

**Finding your runtime path:**
```bash
# Automatic detection:
ls $(dirname $(which hx))/../share/helix/runtime

# Manual check for system-installed Helix:
which hx  # e.g., /usr/bin/hx → runtime at /usr/share/helix/runtime

# For custom builds:
ls ~/helix/runtime  # Should contain themes/, grammars/, queries/ directories

# Verify runtime is valid:
ls $HELIX_RUNTIME  # Should show: grammars/ languages.toml queries/ themes/
```

### Using Neovim

```nix
# In yazelix.nix:
editor_command = "nvim";         # Use Neovim
helix_runtime_path = null;       # Not needed for Neovim
```

**Benefits:**
- ✅ **Full integration** - All yazelix features work (reveal in sidebar, open in same instance, etc.)
- ✅ **Smart instance management** - Files open in existing Neovim instance when possible
- ✅ **Pane detection** - Yazelix finds and reuses your Neovim panes intelligently

**Setup Required:**
- ⚠️ **Add keybinding** - See [Neovim Keybindings](./neovim_keybindings.md) for Alt+y setup

**Popular Neovim commands:**
- `"nvim"` - Neovim from PATH
- `"/usr/bin/nvim"` - System Neovim with full path
- `"/nix/store/.../bin/nvim"` - Nix-provided Neovim

### Using Other Editors

```nix
# In yazelix.nix:
editor_command = "vim";          # vim, nano, emacs, etc.
helix_runtime_path = null;       # Not needed for non-Helix editors
```

**Benefits:**
- ✅ **Works reliably** - Basic Zellij integration (new panes, tab naming)
- ✅ **Use any editor** - Full flexibility

**Limitations:**
- ❌ **Limited features** - No advanced integration (reveal in sidebar, same-instance opening)
- ❌ **No editor-specific shortcuts** - Alt+y (reveal in Yazi) won't work

**Popular editor commands:**
- `"vim"` - Vi/Vim
- `"nano"` - GNU Nano
- `"emacs"` - GNU Emacs
- `"kak"` - Kakoune
- `"/path/to/custom/editor"` - Custom editor with full path

## Integration Features

### Helix-Specific Features (when using Helix)

**Reveal in Yazi (Alt+y):**
- Jump from Helix buffer to the same file in Yazi sidebar
- Only works in sidebar mode with Helix
- Setup: [Helix Keybindings](./helix_keybindings.md)

**File Picker (Ctrl+y):**
- Native Helix file picker integration
- Works in both sidebar and no-sidebar modes

**Smart Instance Management:**
- Opening files from Yazi reuses existing Helix instance when possible
- New panes created intelligently based on layout

**Buffer Navigation:**
- Yazelix tracks Helix buffers for navigation features

### Neovim-Specific Features (when using Neovim)

**Reveal in Yazi (Alt+y):**
- Jump from Neovim buffer to the same file in Yazi sidebar
- Only works in sidebar mode with Neovim
- Setup: [Neovim Keybindings](./neovim_keybindings.md)

**Smart Instance Management:**
- Opening files from Yazi reuses existing Neovim instance when possible
- Checks up to 4 panes to find existing Neovim instances
- New panes created intelligently based on layout

**Pane Detection:**
- Yazelix automatically detects running Neovim instances
- Moves found instances to top of pane stack for focus

**Command Integration:**
- Files opened via `:edit` command in existing instances
- Working directory changed via `:cd` command automatically

### Generic Editor Features (all editors)

**Basic File Opening:**
- Files open in new Zellij panes
- Tab naming based on file directory
- Working directory set correctly

**Zellij Integration:**
- Proper pane management and focus
- Terminal multiplexer benefits

## Troubleshooting

**📋 [Complete Troubleshooting Guide →](./troubleshooting.md)** - Quick fixes for common issues

**Quick reset:** Delete `yazelix.nix` and run `yzx launch` to regenerate defaults.

## Advanced Scenarios

### Multiple Helix Versions

If you have multiple Helix installations:

```nix
# Use specific version
editor_command = "/opt/helix-custom/bin/hx";
helix_runtime_path = "/opt/helix-custom/share/helix/runtime";
```

### Development Setup

For Helix development:

```nix
# Use your development build
editor_command = "/home/user/helix/target/release/hx";
helix_runtime_path = "/home/user/helix/runtime";
```

### Fallback Configuration

For maximum reliability:

```nix
# Always use yazelix's Helix
editor_command = null;
helix_runtime_path = null;
```

## Home Manager Integration

When using Home Manager, the same options apply:

```nix
programs.yazelix = {
  enable = true;
  
  # Editor configuration
  editor_command = null;        # Default: yazelix's Helix
  helix_runtime_path = null;    # Default: matching runtime
  
  # Or custom:
  # editor_command = "hx";
  # helix_runtime_path = "/home/user/helix/runtime";
};
```

See `home_manager/examples/example.nix` for complete configuration examples.

## Common Configuration Examples

### Most Users (Recommended)
```nix
# yazelix.nix
{
  editor_command = null;           # Use yazelix's Helix
  helix_runtime_path = null;       # Use matching runtime
  # ... other settings
}
```

### Helix Developer
```nix
# yazelix.nix  
{
  editor_command = "/home/user/helix/target/release/hx";
  helix_runtime_path = "/home/user/helix/runtime";
  # ... other settings
}
```

### Neovim User
```nix
# yazelix.nix
{
  editor_command = "nvim";         # Use Neovim
  helix_runtime_path = null;       # Not needed for Neovim
  # ... other settings
}
```

**Remember:** Add Alt+y keybinding to your Neovim config - see [Neovim Keybindings](./neovim_keybindings.md)

### Vim/Other Editor User
```nix
# yazelix.nix
{
  editor_command = "vim";          # Or "nano", "emacs", etc.
  helix_runtime_path = null;       # Not needed for non-Helix
  # ... other settings
}
```

### System Helix User (Advanced)
```nix
# yazelix.nix
{
  editor_command = "hx";                              # Use system Helix
  helix_runtime_path = "/usr/share/helix/runtime";    # Match system runtime
  # ... other settings
}
```

## Integration Feature Matrix

| Editor Type | File Opening | Reveal in Sidebar | Same Instance | File Picker | Tab Naming |
|-------------|--------------|-------------------|---------------|-------------|------------|
| Yazelix Helix (null) | ✅ | ✅ | ✅ | ✅ (Ctrl+y) | ✅ |
| System Helix ("hx") | ✅ | ✅ | ✅ | ✅ (Ctrl+y) | ✅ |
| Custom Helix (path) | ✅ | ✅ | ✅ | ✅ (Ctrl+y) | ✅ |
| Neovim ("nvim") | ✅ | ✅ (with setup) | ✅ | ✅ (Telescope) | ✅ |
| Vim | ✅ | ❌ | ❌ | ❌ | ✅ |
| Other Editors | ✅ | ❌ | ❌ | ❌ | ✅ |

**Legend:**
- **File Opening**: Click files in Yazi to open in editor
- **Reveal in Sidebar**: Alt+y from editor jumps to file in Yazi
- **Same Instance**: Files open in existing editor instance when possible
- **File Picker**: Native file picking integration (Helix: Ctrl+y, Neovim: Telescope/fzf-lua)
- **Tab Naming**: Zellij tabs named after project/directory

**Notes:**
- Neovim requires [keybinding setup](./neovim_keybindings.md) for reveal in sidebar (Alt+y)
- File picker in Neovim works with your existing plugins (Telescope, fzf-lua, etc.)