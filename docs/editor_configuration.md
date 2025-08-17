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

### Using Other Editors

```nix
# In yazelix.nix:
editor_command = "nvim";         # vim, nano, emacs, etc.
helix_runtime_path = null;       # Not needed for non-Helix editors
```

**Benefits:**
- ✅ **Works reliably** - Basic Zellij integration (new panes, tab naming)
- ✅ **Use any editor** - Full flexibility

**Limitations:**
- ❌ **Limited features** - No advanced integration (reveal in sidebar, same-instance opening)
- ❌ **No Helix-specific shortcuts** - Alt+y (reveal in Yazi), Ctrl+y (file picker) won't work

**Popular editor commands:**
- `"vim"` - Vi/Vim
- `"nvim"` - Neovim  
- `"nano"` - GNU Nano
- `"emacs"` - GNU Emacs
- `"kak"` - Kakoune
- `"/path/to/custom/editor"` - Custom editor with full path

## Integration Features

### Helix-Specific Features (when using Helix)

**Reveal in Yazi (Alt+y):**
- Jump from Helix buffer to the same file in Yazi sidebar
- Only works in sidebar mode with Helix

**File Picker (Ctrl+y):**
- Native Helix file picker integration
- Works in both sidebar and no-sidebar modes

**Smart Instance Management:**
- Opening files from Yazi reuses existing Helix instance when possible
- New panes created intelligently based on layout

**Buffer Navigation:**
- Yazelix tracks Helix buffers for navigation features

### Generic Editor Features (all editors)

**Basic File Opening:**
- Files open in new Zellij panes
- Tab naming based on file directory
- Working directory set correctly

**Zellij Integration:**
- Proper pane management and focus
- Terminal multiplexer benefits

## Troubleshooting

### Runtime Mismatch Errors

If you see errors like "runtime not found", "failed to load grammar", or version mismatches:

1. **Check your Helix version and yazelix's version:**
   ```bash
   hx --version                    # Your system Helix
   echo $EDITOR | xargs -- --version  # Yazelix's Helix
   ```

2. **Verify runtime path exists and is valid:**
   ```bash
   echo "HELIX_RUNTIME: $HELIX_RUNTIME"
   ls $HELIX_RUNTIME  # Should show: grammars/ languages.toml queries/ themes/
   ```

3. **Quick fix - use yazelix's Helix:**
   ```nix
   editor_command = null;        # Use yazelix's Helix
   helix_runtime_path = null;    # Use matching runtime
   ```

4. **Debug your custom setup:**
   ```bash
   # Test if your Helix works with its runtime
   HELIX_RUNTIME=/your/runtime/path hx --version
   ```

### Missing Integration Features

If Helix-specific features don't work:

1. **Verify Helix detection:**
   - Check that your `editor_command` ends with `hx` or `helix`
   - Full paths like `/usr/bin/hx` should work

2. **Check sidebar mode:**
   - Reveal in Yazi (Alt+y) only works with `enable_sidebar = true`

3. **Restart yazelix:**
   ```bash
   exit  # Exit current session
   yazelix  # Start fresh session
   ```

### File Opening Not Working

If files don't open when clicked in Yazi:

1. **Check EDITOR is set:**
   ```bash
   echo "EDITOR: $EDITOR"  # Should show your editor path
   ```

2. **Restart yazelix** to pick up configuration changes:
   ```bash
   exit  # Exit current session
   yazelix  # Start fresh
   ```

3. **Check logs for errors:**
   ```bash
   tail ~/.config/yazelix/logs/open_editor.log
   tail ~/.config/yazelix/logs/open_helix.log  # If using Helix
   ```

### Performance Issues

If editor startup is slow:

1. **Use default configuration** for fastest startup
2. **Check runtime path** - incorrect paths cause delays  
3. **Verify Helix plugins** - Custom configs can slow startup
4. **Profile startup time:**
   ```bash
   time $EDITOR --version  # Quick test
   ```

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

### Vim User
```nix
# yazelix.nix
{
  editor_command = "nvim";         # Or "vim", "nano", etc.
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
| Yazelix Helix (null) | ✅ | ✅ | ✅ | ✅ | ✅ |
| System Helix ("hx") | ✅ | ✅ | ✅ | ✅ | ✅ |
| Custom Helix (path) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Vim/Neovim | ✅ | ❌ | ❌ | ❌ | ✅ |
| Other Editors | ✅ | ❌ | ❌ | ❌ | ✅ |

**Legend:**
- **File Opening**: Click files in Yazi to open in editor
- **Reveal in Sidebar**: Alt+y from Helix jumps to file in Yazi
- **Same Instance**: Files open in existing editor instance when possible  
- **File Picker**: Ctrl+y in Helix for native file picking
- **Tab Naming**: Zellij tabs named after project/directory