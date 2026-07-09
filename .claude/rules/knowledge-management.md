# Knowledge Management

Maintain the GitKB knowledge base as you work. Documents are your persistent memory across sessions.

## Before Starting Work

- Check `kb_board` or `git-kb board --json` to see what's active and what's blocked
- If you're about to do non-trivial work and no task exists for it, create one first
- Search before creating: `kb_search` with keywords to avoid duplicates
- Agents must use MCP or `--json` for slug/ID discovery. Never copy slugs, IDs, symbol IDs, or relationship targets from human-readable table/tree/board output because it may be truncated.

## While Working

- Add progress entries to the active task document as you make progress
- Include `[[tasks/...]]` wikilinks in git commit messages for related tasks:
  ```
  fix: resolve timeout issue

  Implements [[tasks/my-task]]
  ```
- When you discover bugs or issues, create incident documents — don't just fix and forget

## After Significant Work

- Update `context/overridable/active` to reflect what changed and what's next
- Check off completed acceptance criteria in task documents
- Add completion evidence (commit hashes, test results) before marking tasks done

## Commit Discipline

- **Always scope your commits** — pass `pathspecs` listing only the documents you modified:
  ```
  kb_commit with message: "Update task progress", pathspecs: ["tasks/my-task"]
  ```
  CLI equivalent: `git-kb commit -m "Update task progress" tasks/my-task`
- **Never commit the entire workspace** — bare `kb_commit` (no pathspecs) commits ALL workspace changes, including other agents' uncommitted edits
- **Check `kb_status` before committing** — if you see documents you didn't modify, exclude them from your pathspecs

## Document Lifecycle

- **Create first, implement second** — the document IS your plan
- **Update as you go** — don't wait until the end to document
- **Complete the body before changing status** — never mark "done" without evidence
- **Link everything** — tasks reference specs, incidents reference fixes, commits reference tasks
