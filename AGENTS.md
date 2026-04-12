# Agent Guidelines

This file is the single source of truth for agent workflow, coding conventions, and implementation expectations in this repository.

## Beads Workflow

Use Beads (`bd`) as the agent memory and triage layer for Yazelix work.

- Use `AGENTS.md` as the single durable source of agent workflow rules and command-surface policy.
- Use `bd ready` to find unblocked work and `bd prime` for agent-optimized context instead of manually reconstructing project state from `.beads`.
- Use `bd` for all issue mutations: create, update, close, dependency management.
- bd uses embedded Dolt (`.beads/embeddeddolt/`) as its storage backend — there is no separate DB/JSONL sync cycle. `bd export` and `bd import` handle JSONL interchange when needed.
- Never fire `bd` write commands in parallel from multiple tools or subshells at once; bd uses file-level locking and a single-writer model.
- Treat scope boundaries strictly:
  - `bd ready` and `bd prime` decide what to work on.
  - `bd` updates issue state.
  - Coordination between multiple agents should use a separate coordination layer, not ad-hoc issue comments or long prompt memory.
- Keep agent guidance short. Do not copy large issue graphs, long triage dumps, or project history into `AGENTS.md`; store dynamic state in Beads and regenerate it when needed.

## File Naming Conventions

**IMPORTANT**: Yazelix uses underscores (`_`) for ALL file and directory names, never hyphens (`-`).

Examples:
- ✅ `home_manager/`
- ❌ `home-manager/`
- ✅ `yazelix_default.toml`
- ❌ `yazelix-default.nix`
- ✅ `start_yazelix.nu`
- ❌ `start-yazelix.nu`

This convention is used consistently throughout:
- Directory names: `configs/terminal_emulators/`, `nushell/scripts/core/`
- File names: `yazelix_default.toml`, `start_yazelix.nu`, `launch_yazelix.nu`
- Script names: All Nushell scripts use underscores

When creating new files or directories, always use underscores to maintain consistency with the existing codebase.

## Project Structure Notes

- Yazelix is a development environment (`devShell`) not a traditional package
- Configuration is handled via `yazelix.toml` (user) and `yazelix_default.toml` (template)
- All paths reference `~/.config/yazelix/` as the base directory
- Scripts are organized in `nushell/scripts/` with subdirectories using underscores

## Upstream Reference Clones

- For upstream dependency code inspection, prefer the local reference clones under `/home/lucca/pjs/open_source/yazelix_related/` before browsing the network.
- Current expected mirrors include `/home/lucca/pjs/open_source/yazelix_related/helix`, `/home/lucca/pjs/open_source/yazelix_related/yazi`, `/home/lucca/pjs/open_source/yazelix_related/zellij`, and `/home/lucca/pjs/open_source/yazelix_related/nushell`.
- Treat those clones as read-only reference checkouts unless the user explicitly asks to modify them.

## Configuration Management Principles

### Synchronization Requirements
1. **Always sync Home Manager module with default config** - When changing `yazelix_default.toml`, update `home_manager/module.nix` to maintain identical options and defaults
2. **Verify both configuration paths work** - Test changes through both direct config files and Home Manager integration

### Code Robustness Requirements
1. **Avoid fallbacks** - Fallback behavior can mask underlying issues and lead to unpredictable behavior across different environments
2. **Fail fast with clear errors** - When something is wrong, provide explicit error messages rather than degraded functionality
3. **Universal robustness** - Yazelix must work reliably for all users, not just maintainers who can manually fix issues
4. **Avoid redundant code** - Focus on elegant, concise code when possible; eliminate duplication and unnecessary complexity
5. **Do not ship or rely on local-only host fixes** - Do not patch user-specific caches, local machine state, or other one-off environment artifacts as a substitute for a real Yazelix fix unless the user explicitly asks for a local recovery workaround. Temporary local probes are allowed for diagnosis or to test a hypothesis, but they must be treated as throwaway investigation steps and either reverted or replaced by a real repo fix before calling the work done.
6. **Never automatically move, delete, or take ownership of user-managed config files outside Yazelix-owned paths** - In particular, do not automatically relocate files like `~/.config/zellij/config.kdl` into Yazelix-managed paths. If Yazelix wants to adopt an external user config, that must be an explicit user action such as an import command, not an implicit startup side effect.

### Error Handling Philosophy
- **No silent failures** - Every error should be visible and actionable
- **Environment independence** - Code should work regardless of host system quirks
- **Consistent behavior** - Same input should produce same output across all user environments

