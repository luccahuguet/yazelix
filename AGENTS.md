# Agent Guidelines

This file is the single source of truth for agent workflow, coding conventions, and implementation expectations in this repository.

## Beads Workflow

Use Beads Rust (`br`) as the agent memory and triage layer for Yazelix work.

- Use `AGENTS.md` as the single durable source of agent workflow rules and command-surface policy.
- Use `br ready` to find unblocked work and `br show <id>` for detailed issue context instead of manually reconstructing project state from `.beads`.
- Use `br` for all issue mutations: create, update, close, dependency management.
- Use `docs/child_repo_beads_ownership_policy.md` as the durable policy for whether main Yazelix Beads, child-repo Beads, or both should track cross-repo work.
- `br` uses `.beads/issues.jsonl` as the tracked durable interchange file and a local ignored SQLite database at `.beads/beads.db`.
- Run `br sync --import-only --rebuild` after a fresh checkout or suspicious local database state, and `br sync --flush-only` before committing if you need an explicit JSONL refresh.
- Never fire `br` write commands in parallel from multiple tools or subshells at once; serialize issue mutations and sync operations.
- Treat scope boundaries strictly:
  - `br ready`, `br list`, and `br show` decide what to work on.
  - `br` updates issue state.
  - Coordination between multiple agents should use a separate coordination layer, not ad-hoc issue comments or long prompt memory.
- Keep agent guidance short. Do not copy large issue graphs, long triage dumps, or project history into `AGENTS.md`; store dynamic state in Beads and regenerate it when needed.
- For reusable Codex workflows, prefer the official Agent Skills model and OpenAI skills catalog (`https://github.com/openai/skills`) over copying large third-party guideline packs into this file.

## File Naming Conventions

**IMPORTANT**: Yazelix uses underscores (`_`) for ALL file and directory names, never hyphens (`-`).

Examples:
- ✅ `home_manager/`
- ❌ `home-manager/`
- ✅ `settings_default.jsonc`
- ❌ `yazelix-default.nix`
- ✅ `shell_nu.nu`
- ❌ `start-yazelix.nu`

This convention is used consistently throughout:
- Directory names: `configs/terminal_emulators/`, `nushell/scripts/core/`
- File names: `settings_default.jsonc`, `release_metadata.toml`, `yazelix_runtime_package.nix`
- Script names: All Nushell scripts use underscores, such as `stack_prompt_guard.nu`

When creating new files or directories, always use underscores to maintain consistency with the existing codebase.

## Project Structure Notes

- Yazelix has packaged runtime surfaces and maintainer development-shell surfaces; keep user runtime behavior distinct from dev tooling
- The canonical user semantic config is `~/.config/yazelix/settings.jsonc`
- Shipped config defaults/templates feed `settings.jsonc` generation through `settings_default.jsonc`, `yazelix_cursors_default.toml`, `config_metadata/yazelix_settings.schema.json`, and `config_metadata/main_config_contract.toml`
- Old mutable `yazelix.toml` and `cursors.toml` files are unsupported legacy inputs, not runtime config sources or automatic migration inputs
- All Yazelix-owned user config paths reference `~/.config/yazelix/` as the base directory unless an explicit XDG/config override is in effect
- Scripts are organized in `nushell/scripts/` with subdirectories using underscores

## README Style

- Treat `README.md` as a distinct prose surface with its own style expectations
- Do not end README prose lines or bullet items with periods unless the punctuation is semantically required, such as code, URLs, abbreviations, or quoted literal text
- When updating generated or manual README surface text, rewrite the sentence instead of leaving trailing periods behind

## Upstream Reference Clones

- For upstream dependency code inspection, prefer the local reference clones under `/home/lucca/pjs/open_source/yazelix_related/` before browsing the network.
- Current expected mirrors include `/home/lucca/pjs/open_source/yazelix_related/helix`, `/home/lucca/pjs/open_source/yazelix_related/yazi`, `/home/lucca/pjs/open_source/yazelix_related/zellij`, and `/home/lucca/pjs/open_source/yazelix_related/nushell`.
- Treat those clones as read-only reference checkouts unless the user explicitly asks to modify them.

