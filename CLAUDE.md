# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this project.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **br (beads_rust)** for issue tracking. Run `br ready` and `br show <id>` for issue context.

### Quick Reference

```bash
br ready              # Find available work
br show <id>          # View issue details
br update <id> --claim  # Claim work
br close <id>         # Complete work
```

### Rules

- Use `br` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Keep `.beads/issues.jsonl` tracked as the durable issue state and `.beads/beads.db` ignored as the local cache
- Do not use the retired tracker workflow

## Session Completion

**When ending a work session**, complete the steps below that apply to the current change. For non-trivial changes, local implementation and validation can be complete before push, but remote push must wait until the user manually tests and approves it. Only trivial changes should follow the immediate push path by default.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - Required only after the user has manually tested non-trivial changes, or immediately for trivial changes / when the user explicitly asks to push:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Do not push non-trivial changes before user manual testing and explicit approval
- Commit finished local work before moving to unrelated work
- If an approved push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->


## Build & Test

### Rebuild the installed runtime — one way only

`lifeos_foundation_yzx` is installed via the default Nix profile against `path:/home/flexnetos/FlexNetOS/src/yazelix`. Rebuild through the active profile frontdoor:

```bash
/home/flexnetos/.nix-profile/bin/yzx update upstream
/home/flexnetos/.nix-profile/bin/yzx doctor --fix
/home/flexnetos/.nix-profile/bin/yzx doctor
```

Because this profile is backed by a local `path:` flake, `yzx update upstream` must fetch and fast-forward the clean tracked checkout before `nix profile upgrade`. It should fail instead of rebuilding from a dirty, detached, ahead-only, or diverged local checkout.

Do not rebuild this package with host-local `FLEXNETOS_*_PATH` inputs or `packaging/*_local_binary.nix` shims. Runtime tools in the foundation package must come from published flake inputs or source-owned package definitions so the profile can be rebuilt in a clean no-override path.

Do not copy raw `nix profile upgrade` commands with hardcoded store hashes. Those hashes go stale on every profile upgrade and re-introduce the exact "three-profiles-that-should-be-one" drift class this package is meant to prevent.

Verify the rebuild:

```bash
grep codex_usage_display ~/.nix-profile/settings_default.jsonc  # should mirror src/yazelix/settings_default.jsonc
readlink -f ~/.nix-profile/bin/yzx                              # store hash should differ from prior
/home/flexnetos/.nix-profile/bin/yzx run codedb --version
```

### Local toolchain notes are not package ownership

The host has useful build accelerators and developer tools, but they are not foundation package ownership:

| Tool | Path | Notes |
|---|---|---|
| `cargo` + `rustc` (fenix rust-mixed 1.96) | `/nix/store/b47aazvj6hmsd1i1a6sy9ch5yx8ylvxg-rust-mixed/bin/{cargo,rustc}` | Newer store hashes may exist; `find /nix/store -maxdepth 5 -name 'cargo' -type f \| xargs -I{} sh -c '{} --version'` to check |
| `cargo` nightly (fenix, if needed) | `/nix/store/l831cb33qjq42psp88zdga9zvgn785ix-auditable-cargo-nightly-latest-2026-05-31/bin/cargo` | Cargo-only bundle, no matching rustc — use the rust-mixed above for a full toolchain |
| `kache-rustc-wrapper` | `/home/flexnetos/FlexNetOS/usr/bin/kache-rustc-wrapper` | Replaces sccache. Set as `RUSTC_WRAPPER=` for local development builds only. |
| `wild` linker | `/home/flexnetos/.local/bin/wild` | **CAVEAT**: gcc rejects absolute paths for `-fuse-ld=<path>`. Do NOT set `RUSTFLAGS="-C link-arg=-fuse-ld=/absolute/path/to/wild"` — it errors with `unrecognized command-line option`. Either use `-fuse-ld=wild` (short name, needs `ld.wild` in the gcc search prefix) or drop wild; kache is the primary speed win. |
| `bun` / `bunx` | `/home/flexnetos/.local/bin/{bun,bunx}` | For node commands invoked during build (e.g. codex tooling). |

If a runtime tool starts as a local experiment, publish or vendor it through the owning child repo first, then consume that published input from Yazelix. Do not feed ad-hoc host binaries into `lifeos_foundation_yzx`.

## Architecture Overview

### Canonical source vs. sibling trees

Three yazelix-named directories exist under `/home/flexnetos/FlexNetOS/src/`. Only one is canonical for this package:

- **`src/yazelix`** — canonical. `nix profile list` shows `lifeos_foundation_yzx` locked to `path:/home/flexnetos/FlexNetOS/src/yazelix`. Edits here are the ones that ship after a rebuild.
- **`src/yazelix_new_worktree`** — a stale git worktree of the same repo on branch `worktree/new_worktree`. Not consumed by any build. Safe to ignore.
- **`src/yazelix-helix`** — a separate repo (helix editor fork) consumed via flake input `yazelixHelix`. Not competing with the two above; touch only when explicitly working on the helix bridge.

### Three-profile convergence

The FlexNetOS agent workspace has three artifacts that must reference the same runtime identity:

