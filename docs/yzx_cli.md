# yzx Command Line Interface

Shell-agnostic CLI for Yazelix management. Works across bash, fish, zsh, and nushell.

## Commands

### `yzx doctor [--verbose] [--fix]`
Health checks and diagnostics
- `--verbose`: Detailed output
- `--fix`: Auto-fix safe issues

### `yzx test [--verbose] [--filter] [--new-window]`
Run Yazelix test suite
- `--verbose`: Show detailed test output
- `--filter`: Filter tests by name pattern
- `--new-window`: Launch tests in a new Yazelix window (useful for debugging crashes)

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
yzx test                      # Run all tests
yzx test --verbose            # Run tests with detailed output
yzx test --filter nix         # Run only Nix-related tests
yzx test --new-window         # Run tests in separate window (for debugging)
yzx launch                    # New terminal window
yzx info                      # System information
yzx config_status bash        # Check bash integration
```