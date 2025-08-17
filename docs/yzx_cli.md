# yzx Command Line Interface

The `yzx` command is Yazelix's unified CLI tool that provides shell-agnostic access to all Yazelix functionality. It works across all supported shells (bash, fish, zsh, nushell) with full subcommand support.

## Installation

The `yzx` command is automatically installed when you:
1. Run Yazelix for the first time (via terminal config or manual launch)
2. The command becomes available in your shell after the initial setup

## Command Reference

### Diagnostics

#### `yzx doctor [--verbose] [--fix]`
Comprehensive health checks and diagnostic tool for Yazelix.

**Options:**
- `--verbose` (`-v`): Show detailed diagnostic information
- `--fix` (`-f`): Attempt to automatically fix detected issues

**What it checks:**
- **Helix runtime conflicts**: Detects old `~/.config/helix/runtime` that breaks syntax highlighting
- **Environment variables**: EDITOR, HELIX_RUNTIME, and other critical settings
- **Configuration health**: yazelix.nix validation and shell integration
- **System status**: Log file sizes, file permissions, git repository state

**Auto-fix capabilities:**
- Backup conflicting runtime directories
- Clean oversized log files
- Create missing configuration files

**Examples:**
```bash
yzx doctor                    # Basic health check
yzx doctor --verbose          # Detailed diagnostics
yzx doctor --fix              # Auto-fix safe issues
yzx doctor -v -f              # Verbose output with auto-fix
```

### Configuration Management

#### `yzx config_status [shell]`
Show the status of shell configurations and Yazelix integration.

**Arguments:**
- `shell` (optional): Specific shell to check (`bash`, `fish`, `zsh`, `nushell`)

**Examples:**
```bash
yzx config_status            # Check all shell configurations
yzx config_status bash       # Check only bash configuration
yzx config_status nushell    # Check only nushell configuration
```

### Version and System Information

#### `yzx versions`
Display version information for all tools in the Yazelix environment.

Shows versions for:
- Yazelix itself
- Core tools (Yazi, Zellij, Helix, Nushell)
- Recommended tools (lazygit, starship, etc.)
- Extension tools (fzf, ripgrep, etc.)

#### `yzx info`
Show comprehensive Yazelix system information.

Displays:
- Yazelix version and description
- Configuration directory paths
- Current configuration settings (shell, terminal, helix mode)
- Persistent session configuration

### Launcher Commands

#### `yzx launch`
Launch Yazelix in a new terminal window using your preferred terminal emulator.

- Opens a new terminal window
- Automatically starts the Yazelix environment
- Respects your `preferred_terminal` setting in `yazelix.nix`

#### `yzx start`
Start Yazelix directly in the current terminal.

- Starts Yazelix in the current terminal session
- Useful for quick access without opening new windows
- Integrates with your current shell environment

#### `yzx restart`
Restart Yazelix while preserving persistent sessions.

**Behavior:**
- **With persistent sessions**: Shows information about session persistence, no restart needed
- **Without persistent sessions**: Launches new Yazelix instance, then kills the old session

### Help

#### `yzx help`
Display the complete command reference and usage information.

#### `yzx` (no arguments)
Alias for `yzx help` - shows the help message.

## Shell Integration

The `yzx` command works identically across all supported shells:

### Bash
```bash
yzx doctor --fix
yzx launch
```

### Fish
```fish
yzx doctor --fix
yzx launch
```

### Zsh
```zsh
yzx doctor --fix
yzx launch
```

### Nushell
```nu
yzx doctor --fix
yzx launch
```

## Common Workflows

### Daily Usage
```bash
# Quick health check
yzx doctor

# Launch Yazelix in new window
yzx launch

# Check system information
yzx info
```

### Troubleshooting
```bash
# Comprehensive diagnostics with auto-fix
yzx doctor --verbose --fix

# Check shell integration status
yzx config_status

# Verify tool versions
yzx versions
```

### Configuration Management
```bash
# Check all shell configurations
yzx config_status

# Check specific shell
yzx config_status bash

# View system info and current settings
yzx info
```

## Notes

- All commands respect your Yazelix configuration in `~/.config/yazelix/yazelix.nix`
- The `yzx` command automatically detects and ensures Nix environment availability
- Commands that modify files (like `yzx doctor --fix`) create backups for safety
- Shell integration commands work across all supported shells without modification

## See Also

- [Troubleshooting Guide](./troubleshooting.md) - For detailed problem-solving
- [Customization Guide](./customization.md) - For configuration options
- [Boot Sequence](./boot_sequence.md) - For understanding Yazelix startup process