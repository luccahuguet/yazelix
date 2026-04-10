# yzx Command Line Interface

Shell-agnostic CLI for Yazelix management. Works across bash, fish, zsh, and nushell.

## Commands

### `yzx doctor [--verbose] [--fix]`
Health checks and diagnostics
- `--verbose`: Detailed output
- `--fix`: Auto-fix safe issues
- Warns when `yazelix.toml` has stale, removed, or invalid fields relative to `yazelix_default.toml`
- Reports the active runtime/distribution tier before deeper checks
- Runs installer-owned runtime-link and stable-launcher diagnostics only when the current mode actually owns those surfaces

### `yzx dev test [--verbose] [--new-window] [--lint-only] [--profile] [--sweep] [--visual] [--all] [--delay SECONDS]`
Run Yazelix test suite
- Default: run the normal non-sweep automated regression suite
- `--verbose`: Show detailed test output
- `--new-window`: Launch tests in a new Yazelix window (useful for debugging crashes)
- `--lint-only`: Run only syntax validation
- `--profile`: Print timing summaries for the default suite and the internal `test_yzx_commands.nu` sub-suites
- `--sweep`: Run only the non-visual configuration sweep
- `--visual`: Run only the visual terminal sweep (launches actual terminal windows)
- `--all`: Run the default suite plus non-visual sweep + visual sweep
- `--delay`: Delay between visual terminal launches in seconds (default: 3)

### `yzx dev bench [-n ITERATIONS] [-t TERMINAL] [--verbose]`
Benchmark terminal launch performance
- `-n, --iterations`: Number of iterations per terminal (default: 3)
- `-t, --terminal`: Test only specific terminal (e.g., ghostty, wezterm, kitty)
- `--verbose`: Show detailed output

### `yzx dev profile [--cold] [--clear-cache]`
Profile launch sequence and identify performance bottlenecks
- Default: Profile the current-terminal startup path and write a structured startup report under `~/.local/share/yazelix/profiles/startup/`
- `--cold`: Profile cold startup from a vanilla terminal (outside Yazelix)
- `--clear-cache`: Clear the runtime project cache plus recorded materialized/launch state first so the profiled run exercises the rebuild-heavy path
- The summary breaks out real startup phases such as preflight, config-state checks, `devenv` shell entry, shellHook setup, and inner startup work

### `yzx dev bump VERSION`
Automate the version bump, release commit, and matching git tag
- Requires a clean git worktree
- Fails if `VERSION` is not a real Yazelix tag like `v14` or `v13.13`
- Refuses to reuse an existing git tag
- Rotates the current `Unreleased` release notes into the requested version, resets a fresh `Unreleased` placeholder, updates `YAZELIX_VERSION`, syncs the README title/version marker, creates a dedicated commit, and creates the matching annotated tag
- Refuses to run if `CHANGELOG.md` or `docs/upgrade_notes.toml` still contain the untouched default `Unreleased` placeholder text

### `yzx launch [--path DIR] [--home] [--terminal TERM] [--verbose] [--reuse] [--skip-refresh] [--force-reenter]`
Launch Yazelix with directory and mode options
- Default: Launch new terminal in current directory
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--terminal TERM`: Override terminal selection (e.g., ghostty, wezterm, kitty)
- `--verbose`: Print detailed launch diagnostics
- `--reuse`: Reuse the last built Yazelix profile without rebuilding (errors if no cached profile exists)
- `--skip-refresh, -s`: Skip explicit refresh trigger and allow potentially stale environment
- `--force-reenter`: Force a fresh `devenv` re-entry before launch

### `yzx enter [--path DIR] [--home] [--verbose] [--reuse] [--skip-refresh] [--force-reenter]`
Start Yazelix in the current terminal
- Default: Start in the current terminal and current directory
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--verbose`: Print detailed startup diagnostics
- `--reuse`: Reuse the last built Yazelix profile without rebuilding (errors if no cached profile exists)
- `--skip-refresh, -s`: Skip explicit refresh trigger and allow potentially stale environment
- `--force-reenter`: Force a fresh `devenv` re-entry before startup

