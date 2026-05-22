# Beads Rust Issue Tracking

Yazelix uses `br` from [beads_rust](https://github.com/Dicklesworthstone/beads_rust) for local issue tracking.

The durable shared state is `.beads/issues.jsonl`. The local SQLite database is generated from that file and is intentionally ignored by git.

## Essential Commands

```bash
br ready
br list --status open
br show <issue-id>
br create "Title" -p 2 -t task
br update <issue-id> --claim
br close <issue-id> --reason "Completed"
br sync --status
br sync --flush-only
br sync --import-only --rebuild
```

## Storage Model

- `.beads/issues.jsonl` is tracked and reviewed like source code
- `.beads/beads.db` is local cache/state and can be regenerated
- `br` auto-imports and auto-flushes for normal commands
- Use `br sync --import-only --rebuild` after a fresh checkout or suspicious local database state
- Use `br sync --flush-only` before committing issue changes when you need an explicit JSONL refresh

Do not use the retired tracker workflow in this repository.
