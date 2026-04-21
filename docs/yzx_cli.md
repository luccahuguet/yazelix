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

### `yzx dev profile [--cold] [--desktop] [--launch] [--clear-cache]`
Profile launch sequence and identify performance bottlenecks
- Default: Profile the current-terminal startup path and write a structured startup report under `~/.local/share/yazelix/profiles/startup/`
- `--cold`: Profile cold startup from a vanilla terminal (outside Yazelix)
- `--desktop`: Profile the desktop-entry fast path, including pre-terminal work and the profiled handoff inside the spawned terminal
- `--launch`: Profile the managed new-window launch path, including wrapper preparation, terminal dispatch, and the profiled handoff inside the spawned terminal
- `--clear-cache`: Clear the runtime project cache plus recorded materialized/launch state first so the profiled run exercises the rebuild-heavy path
- `--terminal`: Override terminal selection for `--launch` profiling
- The summary breaks out real startup phases such as preflight, config-state checks, maintainer-shell entry, shellHook setup, and inner startup work
- Profiling works from either a repo checkout or the active installed runtime

### `yzx dev bump VERSION`
Automate the version bump, release commit, and matching git tag
- Requires a clean git worktree
- Fails if `VERSION` is not a real Yazelix tag like `v15` or `v15.1`
- Refuses to reuse an existing git tag
- Rotates the current `Unreleased` release notes into the requested version, resets a fresh `Unreleased` placeholder, updates `YAZELIX_VERSION`, syncs the README title/version marker, creates a dedicated commit, and creates the matching annotated tag
- Refuses to run if `CHANGELOG.md` or `docs/upgrade_notes.toml` still contain the untouched default `Unreleased` placeholder text

### `yzx launch [--path DIR] [--home] [--terminal TERM] [--verbose]`
Launch Yazelix with directory and mode options
- Default: Launch new terminal in current directory
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--terminal TERM`: Override terminal selection (e.g., ghostty, wezterm, kitty)
- `--verbose`: Print detailed launch diagnostics

### `yzx enter [--path DIR] [--home] [--verbose]`
Start Yazelix in the current terminal
- Default: Start in the current terminal and current directory
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--verbose`: Print detailed startup diagnostics

