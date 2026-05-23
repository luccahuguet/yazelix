# yzx Command Line Interface

Shell-agnostic CLI for Yazelix management. Works across bash, fish, zsh, and nushell.

## Commands

### `yzx doctor [--verbose] [--fix] [--fix-plan]`
Health checks and diagnostics
- `--verbose`: Detailed output
- `--fix`: Auto-fix safe issues
- `--fix-plan`: Print exact recovery commands without mutating anything
- `--fix-plan --json`: Emit a machine-readable recovery plan for docs, support, and agents
- Warns when `settings.jsonc` has stale, removed, or invalid fields relative to the current settings contract
- Reports the active runtime/distribution tier before deeper checks
- Detects missing or stale workspace assets such as generated Zellij config, layouts, and plugin wasm artifacts
- Runs installer-owned runtime-link and stable-launcher diagnostics only when the current mode actually owns those surfaces

### `yzx inspect [--json]`
Inspect active Yazelix runtime truth
- `--json`: Emit the full machine-readable runtime, config, install, generated-state, tool-version, and session report
- Works outside Zellij and marks session state unavailable instead of failing
- Intended as the stable fact source for recovery diagnostics, support, live docs examples, and AI coding agents

### `yzx onboard [--force] [--dry-run]`
Generate a focused first-run Yazelix config
- Arrow keys move through single-choice prompts and `Enter` confirms
- `Space` toggles status-bar widget choices in the multi-select prompt
- `--force`: Overwrite the managed user `settings.jsonc` when it already exists
- `--dry-run`: Print the generated config instead of writing it
- Generates only the current supported main config surface; it does not recreate removed pack sidecars

### `yzx dev`
Runtime diagnostics
- Installed/runtime `yzx dev` intentionally exposes only runtime-safe diagnostic commands
- `yzx dev inspect_session [--json]`: Inspect the current Yazelix/Zellij tab session snapshot from the pane orchestrator
- `yzx dev profile [--cold] [--desktop] [--launch] [--clear-cache]`: Profile startup and launch paths from the active runtime
- Repo-only commands such as tests, release bumps, issue sync, plugin wasm sync, Nu lint, Rust checks, and repo updates belong to the Yazelix maintainer shell
- Running a repo-only `yzx dev` command from the installed runtime prints a maintainer-shell remediation instead of trying to execute repository tooling

### `yzx dev profile [--cold] [--desktop] [--launch] [--clear-cache]`
Profile launch sequence and identify performance bottlenecks
- Default: Profile the current-terminal startup path and write a structured startup report under `~/.local/share/yazelix/profiles/startup/`
- `--cold`: Profile cold startup from a vanilla terminal (outside Yazelix)
- `--desktop`: Profile the desktop-entry fast path, including pre-terminal work and the profiled handoff inside the spawned terminal
- `--launch`: Profile the managed new-window launch path, including wrapper preparation, terminal dispatch, and the profiled handoff inside the spawned terminal
- `--clear-cache`: Clear the runtime project cache plus recorded materialized/launch state first so the profiled run exercises the rebuild-heavy path
- `--terminal`: Override terminal selection for `--launch` profiling
- `yzx dev profile compare <baseline-report> <candidate-report>`: Compare two saved reports without rerunning startup, including total and per-step deltas
- `yzx dev profile save-baseline <name> <report>`: Copy a saved report into the local baseline directory
- `yzx dev profile compare-baseline <name> <candidate-report>`: Compare a named local baseline with another saved report
- The summary breaks out real startup phases such as preflight, config-state checks, maintainer-shell entry, Rust setup preflight, and inner startup work
- Profiling works from either a repo checkout or the active installed runtime
- Startup profile comparison is a local evidence tool, not a hosted CI timing gate

