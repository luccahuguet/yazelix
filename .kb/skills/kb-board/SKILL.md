---
name: kb-board
description: Show GitKB kanban board with task status columns
---

Display the kanban board and provide actionable context about the current workstream.

**Input:** `$ARGUMENTS`

Optional arguments: `--group-by <field>`, `--columns <list>`, `--sort-by <field>`, `--sort-direction <asc|desc>`, `--all`

For agent work, always use `--json`. Plain board output is for humans and can truncate slugs.

## Steps

### 1. Show the Board

If no arguments provided, show the default status board:

```bash
git-kb board --all --json
```

If `--group-by` was specified, pass it through:

```bash
git-kb board --group-by priority --all --json
git-kb board --group-by tags --all --json
git-kb board --group-by priority --columns critical,high,medium,low --all --json
git-kb board --group-by status --sort-by priority --sort-direction asc --all --json
```

**Available flags:**
- `--group-by <field>` — Group by any frontmatter property instead of status (e.g. `priority`, `tags`, `assignee`, `component`, or custom fields)
- `--columns <list>` — Explicit column order (comma-separated, requires `--group-by`). Unlisted values appear as extra columns at the end.
- `--sort-by <field>` — Sort items within columns by a frontmatter property (requires `--group-by`)
- `--sort-direction <asc|desc>` — Sort direction (default: asc, requires `--group-by`)
- `--json` — Machine-readable JSON output with complete slugs/IDs
- `--all` — Show all items and document types

**Notes:**
- Array properties (like `tags`) place documents in multiple columns
- Documents without the grouped field appear in an `(unset)` column
- Status and priority columns use enum ordering; other fields sort alphabetically

### 2. Analyze Blocked Tasks

If any tasks are in the BLOCKED column (or have `blockedBy` relationships):
- Use `kb_show` to load each blocked task
- Identify what's blocking them
- Summarize: "X is blocked by Y because Z"

### 3. Suggest Next Task

Look at ACTIVE and DRAFT tasks. Suggest what to work on next based on:
- **Priority**: high > medium > low
- **Dependencies**: unblocked tasks first
- **Momentum**: tasks related to recently completed work

### 4. Flag Staleness

If any task has been ACTIVE with no progress log entries in the last 7 days, flag it:
- "task-slug has been active since [date] with no progress updates in 7+ days — is it still being worked on?"

### 5. Present Summary

Show the board output, then add:
- Count of tasks by status (or by the grouped field)
- Any blocked items with reasons
- Suggested next task with rationale
