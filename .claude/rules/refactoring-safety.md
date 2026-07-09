# Refactoring Safety

Before making changes that could break callers, always check the blast radius using code intelligence tools.

## Required Checks

| Before doing this | Run this first |
|-------------------|----------------|
| Changing a function signature | `kb_callers` to find all call sites that need updating |
| Renaming a public symbol | `kb_callers` to find all references |
| Modifying an interface or type's fields | `kb_callers` on the type to find all usages |
| Deleting a symbol | `kb_dead_code` or `kb_callers` to confirm zero callers |
| Changing a file's public API | `kb_impact` to see transitive dependents |

## Risk Assessment

After checking callers, assess the risk level:

| Callers | Risk | Action |
|---------|------|--------|
| 0-2 callers, same module | **Low** | Proceed with the change |
| 3-10 callers, multiple modules | **Medium** | Update all call sites carefully, run tests |
| 10+ callers or public API | **High** | Confirm with user before proceeding |

## Workflow

1. **Check callers**: `kb_callers with symbol: "<symbol>"`
2. **Check impact**: `kb_impact with file_path: "<file>"`
3. **Plan updates**: List all files that need changes
4. **Update leaf callers first**: Start with callers that have no further dependents
5. **Update tests**: Tests are callers too — update them alongside production code
6. **Verify**: Run tests after all changes are complete

## Example

Before renaming `processEvent()` to `handleEvent()`:

```text
kb_callers with symbol: "src/events.ts::processEvent"
```

If this returns 5 callers across 3 files, that's a **medium** risk refactor. Update all 5 call sites, then verify with tests.
