---
description: Code Intelligence
alwaysApply: true
trigger: always_on
---

# Code Intelligence

This project has code intelligence tools available via MCP (`kb_callers`, `kb_symbols`, `kb_code_doctor`, `kb_code_query`, etc.) and CLI (`git-kb code callers <symbol> --json`, `git-kb code doctor --json`, `git-kb code query hotspots --json`, etc.). **Prefer MCP tools** — they support parallel calls and return structured JSON. Fall back to CLI via Bash if MCP is disconnected.

The daemon automatically re-indexes files on save via file watching (500ms debounce). No manual re-indexing needed during normal coding.

## Use Code Intelligence Instead of Grep

Do NOT use Grep or `grep` to find callers, usages, or definitions of functions/methods/types. Use code intelligence tools instead — they understand the AST, not just text matches.

| Instead of | Use |
|------------|-----|
| `Grep` for function callers | `kb_callers` — returns actual call sites from the call graph |
| `Grep` for function definitions | `kb_symbols` with `search:` — finds by name with signature and location |
| `Grep` to understand what a function calls | `kb_callees` — returns actual callees from the call graph |
| `Grep` to assess change impact | `kb_impact` with `file_path:` — transitive blast radius analysis |
| `Glob` + `Grep` to find dead code | `kb_dead_code` — finds symbols with zero callers |
| manual DB inspection for index health | `kb_code_doctor` — symbol/call/unresolved breakdowns and recommendations |
| manual entrypoint tracing | `kb_code_entrypoints` and `kb_code_flows` |
| ad hoc SQL for graph questions | `kb_code_query` — typed graph query templates |

Grep is still appropriate for searching config files, string literals, error messages, and non-code content.

## Before Modifying Functions

Before changing a function signature, renaming a symbol, or modifying a struct's fields, check callers:

```text
kb_callers with symbol: "<symbol_name>"
```

This shows every call site that would break. Use this to assess blast radius before making changes.

## When Exploring Unfamiliar Code

When you need to understand a module or file you haven't seen before, run these in parallel:

```text
kb_symbols with file_path: "<file_path>"     # List all symbols in a file
kb_callers with symbol: "<symbol_name>"      # Who calls this?
kb_callees with symbol: "<symbol_name>"      # What does this call?
kb_code_doctor                               # Index and call-resolution health
```

## Task-Aware Code Context

When starting a task that involves code changes, use `kb_smart_context` to get a token-budgeted assembly of relevant code and documents:

```text
kb_smart_context with task: "<task-slug>"
```

This extracts signals from the task (code references, file paths, keywords), resolves symbols, traverses the call graph for callers and callees, and returns ranked results within a token budget. Each item includes a relevance score and why it was included (direct reference, caller, callee, or semantic match).

Key parameters:
- `token_budget` (default 8000) — max tokens of context to return
- `include_callers` / `include_callees` (default true) — enable call graph traversal
- `call_depth` (default 2) — how deep to traverse
- `min_score` (default 0.3) — minimum relevance to include

For lighter-weight enrichment, use `kb_context` with `include_code_refs: true` to resolve `[[code:...]]` wikilinks in a task to their symbol metadata (file path, line range, signature) without call graph traversal.

## Skills

Use these skills for structured code intelligence workflows:

- `/understand <file|symbol>` — Analyze structure, callers, callees, and related docs
- `/refactor-safety <symbol>` — Safety check with blast radius and all call sites
- `/explore <query>` — Semantic search across code and documents (requires embeddings; enable in `.kb/config.toml` first)

## Initial Indexing

If symbols commands return empty for a directory that hasn't been indexed yet:

```bash
git-kb code index <directory_or_file>
```

After indexing, run `git-kb code doctor --json` when results look empty or suspicious. In nested meta/worktree checkouts, the installed CLI may resolve the KB root incorrectly; if `git-kb code index` reports `Indexed 0 symbols from 0 files` with many skipped files, verify you are running the intended binary and KB root before trusting `git-kb code` output.

After initial indexing, the file watcher keeps the index current automatically.

## MCP Tool Reference

| Tool | Purpose |
|------|---------|
| `kb_symbols` | Search/list indexed symbols (filter by file, kind, language) |
| `kb_callers` | Find all callers of a function |
| `kb_callees` | Find all functions called by a symbol |
| `kb_impact` | Analyze change blast radius across the call graph |
| `kb_dead_code` | Find potentially dead code (symbols with no callers) |
| `kb_code_doctor` | Report code index and call-resolution health |
| `kb_code_entrypoints` | List inferred code entrypoints |
| `kb_code_flows` | List entrypoint-derived code flows |
| `kb_code_flow` | Inspect one entrypoint-derived flow |
| `kb_code_query` | Run typed code graph query templates |
| `kb_symbol_refs` | Find KB documents that reference a code symbol |
| `kb_smart_context` | Task-aware code context: call graph traversal + token budgeting |
| `kb_index` | Initial indexing of source files |