### `yzx env [--no-shell]`
Load Yazelix environment without UI
- Default: Drop into your configured shell with the curated Yazelix tool surface available
- `--no-shell`: Stay in current shell (doesn't switch shells)
- Runtime-private helpers stay under the runtime root instead of being exported into your interactive PATH
- This remains the supported non-UI shell-entry surface for editor and terminal integration

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

### `yzx restart`
Restart Yazelix (handles persistent sessions)
- Relaunches through the stable owner-provided `yzx` wrapper when one exists
- Profile installs relaunch through the default-profile `yzx`; Home Manager installs relaunch through the Home Manager-owned `yzx`
- Already-open Yazelix windows keep running their current live runtime until they are explicitly relaunched or restarted

### `yzx status [--versions]`
Show current Yazelix status
- Default: show the structured runtime/config summary table
- `--versions, -V`: include the full tool version matrix

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
Upgrade the active Yazelix package in the default Nix profile
- Prints the exact command it will run
- Runs `nix profile upgrade --refresh <matching-yazelix-profile-entry>`
- Intended for installs owned by the default Nix profile
- Fresh launches use the updated installed runtime; already-open windows continue on their current live runtime until relaunch or `yzx restart`

### `yzx update home_manager`
Refresh the current Home Manager flake input, then print the manual switch step
- Must be run from the Home Manager flake directory that owns the install
- Assumes the Yazelix flake input in that directory is named `yazelix`
- Prints the exact command it will run
- Runs `nix flake update yazelix`
- If your flake uses a different input name, run `nix flake update <your-input-name>` yourself
- Prints `home-manager switch` for the user to copy and run manually
- After `home-manager switch`, fresh launches and `yzx restart` use the profile-owned wrapper; already-open windows do not hot-swap invisibly

### `yzx home_manager prepare [--apply] [--yes]`
Preview or archive manual-install artifacts before Home Manager takeover
- Default: preview takeover blockers and manual-install artifacts without changing anything
- `--apply`: archive the takeover artifacts so `home-manager switch` can take ownership cleanly
- `--yes`: skip the confirmation prompt when `--apply` is used
- It also archives a stale legacy `~/.local/bin/yzx` wrapper when that old manual path would shadow the profile-owned command after migration
- Use this when migrating an existing upstream/manual install to Home Manager

### `yzx update nix [--yes] [--verbose]`
Upgrade Determinate Nix
- `yzx update nix`: Upgrade Determinate Nix via `determinate-nixd` (`--yes` skips prompt, `--verbose` shows command; sudo required; only works if Determinate Nix is installed)

Maintainer-only updates:
- `yzx dev update`: Refresh the repo runtime inputs by updating `flake.lock` `nixpkgs`, run canary generated-state/build checks (`default`, `shell_layout`), then sync pinned runtime expectations, refresh the vendored `configs/zellij/plugins/zjstatus.wasm`, refresh vendored Yazi plugin runtime files from the pinned source map in `config_metadata/vendored_yazi_plugins.toml`, and perform one explicit activation step selected by the required `--activate profile|home_manager|none` flag (`profile` replaces older default-profile Yazelix entries with the current repo package for local dogfooding, `home_manager` refreshes the Home Manager flake input before `home-manager switch`, and `none` leaves local activation untouched). `--canary-only` is the only path that does not require `--activate`.
- `yzx dev build_pane_orchestrator [--sync]`: Build the Zellij pane orchestrator wasm for `wasm32-wasip1`; `--sync` also updates the tracked/runtime plugin paths after a successful build, preserves previously granted plugin permissions onto the stable runtime path when possible, and regenerates Zellij config. After syncing, prefer restarting Yazelix over reloading the plugin in place. If the toolchain is missing, install a WASI-capable Rust toolchain first.

### `yzx menu [--popup]`
Interactive command palette (fuzzy search)
- Default: inline mode in current terminal
- `--popup`: open in a Zellij floating pane (errors if not in Zellij)
- Lists most `yzx` commands while hiding maintenance-heavy or low-signal entries (`yzx dev*`, `yzx env`, `yzx run`)
- Cancel with `Esc` before running a command
- In popup mode after running a command: `Backspace` returns to menu and `Enter` closes the popup
- Keybind: `Alt Shift M` opens the popup menu in Zellij
- Popup pane is named `yzx_menu` to avoid duplicate menu instances

### `yzx screen [STYLE]`
Preview the animated welcome screen directly in the current terminal
- `STYLE`: one of `logo`, `boids`, `game_of_life`, or `random`
- Runs until a key is pressed
- Requires an interactive terminal that supports timed keypress reads via bash
- Useful for previewing welcome styles without launching the full Yazelix UI

### `yzx popup [COMMAND ...]`
Open a transient floating-pane command inside Zellij
- Default: runs `zellij.popup_program` from `yazelix.toml`
- Special token: `editor` resolves to the current Yazelix `editor.command` with its managed runtime/env
- `COMMAND ...`: override the configured popup command for one invocation
- Uses the current tab workspace root as cwd when available; otherwise uses the current shell cwd
- Errors if not in Zellij
- Default keybind: `Alt t`
- Popup pane is named `yzx_popup`

### `yzx config [--path]`
Show the active Yazelix configuration through the Rust-owned control path
- Default: print the active config
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

### `yzx edit <target> [--print]`
Open one of the managed config surfaces through explicit or fuzzy target selection
- Supported targets include `config`, `helix`, `zellij`, `yazi`, `yazi-keymap`, and `yazi-init`
- Yazi targets stay inside `user_configs/yazi/` and do not expose host-owned `~/.config/yazi/` files
- `--print`: print the resolved managed path without opening

### `yzx config reset [--yes] [--no-backup]`
Replace `yazelix.toml` with a fresh copy of the shipped template
- Backs up the current config file to `*.backup-<timestamp>` first when it exists
- `--yes`: skip the confirmation prompt
- `--no-backup`: discard the previous config file instead of renaming it to a backup first
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

# Environment-only mode (no UI)
yzx env                       # Drop into configured shell with Yazelix tools
yzx env --no-shell            # Load tools but stay in current shell
yzx run lazygit              # Run single command and exit
yzx run bash -lc "lazygit"   # Run through a shell
yzx run bd ready             # Outside-shell fallback for Beads issue triage
yzx run bd prime             # Outside-shell fallback for agent-oriented Beads context
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
yzx restart                   # Reopen Yazelix in a fresh window

# Diagnostics and info
yzx doctor --fix              # Health check with auto-fix
yzx config                    # Show active config
yzx config --path             # Print the active config path
yzx import zellij             # Import ~/.config/zellij/config.kdl into managed overrides
yzx import yazi               # Import native Yazi override files into managed overrides
yzx import helix              # Import ~/.config/helix/config.toml into managed overrides
yzx import zellij --force     # Backup and replace the managed Zellij override
yzx edit config               # Open the main managed config
yzx edit keymap               # Open managed Yazi keymap.toml
yzx edit init                 # Open managed Yazi init.lua
yzx config reset              # Replace the managed config with a fresh template after confirmation
yzx config reset --yes        # Replace the managed config with a fresh template and keep backups
yzx config reset --yes --no-backup  # Replace the managed config without writing backups
yzx status                    # System information
yzx status --versions         # Show all tool versions
yzx sponsor                   # Open the Yazelix sponsor page

# Updates
yzx update                    # Show the supported update-owner paths
yzx update upstream           # Print and run nix profile upgrade --refresh <matching-yazelix-profile-entry>
yzx update home_manager       # Run nix flake update yazelix here, then print home-manager switch
yzx home_manager prepare      # Preview manual-install takeover blockers before Home Manager switch
yzx home_manager prepare --apply --yes  # Archive takeover artifacts, then hand off to home-manager switch
yzx update nix                # Upgrade Determinate Nix via determinate-nixd (sudo)
yzx dev update --yes --activate profile  # Refresh all inputs, run canaries, sync pins, refresh vendored zjstatus and Yazi plugins, then activate the local repo package in the default profile
yzx dev update --yes --activate none  # Refresh the repo state only and skip local activation
yzx dev update --yes --activate home_manager --home-manager-attr 'you@host'  # Refresh the repo, update the Home Manager yazelix-hm input, then run home-manager switch
yzx dev update --canary-only --canaries [default]  # Run only the default canary
yzx dev update --canary-only --canaries [shell_layout]  # Run the alternate shell/layout canary
yzx dev build_pane_orchestrator --sync  # Build and sync the pane orchestrator wasm
yzx screen game_of_life       # Preview the live game_of_life welcome animation in the terminal

# Development verification
yzx dev test                  # Run the default non-sweep regression suite
yzx dev test --verbose        # Run the default suite with detailed output
yzx dev test --new-window     # Run tests in separate window (for debugging)
yzx dev test --lint-only      # Run only syntax validation
yzx dev test --sweep          # Run only the non-visual config/shell sweep
yzx dev test --visual         # Run only the visual terminal sweep
yzx dev test --all            # Run the default suite plus sweep + visual lanes

# Profiling
# Note: Different launch scenarios have different performance characteristics
yzx dev profile               # Profile the current-terminal startup path
yzx dev profile --cold        # Profile cold start from a vanilla terminal
yzx dev profile --cold --clear-cache  # Force a rebuild-heavy cold profile run
yzx dev profile --desktop     # Profile the desktop-entry launch path
yzx dev profile --launch --terminal ghostty  # Profile managed new-window launch with a terminal override

# Performance scenarios explained:
# 1. Warm start (~130ms): Already in Yazelix, launching tools/commands
# 2. Cold cached (~300-500ms): Desktop entry or vanilla terminal launch, config unchanged
# 3. Config change (~3-8s): After clearing cache (full Nix re-evaluation)
```

Note: if `zellij.persistent_sessions = true` and the named session already exists, Zellij reattaches to that session and `yzx enter --path ...` is ignored. Yazelix warns about this and tells you to kill the session first if you want a fresh working directory.
