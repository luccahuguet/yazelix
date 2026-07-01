---
name: kb-close
description: Complete a task with verification — check acceptance criteria, require evidence, update status
---

Close a task after verifying all acceptance criteria are met. This enforces AGENTS.md rule #8: "Complete document body before status updates."

**Input:** `$ARGUMENTS`

The argument should be a task slug (e.g. `tasks/my-task`).

## Steps

### 1. Load the Task

Use `kb_show` to load the task document. If not found, show available tasks and ask.

### 2. Audit Acceptance Criteria

Parse the document body for acceptance criteria (lines matching `- [ ]` or `- [x]`).

**Count and report:**
- Total criteria
- Checked (`[x]`) — completed
- Unchecked (`[ ]`) — remaining

**If unchecked items remain**, show each one and ask the user:
> "These acceptance criteria are not yet checked:
> - [ ] Criterion A
> - [ ] Criterion B
>
> Are these actually done (I'll check them off), no longer relevant (I'll remove them), or still outstanding (we should not close yet)?"

Do NOT proceed to close if the user says items are still outstanding.

### 3. Verify Completion Evidence

Check if the document has a "Completion Evidence" section (commit hashes, PR links, test results, verification steps). This is distinct from the "Progress Log" (chronological work entries). Both are valuable, but completion evidence is the proof that the work was actually done.

If neither section exists, warn:
> "This task has no completion evidence. Before closing, consider adding:
> - Commit hashes or PR links
> - Test results
> - Verification steps taken"

Offer to add a "Completion Evidence" section based on what's known.

### 4. Check for Orphaned Work

Use `kb_graph` to see if this task has children or blocks other tasks:
- If it has uncompleted children, warn and ask: "This task has open child tasks: [list]. Close them first, or unlink them with `kb_unlink` (e.g. `kb_unlink` with `child: "<child-slug>"`, `container: "<this-slug>"`). Continue anyway?"
  - Do NOT proceed without explicit user confirmation.
- If it blocks other tasks, note: "Closing this will unblock: [list]."

### 5. Update the Document

Checkout the task and update it:
1. Check off any acceptance criteria confirmed as complete
2. Add a "Completion Evidence" section if missing (separate from Progress Log)
3. Add a dated entry to "Progress Log": `### YYYY-MM-DD\n- Task completed. [summary]`
4. Commit the body changes first: `"Complete task: <title>"`

### 6. Set Status

Only after the body is updated and committed:

```text
kb_set with slug: "<task-slug>", status: "completed"
```

### 7. Confirm

Show:
- Task slug and title — marked complete
- Final acceptance criteria state (all checked)
- Tasks that are now unblocked (if any)
- Suggest updating context docs if this was a major milestone
