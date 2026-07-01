---
name: gitkb
description: Manage GitKB knowledge base for project documentation, tasks, and context. Use when working with KB documents, viewing tasks, updating progress, or managing project knowledge.
---

# GitKB Knowledge Base Skill

GitKB is a distributed knowledge base with a git-like CLI. Documents are stored in a local database and materialized to `.kb/workspace/` for editing.

## Common Gotchas

1. **No type-specific subcommands**: Use `git-kb show <slug> --json`, NOT `git-kb task show`
2. **Machine-readable first**: Use MCP tools when available; otherwise add `--json` to every command that supports it
3. **Never copy identifiers from tables**: Human-readable tables/boards/tree output can truncate slugs, IDs, and symbol IDs with `…`
4. **Board rendering**: Use `kb_board` or `git-kb board --json` for agent work; plain `git-kb board` is for human display only
5. **Always check numbering**: Run `git-kb list <type> --json` before creating new documents

## CLI Reference

### Document Operations

```bash
# List and search
git-kb list --json                            # All documents
git-kb list --type task --status active --json # Filter by type/status
git-kb show <slug> --json                    # View document content
git-kb search "<keywords>" --json            # Ranked full-text search
git-kb search "<keywords>" --all-terms --json # Require every term

# Create and modify
git-kb create --type task --slug tasks/my-task --title "My Task" --json
git-kb set <slug> --status active            # Quick metadata update (auto-commits)
```

### Workspace Operations

```bash
git-kb checkout <slug>                       # Materialize for editing
git-kb checkout --path context/              # Checkout by path prefix
git-kb status --json                         # Show pending changes
git-kb diff                                  # Show line-level diffs
git-kb commit -m "msg" <pathspecs...>        # Save changes to database
git-kb stash                                 # Stash workspace changes
git-kb reset                                 # Discard workspace changes
git-kb clear                                 # Remove from workspace
```

### Board and Context

```bash
# Kanban board
git-kb board --json                          # Tasks grouped by status
git-kb board --all --json                    # All document types
git-kb board --group-by priority --json      # Group by priority
git-kb board --group-by tags --json          # Group by tags
git-kb board --sort-by priority --json       # Sort items within columns

# Context bootstrap
git-kb context --compact --code-refs         # Task-aware context bundle
```

### Relationship and History

```bash
git-kb graph <slug> --json                   # Show document relationships
git-kb log <slug>                            # Commit history
git-kb link --child <child> --container <parent>  # Link documents
```

### Code Intelligence

```bash
git-kb code symbols --file <path> --json     # List symbols in a file
git-kb code symbols "<name>" --json          # Search symbols by name
git-kb code callers "<symbol>" --json        # Find callers
git-kb code callees "<symbol>" --json        # Find callees
git-kb code impact <file> --json             # Blast radius analysis
git-kb code dead --json                      # Find dead code
git-kb code index                            # Index source files
git-kb code doctor --json                    # Diagnose index and call-resolution health
git-kb code entrypoints --refresh --json     # Inspect inferred entrypoints
git-kb code flows --refresh --json           # List entrypoint-derived flows
git-kb code flow <flow-id> --json            # Inspect one flow
git-kb code query hotspots --json            # Run typed graph query templates
```

### Service Control

```bash
git-kb daemon start                          # Start daemon
git-kb daemon stop                           # Stop daemon
git-kb daemon status                         # Check status
```

## MCP Tools

When MCP tools are available, prefer them for structured JSON output and parallel invocation:

| CLI | MCP | Purpose |
|-----|-----|---------|
| `git-kb list --json` | `kb_list` | List documents with filtering |
| `git-kb show <slug> --json` | `kb_show` | Get document(s) by slug (supports batch via `slugs: [...]`) |
| `git-kb create --type task --slug tasks/my-task --title "My Task" --json` | `kb_create` | Create new document |
| `git-kb set` | `kb_set` | Quick metadata update |
| `git-kb checkout` | `kb_checkout` | Materialize to workspace |
| `git-kb status --json` | `kb_status` | Show pending changes |
| `git-kb commit` | `kb_commit` | Save workspace changes |
| `git-kb diff` | `kb_diff` | Show line-level diffs |
| `git-kb board --json` | `kb_board` | Kanban board view |
| `git-kb search "<query>" --json` | `kb_search` | Ranked full-text search |
| `git-kb graph <slug> --json` | `kb_graph` | Relationship graph |
| `git-kb code symbols --json` | `kb_symbols` | Query code symbols |
| `git-kb code callers "<symbol>" --json` | `kb_callers` | Find callers |
| `git-kb code callees "<symbol>" --json` | `kb_callees` | Find callees |
| `git-kb code impact <path> --json` | `kb_impact` | Change impact analysis |
| `git-kb code dead --json` | `kb_dead_code` | Find dead code |
| `git-kb code doctor --json` | `kb_code_doctor` | Report index and call-resolution health |
| `git-kb code entrypoints --json` | `kb_code_entrypoints` | List inferred entrypoints |
| `git-kb code flows --json` | `kb_code_flows` | List entrypoint-derived flows |
| `git-kb code flow <flow-id> --json` | `kb_code_flow` | Inspect one flow |
| `git-kb code query hotspots --json` | `kb_code_query` | Run typed graph query templates |
| `git-kb ai semantic "<query>" --json` | `kb_semantic` | Semantic search |
| — | `kb_smart_context` | Task-aware code context (MCP-only) |
| — | `kb_context` | Context bootstrap bundle (MCP-only) |

**Related skills:**
- `/understand <file|symbol>` — Analyze structure and dependencies
- `/refactor-safety <symbol>` — Safety check: callers, callees, impact analysis
- `/explore <query>` — Semantic search to find relevant code and docs

## Workflows

### Starting Work on a Task

1. View available tasks:
   ```bash
   git-kb board --json
   git-kb list --type task --status active --json
   ```

2. Checkout task to workspace:
   ```bash
   git-kb checkout tasks/my-task
   ```

3. Edit the file at `.kb/workspace/tasks/my-task.md`

4. Commit changes (always scope to your documents):
   ```bash
   git-kb commit -m "Progress on feature" tasks/my-task
   ```

### Completing a Task

Before changing status to `completed`:

1. Update the document content with completion evidence:
   - Mark acceptance criteria as checked (`- [x]`)
   - Add completion evidence with commit hashes
   - Document any follow-up items

2. Then update the status:
   ```bash
   git-kb set tasks/my-task --status completed
   ```

### Creating a New Task

```bash
git-kb create --type task --slug tasks/my-task --title "Implement feature X" --json
git-kb checkout tasks/my-task
# Edit .kb/workspace/tasks/my-task.md
git-kb commit -m "Add my-task" tasks/my-task
```

### Traceability

Include task slugs in git commit messages:
```
fix: resolve auth timeout issue

Implements [[tasks/my-task]]
```

## Key Concepts

| Term | Definition |
|------|------------|
| **Workspace** | `.kb/workspace/` — Files materialized for editing |
| **Checkout** | Materialize document from DB to workspace |
| **Commit** | Sync workspace changes back to database |
| **Slug** | Human-readable document ID (e.g., `tasks/my-task`) |
| **Wikilink** | `[[slug]]` reference between documents |

## Document Types

- `task` — Work items with status tracking
- `note` — General documentation
- `spec` — Technical specifications
- `context` — Project context documents
- `brief` — Project brief
- `architecture` — Architecture documentation
- `patterns` — Design patterns

## Status Values

`draft` → `backlog` → `active` → `blocked` → `completed` → `done`

## Document Naming Conventions

| Type | Pattern | Example |
|------|---------|---------|
| Task | `tasks/{prefix}-{N}` | `tasks/my-project-1` |
| Incident | `incidents/inc-{NNN}-{slug}` | `incidents/inc-001-auth-timeout` |
| Context | `context/{category}/{name}` | `context/overridable/active` |
| Note | `notes/{slug}` | `notes/api-design` |
| Spec | `specs/{slug}` | `specs/federation-protocol` |

Always check existing documents before creating to ensure consistent numbering:
```bash
git-kb list <type> --json
```
