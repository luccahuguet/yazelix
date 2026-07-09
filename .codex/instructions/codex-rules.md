# Codex Rules (Harmony)

## GitKB First

- For non-trivial work, create or update a GitKB document before coding.
- Use `git-kb` for tasks, context, and traceability.

## Code Intelligence

- Do not use grep to find callers/definitions.
- Use GitKB code tools (`kb_callers`, `git-kb code callers "<symbol>" --json`, etc.).
- Use `--json` for every GitKB command that supports it when discovering slugs, IDs, symbols, relationships, or task state.

## Commit Discipline

- Always scope `git-kb commit` with pathspecs.
- Include task slugs in commit messages (e.g., `[[tasks/<task-slug>]]`).
