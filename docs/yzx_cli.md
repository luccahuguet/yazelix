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
- Default: Profile warm start (environment setup components from within Yazelix)
- `--cold`: Profile cold start from vanilla terminal (emulates desktop entry or fresh terminal launch)
- `--clear-cache`: Toggle yazelix.toml option and clear cache to force full Nix re-evaluation (simulates config change)

### `yzx launch [--here] [--path DIR] [--home] [--terminal TERM] [--verbose]`
Launch Yazelix with directory and mode options
- Default: Launch new terminal in current directory
- `--here`: Start in current terminal (instead of new terminal)
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--terminal TERM`: Override terminal selection (e.g., ghostty, wezterm, kitty)
- `--verbose`: Print detailed launch diagnostics

### `yzx env [--no-shell]`
Load Yazelix environment without UI
- Default: Drop into your configured shell with all Yazelix tools available
- `--no-shell`: Stay in current shell (doesn't switch shells)

### `yzx run <command> [args...]`
Run a single command in the Yazelix environment and exit
- Quote args that start with `-` to avoid flag parsing (e.g., `"-lc"`)

### `yzx restart`
Restart Yazelix (handles persistent sessions)

### `yzx info`
Show system information and settings

### `yzx update`
Manage Yazelix updates
- `yzx update devenv`: Update the devenv CLI in your Nix profile (`--verbose` shows underlying commands)
- `yzx update lock`: Refresh `devenv.lock` via `devenv update` (`--yes` skips prompt, `--verbose` shows command)
- `yzx update zjstatus`: Update bundled zjstatus.wasm plugin
- `yzx update nix`: Upgrade Determinate Nix via `determinate-nixd` (`--yes` skips prompt, `--verbose` shows command; sudo required)
- `yzx update repo`: Pull latest Yazelix updates (`--stash` auto-stashes changes, `--verbose` shows git commands)
- `yzx update all`: Run `devenv`, `lock --yes`, and `zjstatus` updates

### `yzx gc [deep [PERIOD] | deeper]`
Garbage collection for Nix store
- `yzx gc`: Clean devenv generations + remove unreferenced paths
- `yzx gc deep`: Also delete generations older than 30 days
- `yzx gc deep 7d`: Delete generations older than 7 days (configurable period)
- `yzx gc deeper`: Delete ALL old generations (most aggressive)

### `yzx packs [--expand] [--all]`
Show enabled packs and their sizes
- `--expand`: Show individual packages within each pack
- `--all`: Show all declared packs (even disabled ones)

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
yzx run lazygit               # Run single command and exit
yzx run bash "-lc" "lazygit"  # Run through a shell

# Diagnostics and info
yzx doctor --fix              # Health check with auto-fix
yzx info                      # System information
yzx versions                  # Show all tool versions
yzx config_status bash        # Check bash integration

# Updates
yzx update devenv             # Update devenv CLI
yzx update lock --yes          # Refresh devenv.lock without prompt
yzx update nix                # Upgrade Determinate Nix via determinate-nixd (sudo)
yzx update repo --stash        # Pull repo updates and reapply local changes

# Garbage collection
yzx gc                        # Safe: clean devenv + remove unreferenced paths
yzx gc deep                   # Medium: also delete generations older than 30d
yzx gc deep 7d                # Medium: delete generations older than 7 days
yzx gc deeper                 # Aggressive: delete ALL old generations

# Packs
yzx packs                     # Show enabled packs summary with sizes
yzx packs --expand            # Show packages within each pack
yzx packs --all               # Show all declared packs (even disabled)

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
yzx profile --cold            # Profile cold start (emulates desktop entry or vanilla terminal launch)
yzx profile --cold --clear-cache  # Profile after config change (toggles option and clears cache)

# Performance scenarios explained:
# 1. Warm start (~130ms): Already in Yazelix, launching tools/commands
# 2. Cold cached (~300-500ms): Desktop entry or vanilla terminal launch, config unchanged
# 3. Config change (~3-8s): After clearing cache (full Nix re-evaluation)
```