## Nushell Development Notes

### 🚨 MOST CRITICAL RULE: Escaping Parentheses in String Interpolation

**Nushell interprets unescaped parentheses `()` in string interpolation as command substitution!**

**The ONLY correct syntax is:** `\(` and `\)` (single backslash)
- ❌ **NEVER use:** `\\(` and `\\)` (double backslash) - this will fail!
- ❌ **NEVER use:** `()` (no backslash) - this executes commands!

**Examples:**
- ✅ Correct: `$"Checking pane \(editor\)"`
- ❌ Wrong: `$"Checking pane \\(editor\\)"` → tries to execute command `editor\\`
- ❌ Wrong: `$"Checking pane (editor)"` → tries to execute command `editor`
- ✅ Correct: `log_to_file $log "Sent Escape \(27\) to enter normal mode"`
- ❌ Wrong: `log_to_file $log "Sent Escape \\(27\\) to enter normal mode"` → fails

**If you get "Command X not found" errors in string interpolation, check for incorrect parentheses escaping first!**

## Python Notes

- Use `python3` explicitly in all commands, scripts, and documentation.
- Avoid `python` as it can point to Python 2 on some systems or be unset.
- Prefer fenced code blocks with `bash` and examples like:
  ```bash
  python3 -m venv .venv
  python3 script.py
  ```

## GitHub Workflow

- Prefer the GitHub CLI (`gh`) for inspecting issues, PRs, comments, and repo metadata instead of scraping GitHub pages manually.
- Use `gh` first when interacting with GitHub state from this repository unless the task specifically requires browser-only behavior.
- GitHub and Beads have a shared-subset contract:
  - GitHub owns the public issue, discussion thread, and open/closed lifecycle.
  - Beads owns planning metadata: dependencies, priority, labels, design notes, acceptance criteria, and execution history.
  - Every GitHub issue in `luccahuguet/yazelix` should have exactly one bead with the issue URL stored as `external_ref`.
  - Every in-contract GitHub issue should also have one canonical visible automated comment of the form `Automated: Tracked in Beads as \`yazelix-...\`.` so the mapping is obvious from the issue page.
  - Open GitHub issues should not map to closed beads, and closed GitHub issues should not map to open beads.
  - Title/body evolution in Beads is allowed after import; `external_ref` and lifecycle sync are the hard contract.
  - The automated validator enforces this contract for issues created on or after `2026-03-22`. Older backlog issues are intentionally grandfathered until they are explicitly imported or touched by the local sync flow.
- GitHub Actions must stay read-only with respect to Beads. Do not let CI mutate or commit `.beads/issues.jsonl`.
- Sync GitHub issue state into Beads locally during normal maintainer work with `yzx dev sync_issues`; that command is also responsible for creating or repairing the canonical Beads comment on GitHub issues. Then commit the Beads changes on your branch.
- Do not block on `yzx dev sync_issues` when it is slow or hanging. Prefer `bd` for Beads mutations whenever possible, continue the implementation work, and repair the GitHub/Beads contract afterward.
- Use `bd export` and `bd import` for JSONL interchange when needed. bd uses embedded Dolt as its storage backend — there is no separate DB/JSONL sync cycle like `br sync`.

## Spec Workflow

- Use `docs/spec_driven_workflow.md` as the durable entrypoint for when a change needs a spec and how specs relate to Beads.
- Store reusable spec templates and concrete specs under `docs/specs/`.
- Prefer specs for user-visible behavior, subsystem contracts, and integration boundaries. Do not create specs for trivial edits or purely mechanical refactors.
- Real specs should include a small `Traceability` section with one Bead id and at least one concrete `Defended by` check or test.

## Tool Invocation Workflow

- Prefer `yzx run ...` for project-scoped tool invocations instead of raw `nix develop -c ...` when running tools provided by the Yazelix environment.
- Use raw `nix develop -c ...` only when `yzx run ...` is not a clean fit for the task, such as larger multi-command shell scripts or environment debugging.
- For agent-driven Yazelix invocations, always suppress the welcome/UI path by default. Prefer entrypoints that already do this, such as `yzx run ...`, or pass the equivalent `--skip-welcome` flow when calling Yazelix bootstrap/runtime scripts through `nix develop -c ...`. Do not launch the interactive welcome screen or its animations unless the task is explicitly about validating that UX.
- Be careful with heavyweight Nix probes during investigation. Prefer cheap read-only commands such as `nix eval`, `nix flake show`, `nix path-info`, `rg`, or repo-local code inspection before running `nix build` on large external inputs. Do not casually launch expensive build jobs just to inspect metadata, and if a diagnostic build is truly needed, say so explicitly and clean it up if it is no longer needed.

