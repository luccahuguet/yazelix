---
name: kb-handoff
description: End-of-session handoff — update context, log progress, commit pending changes
---

Prepare for session end or agent handoff. Ensures all work-in-progress is captured and context documents are fresh for the next session.

**Input:** `$ARGUMENTS`

Optional: a summary of what was accomplished this session. If not provided, the command will infer it from workspace changes and recent commits.

## Steps

### 1. Commit Pending Changes

Check `kb_status` for uncommitted workspace changes.

If changes exist:
1. Use `kb_diff` to review them
2. Commit with a descriptive message
3. Warn if any changes look like they should not be committed (debug content, temporary notes)

### 2. Log Progress on Active Tasks

Find all active tasks: `kb_list` with `type: "task"`, `status: "active"`.

For each active task that was worked on this session:
1. Checkout the task
2. Add a progress log entry with today's date summarizing what was done
3. Update acceptance criteria checkboxes if any were completed
4. Commit: `"Progress: <summary>"`

If the user provided a session summary in `$ARGUMENTS`, use it to write better progress entries.

### 3. Update Active Context

Checkout `context/overridable/active` and update it to reflect:
- **Current focus**: What's being worked on now
- **Recent completions**: Tasks completed or major progress this session
- **Task board**: Updated counts (completed, active, draft)
- **What's next**: Suggested next steps for the next session

Use `kb_list` and `kb_board` (or `git-kb board --json` if MCP is unavailable) to get accurate counts.

### 4. Update Progress (if milestone reached)

If a significant milestone was completed this session (task completed, phase finished, PR merged):
1. Checkout `context/overridable/progress`
2. Update the relevant section (mark phases complete, update metrics, add PR links)
3. Commit

### 5. Verify Clean State

Final checks:
1. `kb_status` — workspace should be clean (all committed)
2. Any active tasks should have recent progress entries
3. Context docs should reflect the current state

### 6. Present Handoff Summary

Show the next agent (or next session) everything they need:

```
## Session Handoff

### What Was Done
- [Summary of work completed]

### Current State
- Active tasks: [list with status]
- Pending work: [what's next]
- Blockers: [if any]

### For Next Session
- Start with: /kb-context to load project state
- Then: /kb-start <suggested-task> to resume work
- Watch out for: [any gotchas or context the next session needs]
```
