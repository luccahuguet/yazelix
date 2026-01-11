# Claude Code Context for Yazelix

## File Naming Conventions

**IMPORTANT**: Yazelix uses underscores (`_`) for ALL file and directory names, never hyphens (`-`).

Examples:
- ✅ `home_manager/` 
- ❌ `home-manager/`
- ✅ `yazelix_default.toml`
- ❌ `yazelix-default.nix`
- ✅ `start_yazelix.nu`
- ❌ `start-yazelix.nu`

This convention is used consistently throughout:
- Directory names: `configs/terminal_emulators/`, `nushell/scripts/core/`
- File names: `yazelix_default.toml`, `start_yazelix.nu`, `launch_yazelix.nu`
- Script names: All Nushell scripts use underscores

When creating new files or directories, always use underscores to maintain consistency with the existing codebase.

## Project Structure Notes

- Yazelix is a development environment (`devShell`) not a traditional package
- Configuration is handled via `yazelix.toml` (user) and `yazelix_default.toml` (template)
- All paths reference `~/.config/yazelix/` as the base directory
- Scripts are organized in `nushell/scripts/` with subdirectories using underscores

## Configuration Management Principles

### Synchronization Requirements
1. **Always sync Home Manager module with default config** - When changing `yazelix_default.toml`, update `home_manager/module.nix` to maintain identical options and defaults
2. **Verify both configuration paths work** - Test changes through both direct config files and Home Manager integration

### Code Robustness Requirements  
1. **Avoid fallbacks** - Fallback behavior can mask underlying issues and lead to unpredictable behavior across different environments
2. **Fail fast with clear errors** - When something is wrong, provide explicit error messages rather than degraded functionality
3. **Universal robustness** - Yazelix must work reliably for all users, not just maintainers who can manually fix issues
4. **Avoid redundant code** - Focus on elegant, concise code when possible; eliminate duplication and unnecessary complexity

### Error Handling Philosophy
- **No silent failures** - Every error should be visible and actionable
- **Environment independence** - Code should work regardless of host system quirks
- **Consistent behavior** - Same input should produce same output across all user environments

## Nushell Development Notes

### 🚨 MOST CRITICAL RULE: Escaping Parentheses in String Interpolation

**Nushell interprets unescaped parentheses `()` in string interpolation as command substitution!**

**The ONLY correct syntax is:** `\(` and `\)` (single backslash)
- ❌ **NEVER use:** `\\(` and `\\)` (double backslash) - this will fail!
- ❌ **NEVER use:** `()` (no backslash) - this executes commands!

**Examples:**
- ✅ Correct: `$"Checking pane \(editor\)"`
- ❌ Wrong: `$"Checking pane \\(editor\\)"` → tries to execute command `editor\\`
- ❌ Wrong: `$"Checking pane (editor)"` → tries to execute command `editor`
- ✅ Correct: `log_to_file $log "Sent Escape \(27\) to enter normal mode"`
- ❌ Wrong: `log_to_file $log "Sent Escape \\(27\\) to enter normal mode"` → fails

**If you get "Command X not found" errors in string interpolation, check for incorrect parentheses escaping first!**

## Python Notes

- Use `python3` explicitly in all commands, scripts, and documentation.
- Avoid `python` as it can point to Python 2 on some systems or be unset.
- Prefer fenced code blocks with `bash` and examples like:
  ```bash
  python3 -m venv .venv
  python3 script.py
  ```

## Planning and Decision Making

**ALWAYS PLAN FIRST** - Before taking significant actions (like git commits, major changes, or file operations), explicitly discuss the approach and get user approval. This includes:
- Git operations: What files to commit, whether to include binaries, commit message strategy
- File changes: Whether to edit, create, or delete files
- Tool selection: Which approach to use when multiple options exist
- Architecture decisions: How to structure or integrate new features

**REASON FROM FIRST PRINCIPLES** - When faced with design decisions or trade-offs, analyze the fundamental requirements and constraints rather than following conventions blindly. Consider:
- What is the core problem being solved?
- What are the fundamental constraints (safety, user expectations, system behavior)?
- What are the actual risks vs. perceived risks?
- What does the user explicitly want vs. what they implicitly expect?
- How do similar tools handle this situation and why?

Example: When deciding whether `yzx doctor --fix` should auto-fix user runtime conflicts, reason through user intent (explicit `--fix` flag), safety (backup approach), and the nature of the problem (system debris vs. user data).

## Yazelix Versioning

**Yazelix versioning:**
- Follow the current project versioning scheme used in tags/releases
- When referencing versions in documentation or migration notes, only use actual version numbers that exist

## Documentation and User Guidance Principles

When documenting limitations or potential issues:
1. **Positive Direction Instead of Warnings** - Instead of telling users "don't do this", provide clear guidance on "do this instead"
2. Point users to specific files they should edit
3. Explain the recommended approach first, then mention alternatives
4. Focus on the workflow that works best rather than listing what doesn't work