### `yzx env [--no-shell] [--reuse] [--skip-refresh]`
Load Yazelix environment without UI
- Default: Drop into your configured shell with all Yazelix tools available
- `--no-shell`: Stay in current shell (doesn't switch shells)
- `--reuse`: Reuse the last built Yazelix profile without rebuilding (errors if no cached profile exists)
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
- `yzx run` is a wrapped argv passthrough: the first token is the child command and the remaining tokens are forwarded unchanged
- Dash-prefixed child args do not need special quoting just to avoid Yazelix flag parsing
- If you want shell parsing, call the shell explicitly, for example: `yzx run bash -lc "lazygit"`

### `yzx cwd [DIR]`
Retarget the current tab workspace root inside Zellij
- Default: use the current directory when `DIR` is omitted
- When `DIR` is not an existing path, Yazelix resolves it with `zoxide query`
- Updates the current tab's Yazelix workspace root and renames the tab
- Also applies the directory change to the current pane after the command returns
- When a managed Helix or Neovim editor pane is present, its cwd is also updated
- When sidebar mode is enabled, the managed Yazi sidebar also follows the updated directory
- Other existing panes keep their current working directories; new managed actions use the updated tab directory
- Errors when run outside Zellij

### `yzx reveal PATH`
Reveal a file or directory in the managed Yazi sidebar
- Targets the managed sidebar in the current Zellij tab
- Focuses the sidebar after revealing the target when the sidebar is available
- Intended as the stable editor-integration surface for Helix/Neovim keybindings
- Errors clearly when run outside a Yazelix/Zellij session or when the managed sidebar is unavailable

### `yzx keys`
Show Yazelix-owned keybindings and remaps
- Default: print the small set of workspace-critical Yazelix bindings
- Ends with pointers to tool-specific discoverability helpers
- `yzx keys yzx`: alias for the default Yazelix view
- `yzx keys yazi`: explain how to view Yazi's built-in keybindings from inside Yazi
- `yzx keys hx`: explain how to discover Helix bindings and commands
- `yzx keys helix`: alias for `yzx keys hx`
- `yzx keys nu`: show a small curated subset of useful Nushell keybindings
- `yzx keys nushell`: alias for `yzx keys nu`

### `yzx tutor`
Show the guided Yazelix overview
- Default: print the Yazelix-specific tutor with the workspace model and next-step commands
- Keeps a clear split with other help surfaces: `yzx help` is command reference, `yzx keys` is keybinding discoverability
- `yzx tutor hx`: launch Helix's built-in tutorial via `hx --tutor`
- `yzx tutor helix`: alias for `yzx tutor hx`
- `yzx tutor nu`: launch Nushell's built-in tutorial in a fresh `nu` process
- `yzx tutor nushell`: alias for `yzx tutor nu`

### `yzx restart [--reuse] [--skip-refresh]`
Restart Yazelix (handles persistent sessions)
- `--reuse`: Reopen Yazelix from the last built profile without rebuilding (errors if no cached profile exists)
- `--skip-refresh, -s`: Skip explicit refresh trigger and allow potentially stale environment

### `yzx status [--versions] [--verbose]`
Show current Yazelix status
- Default: show active config, refresh state, shell hook summary, and key runtime settings
- `--versions, -V`: include the full tool version matrix
- `--verbose, -v`: include detailed shell hook status table

### `yzx sponsor`
Open the Yazelix GitHub Sponsors page
- Opens `https://github.com/sponsors/luccahuguet` when possible
- Falls back to printing the URL if no opener is available

### `yzx whats_new`
Show the current Yazelix release summary on demand
- Prints the current version entry from `docs/upgrade_notes.toml`
- Marks the current version as seen so startup does not need to repeat the same note
- Reuses the same migration-aware guidance shown on first interactive run after an upgrade

### `yzx update`
Show available update targets
- Bare `yzx update` prints the supported update-owner choices
- It points users at `yzx update upstream` or `yzx update home_manager`
- It warns users not to mix both update paths for the same installed Yazelix runtime

### `yzx update upstream`
Refresh Yazelix from the upstream installer surface
- Prints the exact command it will run
- Runs `nix run github:luccahuguet/yazelix#install`
- Intended for installs driven by the upstream/manual installer path

### `yzx update home_manager`
Refresh the current Home Manager flake input, then print the manual switch step
- Must be run from the Home Manager flake directory that owns the install
- Prints the exact command it will run
- Runs `nix flake update yazelix`
- Prints `home-manager switch` for the user to copy and run manually

### `yzx update nix [--yes] [--verbose]`
Upgrade Determinate Nix
- `yzx update nix`: Upgrade Determinate Nix via `determinate-nixd` (`--yes` skips prompt, `--verbose` shows command; sudo required; only works if Determinate Nix is installed)

Maintainer-only updates:
- `yzx dev update`: Refresh `devenv.lock` via `devenv update`, run canary refresh/build checks (`default`, `maximal`), then sync pinned runtime expectations, refresh the vendored `configs/zellij/plugins/zjstatus.wasm`, refresh vendored Yazi plugin runtime files from the pinned source map in `config_metadata/vendored_yazi_plugins.toml`, and perform one explicit activation step selected by the required `--activate installer|home_manager|none` flag (`home_manager` refreshes the Home Manager flake input before `home-manager switch`; `none` leaves local activation untouched). `--canary-only` is the only path that does not require `--activate`.
- `yzx dev sync_terminal_configs`: Regenerate terminal configs and sync snapshots into `configs/terminal_emulators/`
- `yzx dev build_pane_orchestrator [--sync]`: Build the Zellij pane orchestrator wasm for `wasm32-wasip1`; `--sync` also updates the tracked/runtime plugin paths after a successful build, preserves previously granted plugin permissions onto the stable runtime path when possible, and regenerates Zellij config. After syncing, prefer restarting Yazelix over reloading the plugin in place. If the toolchain is missing, enable the `rust_wasi` pack.

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
- Keybind: `Alt Shift M` opens the popup menu in Zellij
- Popup pane is named `yzx_menu` to avoid duplicate menu instances

### `yzx popup [COMMAND ...]`
Open a transient floating-pane command inside Zellij
- Default: runs `zellij.popup_program` from `yazelix.toml`
- `COMMAND ...`: override the configured popup command for one invocation
- Uses the current tab workspace root as cwd when available; otherwise uses the current shell cwd
- Errors if not in Zellij
- Default keybind: `Alt t`
- Popup pane is named `yzx_popup`

### `yzx config [--full] [--path]`
Show the active Yazelix configuration via Nushell `open`
- Default: print the active config with `packs` hidden to reduce noise
- `--full`: include the `packs` section
- `--path`: print the resolved config path

### `yzx import zellij|yazi|helix [--force]`
Import native Zellij, Yazi, or Helix config into Yazelix-managed overrides
- `yzx import zellij`: copies `~/.config/zellij/config.kdl` into `user_configs/zellij/config.kdl`
- `yzx import yazi`: imports `yazi.toml`, `keymap.toml`, and `init.lua` from `~/.config/yazi/` into `user_configs/yazi/`
- `yzx import helix`: copies `~/.config/helix/config.toml` into `user_configs/helix/config.toml`
- Fails clearly when no native source files are available for the selected target
- Refuses to overwrite existing managed destination files by default
- `--force`: writes `*.backup-<timestamp>` backups before replacing managed destination files

### `yzx edit config [--print]`
Open the main Yazelix config file in your editor
- Uses `$EDITOR` (set by Yazelix from `[editor] command` in yazelix.toml)
- Targets `user_configs/yazelix.toml`
- `--print`: print the resolved config path without opening

### `yzx edit packs [--print]`
Open the Yazelix pack sidecar in your editor
- Uses `$EDITOR` (set by Yazelix from `[editor] command` in yazelix.toml)
- Targets `user_configs/yazelix_packs.toml`
- `--print`: print the resolved config path without opening

### `yzx edit <target> [--print]`
Open one of the managed config surfaces through explicit or fuzzy target selection
- Supported targets include `config`, `packs`, `helix`, `zellij`, `yazi`, `yazi-keymap`, and `yazi-init`
- Yazi targets stay inside `user_configs/yazi/` and do not expose host-owned `~/.config/yazi/` files
- `--print`: print the resolved managed path without opening

### `yzx config migrate [--apply] [--yes]`
Preview or apply known Yazelix config migrations
- Default: preview known safe rewrites and manual-only follow-up without changing the file
- `--apply`: write only deterministic rewrites back to `yazelix.toml`, and create or rewrite `yazelix_packs.toml` when packs are migrated out of the main config
- `--yes`: skip the confirmation prompt for `--apply`
- Always uses a backup-first write path on apply
- Never guesses on ambiguous legacy config; those cases stay manual-only
- Rewrites from parsed TOML, so comments and key ordering may be normalized on apply

### `yzx config reset [--yes] [--no-backup]`
Replace `yazelix.toml` and `yazelix_packs.toml` with fresh copies of the shipped templates
- Backs up the current config surfaces to `*.backup-<timestamp>` first when they exist
- `--yes`: skip the confirmation prompt
- `--no-backup`: discard the previous config surfaces instead of renaming them to backups first
- Use this as a blunt recovery path when `yzx doctor` reports stale config fields

### `yzx help`
Show command reference

## Examples

```bash
# Launch Yazelix
yzx launch                    # New terminal in current directory
yzx enter                     # Start in current terminal
yzx launch --home             # New terminal in home directory
yzx enter --path ~/project    # Current terminal, specific directory
yzx launch --terminal wezterm # Force WezTerm for this launch
yzx launch --verbose          # Detailed launch diagnostics
yzx launch --reuse            # Reuse the last built profile without rebuilding
yzx launch -s                 # Launch while skipping explicit refresh trigger

# Environment-only mode (no UI)
yzx env                       # Drop into configured shell with Yazelix tools
yzx env --no-shell            # Load tools but stay in current shell
yzx env --reuse               # Reuse the last built profile without rebuilding
yzx env -s                    # Load env while skipping explicit refresh trigger
yzx refresh                   # Refresh devenv cache if changes were detected
yzx refresh --force           # Force refresh even when up to date
yzx refresh -v                # Refresh with high-level progress
yzx refresh -V                # Refresh with full build logs (-vv equivalent)
yzx run lazygit              # Run single command and exit
yzx run bash -lc "lazygit"   # Run through a shell
yzx run br init              # Outside-shell fallback for Beads Rust
yzx run bv --robot-triage    # Outside-shell fallback for Beads Viewer robot mode
yzx cwd                       # Set the current tab directory to $PWD
yzx cwd ~/project             # Set the current tab directory explicitly
yzx cwd yazelix               # Resolve a project via zoxide, then retarget the current tab
yzx keys                      # Show Yazelix-owned bindings and remaps
yzx keys yazi                 # How to view Yazi's own bindings
yzx keys hx                   # How to discover Helix bindings
yzx keys nu                   # Small curated Nushell keybinding subset
yzx tutor                     # Guided Yazelix overview
yzx tutor hx                  # Launch Helix's built-in tutor
yzx tutor nu                  # Launch Nushell's built-in tutor
yzx restart --reuse           # Reopen from the last built profile without rebuilding

# Diagnostics and info
yzx doctor --fix              # Health check with auto-fix
yzx config                    # Show active config without the packs section
yzx config --full             # Show the full config including packs
yzx config --path             # Print the active config path
yzx import zellij             # Import ~/.config/zellij/config.kdl into managed overrides
yzx import yazi               # Import native Yazi override files into managed overrides
yzx import helix              # Import ~/.config/helix/config.toml into managed overrides
yzx import zellij --force     # Backup and replace the managed Zellij override
yzx edit config               # Open the main managed config
yzx edit packs                # Open the pack sidecar
yzx edit keymap               # Open managed Yazi keymap.toml
yzx edit init                 # Open managed Yazi init.lua
yzx config migrate            # Preview known config migrations
yzx config migrate --apply --yes  # Apply safe migrations with backup
yzx config reset              # Replace both config surfaces with fresh templates after confirmation
yzx config reset --yes        # Replace both config surfaces with fresh templates and keep backups
yzx config reset --yes --no-backup  # Replace both config surfaces without writing backups
yzx status                    # System information
yzx status --versions         # Show all tool versions
yzx status --verbose          # Show detailed shell hook status
yzx sponsor                   # Open the Yazelix sponsor page

# Updates
yzx update                    # Show the supported update-owner paths
yzx update upstream           # Print and run nix run github:luccahuguet/yazelix#install
yzx update home_manager       # Run nix flake update yazelix here, then print home-manager switch
yzx update nix                # Upgrade Determinate Nix via determinate-nixd (sudo)
yzx dev update --yes --activate installer  # Refresh all inputs, run canaries, sync pins, refresh vendored zjstatus and Yazi plugins, then activate the installer-owned runtime
yzx dev update --yes --activate none  # Refresh the repo state only and skip local activation
yzx dev update --yes --activate home_manager --home-manager-attr 'you@host'  # Refresh the repo, update the Home Manager yazelix-hm input, then run home-manager switch
yzx dev update --canary-only --canaries [default]  # Run only the default canary
yzx dev build_pane_orchestrator --sync  # Build and sync the pane orchestrator wasm (enable rust_wasi first)

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
yzx dev test                  # Run the default non-sweep regression suite
yzx dev test --verbose        # Run the default suite with detailed output
yzx dev test --new-window     # Run tests in separate window (for debugging)
yzx dev test --lint-only      # Run only syntax validation
yzx dev test --sweep          # Run only the non-visual config/shell sweep
yzx dev test --visual         # Run only the visual terminal sweep
yzx dev test --all            # Run the default suite plus sweep + visual lanes

# Benchmarking
yzx dev bench                 # Benchmark all available terminals (3 iterations each)
yzx dev bench -n 5            # Run 5 iterations per terminal
yzx dev bench -t ghostty      # Benchmark only Ghostty
yzx dev bench -t wezterm -n 10 # Benchmark WezTerm with 10 iterations

# Profiling
# Note: Different launch scenarios have different performance characteristics
yzx dev profile               # Profile the current-terminal startup path
yzx dev profile --cold        # Profile cold start from a vanilla terminal
yzx dev profile --cold --clear-cache  # Force a rebuild-heavy cold profile run

# Performance scenarios explained:
# 1. Warm start (~130ms): Already in Yazelix, launching tools/commands
# 2. Cold cached (~300-500ms): Desktop entry or vanilla terminal launch, config unchanged
# 3. Config change (~3-8s): After clearing cache (full Nix re-evaluation)
```

Note: if `zellij.persistent_sessions = true` and the named session already exists, Zellij reattaches to that session and `yzx enter --path ...` is ignored. Yazelix warns about this and tells you to kill the session first if you want a fresh working directory.
