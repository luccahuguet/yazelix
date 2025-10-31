# Yazelix Development Insights

This document captures important debugging insights, architectural lessons, and "gotchas" discovered during Yazelix development.

---

## The Nix Environment ARG_MAX Problem (October 2025)

### Problem Description

When calling `yzx launch --here` from within a `yzx env` shell (or even inside an existing Yazelix session), the command would:
1. Hang indefinitely (no output, no progress)
2. After multiple Ctrl+C interrupts, show random different errors each time:
   - "Argument list too long (os error 7)"
   - "Command `main` not found"
   - Random tool help messages (e.g., `mise --help`)
   - File not found errors with garbled paths

This "casino effect" of getting different random errors was the key insight that revealed the underlying issue.

### Root Cause

**Spawning external processes from within Nix environments passes massive environment variables that exceed kernel limits.**

When you're inside a Nix shell (`nix develop`), the environment contains:
- Store paths for all dependencies
- Build inputs and outputs
- Compiler flags (NIX_CFLAGS_COMPILE, etc.)
- Path entries for every tool
- Configuration variables

These can easily total **several megabytes** of data.

When Nushell spawns an external command with `^nu` or `^bash`, it passes:
- `argv[]` - The command arguments
- `envp[]` - The entire environment

The kernel has a combined limit (`ARG_MAX`) of ~2MB for `argv + envp`. In Nix environments, `envp` alone can exceed this, causing:
- Error 7: "Argument list too long"
- Hanging as the kernel struggles with the spawn
- Corrupted data leading to random command execution
- Unpredictable behavior as different parts of the environment get truncated

### Failed Solutions

1. **Using `env -i` to clean environment**: Still spawns with massive env before cleaning
2. **Piping commands to bash stdin**: Bash process itself spawned with huge env
3. **Temporary script files**: File created, but bash execution still hits limit
4. **Nested Nushell scoping tricks**: Module scoping issues prevented function calls

### The Solution

**Run scripts directly without spawning external processes.**

Instead of:
```nushell
^nu /path/to/script.nu args...  # Spawns process, passes environment
```

Use:
```nushell
nu /path/to/script.nu args...   # Direct execution in current interpreter
```

Or better yet, for modules:
```nushell
use module.nu function
function args...                # Function call, no spawning
```

The key insight: Look at what works (`yzx versions`, `yzx doctor`) and copy that pattern:

```nushell
export def "yzx versions" [] {
    nu ~/.config/yazelix/nushell/scripts/utils/version_info.nu
}
```

Simple, direct, no spawning, no environment passing.

### Implementation

The first fix in 2024 reintroduced a standalone launcher command that invoked `start_yazelix.nu` directly. In October 2025 we carried that idea forward: `yzx launch --here` now imports the shared `start_yazelix_session` helper and runs it inside the current Nushell engine. No external `^nu` spawn, no recursive `nix develop`, same fast path whether you enter from a vanilla shell or an active Yazelix session.

### Commands After Fix

- **`yzx launch --here`** - Start Yazelix in current terminal (direct call, no spawning)
- **`yzx launch`** - Launch Yazelix in a new terminal window
- **`yzx env`** - Load tools without UI

### Lessons Learned

1. **Simplicity wins**: The simplest solution (direct script execution) worked best
2. **Copy what works**: When stuck, find similar working code and use that pattern
3. **Random errors = environment issue**: Different errors each run suggests data corruption/truncation
4. **Kernel limits are real**: ARG_MAX isn't just theoretical, Nix environments hit it
5. **Fail fast with clear errors**: Better than mysterious hangs and random behavior
6. **Test in real conditions**: `yzx env` → `yzx launch --here` workflow revealed the issue

### Prevention

When designing new commands:
- ✅ Prefer direct function calls or script execution
- ✅ Avoid spawning processes when already in Nix shells
- ✅ Test workflows that transition between env modes
- ✅ Use `nu script.nu` instead of `^nu script.nu`
- ❌ Don't spawn external processes unnecessarily
- ❌ Don't assume environment size is bounded

### Testing

Verified with comprehensive sweep tests:
- Shell sweep: 8/8 passed (nu, bash, fish, zsh + variations)
- Terminal sweep: 5/5 passed (ghostty, wezterm, kitty, alacritty, foot)
- Total: 13/13 tests passed

### References

- Kernel ARG_MAX limit: `getconf ARG_MAX` (typically 2097152 bytes on Linux)
- Nix environment inspection: `env | wc -c` in `nix develop` shell
- Related commits:
  - `64e09a8` - Reintroduced the standalone launcher command
  - `2f1a6d3` - "refactor: inline start_yazelix into yzx launch --here"

---

## Development Philosophy Reinforced

This debugging session reinforced several key principles:

### Reason from First Principles
When faced with complex problems, analyze fundamental constraints:
- What is the actual error (not what it seems to be)?
- What are the system limits involved?
- What is the simplest solution that respects these constraints?

### Avoid Over-Engineering
The solution wasn't:
- Complex environment filtering
- Sophisticated process management
- Elaborate scoping workarounds

It was: **Run the script directly, like other commands already do.**

### User Experience Matters
Random errors create terrible UX. When debugging:
- Pay attention to "casino" patterns (different errors each time)
- This usually indicates data corruption or resource exhaustion
- Fix the root cause, not the symptoms

---

*Last updated: October 25, 2025*
