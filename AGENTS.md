# Agent Guidelines

This file is the single source of truth for agent workflow, coding conventions, and implementation expectations in this repository.

## Beads Workflow

Use Beads as the agent memory and triage layer for Yazelix work.

- Prefer `AGENTS.md` for durable agent workflow rules; keep `CLAUDE.md` focused on repo conventions and implementation guidance.
- Use `bv --robot-triage` as the default entrypoint for task context instead of manually reconstructing project state from `.beads`.
- When token budget matters, prefer `bv --robot-triage --format toon`.
- For bounded handoff context, use `bv --agent-brief <dir>` to export a compact bundle (`triage.json`, `insights.json`, `brief.md`, `helpers.md`).
- Use `br` for issue mutations and sync: create, update, close, dependency management, and `br sync --flush-only`.
- Treat scope boundaries strictly:
  - `bv` decides what to work on.
  - `br` updates issue state.
  - Coordination between multiple agents should use a separate coordination layer, not ad-hoc issue comments or long prompt memory.
- Keep agent files short. Do not copy large issue graphs, long triage dumps, or project history into `AGENTS.md` / `CLAUDE.md`; store dynamic state in Beads and regenerate it when needed.

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

## Spec Workflow

- Use `docs/spec_driven_workflow.md` as the durable entrypoint for when a change needs a spec and how specs relate to Beads.
- Store reusable spec templates and concrete specs under `docs/specs/`.
- Prefer specs for user-visible behavior, subsystem contracts, and integration boundaries. Do not create specs for trivial edits or purely mechanical refactors.
- Real specs should include a small `Traceability` section with one Bead id and at least one concrete `Defended by` check or test.

## Tool Invocation Workflow

- Prefer `yzx run ...` for project-scoped tool invocations instead of raw `devenv shell ...` when running tools provided by the Yazelix environment.
- Use raw `devenv shell ...` only when `yzx run ...` is not a clean fit for the task, such as larger multi-command shell scripts or environment debugging.

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
- If the current shell toolchain cannot build `wasm32-wasip1`, use the pinned Yazelix environment:
  ```bash
  devenv shell -- nu -c 'source nushell/scripts/yzx/dev.nu; yzx dev build_pane_orchestrator --sync'
  ```
- For popup-runner rebuilds in the pinned environment:
  ```bash
  devenv shell -- nu -c 'source nushell/scripts/yzx/dev.nu; yzx dev build_popup_plugin --sync'
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

1. Make the requirements less "dumb": Question every requirement, especially those from smart people, to ensure they are not illogical or based on flawed assumptions.
2. Delete the part or process: Actively remove unnecessary steps, components, or processes. If you are not occasionally adding parts back in, you are not deleting enough.
3. Simplify or optimize: Streamline the remaining essential components. Never optimize a part or process that should have been deleted.

## Verification Requirements

- **Always test the exact functions or commands you change** before committing.
- If a command cannot be executed in this environment, explain why and provide the nearest safe alternative.
- **Prefer high-signal behavior and regression tests** over shallow command-discovery checks.
- **Do not add weak tests that mostly create noise**, such as tests that only verify a command exists in help output, that a subcommand name is listed, or that implementation trivia appears in generated text without protecting meaningful behavior.
- **Every new test should defend a real contract, regression, or failure mode**: user-visible behavior, config/state invariants, integration boundaries, or a bug that has already happened.
- **Do not add packaging/config-sync tests by default** just because two files should match. Only keep them when they defend a maintained source-of-truth invariant in the right lane; otherwise prefer behavior tests, spec-backed validation, or cheaper dedicated validators.
- When in doubt, **remove or avoid low-value tests** and spend the budget on fewer, stronger assertions.

## Yazelix Versioning

**Yazelix versioning:**
- Follow the current project versioning scheme used in tags/releases
- When referencing versions in documentation or migration notes, only use actual version numbers that exist
- **Keep `YAZELIX_VERSION` in sync with git tags**: When creating a new git tag, update `nushell/scripts/utils/constants.nu` to match (e.g., `export const YAZELIX_VERSION = "v12.3"`). This version is displayed in the zjstatus bar.

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
