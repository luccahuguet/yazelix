---
name: kb-create
description: Create a new KB document by type with discovery, context-aware content, and auto-incrementing slug
---

Create a new GitKB document based on the user's input.

**Input:** `$ARGUMENTS`

The input format is: `<type> [slug] <description>`

- **type** (required): `task`, `incident`, `spec`, `epic`, `note`, or `context`
- **slug** (optional): A path containing `/`. Three forms:
  - **Complete slug** (e.g. `tasks/my-project-28`) — use as-is
  - **Slug prefix** ending with `-` (e.g. `tasks/my-project-`) — auto-append next number
  - **Slug base** without trailing `-` (e.g. `tasks/my-project`) — auto-append `-{next number}`
- **description** (required): Natural language description of the document

Examples:
- `/kb-create task tasks/my-project-28 Add sort flag` — exact slug
- `/kb-create task tasks/my-project- Add sort flag` — auto-increments to next available
- `/kb-create task tasks/my-project Add sort flag` — same as above
- `/kb-create task Add sort flag to git-kb list` — fully auto-generates slug
- `/kb-create spec specs/federation-protocol Design the sync protocol`
- `/kb-create incident The daemon crashes on startup with empty KB`

## Steps

### 1. Parse Input

Extract the **type** (first word), optional **slug** (second word if it contains `/`), and **description** (remainder).

If no type is provided, ask the user what type of document to create.
If no description is provided, ask the user to describe what the document is about.

### 2. Discover: Search Before Create

**This step is critical.** Before creating anything, search for existing related work:

1. **Search for duplicates:** `kb_search` with keywords from the description. If a document already covers this topic, tell the user and ask whether to extend the existing doc or create a new one.

2. **Check the board:** use `kb_board`, or `git-kb board --json` if MCP is unavailable, to understand what's active, what's blocked, and where this new document fits in the current workstream.

3. **Find related documents:** use `kb_search`/`kb_graph`, or `git-kb search "<query>" --json`/`git-kb graph <slug> --json` if MCP is unavailable. Note any documents that should be linked from the new one (parent tasks, related specs, prior incidents with similar symptoms).

Run these searches in parallel to save time. If you find a clear duplicate, stop and ask the user before proceeding.

### 3. Load Project Context

Use `kb_context` to understand the project's current state. This ensures the document you create:
- Uses the right terminology and conventions
- References the correct architecture and patterns
- Fits into the current active workstream
- Doesn't contradict existing decisions

If the description mentions specific code (files, functions, modules), use `kb_symbols` or `kb_impact` to understand the relevant code and write more precise goals and acceptance criteria.

### 4. Determine Slug

**A. Complete slug** — the second word contains `/` and is a fully-formed slug. Use it as-is.
  - For **non-numbered types** (spec, note, context): any slug with `/` is complete (e.g. `specs/federation-protocol`, `notes/api-design`).
  - For **numbered types** (task, epic, incident): complete when the last path segment ends with a digit (e.g. `tasks/my-project-28`, `incidents/inc-001-auth-timeout`).

**B. Slug prefix** — only for numbered types (task, epic, incident). The second word contains `/` and either ends with `-` or is a base without a trailing number (e.g. `tasks/my-project-` or `tasks/my-project`). Auto-increment:

1. Normalize: strip any trailing `-` to get the base (e.g. `tasks/my-project`)
2. Run `git-kb list --json` and find all slugs matching `{base}-{N}` (use the appropriate `--type` for the document being created — e.g. `--type task` for tasks, `--type incident` for incidents). Never derive slugs from table output.
3. Extract the highest `N`, increment by 1
4. Final slug: `{base}-{N+1}`

**C. No slug provided** — fully auto-generate:

| Type | Method |
|------|--------|
| task | Run `git-kb list --type task --json`, detect naming pattern (e.g. `tasks/my-project-{N}`), increment highest. If no tasks exist, ask the user for a prefix. |
| epic | Run `git-kb list --type epic --json`, detect naming pattern, increment highest. If no epics exist, ask the user for a prefix. |
| incident | Run `git-kb list --type incident --json`, pattern `incidents/inc-{NNN}-{short-slug}`, increment. Derive short slug from description. |
| spec | `specs/{short-slug}` derived from description (lowercase, hyphens, 2-5 words) |
| note | `notes/{short-slug}` derived from description |
| context | Ask user for stability level (`immutable`/`extensible`/`overridable`), then `context/{level}/{short-slug}` |

### 5. Generate Title

Create a concise, descriptive title from the user's description. Use title case. Keep it under 80 characters.

### 6. Generate Content

Build the document body based on type. Write **substantive, project-aware content** — not placeholder text. Use what you learned from discovery and context loading to write content that a cold-starting agent could pick up and make progress on.

**Task / Epic:**
```markdown
## Overview

[1-2 paragraphs explaining WHAT this task is and WHY it exists.
Reference the current project state. Link to related documents
with [[wikilinks]]. Mention what prompted this work.]

## Goals

- [Concrete goal derived from description and project context]
- [Each goal should be independently verifiable]

## Implementation

[Sketch the approach if obvious from context. Reference specific
files, modules, or functions discovered via code intelligence.
If the approach isn't clear yet, say so — that's fine for a draft.]

## Acceptance Criteria

- [ ] [Specific, verifiable criterion — not vague]
- [ ] [Another criterion — think "how would I test this?"]
- [ ] [Tests pass / no regressions]

## Spec References

- [[related-spec-or-task]] — [why it's related]
```

**Incident:**
```markdown
## Overview

[What happened. When it was discovered. How it was noticed.]

## Symptoms

- [Observable behavior — error messages, incorrect output, crashes]
- [Include exact error text if available]

## Impact

[Who/what is affected. Severity. Is there a workaround?]

## Investigation

[Initial findings. What's been checked so far.
Hypotheses for root cause.]

## Related

- [[related-docs]] — [prior incidents, relevant specs]
```

**Spec:**
```markdown
## Overview

[What this spec covers and why it's needed. Reference the
motivating task or problem.]

## Goals

- [What the design must achieve]
- [Constraints it must satisfy]

## Design

[Proposed approach. Include diagrams (Mermaid) if helpful.
Reference existing architecture from context docs.]

## Alternatives Considered

[Other approaches and why they were rejected.
Document the reasoning so it doesn't get re-litigated.]

## Open Questions

- [Unresolved decisions that need input]
```

**Note:**
```markdown
[Freeform content based on description. Notes don't have a rigid
structure, but should still be clear and useful to a future reader.]
```

### 7. Quality Self-Check

Before creating, verify the document passes these checks:

- [ ] Someone with zero project context can understand the goal
- [ ] Acceptance criteria are specific and verifiable (not vague)
- [ ] Links exist to related documents (parent task, spec, incident)
- [ ] Key decisions or assumptions are stated explicitly
- [ ] No placeholder text remains — all content is substantive

If any check fails, improve the content before proceeding.

### 8. Create and Commit

1. Use `kb_create` with the computed `slug`, `title`, `type`, `status: "draft"`, generated `content`, and inferred `tags`.
2. Use `kb_commit` with message: `"Create {type}: {title}"`.
3. Show the user:
   - The created document slug
   - A brief summary of what was created
   - Any related documents found during discovery (as suggested links)

## Tag Inference

Infer tags from the description and project context:
- Technology names mentioned (e.g. `sqlite`, `rust`, `terraform`)
- Area of codebase (e.g. `store`, `sync`, `cli`, `embedding`)
- Work category (e.g. `bug`, `feature`, `refactor`, `dx`, `perf`)
- Phase or milestone if obvious (e.g. `phase0`, `phase2`)

## Relationship Awareness

- If the user mentions a parent task, add `parent: {slug}` to frontmatter
- If this task blocks or is blocked by another, mention it in the body with `[[wikilinks]]`
- If this implements a spec, reference it in "Spec References"
- If this resolves an incident, add `resolves: {slug}` to frontmatter
