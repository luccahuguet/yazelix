# Yazelix baseline: Kache/Nushell enforcement

- Baseline commit: `9234c41b7d39ee029b1d01578808dbd1abd5bd4b`.
- Active install owner: the `lifeos_foundation_yzx` Nix profile output.
- Editable user input remains `~/.config/yazelix`.
- Generated runtime proof remains `~/.local/share/yazelix`.
- Active frontdoor remains `~/.nix-profile/bin/yzx`.
- The flake advertised `https://yazelix.cachix.org` and shipped a
  `Publish Nix Cache` workflow using `cachix/cachix-action`.
- All four GitHub workflows selected Bash as their run-step shell.
- The profile's `kache-rustc-wrapper` and its cargo-auditable shim were
  generated as POSIX shell scripts.
- The profile did not package `fxrun`, `fxrun-actions`, or `fxrun-dispatch`.
- The profile did not own a Nushell runner service or a fail-closed
  Kache/Nushell runtime policy check.

The owner-approved contract is stricter: Kache is the only persistent build
cache, Nushell is the only automated runner shell, and non-compliant runners
remain stopped.
