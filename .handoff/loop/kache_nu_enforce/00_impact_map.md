# Yazelix impact map: Kache/Nushell enforcement

## Owners and downstream surfaces

| Owner | Change | Downstream consumer |
| --- | --- | --- |
| `flake.nix` inputs | Pin `flexnetos_runner_source` | Nix lock and local `--override-input` verification |
| `packaging/flexnetos_runner_release.nix` | Build the three runner binaries | `lifeos_foundation_yzx` profile `bin/` and `toolbin/` |
| `packaging/kache_rustc_wrapper.rs` | Replace generated shell wrappers with ELF executables | Cargo `RUSTC_WRAPPER`/`CARGO_BUILD_RUSTC_WRAPPER` |
| `nushell/runner/*.nu` | Own volatile runner lifecycle and runtime policy | profile commands and systemd user unit |
| `systemd/user/flexnetos_runner@.service` | Point services only at profile-owned Nu/Kache/runner paths | `flexnetos_runner@01` and `@02` instances after explicit activation |
| `.github/workflows/*.yml` | Remove remote cache publication and select Nu for run steps | GitHub-hosted CI |
| `checks/cache_shell_policy.nu` | Reject cache/shell drift | `nix flake check` and focused policy builds |

## Risk

- High: advancing the runner pin before the runner repository removes its
  generic cache surface would ship a non-compliant CLI. The published
  `0398173` pin is build-only baseline and must not be the final activated pin.
- High: enabling the service before profile build and policy checks pass could
  recreate runner state. This slice does not activate or restart services.
- Medium: GitHub-hosted jobs require a Nu setup action before the first run
  step. Workflow ordering is part of the policy check.
- Low: Kache wrapper behavior is preserved as exact argv forwarding through a
  Rust `exec`, without a shell process.

No existing Rust product symbol is edited. New Rust is a standalone wrapper
compiled by Nix, so the source-level blast radius is limited to the profile
executable graph above.