### `yzx launch [--path DIR] [--home] [--config FILE] [--with KEY=VALUE] [--terminal TERM] [--verbose]`
Launch Yazelix with directory and mode options
- Default: Launch new terminal in current directory
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--config FILE`: Use an alternate complete `settings.jsonc` for this window
- `--with KEY=VALUE`: Apply a repeatable session-only settings override, such as `--with editor.command=nvim`
- `--terminal TERM`: Override terminal selection (e.g., ghostty, wezterm, ratty, kitty)
- `--verbose`: Print detailed launch diagnostics

### `yzx enter [--path DIR] [--home] [--config FILE] [--with KEY=VALUE] [--verbose]`
Start Yazelix in the current terminal
- Default: Start in the current terminal and current directory
- `--path DIR`: Start in specific directory
- `--home`: Start in home directory
- `--config FILE`: Use an alternate complete `settings.jsonc` for this current-terminal session
- `--with KEY=VALUE`: Apply a repeatable session-only settings override, such as `--with core.welcome_style=static`
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

### `yzx agent`
Launch host-installed Codex
- Used by the `Alt+Shift+L` right-side agent pane binding
- Fails clearly when `codex` is missing from `PATH`
- Does not bundle Codex or fall back to another agent

### `yzx reveal PATH`
Reveal a file or directory in the managed Yazi file-tree sidebar
- Targets the managed sidebar in the current Zellij tab
- Focuses the sidebar after revealing the target when the sidebar is available
- Intended as the stable editor-integration surface for Helix/Neovim keybindings
- Errors clearly when run outside a Yazelix/Zellij session or when the managed sidebar is unavailable

### `yzx sidebar refresh`
Refresh the managed Yazi file-tree sidebar
- Targets the active tab's managed sidebar through pane-orchestrator session state
- Refreshes the Yazi file list and bundled git/starship sidebar status widgets when the sidebar is available
- Used by the `yzpp` popup close hook for the managed popup program

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
Show the guided Yazelix tutor
- Default: print the Yazelix-specific tutor overview with the workspace model and next-step commands
- `yzx tutor begin`: start the first Yazelix lesson
- `yzx tutor list`: list short Yazelix lessons
- `yzx tutor workspace`: practice workspace roots, managed panes, and Yazi handoff
- `yzx tutor discovery`: practice `yzx help`, `yzx keys`, `yzx menu`, and `yzx doctor`
- `yzx tutor tool_tutors`: point to upstream Helix and Nushell tutors
- Keeps a clear split with other help surfaces: `yzx help` is command reference, `yzx keys` is keybinding discoverability
- `yzx tutor hx`: launch Helix's built-in tutorial via `hx --tutor`
- `yzx tutor helix`: alias for `yzx tutor hx`
- `yzx tutor nu`: launch Nushell's built-in tutorial in a fresh `nu` process
- `yzx tutor nushell`: alias for `yzx tutor nu`

### `yzx restart [-s | --skip] [--config FILE] [--with KEY=VALUE]`
Restart the current Yazelix window
- Relaunches through the stable owner-provided `yzx` wrapper when one exists
- Profile installs relaunch through the default-profile `yzx`; Home Manager installs relaunch through the Home Manager-owned `yzx`
- Already-open Yazelix windows keep running their current live runtime until they are explicitly relaunched or restarted
- `--skip, -s`: skip the welcome screen for the restarted window only
- `--config FILE`: Use an alternate complete `settings.jsonc` for the restarted window
- `--with KEY=VALUE`: Apply a repeatable session-only settings override for the restarted window

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
- This still matters for `path:` inputs because `flake.lock` pins a snapshot of that local path until you refresh it
- Prints `home-manager switch` for the user to copy and run manually
- After `home-manager switch`, fresh launches and `yzx restart` use the profile-owned wrapper; already-open windows do not hot-swap invisibly

### `yzx home_manager prepare [--apply] [--yes]`
Preview or remove manual-install takeover blockers before Home Manager takeover
- Default: preview takeover blockers and manual-install artifacts without changing anything
- `--apply`: archive file-based takeover artifacts and remove standalone default-profile Yazelix entries so `home-manager switch` can take ownership cleanly
- `--yes`: skip the confirmation prompt when `--apply` is used
- It also archives a stale legacy `~/.local/bin/yzx` wrapper when that old manual path would shadow the profile-owned command after migration
- Use this when migrating an existing upstream/manual install to Home Manager

### `yzx update nix [--yes] [--verbose]`
Upgrade Determinate Nix
- `yzx update nix`: Upgrade Determinate Nix via `determinate-nixd` (`--yes` skips prompt, `--verbose` shows command; sudo required; only works if Determinate Nix is installed)

Runtime diagnostics:
- `yzx dev inspect_session [--json]`: Inspect the current Yazelix/Zellij tab session snapshot from the pane orchestrator; useful for debugging workspace root, focus context, layout state, managed panes, and sidebar Yazi identity
- `yzx_control zellij status-cache-heartbeat --json`: Read the last window-local pane-orchestrator heartbeat from `status_bar_cache.json` without piping into the plugin; useful during stalls because it shows stale heartbeat age, last timer tick, last handled pipe, and recent status-refresh timestamps

Maintainer shell commands:
- Inside `nix develop`, the maintainer-shell `yzx` wrapper routes repo-only `yzx dev` commands to `yzx_repo_maintainer`
- `yzx dev rust fmt [core|maintainer|pane_orchestrator|all] [--check]`: Run `cargo fmt` directly from the current maintainer environment. Default target is `all`
- `yzx dev rust check [core|maintainer|pane_orchestrator|all]`: Run fast `cargo check` directly. Default target is `core`
- `yzx dev rust test [core|maintainer|pane_orchestrator|all] [cargo test args...]`: Run fast `cargo test` directly. Default target is `core`; pass a focused test filter directly or after the target
- `yzx dev test [--verbose] [--new-window] [--lint-only] [--profile] [--sweep] [--all]`: Run the repo test lanes
- `yzx dev update`: Refresh repo runtime inputs, run canaries, sync pinned runtime expectations, refresh vendored runtime assets, and perform the required `--activate profile|home_manager|none` step; `--canary-only` is the only path that does not require `--activate`
- `yzx dev sync_issues [--dry-run]`: Sync the GitHub/Beads public issue contract from a repo checkout
- `yzx dev lint_nu [--format pretty|compact] [paths...]`: Run the repo Nu linter wrapper
- `yzx dev bump VERSION`: Rotate release notes, update `YAZELIX_VERSION`, sync the README version marker, create the release commit, and create the matching annotated tag
- `yzx_repo_validator validate-package-rust-test-purity`: Guard default/package-time Rust tests from host-only commands such as `nix` and `home-manager`; Nix-dependent checks belong in explicit validators or package gates
- `yzx_repo_validator validate-workspace-session-contract`: Check built-in layout metadata, workspace runtime assets, internal Zellij command routing, pane-orchestrator pipe commands, and Yazi workspace entrypoints
- `yzx_repo_validator validate-rust-ownership-budget`: Optional manual audit for the canonical Rust ownership manifest, unexpected `.rs` files, and historical LOC/file ceilings

### `yzx menu [--popup]`
Interactive command palette (fuzzy search)
- Default: inline mode in current terminal
- `--popup`: toggle the `zellij.popup_commands.menu` `yzpp` floating pane (errors if not in Zellij)
- `--pane`: run the popup-pane menu UI in the current pane
- Lists most `yzx` commands while hiding maintenance-heavy or low-signal entries (`yzx dev*`, `yzx env`, `yzx run`)
- Cancel with `Esc` before running a command
- In pane mode after running a command: `Backspace` returns to menu and `Enter` returns control to the pane
- Keybind: `Alt Shift M` opens the popup menu in Zellij
- Popup pane is named `yzx_menu` to avoid duplicate menu instances

### `yzx screen [STYLE]`
Preview the animated welcome screen directly in the current terminal
- `STYLE`: one of `logo`, `boids`, `boids_predator`, `boids_schools`, `mandelbrot`, `magician`, `game_of_life_gliders`, `game_of_life_oscillators`, `game_of_life_bloom`, or `random`
- `random`: rotates across the retained Game of Life, boids, and Mandelbrot animation families
- Runs until a key is pressed
- Requires an interactive terminal that supports timed keypress reads via bash
- Useful for previewing welcome styles without launching the full Yazelix UI

### `yzx popup [COMMAND ...]`
Open a transient floating-pane command inside Zellij
- Default: runs `zellij.popup_program` from `settings.jsonc`
- Special token: `editor` resolves to the current Yazelix `editor.command` with its managed runtime/env
- `COMMAND ...`: override the configured popup command for one invocation
- Uses the current tab workspace root as cwd when available; otherwise uses the current shell cwd
- Errors if not in Zellij
- Named generated popup specs use `zellij.popup_commands`: `bottom_popup` defaults to `["lazygit"]`, `top_popup` defaults to `["yzx", "config", "ui"]`, and `menu` defaults to `["yzx", "menu"]`
- Default surface keys: `Alt Shift J` for `bottom_popup` and `Alt Shift K` for `top_popup`
- The unplaced `popup` action remains configurable and unbound by default
- Popup panes are named `yzx_popup`, `yzx_bottom_popup`, `yzx_top_popup`, and `yzx_menu`

### `yzx config [--path]`
Show the active Yazelix configuration through the Rust-owned control path
- Default: print the active config
- `--path`: print the resolved config path

### `yzx config ui`
Open Yazelix's ratconfig-backed JSONC settings browser and editor
- Lists schema-known settings by user-intent tab
- Shows explicit, defaulted, unset, and invalid values
- Shows stale-field diagnostics and managed sidecar status under the Advanced tab
- Usually opens as the config UI popup through `Alt+Shift+C`; the top popup defaults to the same command through `Alt+Shift+K`
- Edits writable main settings and cursor settings while preserving unrelated comments and formatting

### `yzx config set <settings.path> <json-value>`
Set a supported config value without rewriting the whole file
- Preserves unrelated comments and formatting
- Accepts JSON literals such as `true`, `20`, `"bash"`, or `["ghostty"]`
- Writes normal settings to `~/.config/yazelix/settings.jsonc` and `cursors.*` paths to `~/.config/yazelix_ghostty_cursors/settings.jsonc`
- Validates the patched config or cursor registry before writing
- Refuses Home Manager-owned, read-only, non-`settings.jsonc`, or unsafe object/array edits

### `yzx config unset <settings.path>`
Remove an explicit config value so Yazelix uses the default
- Preserves unrelated comments and formatting
- Writes normal settings to `~/.config/yazelix/settings.jsonc` and `cursors.*` paths to `~/.config/yazelix_ghostty_cursors/settings.jsonc`
- Validates the patched config or cursor registry before writing
- Leaves the file unchanged when the value is already absent

### `yzx import zellij|yazi|helix [--force]`
Import native Zellij, Yazi, or Helix config into Yazelix-managed overrides
- `yzx import zellij`: copies `~/.config/zellij/config.kdl` into `zellij.kdl` only when the source has no `keybinds` blocks
- `yzx import yazi`: imports `yazi.toml`, `keymap.toml`, `init.lua`, `package.toml`, `plugins/`, and `flavors/` from `~/.config/yazi/` into `~/.config/yazelix/yazi/`
- `yzx import helix`: copies `~/.config/helix/config.toml` into `helix.toml`
- Fails clearly when no native source files are available for the selected target
- Fails clearly for `yzx import zellij` when the native file contains `keybinds` blocks; Yazelix-session keybindings belong in `settings.jsonc`
- Refuses to overwrite existing managed destination files by default
- `--force`: writes `*.backup-<timestamp>` backups before replacing managed destination files

### `yzx edit config [--print]`
Open the main Yazelix config file in your editor
- Uses `$EDITOR` (set by Yazelix from `editor.command` in `settings.jsonc`)
- Targets `settings.jsonc`
- `--print`: print the resolved config path without opening

### `yzx cursors`
Inspect Ghostty cursor presets and resolved colors
- Shows the active cursor `settings.jsonc` path
- Shows global trail, effect, glow, duration, and Kitty fallback settings
- Shows resolved colors for enabled presets, including derived mono accents

### `yzx edit <target> [--print]`
Open one of the managed config surfaces through explicit or fuzzy target selection
- Supported targets include `config`, `cursors`, `helix`, `zellij`, `yazi`, `yazi-keymap`, and `yazi-init`
- `cursors` opens `~/.config/yazelix_ghostty_cursors/settings.jsonc` for full cursor registry edits
- Yazi targets stay inside `~/.config/yazelix/yazi/` and do not expose host-owned `~/.config/yazi/` files
- `--print`: print the resolved managed path without opening

### `yzx reset config [--yes] [--no-backup]`
Replace `settings.jsonc` with a fresh copy of the shipped settings template
- Backs up the current config file to `*.backup-<timestamp>` first when it exists
- `--yes`: skip the confirmation prompt
- `--no-backup`: discard the previous config file instead of renaming it to a backup first
- Use this as a blunt recovery path when `yzx doctor` reports stale config fields
- Only replaces `~/.config/yazelix/settings.jsonc`
- Preserves managed override sidecars such as `helix.toml`, `zellij.kdl`, `yazi/`, `terminal_*.conf|toml|ini`, and `shell_*.sh|zsh|fish|nu`
- Preserves unknown adjacent files under `~/.config/yazelix/` and prints a warning instead of deleting or adopting them
- Cursor presets live in `~/.config/yazelix_ghostty_cursors/settings.jsonc`; `reset config` only resets the main Yazelix settings file

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
yzx launch --config ./minimal.jsonc # Use an alternate complete settings file
yzx launch --with editor.command=nvim # Override one setting for this window

# Environment-only mode (no UI)
yzx env                       # Drop into configured shell with Yazelix tools
yzx env --no-shell            # Load tools but stay in current shell
yzx run lazygit              # Run single command and exit
yzx run bash -lc "lazygit"   # Run through a shell
yzx run br ready             # Outside-shell fallback for Beads issue triage
yzx run br show <issue-id>   # Outside-shell fallback for agent-oriented Beads context
yzx keys                      # Show Yazelix-owned bindings and remaps
yzx keys yazi                 # How to view Yazi's own bindings
yzx keys hx                   # How to discover Helix bindings
yzx keys nu                   # Small curated Nushell keybinding subset
yzx tutor                     # Guided Yazelix overview
yzx tutor begin               # Start the first Yazelix lesson
yzx tutor list                # List Yazelix tutor lessons
yzx tutor hx                  # Launch Helix's built-in tutor
yzx tutor nu                  # Launch Nushell's built-in tutor
yzx restart                   # Reopen Yazelix in a fresh window
yzx restart -s                # Reopen Yazelix and skip the welcome screen once
yzx restart --config ./minimal.jsonc # Reopen with an alternate complete settings file
yzx restart --with core.welcome_style=static # Reopen with a one-shot config override

# Diagnostics and info
yzx doctor --fix              # Health check with auto-fix
yzx config                    # Show active config
yzx config --path             # Print the active config path
yzx config ui                 # Open the ratconfig-backed JSONC settings editor
yzx config set editor.hide_sidebar_on_file_open true # Set a config value with a JSON literal
yzx config unset editor.hide_sidebar_on_file_open # Remove an explicit config value
yzx cursors                   # Inspect Ghostty cursor presets and resolved colors
yzx edit cursors              # Open the Ghostty cursor settings file
yzx import zellij             # Import ~/.config/zellij/config.kdl when it has no keybinds blocks
yzx import yazi               # Import native Yazi override files and plugins into managed overrides
yzx import helix              # Import ~/.config/helix/config.toml into managed overrides
yzx import zellij --force     # Backup and replace zellij.kdl when the source has no keybinds blocks
yzx edit config               # Open the main managed config
yzx edit keymap               # Open managed Yazi keymap.toml
yzx edit init                 # Open managed Yazi init.lua
yzx reset config              # Replace the managed config with a fresh template after confirmation
yzx reset config --yes        # Replace the managed config with a fresh template and keep backups
yzx reset config --yes --no-backup  # Replace the managed config without writing backups
yzx status                    # System information
yzx status --versions         # Show all tool versions
yzx sponsor                   # Open the Yazelix sponsor page

# Updates
yzx update                    # Show the supported update-owner paths
yzx update upstream           # Print and run nix profile upgrade --refresh <matching-yazelix-profile-entry>
yzx update home_manager       # Run nix flake update yazelix here, then print home-manager switch
yzx home_manager prepare      # Preview manual-install takeover blockers before Home Manager switch
yzx home_manager prepare --apply --yes  # Archive file blockers, remove standalone profile yazelix entries, then hand off to home-manager switch
yzx update nix                # Upgrade Determinate Nix via determinate-nixd (sudo)
yzx screen game_of_life_gliders  # Preview the glider-swarm Game of Life welcome animation in the terminal

# Maintainer shell updates
# These commands require `nix develop` from a Yazelix repo checkout
yzx dev update --yes --activate profile  # Refresh all inputs, run canaries, sync pins, refresh vendored zjstatus and Yazi plugins, then activate the local repo package in the default profile
yzx dev update --yes --activate none  # Refresh the repo state only and skip local activation
yzx dev update --yes --activate home_manager --home-manager-attr 'you@host'  # Refresh the repo, update the Home Manager yazelix-hm input, then run home-manager switch
yzx dev update --canary-only --canaries [default]  # Run only the default canary
yzx dev update --canary-only --canaries [shell_layout]  # Run the alternate shell/layout canary
yzx_repo_maintainer canary-session --dry-run  # Preview the disposable packaged-runtime Zellij canary
yzx_repo_maintainer canary-session --session-name yazelix-canary-manual  # Open an isolated canary session and keep evidence under /tmp
yzx_repo_validator validate-child-release-transaction  # Verify locked first-party child commits are published before landing a main lock update
nix build ../yazelix-zellij-pane-orchestrator#yazelix_zellij_pane_orchestrator --no-link  # Build the local pane-orchestrator package

# Maintainer verification
# These commands require `nix develop` from a Yazelix repo checkout
yzx dev test                  # Run the default non-sweep regression suite
yzx dev test --verbose        # Run the default suite with detailed output
yzx dev test --new-window     # Run tests in separate window (for debugging)
yzx dev test --lint-only      # Run only syntax validation
yzx dev test --sweep          # Run only the config/shell sweep
yzx dev test --all            # Run the default suite plus the config/shell sweep

# Profiling
# Note: Different launch scenarios have different performance characteristics
yzx dev profile               # Profile the current-terminal startup path
yzx dev profile --cold        # Profile cold start from a vanilla terminal
yzx dev profile --cold --clear-cache  # Force a rebuild-heavy cold profile run
yzx dev profile --desktop     # Profile the desktop-entry launch path
yzx dev profile --launch --terminal ghostty  # Profile managed new-window launch with a terminal override
yzx dev profile compare ~/.local/share/yazelix/profiles/startup/old.jsonl ~/.local/share/yazelix/profiles/startup/new.jsonl
yzx dev profile save-baseline warm-v16 ~/.local/share/yazelix/profiles/startup/startup_profile_20260428_120000_000.jsonl
yzx dev profile compare-baseline warm-v16 ~/.local/share/yazelix/profiles/startup/startup_profile_20260428_121500_000.jsonl

# Performance scenarios explained:
# 1. Warm start (~130ms): Already in Yazelix, launching tools/commands
# 2. Cold cached (~300-500ms): Desktop entry or vanilla terminal launch, config unchanged
# 3. Config change (~3-8s): After clearing cache (full Nix re-evaluation)
```