1. **Custom layout** — `configs/zellij/layouts/flexnetos_agent_workspace.kdl` (a template consumed by `runtime_materialization::resolve_zellij_layout_path`; the runtime detects `__YAZELIX_ZJSTATUS_TAB_TEMPLATE__` and renders it into `~/.local/share/yazelix/configs/zellij/layouts/`). Also shipped into the nix-store profile at `~/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl` — identical sha256.
2. **Launch app** — `~/.local/share/applications/com.flexnetos.Yazelix.Agent.desktop` (hand-installed, NOT home-manager managed, safe to edit directly; ownership marker `X-FlexNetOS-Managed=true` keeps `install_ownership_report.rs` from repairing it).
3. **Runtime binary/profile** — `~/.nix-profile/bin/yzx` → `/nix/store/…-lifeos-foundation-yzx` (variant `mars`, profile `mars-full`).

## Conventions & Patterns

### Desktop `Exec` lines reference the stable profile, not source

Never point `YAZELIX_LAYOUT_OVERRIDE` (or any other launcher-embedded path) at an absolute path under `/home/flexnetos/FlexNetOS/src/`. Use `~/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl` (or another `$HOME/.nix-profile/...` path) instead. The stable-profile symlink follows `nix profile upgrade` automatically; a source-tree absolute path becomes wrong the moment the repo moves or the layout template regenerates.

### The `com.yazelix.Yazelix.Mars.desktop` entry is runtime-owned

It has `NoDisplay=true` and `X-Yazelix-Managed=true`, and `rust_core/yazelix_core/src/install_ownership_report.rs` will repair it if drifted. Do not delete it or edit its `Exec` line — install a sibling FlexNetOS-specific entry (like `com.flexnetos.Yazelix.Agent.desktop`) with `X-FlexNetOS-Managed=true` instead.

### `yzx doctor` warnings after a rebuild — session carryover, not persistent drift

When a shell is spawned by the Yazelix desktop entry, its PATH and several env vars (`EDITOR`, `VISUAL`, `SHELL`, `LG_CONFIG_FILE`, …) are baked with the store hash that was current at launch time. After `nix profile upgrade` swaps the profile to a new store hash MID-SESSION, `yzx doctor` in that same session reports:

- "A stale host-shell yzx function or alias is shadowing the current profile command" — because the old `/nix/store/<old-hash>/bin` still precedes `~/.nix-profile/bin` in that session's PATH. `yzx doctor` sees `type -a yzx` return the old hash first and interprets it as a startup-file shadow.
- "Host <terminal> environment may be contaminated by Mars Terminal launch state" — because `MARS_CONFIG_HOME` (or similar) is a per-launch temp dir path baked into env.

Neither is a startup-file edit. Fix by re-launching the Yazelix desktop entry after a rebuild; the new launch inherits the current profile symlink. To confirm before relaunching, run `type yzx` in a fresh `env -i HOME="$HOME" PATH="$PATH" bash -lc` — it should resolve to `~/.nix-profile/bin/yzx`.

If the warning persists across a fresh login, THEN look at `~/.local/share/yazelix/initializers/{bash,nushell}/*` for hardcoded store paths and search shell rc files for actual `alias yzx=` / `function yzx` definitions.

### Profile frontdoor handoff

Runtime configuration required to rebuild is owned by the profile package, not host env-var shims:

- `/home/flexnetos/.nix-profile/bin/yzx` is the active frontdoor.
- `/home/flexnetos/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl` is the profile layout override path.
- `/home/flexnetos/.config/yazelix/` is the editable user input surface.
- `/home/flexnetos/.local/share/yazelix/` is generated runtime output and must be used only as proof.

Avoid `~/.local/bin/yzx` and user-local stale launchers as parallel ownership paths.

### `~/.config/yazelix/zellij.kdl` sidecar — merge trigger caveat

The sidecar is merged into `~/.local/share/yazelix/configs/zellij/config.kdl` at materialization time, but yazelix's freshness hash does NOT include the sidecar file. Editing the sidecar alone will not cause `yzx doctor` to detect drift or re-materialize. To pick up sidecar changes without waiting for the next desktop launch:

```bash
rm -f ~/.local/share/yazelix/state/rebuild_hash
yzx doctor --fix   # detects "needs repair", regenerates config.kdl
```

The sidecar is intended for native zellij keys that yazelix does NOT already render (from `settings.jsonc`) or enforce (from `enforced_top_level_settings` in `rust_core/yazelix_zellij_config_pack/src/lib.rs`). For example: `scrollback_lines_to_serialize` is a good sidecar key; `session_serialization` and `serialize_pane_viewport` are already enforced and don't need duplication.

### Adding a new runtime binary to the foundation runtime

Follow the source-owned pattern used for `git-kb`, `rtk`, Claude Code, Codex, and CodeDB:

1. Prefer a published flake input, fixed-output release package, or first-party child repo package.
2. If the tool is owned by a child repo, publish and merge the child repo first.
3. Update Yazelix's flake input/lock to the published child revision.
4. Edit `packaging/flake_outputs.nix` to add the package to `extraRuntimePackages`, `extraRuntimeCommands`, and `exportedBinCommands` only after it builds without host-local paths.
5. Validate with `validate-child-release-transaction` and a no-override `nix build .#lifeos_foundation_yzx`.

Do not create `packaging/<name>_local_binary.nix` for the foundation runtime. That pattern makes the Nix profile depend on whatever happened to be present on one workstation and breaks clean rebuilds.

## GitKB

This project uses GitKB for knowledge management.

@.kb/AGENTS.md