## Code Robustness Requirements
1. **Avoid fallbacks** - Fallback behavior can mask underlying issues and lead to unpredictable behavior across different environments
2. **Fail fast with clear errors** - When something is wrong, provide explicit error messages rather than degraded functionality
3. **Universal robustness** - Yazelix must work reliably for all users, not just maintainers who can manually fix issues
4. **Avoid redundant code** - Focus on elegant, concise code when possible; eliminate duplication and unnecessary complexity
5. **No duplicate implementation code** - Duplicate code is unacceptable. If a change would copy logic from another module or child repo, fix the ownership boundary or consume a shared/child-owned artifact instead.
6. **Do not ship or rely on local-only host fixes** - Do not patch user-specific caches, local machine state, or other one-off environment artifacts as a substitute for a real Yazelix fix unless the user explicitly asks for a local recovery workaround. Temporary local probes are allowed for diagnosis or to test a hypothesis, but they must be treated as throwaway investigation steps and either reverted or replaced by a real repo fix before calling the work done.
7. **Never automatically move, delete, or take ownership of user-managed config files outside Yazelix-owned paths** - In particular, do not automatically relocate files like `~/.config/zellij/config.kdl` into Yazelix-managed paths. If Yazelix wants to adopt an external user config, that must be an explicit user action such as an import command, not an implicit startup side effect.
8. **No accidental Linux assumptions in shared surfaces** - Linux-only packages, paths, helpers, `/proc` reads, desktop integration, graphics wrappers, clipboard tools, or terminal variants must be explicitly platform-gated before they enter shared Rust, Nix, Home Manager, docs, or runtime code. Use clear gates such as Rust `target_os`, Nix `stdenv.hostPlatform`, or Home Manager platform conditionals, and define the Darwin/macOS or unsupported-platform behavior instead of letting Linux-specific code fail later.

### Error Handling Philosophy
- **No silent failures** - Every error should be visible and actionable
- **Environment independence** - Code should work regardless of host system quirks
- **Consistent behavior** - Same input should produce same output across all user environments
- **Explicit platform support** - If a feature is Linux-only, say so in the package/config surface and fail fast when selected elsewhere. If a feature is meant to be shared, keep it free of Linux-only assumptions or gate those assumptions out of non-Linux builds.

## GitHub Workflow

- Prefer the GitHub CLI (`gh`) for inspecting issues, PRs, comments, and repo metadata instead of scraping GitHub pages manually.
- Use `gh` first when interacting with GitHub state from this repository unless the task specifically requires browser-only behavior.
- Prefix non-automated agent-authored GitHub issue comments with `Agent context: posted by Lucca's Codex coding agent on behalf of the Yazelix maintainers. Model: <model>. Effort: <effort>. Surface: Codex.` When unsure, use `Model: GPT-5.5. Effort: xhigh.`
- GitHub and Beads have a shared-subset contract:
  - GitHub owns the public issue, discussion thread, and open/closed lifecycle.
  - Beads owns planning metadata: dependencies, priority, labels, design notes, acceptance criteria, and execution history.
  - Every GitHub issue in `luccahuguet/yazelix` should have exactly one bead with the issue URL stored as `external_ref`.
  - Every in-contract GitHub issue should also have one canonical visible automated comment of the form `Automated: Tracked in Beads as \`yazelix-...\`.` so the mapping is obvious from the issue page.
  - Open GitHub issues should not map to closed beads, and closed GitHub issues should not map to open beads.
  - Title/body evolution in Beads is allowed after import; `external_ref` and lifecycle sync are the hard contract.
  - The automated validator enforces this contract for issues created on or after `2026-03-22`. Older backlog issues are intentionally grandfathered until they are explicitly imported or touched by the local sync flow.
