---
name: explore
description: Explore codebase with code intelligence and search across code and documents. Use when searching by concept, finding where functionality lives, or investigating unfamiliar code.
---

# /explore <query>

Find relevant code and documents. Use code intelligence first, then search for broader discovery.

## When to Use

- Exploring unfamiliar code
- Searching by concept rather than exact name
- Finding "where does X happen?"
- Investigating functionality across code and docs

## Steps

1. **Try code intelligence first (symbol search):**

   ```bash
   git-kb code symbols "<query>" --json
   ```

2. **For matched symbols, explore structure:**

   ```bash
   git-kb code callers "<matched-symbol>" --json
   git-kb code callees "<matched-symbol>" --json
   ```

3. **Search KB documents for related context:**

   ```bash
   git-kb search "<keywords>" --json
   ```

   Full-text search uses ranked keyword retrieval by default, so longer keyword bags can return strong partial matches. Add `--all-terms` only when every term is required.

4. **Use semantic search for broader discovery:**

   ```bash
   git-kb ai semantic "<query>" --json
   ```


If MCP tools are available, prefer `kb_symbols`, `kb_callers`, `kb_callees`, `kb_search`, and `kb_semantic` for structured JSON output. Never copy slugs or symbol IDs from table output.

## Output Format

```
## Exploring: "<query>"

### Code Matches

1. **src/services/auth.ts::login** (function) - Score: 0.89
   ```
   export async function login(credentials: Credentials): Promise<Token>
   ```
   Handles user authentication and token generation.

2. **src/middleware/auth.ts::validate** (function) - Score: 0.82
   ```
   export function validate(token: Token): boolean
   ```
   Validates JWT tokens.

### Document Matches

1. **specs/auth-spec** (spec) - Score: 0.85
   "Authentication System Specification"

2. **tasks/my-task** (task) - Score: 0.78
   "Auth Service Refactoring"

### Suggested Next Steps

- `/understand src/services/auth.ts` - Deep dive into auth module
- `/refactor-safety login` - Check impact before changes
- `git-kb show specs/auth-spec --json` - Read the auth spec
```

## Scope Options

When using semantic search:
- `scope: "all"` — Search both code and documents (default)
- `scope: "code"` — Search only code symbols
- `scope: "documents"` — Search only KB documents

## Prerequisites

For code intelligence, the index must be generated:
```bash
git-kb code index
```

For semantic search, embeddings must be generated:
```bash
git-kb ai embed
```

Document search uses FTS and does not require embeddings.
