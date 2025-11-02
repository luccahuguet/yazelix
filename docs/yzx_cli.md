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

### `yzx bench [-n ITERATIONS] [-t TERMINAL] [--verbose]`
Benchmark terminal launch performance
- `-n, --iterations`: Number of iterations per terminal (default: 3)
- `-t, --terminal`: Test only specific terminal (e.g., ghostty, wezterm, kitty)
- `--verbose`: Show detailed output

### `yzx profile [--cold] [--clear-cache]`
Profile launch sequence and identify performance bottlenecks
- Default: Profile warm start (environment setup components)
- `--cold`: Profile cold start from vanilla terminal (measures real-world launch)
- `--clear-cache`: Clear devenv cache before profiling (simulates config change)

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

# Testing and benchmarking
yzx test                      # Run all tests (non-visual)
yzx test --verbose            # Run tests with detailed output
yzx test --new-window         # Run tests in separate window (for debugging)
yzx test --all                # Run all tests including visual terminal sweep

# Benchmarking
yzx bench                     # Benchmark all available terminals (3 iterations each)
yzx bench -n 5                # Run 5 iterations per terminal
yzx bench -t ghostty          # Benchmark only Ghostty
yzx bench -t wezterm -n 10    # Benchmark WezTerm with 10 iterations

# Profiling
# Note: Different launch scenarios have different performance characteristics
yzx profile                   # Profile warm start (from within Yazelix shell)
yzx profile --cold            # Profile cold start with cache (run from vanilla terminal)
yzx profile --cold --clear-cache  # Profile cold start after config change (clears cache)

# Performance scenarios:
# 1. Warm start (~130ms): Already in Yazelix, launching tools/commands
# 2. Cold cached (~300-500ms): Desktop entry launch, config unchanged
# 3. Cold no-cache (~2-5s): First launch or config changed
```