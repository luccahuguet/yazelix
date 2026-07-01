# GitKB Process Discipline

## Session Start

```bash
git-kb context --compact --code-refs
git-kb board --json
```

Use `--json` for every GitKB command that supports it when discovering slugs, IDs, symbols, relationships, or task state. Human-readable output is for display and may be truncated.

## During Work

- Log progress in the task document.
- Keep acceptance criteria updated.

## Completion

- Add completion evidence before marking a task complete.
- Only then set status to `completed`.
