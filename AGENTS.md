# Agent Guidelines

Yazelix Next is a clean architecture track for a Yazelix-like runtime with the
fewest practical lines of code and the simplest ownership model

## Core Rule

The user decides scope. Do not create a feature, compatibility surface, module,
or planning bead until the user has chosen that direction

## Method

Use contract-driven, check-backed development, not mechanical porting from main
Yazelix:

1. State the irreducible user-visible behavior in one paragraph
2. Name the current Yazelix sources of truth and decide what survives
3. Choose one owner in this repo
4. Choose the cheapest check that proves the contract
5. Implement the smallest slice that satisfies the contract
6. Avoid duplicate owners, adapters, generated fixtures, and compatibility shims
7. Record important rejected alternatives in Beads

Start with the smallest usable vertical slice and polish it before expanding

Use TDD where it fits: Rust helpers, parsers, deterministic CLI behavior, and
regression fixes. Do not use classic TDD as the default for layout design,
runtime integration, fork decisions, or dogfooding surfaces; write the contract
and the focused check first.

## Testing Discipline

Keep tests strong and few. A test should prove a contract, regression, boundary,
or failure mode that matters to users or future agents.

Do not keep weak tests. If a test only repeats another check, asserts
implementation trivia, or mostly preserves scaffolding, either merge its useful
assertion into a stronger test or delete it.

Prefer one contract test with clear setup and meaningful assertions over several
thin tests that make refactors harder without increasing confidence.

## Current Runtime

Current chain:

```text
yzn -> Mars -> Yazelix Zellij fork
```

The project interface is a Nix/Lix-compatible flake. `yzn` is the installed
command name so it does not conflict with main Yazelix `yzx`

After changing the flake runtime, keep the user's installed runtime current:

```sh
nix profile upgrade --refresh yazelix-next
```

Do not add Home Manager, layouts, config generation, plugins, pane policy, or
legacy compatibility unless the user explicitly chooses that feature

## Beads

Use `br` for all issue work. Do not edit `.beads/` files directly

Serialize `br` write commands. Keep decisions that matter later in Beads rather
than relying on chat history

## LOC Discipline

Update the README LOC scorecard whenever project files change

Update `CHANGELOG.md` when user-visible runtime behavior, commands, keymaps,
packaged tools, or runtime contracts change

Prefer deleting scope, avoiding abstractions, and reusing existing package
outputs over adding local wrappers. If LOC grows, the added behavior should be
visible in the scorecard and justified by the slice

Formatting rules outrank LOC pressure. For Rust, run and keep `rustfmt` output;
do not manually compress formatted Rust just to lower the scorecard

## Verification

Run the cheapest exact checks for the changed surface. For runtime flake
changes, normally verify:

```sh
nix flake check
nix flake show --all-systems
nix build .#yzn --no-link --print-build-logs
nix profile add --refresh /home/lucca/pjs/yazelix-dir/yazelix-next --profile <tmp>
```

Do not launch GUI sessions unless the user asks or reports manual dogfooding

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
