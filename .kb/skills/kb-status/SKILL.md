---
name: kb-status
description: Show workspace status and pending changes
---

Show the current workspace status including any uncommitted changes.

```bash
git-kb status --json           # Created/modified/deleted documents
git-kb diff                    # Detailed line-level diff
```

If MCP tools are available, prefer `kb_status` and `kb_diff` for structured JSON output.