- Do not mirror every bead to GitHub by default. Use GitHub issues for user-visible bugs, features, contract changes, or work that benefits from public discussion or contributor visibility. Keep decomposition slices, architecture sequencing, maintainer-only tooling, experiments, postmortems, and other planning-only beads internal unless there is a clear reason to publish them.
- Keep the current publication boundary and reviewed internal-only backlog list in `docs/backlog_publication_policy.md`.
- GitHub Actions must stay read-only with respect to Beads. Do not let CI mutate or commit `.beads/issues.jsonl`.
- Sync GitHub issue state into Beads locally during normal maintainer work with `yzx dev sync_issues`; that command is also responsible for creating or repairing the canonical Beads comment on GitHub issues. Then commit the Beads changes on your branch.
- Do not block on `yzx dev sync_issues` when it is slow or hanging. Prefer `br` for Beads mutations whenever possible, continue the implementation work, and repair the GitHub/Beads contract afterward.
- Use `br sync --flush-only`, `br sync --import-only --rebuild`, and `br sync --merge` for JSONL interchange when needed.

## Contract Workflow

- Use `docs/contract_driven_development.md` as the durable entrypoint for when a change needs a canonical contract and how contracts relate to Beads.
- Store canonical contracts under `docs/contracts/`.
- Prefer contracts for durable user-visible behavior, subsystem contracts, integration boundaries, source-of-truth rules, and supported failure modes. Do not create contracts for trivial edits, purely mechanical refactors, research notes, prototype outcomes, or implementation diaries.
- Beads own planning state, decision history, rejected alternatives, implementation sequencing, and closure evidence.
- Contracts own current supported behavior and verification paths. Contracts should not mention Bead ids unless the contract is specifically about Beads/GitHub planning architecture.

## Tool Invocation Workflow

- Prefer `yzx run ...` for project-scoped tool invocations instead of raw `nix develop -c ...` when running tools provided by the Yazelix environment.
- Use raw `nix develop -c ...` only when `yzx run ...` is not a clean fit for the task, such as larger multi-command shell scripts or environment debugging.
- For Rust inner-loop work, prefer the direct maintainer commands before reaching for Nix:
  - `yzx dev rust fmt --check`
  - `yzx dev rust check`
  - `yzx dev rust test <filter>`
  These commands require `cargo` and `rustc` on `PATH` and intentionally avoid re-entering `nix develop`. Treat Nix builds, Home Manager switches, and package validators as explicit final gates, not the default edit-check loop.
- For agent-driven Yazelix invocations, always suppress the welcome/UI path by default. Prefer entrypoints that already do this, such as `yzx run ...`, or pass the equivalent `--skip-welcome` flow when calling Yazelix bootstrap/runtime scripts through `nix develop -c ...`. Do not launch the interactive welcome screen or its animations unless the task is explicitly about validating that UX.
- Be careful with heavyweight Nix probes during investigation. Prefer cheap read-only commands such as `nix eval`, `nix flake show`, `nix path-info`, `rg`, or repo-local code inspection before running `nix build` on large external inputs. Do not casually launch expensive build jobs just to inspect metadata, and if a diagnostic build is truly needed, say so explicitly and clean it up if it is no longer needed.
- For runtime packaging work, use the verification ladder instead of starting with a full runtime build:
  1. Run focused Rust checks/tests for touched code, such as `yzx dev rust check core` or `yzx dev rust test <filter>`.
  2. Run eval-fast package contracts such as `nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).kgp_package_contracts --no-link --no-write-lock-file` for KGP override metadata changes.
  3. Build only the touched package output when needed, such as `nix build .#yazelix_kgp_zellij --no-link --no-write-lock-file`.
  4. Run `nix build .#runtime_ghostty --no-link --no-write-lock-file` once as the final package gate after the smaller checks pass.
- Avoid launching multiple `nix develop`, `nix eval`, or package-build commands in parallel during validation. They contend on Nix eval caches, store locks, and Cargo/Nix build directories, which makes the session slower and noisier than serialized checks.
- **Do not run `yzx restart` as an agent.** It kills the user's live Zellij session. If a runtime change needs a fresh Yazelix session, ask the maintainer to launch one or explicitly approve the destructive restart first.
- For mars runtime updates, expect `nix build .#runtime_mars --no-link --no-write-lock-file` and the normal Home Manager switch to build the terminal through release LTO and package tests when the terminal input changes. Treat that path as a final runtime gate. Use focused terminal-repo Rust checks and builds before switching, and when improving this path, research current Rust/Nix build-speed tools instead of assuming the bottleneck is only one command.
- Do not run mars-related compile-heavy commands (`cargo`, `nix build`, or Home Manager switch) again until the rebuild-speed optimization beads are addressed, unless the maintainer explicitly overrides this gate for a specific command.
- For mars dogfooding after that gate is addressed, prefer the explicit fast outputs `#runtime_mars_fast` and `#mars_fast`; see `docs/mars_fast_dogfooding.md`. Do not treat those fast outputs as release evidence.

