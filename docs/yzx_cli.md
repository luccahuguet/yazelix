# yzx Command Line Interface

Shell-agnostic CLI for Yazelix management. Works across bash, fish, zsh, and nushell.

## Commands

### `yzx doctor [--verbose] [--fix]`
Health checks and diagnostics
- `--verbose`: Detailed output
- `--fix`: Auto-fix safe issues

### `yzx launch`
Launch Yazelix in new terminal window

### `yzx start` 
Start Yazelix in current terminal

### `yzx restart`
Restart Yazelix (handles persistent sessions)

### `yzx info`
Show system information and settings

### `yzx versions`
Display all tool versions

### `yzx config_status [shell]`
Check shell configuration status

### `yzx help`
Show command reference

## Examples

```bash
yzx doctor --fix              # Health check with auto-fix
yzx launch                    # New terminal window
yzx info                      # System information
yzx config_status bash        # Check bash integration
```