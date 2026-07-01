---
name: code-intelligence
description: Use GitKB code intelligence tools instead of grep for finding callers, definitions, usages, and dead code. Trigger when exploring code relationships or searching for symbol usage.
---

# /code-intelligence

Use GitKB code intelligence instead of grep for code relationships. These tools understand the AST, not just text matches.

## When to Use

- Finding callers of a function or method
- Finding what a function calls (callees)
- Locating symbol definitions
- Assessing change blast radius
- Finding dead code (symbols with zero callers)
- Diagnosing index/resolution health before changing matching behavior
- Inspecting inferred entrypoints, execution flows, and typed graph queries

## Preferred Tools (CLI)

```bash
# List/search symbols
git-kb code symbols --file <path> --json # All symbols in a file
git-kb code symbols "<name>" --json      # Search by name

# Call graph
git-kb code callers "<symbol>" --json   # Who calls this?
git-kb code callees "<symbol>" --json   # What does this call?

# Impact and dead code
git-kb code impact <file> --json        # Transitive blast radius
git-kb code dead --json                 # Symbols with zero callers

# Index health and derived graph primitives
git-kb code doctor --json               # Index and call-resolution health
git-kb code entrypoints --refresh --json # Inferred entrypoints
git-kb code flows --refresh --json      # Entrypoint-derived flows
git-kb code flow <flow-id> --json       # Inspect one flow
git-kb code query hotspots --json       # hotspots, entrypoints, public-api, unresolved-by-reason, cross-service-impact, dead-code-explain
```

If MCP tools are available, prefer `kb_symbols`, `kb_callers`, `kb_callees`, `kb_impact`, `kb_dead_code`, `kb_code_doctor`, `kb_code_entrypoints`, `kb_code_flows`, `kb_code_flow`, and `kb_code_query` for structured JSON output and parallel invocation.
Always use JSON output when copying symbol IDs; table output may be abbreviated for display.

## Instead of Grep

| Instead of | Use |
|------------|-----|
| `grep` for function callers | `git-kb code callers "<symbol>" --json` — actual call sites from call graph |
| `grep` for definitions | `git-kb code symbols --json` — finds by name with signature and location |
| `grep` to understand dependencies | `git-kb code callees "<symbol>" --json` — actual callees from call graph |
| `grep` to assess change impact | `git-kb code impact <path> --json` — transitive blast radius analysis |
| `grep` + manual review for dead code | `git-kb code dead --json` — symbols with zero callers |
| manual DB inspection for index health | `git-kb code doctor --json` — symbol/call/unresolved breakdowns and recommendations |
| manual entrypoint tracing | `git-kb code entrypoints --json` and `git-kb code flows --refresh --json` |
| ad hoc SQL for graph questions | `git-kb code query hotspots --json` |

Grep is still appropriate for config files, string literals, error messages, and non-code content.

## Disambiguating Symbols

When a symbol name matches multiple definitions:

```bash
# Ambiguous — multiple functions named "apply"
git-kb code callers "apply" --json
# → hint: Re-run with the full symbol ID

# Disambiguated — use file::kind::name
git-kb code callers "src/store.rs::function::apply" --json
```

## Prerequisites

Code must be indexed first:
```bash
git-kb code index
```

After indexing, run `git-kb code doctor --json` when results look empty or suspicious. In nested meta/worktree checkouts, the installed CLI may resolve the KB root incorrectly; if `git-kb code index` reports `Indexed 0 symbols from 0 files` with many skipped files, verify you are running the intended binary and KB root before trusting `git-kb code` output.

After initial indexing, the file watcher keeps the index current automatically.
