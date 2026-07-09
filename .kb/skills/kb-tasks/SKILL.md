---
name: kb-tasks
description: List tasks with filtering, relationships, and status details
---

List tasks from the knowledge base with rich detail.

**Input:** `$ARGUMENTS`

Optional filter: `active`, `draft`, `completed`, `blocked`, `all`, or a search term.

## Steps

### 1. Parse Filter

| Argument | Action |
|----------|--------|
| *(empty)* | Show all tasks grouped by status |
| `active` | `kb_list` with `type: "task"`, `status: "active"` |
| `draft` | `kb_list` with `type: "task"`, `status: "draft"` |
| `completed` | `kb_list` with `type: "task"`, `status: "completed"` |
| `blocked` | `kb_list` with `type: "task"`, `status: "blocked"` |
| `all` | `kb_list` with `type: "task"` (all statuses) |
| *other text* | Search tasks: `kb_list` with `type: "task"`, then filter by matching title/tags |

### 2. Enrich Results

For each task returned, show:
- **Slug** and **title**
- **Status** and **priority**
- **Tags** (if any)
- **Blocked by** (if the task has `blocked_by` relationships)

If there are fewer than 10 tasks in the result, use `kb_show` to load them and display richer detail. For larger lists, show the summary table.

### 3. Present

Format as a table:

```markdown
| Slug | Title | Status | Priority | Tags |
|------|-------|--------|----------|------|
```

If any tasks are blocked, add a "Blocked Tasks" section explaining what's blocking each one.
