---
name: kb-search
description: Search the knowledge base and present results with suggested actions
---

Search across all documents and present results with context and next steps.

**Input:** `$ARGUMENTS`

The full input is the search query.

Examples:
- `/kb-search sqlite migration`
- `/kb-search sync protocol`
- `/kb-search frontmatter`

## Steps

### 1. Search

Use `kb_search` with the query from `$ARGUMENTS`.

Search defaults to ranked keyword retrieval: multi-term queries may return documents that match the strongest subset of terms, with better term coverage ranked higher. Use `all_terms: true` only when every query term must be present.

If no results, try:
- Broader terms (drop adjectives, try synonyms)
- Partial matches
- Report "No results found for '[query]'. Try broader terms."

### 2. Present Results

For each result, show:
- **Slug** and **title**
- **Type** and **status**
- **Snippet**: The matching excerpt (if available from search results)
- **Tags** (if any)

Format as a numbered list for easy reference:

```text
1. [tasks/my-project-3] "SQLite Index" (task, completed)
   ...modular SQLite store implementation with FTS5...

2. [specs/data-model] "Data Model" (spec, draft)
   ...SQLite schema for documents, relationships...
```

### 3. Offer Actions

After presenting results, suggest relevant next steps:

- **"Show details"**: "Use `kb_show` to read the full document, or `/kb-review <slug>` to check it against its acceptance criteria"
- **"Find related"**: "Use `kb_graph` to see how a document connects to others"
- **"Start work"**: If a task result is in draft/backlog: "Run `/kb-start <slug>` to begin working on it"
- **"Extend"**: If the user was about to create something similar: "Consider extending [slug] instead of creating a new document"