## Shell Boundary Rule

- Do not add new inline quoted shell-script bodies assembled inside Nushell just to pass them to `bash -lc`, `sh -c`, or similar entrypoints.
- In particular, avoid patterns like arrays of shell lines joined with `str join "\n"` or interpolated multi-line shell snippets whose dynamic values are baked directly into the script text.
- Prefer one of these instead:
  - a dedicated checked-in POSIX helper script under `shells/posix/` or another clearly owned runtime path
  - structured argv execution without a shell when possible
  - if a tiny shell trampoline is truly unavoidable, keep the script body fixed and pass dynamic values as positional arguments or environment variables instead of interpolating them into the shell program text
- Treat existing inline quoted-script seams as refactor targets, not as patterns to copy.

## Command Surface Policy

- When renaming or simplifying a user-facing command surface, do not keep legacy aliases by default.
- Only preserve old command names as aliases when the user explicitly asks for a compatibility transition.
- Do not add legacy support for command/config/API surfaces that have not been pushed yet. Amend or replace the local commits and delete the old surface instead.
- For recently pushed surfaces, ask the maintainer before adding compatibility support. Bias toward removing the stale surface unless there is a clear release, upgrade, or user-support reason to keep it temporarily.

## Cross-Repo Release Transactions

- Use `docs/child_repo_beads_ownership_policy.md` for the Beads ownership split around child source edits, main integration parents, GitHub issue mapping, and release transaction evidence.
- Treat a main-repo `flake.lock` update that consumes a child-repo change as a coupled release transaction, not as a local-only integration. Trivial child-only docs, tests, CI, or internal package changes can be handled in the child repo by themselves.
- Child extraction beads must name the main code, contract, or runtime closure they will delete or relinquish. A child stringifier plus a main planner is not an extraction unless that planner is intentionally retained and documented as main-owned.
- Local `--override-input` validation is only a development smoke test because it can pass against unpublished child commits. Before committing, closing beads, or pushing the main repo for a coupled change, push the child repo first, update the main `flake.lock` to that GitHub revision, and run the main validation without overrides.
- Close Beads and flush `.beads/issues.jsonl` with the published main change, after manual test approval when the coupled runtime change is non-trivial. If the main lock update or no-overrides validation fails after the child push, leave the child commit published but unused unless the child repo itself needs a fix or revert.

## Rust Plugin Workflow

- **Rust pane-orchestrator source edits are not live by themselves.** Source lives in the public child repo `https://github.com/luccahuguet/yazelix-zellij-pane-orchestrator`, checked out as sibling `../yazelix-zellij-pane-orchestrator` by default, or in the path set by `YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_SOURCE_DIR`. Normal Yazelix runtimes consume the wasm from the locked child package, not from a copied main-repo binary.
- After changing the pane orchestrator, build the child package and test Yazelix through an explicit local flake override before claiming integrated behavior is fixed:
  ```bash
  nix build ../yazelix-zellij-pane-orchestrator#yazelix_zellij_pane_orchestrator --no-link
  nix build .#runtime --override-input yazelixZellijPaneOrchestrator ../yazelix-zellij-pane-orchestrator --no-link
  ```
- Follow the cross-repo release transaction rule before landing any main-repo lock update that consumes a new pane-orchestrator child commit.
- **Do not treat `cargo test` or `cargo check` as sufficient verification for live plugin behavior.** They only validate the Rust source. Real behavior changes require the packaged wasm, runtime build validation, and a fresh Yazelix session.
- After switching to a new packaged plugin wasm, use a fresh Yazelix window for live validation. Do not run `yzx restart` unless the maintainer explicitly approves killing the current Zellij session.
- Run `yzx_repo_validator validate-workspace-session-contract` or `yzx dev test` before committing pane-orchestrator integration work; the validator checks generated workspace assets against the packaged runtime shape.
- Built-in Zellij layout templates live in the in-tree `rust_core/yazelix_zellij_config_pack` crate. After changing them, run `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_zellij_config_pack` and `yzx_repo_validator validate-workspace-session-contract`; `yzx doctor` validates generated layouts against the packaged runtime shape.

