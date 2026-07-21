# Architect plan: Yazelix install-owner slice

VERDICT: GO

1. Delete Cachix configuration and its cache-publication workflow.
2. Make every remaining first-party workflow run step use Nushell.
3. Pin `flexnetos_runner_source`, package its three binaries, and allow a local
   source override for coordinated verification.
4. Replace Kache's generated POSIX wrappers with one standalone Rust/ELF
   dispatcher installed at both required paths.
5. Package profile-owned Nushell runner service and policy commands plus the
   profile-owned systemd user unit. Keep work/home/tool state volatile under
   `$XDG_RUNTIME_DIR`; persist only `KACHE_CACHE_DIR`.
6. Add a fail-closed source/runtime policy check and wire it into flake checks.
7. Update the changelog, development contract, and LOC scorecard.
8. Evaluate/build only. Do not activate the profile, enable runners, edit
   generated profile runtime, push, or bypass the manual dogfood gate.

Final activation is blocked until the runner repository publishes its
Kache-only/Nushell-only commit and this flake pin advances to that immutable
revision.
