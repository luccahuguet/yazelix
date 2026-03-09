# yzx Command Line Interface

Shell-agnostic CLI for Yazelix management. Works across bash, fish, zsh, and nushell.

## Commands

### `yzx doctor [--verbose] [--fix]`
Health checks and diagnostics
- `--verbose`: Detailed output
- `--fix`: Auto-fix safe issues

### `yzx dev test [--verbose] [--new-window] [--lint-only] [--sweep] [--visual] [--all] [--delay SECONDS]`
Run Yazelix test suite
- `--verbose`: Show detailed test output
- `--new-window`: Launch tests in a new Yazelix window (useful for debugging crashes)
- `--lint-only`: Run only syntax validation
- `--sweep`: Run only the non-visual configuration sweep
- `--visual`: Run only the visual terminal sweep (launches actual terminal windows)
- `--all`: Run the full suite plus the visual terminal sweep
- `--delay`: Delay between visual terminal launches in seconds (default: 3)

### `yzx dev bench [-n ITERATIONS] [-t TERMINAL] [--verbose]`
Benchmark terminal launch performance
- `-n, --iterations`: Number of iterations per terminal (default: 3)
- `-t, --terminal`: Test only specific terminal (e.g., ghostty, wezterm, kitty)
- `--verbose`: Show detailed output

### `yzx dev profile [--cold] [--clear-cache]`
Profile launch sequence and identify performance bottlenecks
- Default: Profile warm start (environment setup components from within Yazelix)
- `--cold`: Profile cold start from vanilla terminal (emulates desktop entry or fresh terminal launch)
- `--clear-cache`: Toggle yazelix.toml option and clear cache to force full Nix re-evaluation (simulates config change)

### `yzx launch [--here] [--path DIR] [--home] [--terminal TERM] [--verbose] [--skip-refresh]`
Launch Yazelix with directory and mode options
- Default: Launch new terminal in current directory
- `--here`: Start in current terminal (instead of new terminal)
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--terminal TERM`: Override terminal selection (e.g., ghostty, wezterm, kitty)
- `--verbose`: Print detailed launch diagnostics
- `--skip-refresh, -s`: Skip explicit refresh trigger and allow potentially stale environment

### `yzx env [--no-shell] [--skip-refresh]`
Load Yazelix environment without UI
- Default: Drop into your configured shell with all Yazelix tools available
- `--no-shell`: Stay in current shell (doesn't switch shells)
- `--skip-refresh, -s`: Skip explicit refresh trigger and allow potentially stale environment

### `yzx refresh [--force] [--verbose] [--very-verbose]`
Refresh Yazelix `devenv` evaluation cache/environment without launching UI
- Default: Refresh only when config or devenv inputs changed
- `--force`: Refresh even if no changes are detected
- `--verbose, -v`: Show configured top-level package scope and concise build progress
- `--very-verbose, -V`: Show full refresh internals and debug-level build output (`-vv` equivalent)
- Note: Refresh does not hot-replace your current Yazelix session. Use `yzx restart` to switch the current window to the refreshed profile, or `yzx launch` to open a separate Yazelix window on the refreshed profile.

### `yzx run <command> [args...]`
Run a single command in the Yazelix environment and exit
- Quote args that start with `-` to avoid flag parsing (e.g., `"-lc"`)

### `yzx gen_config <terminal>`
Print a terminal emulator config generated from `yazelix_default.toml`
- Example: `yzx gen_config alacritty`

### `yzx restart [--skip-refresh]`
Restart Yazelix (handles persistent sessions)
- `--skip-refresh, -s`: Skip explicit refresh trigger and allow potentially stale environment

### `yzx status [--versions] [--verbose] [--save]`
Show current Yazelix status
- Default: show active config, refresh state, shell hook summary, and key runtime settings
- `--versions, -V`: include the full tool version matrix
- `--verbose, -v`: include detailed shell hook status table
- `--save`: write the version matrix to `docs/version_table.md` (implies `--versions`)

### `yzx sponsor`
Open the Yazelix GitHub Sponsors page
- Opens `https://github.com/sponsors/luccahuguet` when possible
- Falls back to printing the URL if no opener is available

### `yzx update [--verbose]`
Show available update targets
- `--verbose`: accepted for consistency with subcommands
- Bare `yzx update` prints the available user-facing and maintainer update commands

### `yzx update all [--stash] [--verbose]`
Run the user-facing update set
- Updates the devenv CLI and then pulls the latest Yazelix repo changes
- `--stash`: stash local changes before the repo update and re-apply them afterwards
- `--verbose`: show verbose output for both update steps

### `yzx update devenv [--verbose]`
Update the devenv CLI in your Nix profile
- `yzx update devenv`: Update the devenv CLI in your Nix profile (`--verbose` shows underlying commands)

### `yzx update nix [--yes] [--verbose]`
Upgrade Determinate Nix
- `yzx update nix`: Upgrade Determinate Nix via `determinate-nixd` (`--yes` skips prompt, `--verbose` shows command; sudo required; only works if Determinate Nix is installed)