## Command Surface Policy

- When renaming or simplifying a user-facing command surface, do not keep legacy aliases by default.
- Only preserve old command names as aliases when the user explicitly asks for a compatibility transition.

## Rust Plugin Workflow

- **Rust pane-orchestrator source edits are not live by themselves.** Changes under `rust_plugins/zellij_pane_orchestrator/` do not affect Yazelix behavior until the wasm is rebuilt and synced into the tracked/runtime plugin paths.
- **Rust popup-runner source edits are not live by themselves either.** Changes under `rust_plugins/zellij_popup_runner/` do not affect Yazelix behavior until that wasm is rebuilt and synced into the tracked/runtime plugin paths.
- After changing the pane orchestrator, rebuild and sync it before claiming behavior is fixed:
  ```bash
  yzx dev build_pane_orchestrator --sync
  ```
- After changing the popup runner, rebuild and sync it before claiming popup behavior is fixed:
  ```bash
  yzx dev build_popup_plugin --sync
  ```
- If the current shell toolchain cannot build `wasm32-wasip1`, use the flake maintainer shell:
  ```bash
  nix develop -c nu -c 'source nushell/scripts/yzx/dev.nu; yzx dev build_pane_orchestrator --sync'
  ```
- For popup-runner rebuilds in the flake maintainer shell:
  ```bash
  nix develop -c nu -c 'source nushell/scripts/yzx/dev.nu; yzx dev build_popup_plugin --sync'
  ```
- **Do not treat `cargo test` or `cargo check` as sufficient verification for live plugin behavior.** They only validate the Rust source. Real behavior changes require the synced wasm plus a fresh Yazelix session.
- After syncing a new plugin wasm, prefer `yzx restart` or a fresh Yazelix window. Avoid in-place plugin reloads as the default validation path because they can leave the current session in a broken permission state.

## Zellij Keybinding Rule

- In Yazelix Zellij config, do not `unbind` a key that Yazelix then intends to `bind` for its own action in the same merged config.
- Empirical rule for this repo: if you `unbind` a key and then try to reuse it for a Yazelix-owned action in the same merged config, that key becomes dead.
- If Yazelix owns the key, emit only the replacement `bind`. Use `unbind` only for keys Yazelix is truly removing without reusing.

## Planning and Decision Making

**ALWAYS PLAN FIRST** - Before taking significant actions (like git commits, major changes, or file operations), explicitly discuss the approach and get user approval. This includes:
- Git operations: What files to commit, whether to include binaries, commit message strategy
- File changes: Whether to edit, create, or delete files
- Tool selection: Which approach to use when multiple options exist
- Architecture decisions: How to structure or integrate new features

**PREFER PLANNING SPACE FIRST** - It is usually much easier, faster, and safer to improve the plan than to correct code after implementation starts. Spend real effort refining the problem framing, scope, dependencies, user impact, migration story, and verification strategy before making code changes.

**REASON FROM FIRST PRINCIPLES** - When faced with design decisions or trade-offs, analyze the fundamental requirements and constraints rather than following conventions blindly. Consider:
- What is the core problem being solved?
- What are the fundamental constraints (safety, user expectations, system behavior)?
- What are the actual risks vs. perceived risks?
- What does the user explicitly want vs. what they implicitly expect?
- How do similar tools handle this situation and why?

### Delete-First Protocol

Use this as the default refactor and audit method in Yazelix, especially before packaging, upstreaming, or broad architecture work:

1. Make the requirements less "dumb": Question every requirement, especially inherited ones. Ask whether the behavior is truly needed, whether the current scope is too broad, and whether two different responsibilities have been bundled together by accident.
2. Delete the part or process: Remove unnecessary steps, compatibility paths, duplicate ownership layers, stale surfaces, or helper indirection before introducing new abstractions. If nothing can be deleted, be explicit about why.
3. Simplify or optimize only what survives: After deletion, make the remaining path smaller, clearer, and more DRY. Never optimize a part that should have been removed.
4. Verify the simpler contract: Test the exact user-visible behavior or subsystem contract that remains after the deletion/simplification. Prefer focused behavior checks and regressions over broad noise.
5. Record the decision and the new seam: When the work changes future planning, capture the outcome in Beads/specs/notes so later refactors build on the clarified boundary instead of reopening the same ambiguity.

