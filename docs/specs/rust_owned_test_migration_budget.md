# Rust-Owned Test Migration Budget

## Summary

This document defines the next delete-first budget for moving large deterministic
Nushell test surfaces onto Rust-owned tests without losing the product
contracts those tests currently defend.

The final governed-test end state is `0` surviving Nu test LOC. The migration
rule is selective:

- port strong tests that defend real contracts, regressions, or invariants onto
  Rust-owned nextest suites
- delete weak, duplicated, or trivia-heavy tests instead of porting them
- treat shell-heavy but strong Nu tests as temporary blockers on Rust harness
  work, not as allowed long-term survivors

## Scope

- large governed Nu test files under `nushell/scripts/dev`
- Rust-owned logic in `rust_core/yazelix_core`
- adjacent helper-heavy files where the core assertions are now deterministic
  and Rust-owned
- the Rust test-harness work needed to retire strong Nu tests cleanly

Out of scope:

- maintainer update/release/issue-sync behavior
- shell/process-heavy integration flows that still need real command execution
- mass one-to-one rewrites that preserve old fixture noise without improving the
  owner boundary

Current measured governed Nu test surface: `13,079` LOC.

## Bucket Classification

| File or bucket | Current role | Migration bucket | Why |
| --- | --- | --- | --- |
| `test_yzx_generated_configs.nu` | deterministic config/materialization coverage | `rust_port_landed_then_split_more` | the strongest generated-config and materialization assertions now defend Rust-owned logic; keep deleting the residual Nu duplicates |
| `test_yzx_core_commands.nu` | deterministic public command/control-plane coverage | `rust_port_landed_then_split_more` | the strongest public command and report assertions now belong under Rust-owned nextest suites |
| `test_yzx_workspace_commands.nu`, `test_zellij_plugin_contracts.nu` | workspace/session/plugin contracts | `rust_port_after_harness` | strong assertions remain, but they need shared fixture/process helpers before they can leave Nu honestly |
| `test_yzx_popup_commands.nu`, `test_yzx_yazi_commands.nu`, `test_yzx_doctor_commands.nu`, `test_yzx_helix_doctor_contracts.nu` | mixed popup/Yazi/doctor/editor flows | `rust_port_after_harness` | the good assertions are real contracts, but the current Nu files still mix them with wrapper-heavy execution noise |
| `test_shell_managed_config_contracts.nu` | shell-managed config and extern-bridge contracts | `rust_port_after_harness` | the strong assertions should move once the Rust harness can execute the real shell boundary; they are not allowlisted as permanent Nu tests |
| `test_yzx_maintainer.nu`, `test_config_sweep.nu`, upgrade-summary and stale-config e2e files | maintainer/sweep/e2e coverage | `delete_if_weak_or_replace_elsewhere` | keep only the strong contracts; do not preserve broad Nu omnibus files by default |
| `test_yzx_commands.nu` | command-surface/trivial routing inventory | `delete_if_weak` | this class is the easiest place to delete low-value command-discovery checks instead of porting them |
| Rust `yazelix_core` and plugin tests | Rust-owned deterministic logic | `already_rust_and_should_grow` | these are the canonical surviving owners and should absorb strong replacement coverage |

## Strong-Only Migration Rules

1. Port only tests that defend explicit contracts, regressions, or invariants
2. Delete help-output trivia, command-discovery noise, and redundant fixture
   churn instead of porting it
3. When a strong test still needs a real shell/process boundary, block it on
   the Rust harness work rather than allowing it to survive in Nu indefinitely
4. New Rust-owned test coverage should be nextest-first by default under
   `docs/specs/rust_test_hardening_tools_decision.md`

## Landed First Wave

The first landed wave already moved these clusters into Rust-owned tests:

- generated-config normalization
- runtime materialization lifecycle and missing-artifact repair
- deterministic public command-surface and report-shaping assertions

Those deletions are not the end state. They are the first cut.

## Next Migration Wave

`yazelix-rdn7.4.5.5` chooses this next wave:

1. `yazelix-rdn7.4.5.15`
   - define the shared Rust nextest harness and fixture boundary needed to
     retire strong Nu tests cleanly
2. `yazelix-rdn7.4.5.16`
   - implement the shared Rust helpers and delete redundant Nu test helpers
3. `yazelix-rdn7.4.5.7`
   - finish the next generated-config/render-plan residuals
4. `yazelix-rdn7.4.5.9`
   - port deterministic workspace/session/doctor assertions
5. `yazelix-rdn7.4.5.11`
   - port deterministic managed-config contract assertions
6. `yazelix-rdn7.4.5.13`
   - port the remaining strong `test_yzx_core_commands.nu` command-family cuts
7. `yazelix-rdn7.4.5.4`
   - delete the remaining redundant Nu tests after the replacement Rust
     coverage lands

## What Cannot Survive

These are not valid long-term steady states:

- "keep this strong test in Nu because it talks to a shell"
- "port every current assertion one-to-one even if the Rust copy is still weak"
- "leave a large omnibus file in Nu because only part of it is ready"
- "keep help-output or route-listing checks because they are cheap"

If a test is weak, delete it. If it is strong, port it once the harness can
defend the real contract honestly.

## Verification Gate

- `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- `nu nushell/scripts/dev/validate_rust_test_traceability.nu`
- `nix develop -c cargo nextest run --profile ci --manifest-path rust_core/Cargo.toml -p yazelix_core`
- later plugin-owned Rust ports should use the same nextest-first policy

## Acceptance

1. The governed Nu end state is explicit: no surviving Nu tests
2. The next strong migration wave is named concretely instead of as a blanket rewrite
3. Weak tests are explicitly deleted instead of quietly preserved
4. Strong shell-heavy tests are explicitly blocked on Rust harness work rather
   than marked as permanent Nu survivors

## Traceability

- Bead: `yazelix-rdn7.4.5.1`
- Bead: `yazelix-rdn7.4.5.5`
- Informed by: `docs/specs/governed_test_traceability_inventory.md`
- Informed by: `docs/specs/rust_test_hardening_tools_decision.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
