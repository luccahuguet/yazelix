---
name: kb-progress
description: Add a progress log entry to a task — quick datestamped note and commit
---

Quickly log progress on a task. This is the most common mid-work operation — add a datestamped note without a full edit cycle.

**Input:** `$ARGUMENTS`

Format: `<task-slug> <progress note>`

Examples:
- `/kb-progress tasks/my-task Implemented warning system for graph-derived fields. 3 of 5 acceptance criteria done.`
- `/kb-progress tasks/my-project-28 Created slash command, testing auto-increment logic`
- `/kb-progress Fixed the serialization bug, all tests passing now` (uses most recently active task)

## Steps

### 1. Parse Input

Extract the **task slug** (first word if it contains `/`) and **progress note** (remainder).

If no slug is provided, find the most recently modified active task:
1. `kb_list` with `type: "task"`, `status: "active"`
2. If no active tasks exist, tell the user: "No active tasks found. Specify a task slug or start a task with `/kb-start`."
3. Pick the one modified most recently
4. Confirm with the user: "Logging progress on [slug] — [title]. Correct?"

### 2. Load and Checkout

Use `kb_show` to load the task, then `kb_checkout` to materialize it.

### 3. Append Progress Entry

Read the workspace file. Find or create the `## Progress Log` section.

Append a new dated entry at the **top** of the progress log (reverse chronological):

```markdown
### YYYY-MM-DD
- [progress note from user]
```

If a `### YYYY-MM-DD` header for today already exists, append the bullet under it instead of creating a new header.

Use the Edit tool to insert the entry. Do not overwrite the file.

### 4. Update Acceptance Criteria (if applicable)

If the progress note mentions completing specific acceptance criteria, check them off (`[ ]` → `[x]`) in the document body.

### 5. Commit

Use `kb_commit` with:
- `message`: `"Progress: <short summary>"`
- `pathspecs`: `["<task-slug>"]` (the single document modified in this session)

### 6. Confirm

Show:
- Task slug and title
- The entry that was added
- Current acceptance criteria state (X of Y done)