## Verification Requirements

- **Always test the exact functions or commands you change** before committing.
- If a command cannot be executed in this environment, explain why and provide the nearest safe alternative.
- **Prefer high-signal behavior and regression tests** over shallow command-discovery checks.
- **Do not add weak tests that mostly create noise**, such as tests that only verify a command exists in help output, that a subcommand name is listed, or that implementation trivia appears in generated text without protecting meaningful behavior.
- **Every new test should defend a real contract, regression, or failure mode**: user-visible behavior, config/state invariants, integration boundaries, or a bug that has already happened.
- **Every new `test_*.nu` file must declare `# Test lane: ...` using an allowed lane** (`default`, `maintainer`, `sweep`, or `manual`).
- **Treat test strength and lane placement as separate decisions.** Use the repo's per-test strength rubric to decide whether a test is worth keeping, and use lane-placement thinking only to decide where a surviving test belongs.
- **Do not create generic `_extended` test files as overflow.** If a nondefault lane needs more coverage, use a file or lane name that reflects its actual ownership.
- **Every new governed `def test_*` must carry a nearby `# Defends:`, `# Regression:`, or `# Invariant:` marker.**
- **Every new governed `def test_*` must also carry a nearby structured strength marker.** Use:
  - `# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10`
- **Lane minimums are enforced mechanically.** Current minimums are:
  - `default`: `7/10`
  - `maintainer`: `6/10`
  - `sweep`: `6/10`
  - `manual`: `6/10` if a governed `def test_*` exists there at all
- **Do not add packaging/config-sync tests by default** just because two files should match. Only keep them when they defend a maintained source-of-truth invariant in the right lane; otherwise prefer behavior tests, spec-backed validation, or cheaper dedicated validators.
- When in doubt, **remove or avoid low-value tests** and spend the budget on fewer, stronger assertions.

## Yazelix Versioning

**Yazelix versioning:**
- Follow the current project versioning scheme used in tags/releases
- When referencing versions in documentation or migration notes, only use actual version numbers that exist
- **Keep `YAZELIX_VERSION` in sync with git tags**: When creating a new git tag, update `nushell/scripts/utils/constants.nu` to match (e.g., `export const YAZELIX_VERSION = "v12.3"`). This version is displayed in the zjstatus bar.

## Beads Workflow Integration

This project uses [gastownhall/beads](https://github.com/gastownhall/beads) (`bd`) for issue tracking. Issues are stored in `.beads/` and tracked in git via Dolt's versioned storage.

### bd Commands for Issue Management

```bash
bd ready              # Show issues ready to work (no blockers)
bd ready --explain    # Show WHY issues are or aren't ready
bd list --status=open  # All open issues
bd show <id>          # Full issue details with dependencies
bd create "Title" -p 0 -t task    # Create a P0 task
bd create "Title" --parent <id>   # Create child with parent
bd update <id> --status=in_progress --claim   # Claim and start work
bd close <id> --reason="Completed"
bd close <id1> <id2>  # Close multiple issues at once
bd dep add <child> <parent> --type parent-child  # Add dependency
bd dep add <blocked> --blocked-by <blocker>     # Add blocks dep
bd prime              # Agent-optimized workflow context
bd graph <id>         # Dependency graph for an issue
bd stale              # Show stale issues
bd blocked            # Show blocked issues
bd export              # Export issues to JSONL
bd import <file>       # Import issues from JSONL
```

### Workflow Pattern

1. **Triage**: Run `bd ready` to find unblocked work, `bd prime` for agent context
2. **Claim**: Use `bd update <id> --status=in_progress --claim`
3. **Work**: Implement the task
4. **Complete**: Use `bd close <id> --reason="Completed"`
5. **Sync**: `git add .beads/ && git commit` — Dolt versioning handles the rest

### Key Concepts

- **Dependencies**: Issues can block other issues. `bd ready` shows only unblocked work. `bd dep add <blocked> --blocked-by <blocker>` adds blocking deps.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers 0-4 or P0-P4 strings)
- **Types**: task, bug, feature, epic, chore, decision
- **Dotted IDs**: bd natively supports hierarchical IDs (e.g., `yazelix-qgj7.4.3.5`)

### Session Protocol

```bash
git status              # Check what changed
git add .beads/         # Stage beads changes (Dolt auto-commits internally)
git add <files>         # Stage code changes
git commit -m "..."     # Commit everything
git push                # Push to remote
```

<!-- end-bd-workflow -->
