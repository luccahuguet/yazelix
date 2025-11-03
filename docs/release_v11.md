# Yazelix v11 Release Notes

## Highlights
- **Devenv-powered launches**: Yazelix now starts via `devenv shell --impure`. Thanks to devenv’s SQLite cache, cold launches from desktop entries or `yzx launch` fall from ~4 seconds to about half a second, while full re-evaluation only happens after editing `yazelix.toml`.
- **TOML-first configuration**: `yazelix.toml` is auto-generated on first launch and is the single source of truth. Legacy `yazelix.nix` configs trigger friendly migration guidance, and the Home Manager module produces matching TOML.
- **Performance toolkit**: `yzx bench` and `yzx profile` help measure launch times and diagnose bottlenecks, and hash-based config detection keeps warm starts fast.

## Upgrade Notes
1. Ensure `devenv` is installed (`nix profile install github:cachix/devenv/latest`).
2. Launch Yazelix as usual (`yzx launch` or the desktop entry); the first run rebuilds the devenv cache, subsequent launches are near-instant.
3. If you previously customized `yazelix.nix`, copy those settings into `~/.config/yazelix/yazelix.toml` (Yazelix prints a warning when the legacy file is detected).

## Links
- [README Improvements](../README.md#improvements-of-v11-over-v10)
- [Full Changelog](./history.md#v11-devenv-launch-workflow-toml-first-config-and-faster-cold-starts)
