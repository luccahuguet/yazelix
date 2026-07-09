---
name: kb-review
description: Review a task against its acceptance criteria and current codebase state
---

Review a task to assess completeness. Check acceptance criteria against the actual codebase, identify what's done, what's remaining, and what's changed since the task was written.

**Input:** `$ARGUMENTS`

The argument should be a task slug (e.g. `tasks/my-task`).

## Steps

### 1. Load the Task

Use `kb_show` to load the full task document. Parse:
- Acceptance criteria (checklist items)
- Goals
- Implementation details
- Referenced files, functions, or modules

### 2. Check Each Criterion Against Code

For each acceptance criterion, assess whether it's been satisfied:

**If the criterion references specific code** (a flag, a function, a file):
- Use `kb_symbols`, `Glob`, or `Grep` to check if it exists in the codebase
- Use `Read` to verify the implementation matches what's described
- Mark as: DONE, PARTIALLY DONE, or NOT DONE

**If the criterion references tests:**
- Check if the test files exist
- Look for test function names that match
- Mark as: DONE, PARTIALLY DONE, or NOT DONE

**If the criterion is behavioral** (e.g. "result is identical to full reindex"):
- Note that it requires manual verification or test execution
- Mark as: NEEDS VERIFICATION

**If the criterion references documentation:**
- Check if the docs exist
- Mark as: DONE or NOT DONE

### 3. Check for Scope Drift

Compare the task's goals and implementation plan against what actually exists:
- Was the implementation done differently than planned? Note the differences.
- Were additional things done that aren't in the task? Note them.
- Has the relevant code changed since the task was written? Flag it.

### 4. Check Dependencies

Use `kb_graph` to see:
- Prerequisites: Are all blockers resolved?
- Children: Are all child tasks complete?
- Related specs: Does the implementation match the spec?

### 5. Present Review

Format as a structured assessment:

```markdown
## Review: [task title]

### Acceptance Criteria: X of Y Done

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | [criterion text] | DONE | [file/function that satisfies it] |
| 2 | [criterion text] | NOT DONE | [what's missing] |
| 3 | [criterion text] | NEEDS VERIFICATION | [how to verify] |

### Summary
- [Overall assessment: ready to close, needs work, or needs verification]
- [Specific remaining items if any]

### Recommendation
- [Close it / Update it / Keep working on it]
```

### 6. Offer Actions

Based on the review:
- If all criteria are met: "Ready to close. Run `/kb-close <slug>` to complete it."
- If some remain: "X criteria still need work. Want me to update the task with current findings?"
- If the task is outdated: "This task appears stale — the codebase has moved past it. Consider closing as superseded or updating the criteria."
