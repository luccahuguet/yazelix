---
name: kb-start
description: Start working on a task — load context, checkout, set active, and understand scope
---

Begin working on a task. This is the "start of work" ritual.

**Input:** `$ARGUMENTS`

The argument should be a task slug (e.g. `tasks/my-task`) or a search term to find a task.

## Steps

### 1. Find the Task

If the argument looks like a slug (contains `/`), load it directly with `kb_show`.

If it's a search term, use `kb_list` with `type: "task"` and filter by matching title. If multiple matches, show them and ask the user to pick one.

If no argument is provided, show the board (`kb_board` or `git-kb board --json`) and ask the user which task to start.

### 2. Load Project Context

Use `kb_context` with the task slug and `include_code_refs: true` to load the full context bundle:
- Project context (architecture, patterns, active state)
- The task document with full content
- Recent commit history
- Code references enriched with symbol data (file paths, line ranges, signatures)

```text
kb_context with task: "<task-slug>", include_code_refs: true
```

### 2b. Load Smart Code Context (if task has code references)

If the task references code files or symbols (`[[code:...]]` wikilinks), use `kb_smart_context` to assemble task-relevant code context with call graph traversal:

```text
kb_smart_context with task: "<task-slug>"
```

This automatically:
- Extracts code signals from the task (file paths, symbol references, keywords)
- Resolves symbols from the code index
- Traverses callers and callees (configurable depth, default 2)
- Packs results within a token budget (default 8000)
- Returns ranked items with provenance (direct reference, caller, callee, semantic match)

Skip this step if the task is documentation-only or has no code references.

### 3. Understand the Task

Present a clear summary:
- **What**: Overview and goals
- **Why**: What prompted this work
- **Scope**: Acceptance criteria checklist (how many items, how many already checked)
- **Dependencies**: What this blocks, what blocks this (`kb_graph`)
- **Related**: Linked specs, parent tasks, incidents

### 4. Understand the Code (if applicable)

If the task references specific files, modules, or functions:
- Use `kb_symbols` to list relevant symbols
- Use `kb_impact` to understand blast radius of proposed changes
- Summarize: "This task will likely touch X files affecting Y callers"

### 5. Set Status to Active

If the task is currently `draft` or `backlog`, set it to `active`:

```text
kb_set with slug: "<task-slug>", status: "active"
```

If it's already `active`, note that and continue.
If it's `completed`, warn the user and ask if they want to reopen it.

### 6. Checkout to Workspace

```text
kb_checkout with slugs: ["<task-slug>"]
```

### 7. Present Working Context

Show the user:
- Task title and slug
- Acceptance criteria (as a checklist they can reference)
- Key files/modules to look at
- Suggested first step based on the task's implementation section (if any)

End with: "Task is active and checked out. Ready to work."
