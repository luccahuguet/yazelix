---
name: understand
description: Understand a file or symbol's structure and dependencies using code intelligence tools. Use before modifying unfamiliar code or investigating how a module works.
---

# /understand <file|symbol>

Analyze a file or symbol using code intelligence tools to understand its structure and dependencies.

## When to Use

- Before modifying unfamiliar code
- When trying to understand how a module works
- When investigating where functionality lives

## Steps

### For a File Path

When the argument looks like a file path (e.g., `src/services/auth.ts`):

1. **List symbols in the file:**
   ```bash
   git-kb code symbols --file "src/services/auth.ts" --json
   ```

2. **For key functions/methods, show their connections:**
   ```bash
   git-kb code callers "src/services/auth.ts::login" --json
   git-kb code callees "src/services/auth.ts::login" --json
   ```

3. **Check if any documents reference this code:**
   ```bash
   git-kb code refs "src/services/auth.ts::login" --json
   ```

### For a Symbol Name

When the argument contains `::` or looks like a function name:

1. **Find the symbol:**
   ```bash
   git-kb code symbols "login" --json
   ```

2. **Show callers (who calls this):**
   ```bash
   git-kb code callers "src/services/auth.ts::login" --json
   ```

3. **Show callees (what this calls):**
   ```bash
   git-kb code callees "src/services/auth.ts::login" --json
   ```

4. **Find related documents:**
   ```bash
   git-kb code refs "src/services/auth.ts::login" --json
   ```

If MCP tools are available, prefer `kb_symbols`, `kb_callers`, `kb_callees`, `kb_symbol_refs` for structured JSON output. Never copy symbol IDs from non-JSON table output.

## Output Format

Provide a summary with:

1. **Symbol Overview**: Kind, signature, location
2. **Who Calls It**: Direct callers (up to 10)
3. **What It Calls**: Direct callees (up to 10)
4. **Related Documents**: Any KB docs that reference this code

## Example

**Input:** `/understand src/services/auth.ts`

**Output:**
```
## src/services/auth.ts - Authentication Module

### Symbols (5)
- `login(credentials: Credentials): Promise<Token>` (function)
- `logout(token: Token): Promise<void>` (function)
- `validateToken(token: Token): boolean` (function)
- `AuthConfig` (interface)
- `AuthError` (class)

### Key Function: login()

**Callers (3):**
- src/routes/auth.ts:45 -> handleRequest()
- src/routes/auth.ts:89 -> handleRefresh()
- src/middleware/auth.ts:22 -> authenticate()

**Callees (2):**
- src/db/users.ts::findUser()
- src/crypto/tokens.ts::generateToken()

### Related Documents
- tasks/my-task (Auth Service Refactoring)
```

## Prerequisites

Code must be indexed first:
```bash
git-kb code index
```
