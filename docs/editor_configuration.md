# Editor Configuration

Yazelix provides smart editor configuration to avoid conflicts with existing installations while maintaining full integration features.

## Quick Start

**Most users should use the default:**
```jsonc
{
  "editor": {
    "command": ""
  }
}
```

**If you have specific needs:**
- Custom Helix fork → [Using A Custom Helix Fork](#using-a-custom-helix-fork)
- Prefer other editors → [Using Other Editors](#using-other-editors)  
- Runtime conflicts → See [Troubleshooting](#troubleshooting)

## How It Works

Yazelix sets your configured editor as the `EDITOR` environment variable throughout the system. The editor choice affects:
- **File opening behavior** from the Yazi file tree
- **Integration features** (reveal in the Yazi file tree, open in same instance, etc.)
- **Zellij pane management** and tab naming
- **Shell commands** that respect `$EDITOR`

## Configuration Options

### Default (Recommended): Yazelix's Helix

```jsonc
{
  "editor": {
    "command": ""
  },
  "helix": {
    "external": null
  }
}
```

**Benefits:**
- ✅ **No conflicts** with existing Helix installations
- ✅ **Always works** - binary and runtime are perfectly matched
- ✅ **Full integration** - All yazelix features work (reveal in the Yazi file tree, open in same instance, etc.)
- ✅ **Zero configuration** - Works out of the box

**How it works:**
- Yazelix uses its own Nix-provided Helix binary (`/nix/store/.../bin/hx`)
- Runtime is automatically set to the matching version (`/nix/store/.../share/helix/runtime`)
- The bundled editor is the thin `luccahuguet/yazelix-helix` Steel fork
- That fork tracks Helix Steel and carries the small `--config-dir` override Yazelix needs for managed sessions
- Managed Helix source files live under `~/.config/yazelix/helix/`, including `config.toml`, `languages.toml`, `themes/`, and custom Steel plugin files
- Custom Yazelix-managed themes belong in `~/.config/yazelix/helix/themes/`; native `~/.config/helix/themes/` remains outside Yazelix, and `~/.config/yazelix/user_conf/helix/themes/` is unsupported legacy state
- No interference with your system's Helix installation

### Using A Custom Helix Fork

```jsonc
{
  "helix": {
    "external": {
      "binary": "/home/user/helix/target/release/hx",
      "runtime_path": "/home/user/helix/runtime"
    }
  }
}
```

**Benefits:**
- ✅ **Full integration** - All yazelix features work if the binary and runtime match
- ✅ **Use your custom build** - Great for Helix developers

**Notes:**
- Leave `helix.external` as `null` to use Yazelix's bundled Helix
- Set `helix.external` only when you need a user-owned Helix fork
- Both `binary` and `runtime_path` are required together
- `yzx doctor` reports both paths and warns that binary/runtime revision mismatches are user-owned risk
- ⚠️ **Requires matching runtime** - Version mismatch causes startup errors
- ⚠️ **Manual configuration** - You must specify the correct binary and runtime path

**Finding your runtime path:**
```bash
# Automatic detection:
ls $(dirname $(which hx))/../share/helix/runtime

# Manual check for system-installed Helix:
which hx  # e.g., /usr/bin/hx → runtime at /usr/share/helix/runtime

# For custom builds:
ls ~/helix/runtime  # Should contain themes/, grammars/, queries/ directories

# Verify Helix can resolve a valid runtime:
hx --health | head -n 5
```

### Using Neovim

```jsonc
{
  "editor": {
    "command": "nvim"
  }
}
```

**Benefits:**
- ✅ **Full integration** - All yazelix features work (reveal in the Yazi file tree, open in same instance, etc.)
- ✅ **Smart instance management** - Files open in existing Neovim instance when possible
- ✅ **Managed pane targeting** - Yazelix finds and reuses your managed Neovim pane deterministically

**Setup Required:**
- ⚠️ **Add keybinding** - See [Neovim Keybindings](./neovim_keybindings.md) for a recommended reveal binding

**Popular Neovim commands:**
- `"nvim"` - Neovim from PATH
- `"/usr/bin/nvim"` - System Neovim with full path
- `"/nix/store/.../bin/nvim"` - Nix-provided Neovim

### Using Other Editors

```jsonc
{
  "editor": {
    "command": "vim"
  }
}
```

**Benefits:**
- ✅ **Works reliably** - Basic Zellij integration (new panes, tab naming)
- ✅ **Use any editor** - Full flexibility

**Limitations:**
- ❌ **Limited features** - No advanced integration (reveal in the Yazi file tree, same-instance opening)
- ❌ **No editor-specific shortcuts** - reveal in Yazi won't work without custom integration

**Popular editor commands:**
- `"vim"` - Vi/Vim
- `"nano"` - GNU Nano
- `"emacs"` - GNU Emacs
- `"kak"` - Kakoune
- `"/path/to/custom/editor"` - Custom editor with full path

## Integration Features

### Helix-Specific Features (when using Helix)

**Reveal in Yazi (managed binding):**
- Jump from Helix buffer to the same file in the Yazi file tree
- Works against the managed sidebar in the current Yazelix tab
- Default binding: `Alt+r`
- Managed through Yazelix's Helix config surface instead of `~/.config/helix/config.toml`
- If you want to adopt an existing personal Helix config, run `yzx import helix`
- Details: [Helix Keybindings](./helix_keybindings.md)

**File Picker:**
- Native Helix file picker integration
- Choose a Helix-local binding that does not conflict with your Yazelix workspace shortcuts

**Steel plugins:**
- Place custom Steel files below `~/.config/yazelix/helix/steel_plugins`
- Select bundled plugins with `helix.steel_plugins.enabled`
- Declare custom plugins in `helix.steel_plugins.extra`
- Only `public_commands` appear in Helix command completion
- `internal_commands` can be imported for plugin use without leaking into completion
- `startup_commands` run when the generated Yazelix Steel module loads
- Minimal example: [hello_yazelix.scm](./examples/helix_steel_plugins/hello_yazelix.scm)

```jsonc
{
  "helix": {
    "steel_plugins": {
      "enabled": ["splash", "spacemacs_theme"],
      "extra": [
        {
          "id": "my_picker",
          "source": "my_picker.scm",
          "public_commands": ["my-picker-open"],
          "internal_commands": ["my-picker-refresh"],
          "startup_commands": ["my-picker-refresh"],
          "command_descriptions": {
            "my-picker-open": "Open my custom picker",
            "my-picker-refresh": "Refresh my custom picker state"
          }
        }
      ]
    }
  }
}
```

For a complete teaching example with the matching manifest entry, see
[docs/examples/helix_steel_plugins](./examples/helix_steel_plugins/README.md).

**Smart Instance Management:**
- Opening files from Yazi reuses existing Helix instance when possible
- New panes created intelligently based on layout

**Buffer Navigation:**
- Yazelix tracks Helix buffers for navigation features

### Helix Wishlist

These are desired managed-Helix improvements, not current Yazelix support guarantees:

- **Code folding**: a real fold/unfold UI for syntax and LSP folding ranges, with clear indicators and keybindings that fit Helix's modal model
- **Sticky headers**: pinned context lines for the current function, type, module, or section while scrolling through larger files
- **Copilot**: a first-class AI completion path that works inside managed Helix without requiring users to assemble their own plugin/runtime setup
- **Inline git blame**: commit and author context rendered inline in the editor, beyond the default `A-g.b` shell shortcut

### Neovim-Specific Features (when using Neovim)

**Reveal in Yazi (custom binding):**
- Jump from Neovim buffer to the same file in the Yazi file tree
- Works against the managed sidebar in the current Yazelix tab
- Recommended binding: `Alt+r`
- Recommended command: `yzx reveal`
- Setup: [Neovim Keybindings](./neovim_keybindings.md)

**Smart Instance Management:**
- Opening files from Yazi reuses the managed Neovim pane when possible
- New panes created intelligently based on layout

**Managed Pane Targeting:**
- Yazelix targets the managed `editor` pane through the pane orchestrator plugin
- New editor panes are titled `editor` so later opens can reuse them deterministically
- Editors started manually from an ordinary shell pane are not automatically adopted as the managed `editor` pane

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
- Generic editors opened manually from shell panes remain ordinary panes; Yazelix-managed routing only applies to the managed `editor` pane

## Troubleshooting

**📋 [Complete Troubleshooting Guide →](./troubleshooting.md)** - Quick fixes for common issues

**Quick reset:** Run `yzx reset config --yes`, then restart Yazelix.

## Advanced Scenarios

### Multiple Helix Versions

If you have multiple Helix installations:

```jsonc
{
  "helix": {
    "external": {
      "binary": "/opt/helix-custom/bin/hx",
      "runtime_path": "/opt/helix-custom/share/helix/runtime"
    }
  }
}
```

### Development Setup

For Helix development:

```jsonc
{
  "helix": {
    "external": {
      "binary": "/home/user/helix/target/release/hx",
      "runtime_path": "/home/user/helix/runtime"
    }
  }
}
```

### Bundled Helix Configuration

For maximum reliability, keep the default managed Helix path:

```jsonc
{
  "editor": {
    "command": ""
  }
}
```

## Home Manager Integration

When using Home Manager, the same options apply:

```nix
programs.yazelix = {
  enable = true;
  
  # Editor configuration
  editor_command = null;        # Default: yazelix's Helix
  helix_external = null;        # Default: bundled matching binary/runtime
  helix_steel_plugins = {
    enabled = [ "splash" "spacemacs_theme" ];
    extra = [ ];
  };
  
  # Or custom:
  # helix_external = {
  #   binary = "/home/user/helix/target/release/hx";
  #   runtime_path = "/home/user/helix/runtime";
  # };
};
```

See `home_manager/examples/example.nix` for complete configuration examples.

## Common Configuration Examples

### Most Users (Recommended)
```jsonc
{
  "editor": {
    "command": ""
  },
  "helix": {
    "external": null
  }
}
```

### Helix Developer
```jsonc
{
  "helix": {
    "external": {
      "binary": "/home/user/helix/target/release/hx",
      "runtime_path": "/home/user/helix/runtime"
    }
  }
}
```

### Neovim User
```jsonc
{
  "editor": {
    "command": "nvim"
  }
}
```

**Remember:** Add a custom reveal binding to your Neovim config - see [Neovim Keybindings](./neovim_keybindings.md)

### Vim/Other Editor User
```jsonc
{
  "editor": {
    "command": "vim"
  }
}
```

### Custom Helix User (Advanced)
```jsonc
{
  "helix": {
    "external": {
      "binary": "/usr/bin/hx",
      "runtime_path": "/usr/share/helix/runtime"
    }
  }
}
```

## Integration Feature Matrix

| Editor Type | File Opening | Reveal in Sidebar | Same Instance | File Picker | Tab Naming |
|-------------|--------------|-------------------|---------------|-------------|------------|
| Yazelix Helix (null) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bundled Helix ("hx") | ✅ | ✅ | ✅ | ✅ | ✅ |
| External Helix pair | ✅ | ✅ | ✅ | ✅ | ✅ |
| Neovim ("nvim") | ✅ | ✅ (with setup) | ✅ | ✅ (Telescope) | ✅ |
| Vim | ✅ | ❌ | ❌ | ❌ | ✅ |
| Other Editors | ✅ | ❌ | ❌ | ❌ | ✅ |

**Legend:**
- **File Opening**: Click files in Yazi to open in editor
- **Reveal in Sidebar**: your editor-local reveal binding jumps to the file in Yazi
- **Same Instance**: Files open in existing editor instance when possible
- **File Picker**: Native file picking integration (Helix: custom binding, Neovim: Telescope/fzf-lua)
- **Tab Naming**: Zellij tabs named after project/directory

**Notes:**
- Neovim requires [keybinding setup](./neovim_keybindings.md) for reveal in the Yazi file tree
- File picker in Neovim works with your existing plugins (Telescope, fzf-lua, etc.)