## Rust Dependency Gate

- Before starting any Rust implementation bead, record a crate-vs-in-house decision in the bead notes or linked contract.
- The decision must list production crates, dev-only crates, logic to build in-house, rejected alternatives, and packaging impact.
- Default to in-house/std for small domain logic. Add crates when they buy stable parsing, serialization, hashing, error modeling, or test coverage that would be wasteful or riskier to recreate.
- Avoid broad frameworks or convenience crates by default, especially for private helpers. If a broad crate is chosen, explain why the narrower option is worse.
- If the crate list changes during implementation, update the bead or linked contract before continuing so dependency drift is explicit.

## Planning and Decision Making

**MANUAL TEST GATE BEFORE PUSH** - For non-trivial changes, do not push to remote before the user has manually tested the behavior and explicitly approved the push. Only trivial changes may be pushed without manual user testing.

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
5. Record the decision and the new seam: When the work changes future planning, capture the outcome in Beads or maintainer notes so later refactors build on the clarified boundary instead of reopening the same ambiguity.

### Spartan LOC Protocol

Use this for every extraction, cleanup, refactor, validator, generated-fixture, and command-surface bead.

- Main-repo ownership is the score. Moving code to a child repo only counts when the main repo deletes code, stops owning a contract, or shrinks the runtime closure for users who opt out.
- Before starting child extraction work, record the expected main deletion/relinquishment target in the bead acceptance criteria.
- Run `shells/posix/yazelix_loc_scorecard.sh <base> HEAD` before closing meaningful refactor/extraction work, and include the result in the bead close reason or notes.
- A cleanup/refactor/extraction bead should not raise main-repo runtime, maintainer, test, generated, or packaging LOC. If product behavior justifies growth, record the rationale and LOC scorecard in the close reason or notes.
- If a change adds more than `100` main-repo code LOC outside Beads/docs while claiming to simplify, it must delete at least that much in the same bead or record an explicit rationale and affected owner.
- Do not let validators, fixtures, docs, compatibility shims, or wrappers grow around an extraction. Delete stale local scaffolding in the same bead unless a concrete risk forces a separate follow-up.
- Treat transitional migrations and compatibility shims as debt after their current supported window. If cleanup touches a migrated surface, delete the old migration path unless a live contract proves it is still needed.
- When a family drops below its Rust budget ceiling, ratchet the ceiling down in the same commit. Do not rebaseline upward for cleanup work.
- Prefer "not doing it" over adding a configurable abstraction that preserves both old and new owners.

## Verification Requirements

- **Always test the exact functions or commands you change** before committing.
- If a command cannot be executed in this environment, explain why and provide the nearest safe alternative.
- **Prefer high-signal behavior and regression tests** over shallow command-discovery checks.
- **Do not add weak tests that mostly create noise**, such as tests that only verify a command exists in help output, that a subcommand name is listed, or that implementation trivia appears in generated text without protecting meaningful behavior.
- **Every new test should defend a real contract, regression, or failure mode**: user-visible behavior, config/state invariants, integration boundaries, or a bug that has already happened.
- **Every new `test_*.nu` file must declare `# Test lane: ...` using an allowed lane** (`default`, `maintainer`, `sweep`, or `manual`).
- **Every first-party Rust file that contains `#[test]` must declare `// Test lane: ...` using an allowed lane** (`default`, `maintainer`, `sweep`, or `manual`).
- **Treat test quality and lane placement as separate decisions.** Decide whether a test is worth keeping before deciding which lane owns it.
- **Do not create generic `_extended` test files as overflow.** If a nondefault lane needs more coverage, use a file or lane name that reflects its actual ownership.
- **Every new governed `def test_*` must carry a nearby `# Defends:`, `# Regression:`, or `# Invariant:` marker.**
- **Every governed Rust `#[test]` must carry a nearby `// Defends:`, `// Regression:`, or `// Invariant:` marker.**
- **A weak test is not rescued by scoring it.** Exact palette constants, help-output trivia, command-name discovery, and implementation-string checks are not enough unless they defend a documented product contract or regression.
- **Do not add packaging/config-sync tests by default** just because two files should match. Only keep them when they defend a maintained source-of-truth invariant in the right lane; otherwise prefer behavior tests, contract-backed validation, or cheaper dedicated validators.
- **Platform-sensitive changes need platform-sensitive verification.** When touching Rust `cfg`, Nix `hostPlatform`, Home Manager platform logic, desktop entries, terminal packages, graphics wrappers, or host helper dependencies, run the cheapest reliable check that proves the active platform still works and the unsupported platform is gated intentionally. If a supported platform such as `aarch64-darwin` cannot be executed locally, state that gap explicitly and prefer eval or compile checks that still exercise the platform boundary.
- When in doubt, **remove or avoid low-value tests** and spend the budget on fewer, stronger assertions.

