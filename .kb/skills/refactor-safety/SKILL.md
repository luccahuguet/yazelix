---
name: refactor-safety
description: Safety check before refactoring — shows callers, callees, and impact analysis to assess blast radius before changing signatures, renaming symbols, or modifying types.
---

# /refactor-safety <symbol>

Safety check before modifying a function, method, or type. Shows all call sites that would need updates if you change the signature, plus impact analysis.

## When to Use

- Before changing a function signature
- Before renaming a public symbol
- Before modifying an interface or type's fields
- Before any change that could break callers

## Steps

1. **Find the symbol and its location:**

   ```bash
   git-kb code symbols "<symbol-name>" --json
   ```

2. **Get all callers (who would break if you change the signature):**

   ```bash
   git-kb code callers "<full-symbol-id>" --json
   ```

3. **Get callees (what this function depends on):**

   ```bash
   git-kb code callees "<full-symbol-id>" --json
   ```

4. **Run impact analysis on the file:**

   ```bash
   git-kb code impact "<file-path>" --json
   ```

5. **Check for related documents:**

   ```bash
   git-kb code refs "<full-symbol-id>" --json
   ```


If MCP tools are available, prefer `kb_symbols`, `kb_callers`, `kb_callees`, `kb_impact`, `kb_symbol_refs` for structured JSON output. Always use JSON output when copying symbol IDs; table output may be abbreviated for display.

## Output Format

```
## Refactor Safety Check: `symbolName()`

### Location
- File: src/services/auth.ts:45-67
- Kind: function
- Signature: `async function symbolName(param: Type): Promise<Result>`

### Direct Callers (5)
These will break if you change the signature:
  - src/routes/auth.ts:45 -> handleRequest()
  - src/routes/auth.ts:89 -> handleRefresh()
  - src/middleware/auth.ts:22 -> authenticate()
  - tests/auth.test.ts:15 -> testLogin()
  - tests/integration.test.ts:88 -> testFullFlow()

### Transitive Impact
- 12 symbols transitively depend on this
- Affects 4 files

### What This Calls
  - src/db/users.ts::findUser()
  - src/crypto/tokens.ts::generateToken()

### Related Documents
  - tasks/my-task (Related Task)

### Recommendation

[LOW|MEDIUM|HIGH] impact refactor:
- LOW: 0-2 callers, all in same module
- MEDIUM: 3-10 callers, multiple modules
- HIGH: 10+ callers or public API surface

Files that need updates if signature changes:
1. src/routes/auth.ts
2. src/middleware/auth.ts
3. tests/auth.test.ts
4. tests/integration.test.ts
```

## Example

**Input:** `/refactor-safety login`

**Output:**
```
## Refactor Safety Check: `login()`

### Location
- File: src/services/auth.ts:45-67
- Kind: function

### Direct Callers (3)
These will break if you change the signature:
  - src/routes/auth.ts:45 -> handleRequest()
  - src/routes/auth.ts:89 -> handleRefresh()
  - src/middleware/auth.ts:22 -> authenticate()

### Transitive Impact
- 8 symbols transitively depend on this
- Affects 3 files

### Recommendation

MEDIUM impact refactor.

Files that need updates if signature changes:
1. src/routes/auth.ts
2. src/middleware/auth.ts
```

## Prerequisites

Code must be indexed first:
```bash
git-kb code index
```