### `yzx update repo [--stash] [--verbose]`
Pull latest Yazelix updates from git
- `yzx update repo`: Pull latest Yazelix updates (`--stash` auto-stashes changes, `--verbose` shows git commands)

Maintainer-only updates:
- `yzx dev update`: Refresh `devenv.lock` via `devenv update`, then sync pinned runtime `nix`/`devenv` versions and refresh the vendored `configs/zellij/plugins/zjstatus.wasm` (`--yes` skips prompt, `--verbose` shows command)
- `yzx dev sync_terminal_configs`: Regenerate terminal configs and sync snapshots into `configs/terminal_emulators/`

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

### `yzx menu [--popup]`
Interactive command palette (fuzzy search)
- Default: inline mode in current terminal
- `--popup`: open in a Zellij floating pane (errors if not in Zellij)
- Lists most `yzx` commands while hiding maintenance-heavy or low-signal entries (`yzx dev*`, `yzx env`, `yzx run`)
- Cancel with `Esc`
- In popup mode after running a command: `Backspace` returns to menu, `Enter`/`Esc` closes popup
- Keybind: `Alt Shift m` opens the popup menu in Zellij
- Popup pane is named `yzx_menu` to avoid duplicate menu instances

### `yzx config [--full] [--path]`
Show the active Yazelix configuration via Nushell `open`
- Default: print the active config with `packs` hidden to reduce noise
- `--full`: include the `packs` section
- `--path`: print the resolved config path
- `yzx config hx`: show the `[helix]` section
- `yzx config yazi`: show the `[yazi]` section
- `yzx config zellij`: show the `[zellij]` section

### `yzx config open [--print]`
Open the active Yazelix configuration file in your editor
- Uses `$EDITOR` (set by Yazelix from `[editor] command` in yazelix.toml)
- `--print`: print the resolved config path without opening

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
yzx launch -s                 # Launch while skipping explicit refresh trigger

# Environment-only mode (no UI)
yzx env                       # Drop into configured shell with Yazelix tools
yzx env --no-shell            # Load tools but stay in current shell
yzx env -s                    # Load env while skipping explicit refresh trigger
yzx refresh                   # Refresh devenv cache if changes were detected
yzx refresh --force           # Force refresh even when up to date
yzx refresh -v                # Refresh with high-level progress
yzx refresh -V                # Refresh with full build logs (-vv equivalent)
yzx run lazygit               # Run single command and exit
yzx run bash "-lc" "lazygit"  # Run through a shell

# Diagnostics and info
yzx doctor --fix              # Health check with auto-fix
yzx config                    # Show active config without the packs section
yzx config --full             # Show the full config including packs
yzx config --path             # Print the active config path
yzx config hx                 # Show the Helix section only
yzx config yazi               # Show the Yazi section only
yzx config zellij             # Show the Zellij section only
yzx status                    # System information
yzx status --versions         # Show all tool versions
yzx status --verbose          # Show detailed shell hook status
yzx sponsor                   # Open the Yazelix sponsor page

# Updates
yzx update                    # Show update targets
yzx update all               # Update devenv CLI + pull Yazelix repo
yzx update devenv             # Update devenv CLI
yzx update nix                # Upgrade Determinate Nix via determinate-nixd (sudo)
yzx update repo --stash       # Pull repo updates and reapply local changes
yzx dev update --yes          # Refresh devenv.lock, sync pins, and refresh vendored zjstatus

# Garbage collection
yzx gc                        # Safe: clean devenv + remove unreferenced paths
yzx gc deep                   # Medium: also delete generations older than 30d
yzx gc deep 7d                # Medium: delete generations older than 7 days
yzx gc deeper                 # Aggressive: delete ALL old generations

# Packs
yzx packs                     # Show enabled packs summary with sizes
yzx packs --expand            # Show packages within each pack
yzx packs --all               # Show all declared packs (even disabled)

# Development verification
yzx dev test                  # Run the default test suite
yzx dev test --verbose        # Run tests with detailed output
yzx dev test --new-window     # Run tests in separate window (for debugging)
yzx dev test --lint-only      # Run only syntax validation
yzx dev test --sweep          # Run only the non-visual config/shell sweep
yzx dev test --visual         # Run only the visual terminal sweep
yzx dev test --all            # Run full suite plus visual terminal sweep

# Benchmarking
yzx dev bench                 # Benchmark all available terminals (3 iterations each)
yzx dev bench -n 5            # Run 5 iterations per terminal
yzx dev bench -t ghostty      # Benchmark only Ghostty
yzx dev bench -t wezterm -n 10 # Benchmark WezTerm with 10 iterations

# Profiling
# Note: Different launch scenarios have different performance characteristics
yzx dev profile               # Profile warm start (from within Yazelix shell)
yzx dev profile --cold        # Profile cold start (emulates desktop entry or vanilla terminal launch)
yzx dev profile --cold --clear-cache  # Profile after config change (toggles option and clears cache)

# Performance scenarios explained:
# 1. Warm start (~130ms): Already in Yazelix, launching tools/commands
# 2. Cold cached (~300-500ms): Desktop entry or vanilla terminal launch, config unchanged
# 3. Config change (~3-8s): After clearing cache (full Nix re-evaluation)
```
