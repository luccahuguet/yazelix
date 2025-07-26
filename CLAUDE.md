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

