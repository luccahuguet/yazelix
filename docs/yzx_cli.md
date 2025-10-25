# yzx Command Line Interface

Shell-agnostic CLI for Yazelix management. Works across bash, fish, zsh, and nushell.

## Commands

### `yzx doctor [--verbose] [--fix]`
Health checks and diagnostics
- `--verbose`: Detailed output
- `--fix`: Auto-fix safe issues

### `yzx test [--verbose] [--new-window] [--all]`
Run Yazelix test suite
- `--verbose`: Show detailed test output
- `--new-window`: Launch tests in a new Yazelix window (useful for debugging crashes)
- `--all`: Include visual terminal sweep tests (launches actual terminal windows)

### `yzx launch [--here] [--path DIR] [--home] [--terminal TERM] [--verbose]`
Launch Yazelix with directory and mode options
- Default: Launch new terminal in current directory
- `--here`: Start in current terminal (instead of new terminal)
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--terminal TERM`: Override terminal selection (e.g., ghostty, wezterm, kitty)
- `--verbose`: Print detailed launch diagnostics

### `yzx env [--no-shell] [--command CMD]`
Load Yazelix environment without UI
- Default: Drop into your configured shell with all Yazelix tools available
- `--no-shell`: Stay in current shell (doesn't switch shells)
- `--command CMD`: Run a single command in Yazelix environment and exit

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
# Launch Yazelix
yzx launch                    # New terminal in current directory
yzx launch --here             # Start in current terminal
yzx launch --home             # New terminal in home directory
yzx launch --here --path ~/project  # Current terminal, specific directory
yzx launch --terminal wezterm # Force WezTerm for this launch
yzx launch --verbose          # Detailed launch diagnostics

# Environment-only mode (no UI)
yzx env                       # Drop into configured shell with Yazelix tools
yzx env --no-shell            # Load tools but stay in current shell
yzx env --command "lazygit"   # Run single command and exit

# Diagnostics and info
yzx doctor --fix              # Health check with auto-fix
yzx info                      # System information
yzx versions                  # Show all tool versions
yzx config_status bash        # Check bash integration

# Testing
yzx test                      # Run all tests (non-visual)
yzx test --verbose            # Run tests with detailed output
yzx test --new-window         # Run tests in separate window (for debugging)
yzx test --all                # Run all tests including visual terminal sweep
```