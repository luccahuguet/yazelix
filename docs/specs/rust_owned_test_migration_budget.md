# Rust-Owned Test Migration Budget

## Summary

This document defines the next delete-first budget for moving large deterministic
Nushell test surfaces onto Rust-owned tests without losing the product
contracts those tests currently defend.

The immediate goal is not "rewrite every test in Rust." The goal is narrower:

- move deterministic tests that now defend Rust-owned logic onto Rust
- keep mixed shell/process behavior in Nu until a Rust port would still defend
  the real contract honestly
- delete or demote redundant Nu tests only after replacement coverage lands

## Scope

- large governed Nu test files under `nushell/scripts/dev`
- Rust-owned logic in `rust_core/yazelix_core`
- adjacent helper-heavy files where the core assertions are now deterministic
  and Rust-owned

Out of scope:

- sweep or visual harnesses
- maintainer update/release/issue-sync behavior
- shell/process-heavy integration flows that still need real command execution
- mass one-to-one rewrites that preserve old fixture noise without improving the
  owner boundary

## Bucket Classification

| File or bucket | Current role | Migration bucket | Why |
| --- | --- | --- | --- |
| `test_yzx_generated_configs.nu` generated-config parser/materialization assertions | default deterministic config/runtime checks | `rust_port_first` | many assertions now defend Rust-owned config normalize, materialization, render-plan, and stale/repair logic |
| `test_yzx_core_commands.nu` public command/report/control-plane assertions | default control-plane coverage | `rust_port_first` | many checks now defend Rust-owned metadata, config, doctor/status, install-ownership, and public control families |
| `test_yzx_workspace_commands.nu` launch/session/workspace coverage | mixed default integration suite | `mixed_keep_nu` | many strongest assertions still defend shell/bootstrap, Zellij handoff, session-local behavior, and CLI entrypoint orchestration |
| `test_yzx_popup_commands.nu` popup/front-door and wrapper behavior | mixed default integration suite | `mixed_keep_nu` | the deterministic popup fact helpers can move, but wrapper/process execution and Zellij interaction still belong in Nu |
| `test_yzx_yazi_commands.nu` sidebar/editor/Yazi integration | mixed default integration suite | `mixed_keep_nu` | still defends pane-orchestrator state and external Yazi adapter behavior |
| `test_yzx_doctor_commands.nu` public report surface | `rust_port_candidate` | public report computation is Rust-owned, but some CLI/prose/fix behavior still stays in Nu |
| `test_shell_managed_config_contracts.nu` shell setup/extern bridge behavior | maintainer/default-adjacent | `keep_in_nu` | still defends shell init, host-surface non-takeover, and extern bridge behavior |
| `test_yzx_maintainer.nu` release/update/profile harness | maintainer suite | `keep_in_nu` | shell, git, Nix, and repo workflow heavy |
| Rust `yzx_core_*` integration tests and `yazelix_core` unit tests | Rust-owned deterministic logic | `already_rust` | these are the surviving owner and should grow where they delete redundant Nu tests |

## First Migration Targets

### `yazelix-rdn7.4.5.2`

Move deterministic generated-config and materialization coverage first:

- parser failure/removed-surface normalization
- default bootstrap and Taplo-support behavior
- runtime materialization lifecycle and missing-artifact repair
- Yazi/Zellij/terminal/Helix render-plan and generated-file assertions

Preferred surviving owner:

- Rust unit/integration tests under `rust_core/yazelix_core`

Nu assertions that should stay after the first cut:

- shell/bootstrap entrypoint behavior
- current-terminal and desktop launch orchestration
- fixture-heavy cases that still depend on real external command wiring

### `yazelix-rdn7.4.5.3`

Move deterministic public command-surface coverage next:

- root help/version/extern metadata parity
- Rust-owned command families such as config, doctor/status JSON/report
  shaping, install ownership, keys, why/sponsor, and control helpers
- error-envelope behavior for Rust-owned `yzx_core` / `yzx_control` surfaces

Preferred surviving owner:

- Rust integration tests in `rust_core/yazelix_core/tests`

Nu assertions that should stay after the first cut:

- shell wrapper entrypoint behavior
- module bootstrap behavior
- mixed `yzx launch`, `yzx enter`, and current-terminal session UX

## Keep-In-Nu Rules

Keep a Nu test or cluster in Nu when at least one of these is still true:

- it defends shell/process execution rather than deterministic typed logic
- it depends on real Zellij/Yazi/editor interaction or wrapper-mode env wiring
- it verifies public CLI entrypoint behavior that still belongs to Nushell
- the Rust replacement would need to fake the real shell boundary more than the
  current Nu test already does

## Delete Or Demote Rules

After Rust replacement coverage lands:

- delete deterministic Nu tests that only duplicate stronger Rust assertions
- demote expensive but still useful integration checks to maintainer or sweep
  only when the default-lane contract stays defended elsewhere
- keep only the Nu tests that still defend shell/process/session/product UX

Do not delete:

- the only executable defense of a startup/session contract
- the last check that proves shell ownership boundaries remain intact
- the only public-entrypoint test for a still-Nu-owned family

## Verification Gate

Before a later deletion bead removes Nu tests from one of these files, it
should keep all of the relevant surviving verification green:

- `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- `nu nushell/scripts/dev/validate_rust_test_traceability.nu`
- `nix develop -c cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core`
- any surviving Nu component suites that still defend shell/process behavior

## Acceptance

1. The large governed Nu files are grouped into Rust-port, mixed, and keep-in-Nu buckets
2. The next Rust-port clusters are named concretely instead of as a blanket rewrite
3. The keep-in-Nu rules make shell/process stop conditions explicit
4. Later test-deletion beads can point here before removing redundant Nu coverage

## Traceability

- Bead: `yazelix-rdn7.4.5.1`
- Informed by: `docs/specs/governed_test_traceability_inventory.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
