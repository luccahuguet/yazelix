# Claude Code Context for Yazelix

## File Naming Conventions

**IMPORTANT**: Yazelix uses underscores (`_`) for ALL file and directory names, never hyphens (`-`).

Examples:
- ✅ `home_manager/` 
- ❌ `home-manager/`
- ✅ `yazelix_default.nix`
- ❌ `yazelix-default.nix`
- ✅ `start_yazelix.nu`
- ❌ `start-yazelix.nu`

This convention is used consistently throughout:
- Directory names: `configs/terminal_emulators/`, `nushell/scripts/core/`
- File names: `yazelix_default.nix`, `start_yazelix.nu`, `launch_yazelix.nu`
- Script names: All Nushell scripts use underscores

When creating new files or directories, always use underscores to maintain consistency with the existing codebase.

## Project Structure Notes

- Yazelix is a development environment (`devShell`) not a traditional package
- Configuration is handled via `yazelix.nix` (user) and `yazelix_default.nix` (template)
- All paths reference `~/.config/yazelix/` as the base directory
- Scripts are organized in `nushell/scripts/` with subdirectories using underscores

## Documentation and User Guidance Principles

When documenting limitations or potential issues:
1. **Positive Direction Instead of Warnings** - Instead of telling users "don't do this", provide clear guidance on "do this instead"
2. Point users to specific files they should edit
3. Explain the recommended approach first, then mention alternatives
4. Focus on the workflow that works best rather than listing what doesn't work