## Yazelix Versioning

**Yazelix versioning:**
- Follow the current project versioning scheme used in tags/releases
- When referencing versions in documentation or migration notes, only use actual version numbers that exist
- **Keep `release_metadata.toml` in sync with git tags**: When creating a new git tag, update `release_metadata.toml` to match (for example, `version = "v17.4"`). The packaged runtime writes this into `runtime_identity.json`, and the zjstatus bar reads the version from that runtime identity.
- **Release notes must name keybindings explicitly**: If a release changes, adds, removes, or meaningfully clarifies default keybindings, `CHANGELOG.md` and `docs/upgrade_notes.toml` must list the concrete key combinations and their actions. Do not hide keybinding changes behind vague wording like "keybinding polish" or "directional keybindings".
- **Audit the full release range before cutting a tag**: Before writing release notes, inspect the complete commit range from the previous tag to the new tag candidate (for example `git log --oneline v17.2..HEAD` and a diff stat). `CHANGELOG.md` and `docs/upgrade_notes.toml` must summarize every user-visible feature, behavioral change, packaging/runtime change, migration or escape hatch, and important maintainer/CI/runtime-infrastructure change in that range. Do not rely on memory or only document the most recent commits.

## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`) for issue tracking. Issues are stored in `.beads/issues.jsonl`; the local SQLite database is ignored and can be regenerated.

### br Commands for Issue Management

```bash
br ready              # Show issues ready to work (no blockers)
br list --status open  # All open issues
br show <id>          # Full issue details with dependencies
br create "Title" -p 0 -t task    # Create a P0 task
br create "Title" --parent <id>   # Create child with parent
br update <id> --status in_progress --claim   # Claim and start work
br close <id> --reason "Completed"
br close <id1> <id2>  # Close multiple issues at once
br dep add <child> <parent> --type parent-child  # Add dependency
br graph <id>         # Dependency graph for an issue
br stale              # Show stale issues
br blocked            # Show blocked issues
br sync --flush-only  # Export local SQLite state to issues.jsonl
br sync --import-only --rebuild  # Rebuild local SQLite from issues.jsonl
```

### Workflow Pattern

1. **Triage**: Run `br ready` to find unblocked work and `br show <id>` for issue context
2. **Claim**: Use `br update <id> --status in_progress --claim`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id> --reason "Completed"`
5. **Commit immediately**: After finishing a bead or a local fix, commit the completed change before starting unrelated work. Include `.beads/` changes with the code/docs/config changes they describe.
6. **Sync**: `br sync --flush-only`, then `git add .beads/issues.jsonl .beads/metadata.json .beads/config.yaml .beads/.gitignore .beads/README.md`

### Key Concepts

- **Dependencies**: Issues can block other issues. `br ready` shows only unblocked work. `br dep add <blocked> <blocker>` adds blocking deps.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers 0-4 or P0-P4 strings)
- **Types**: task, bug, feature, epic, chore, decision
- **Dotted IDs**: Beads supports hierarchical IDs (e.g., `yazelix-qgj7.4.3.5`)

### Session Protocol

```bash
git status              # Check what changed
git add .beads/         # Stage tracked Beads JSONL/config changes
git add <files>         # Stage code changes
git commit -m "..."     # Commit everything
git push                # Push to remote
```

<!-- end-br-workflow -->

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **br (beads_rust)** for issue tracking. Run `br ready` and `br show <id>` for workflow context.

### Quick Reference

```bash
br ready              # Find available work
br show <id>          # View issue details
br update <id> --claim  # Claim work
br close <id>         # Complete work
```

### Rules

- Use `br` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `br ready`, `br list`, and `br show` for issue context
- Use Beads notes/comments for persistent project knowledge — do NOT use MEMORY.md files

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
- Commit after finishing each bead or local fix before moving to unrelated work
- Once a push is approved or otherwise required, finish it fully: `git pull --rebase`, `git push`, then verify status. If Beads changed after the last commit, run `br sync --flush-only` and commit the JSONL before pushing.
- Do not claim remote completion for unpushed work
- If an approved push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->

<!-- bv-agent-instructions-v2 -->

---

## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`) for issue tracking and [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) (`bv`) for graph-aware triage. Issues are stored in `.beads/` and tracked in git.

### Using bv as an AI sidecar

bv is a graph-aware triage engine for Beads projects (.beads/beads.jsonl). Instead of parsing JSONL or hallucinating graph traversal, use robot flags for deterministic, dependency-aware outputs with precomputed metrics (PageRank, betweenness, critical path, cycles, HITS, eigenvector, k-core).

**Scope boundary:** bv handles *what to work on* (triage, priority, planning). `br` handles creating, modifying, and closing beads.

**CRITICAL: Use ONLY --robot-* flags. Bare bv launches an interactive TUI that blocks your session.**

#### The Workflow: Start With Triage

**`bv --robot-triage` is your single entry point.** It returns everything you need in one call:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores, reasons, unblock info
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `project_health`: status/type/priority distributions, graph metrics
- `commands`: copy-paste shell commands for next steps

```bash
bv --robot-triage        # THE MEGA-COMMAND: start here
bv --robot-next          # Minimal: just the single top pick + claim command

# Token-optimized output (TOON) for lower LLM context usage:
bv --robot-triage --format toon
```

Before claiming, verify current state with `br show <id> --json` or `br ready --json`. `recommendations` can include graph-important blocked or assigned work; only `quick_ref.top_picks` and non-empty `claim_command` fields represent claimable work.

#### Other bv Commands

| Command | Returns |
|---------|---------|
| `--robot-plan` | Parallel execution tracks with unblocks lists |
| `--robot-priority` | Priority misalignment detection with confidence |
| `--robot-insights` | Full metrics: PageRank, betweenness, HITS, eigenvector, critical path, cycles, k-core |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, label suggestions, cycle breaks |
| `--robot-diff --diff-since <ref>` | Changes since ref: new/closed/modified issues |
| `--robot-graph [--graph-format=json\|dot\|mermaid]` | Dependency graph export |

#### Scoping & Filtering

```bash
bv --robot-plan --label backend              # Scope to label's subgraph
bv --robot-insights --as-of HEAD~30          # Historical point-in-time
bv --recipe actionable --robot-plan          # Pre-filter: ready to work (no blockers)
bv --recipe high-impact --robot-triage       # Pre-filter: top PageRank scores
```

### br Commands for Issue Management

```bash
br ready              # Show issues ready to work (no blockers)
br list --status=open # All open issues
br show <id>          # Full issue details with dependencies
br create --title="..." --type=task --priority=2
br update <id> --status=in_progress
br close <id> --reason="Completed"
br close <id1> <id2>  # Close multiple issues at once
br sync --flush-only  # Export DB to JSONL
```

### Workflow Pattern

1. **Triage**: Run `bv --robot-triage` to find the highest-impact actionable work
2. **Claim**: Use `br update <id> --status=in_progress`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id>`
5. **Sync**: Always run `br sync --flush-only` at session end

### Key Concepts

- **Dependencies**: Issues can block other issues. `br ready` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers 0-4, not words)
- **Types**: task, bug, feature, epic, chore, docs, question
- **Blocking**: `br dep add <issue> <depends-on>` to add dependencies

### Session Protocol

```bash
git status              # Check what changed
git add <files>         # Stage code changes
br sync --flush-only    # Export beads changes to JSONL
git commit -m "..."     # Commit everything
git push                # Push to remote
```

<!-- end-bv-agent-instructions -->
