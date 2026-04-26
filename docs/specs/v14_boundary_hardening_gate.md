# v14 Boundary-Hardening Gate

## Summary

Yazelix should treat `v14` as a boundary-hardening release gate, not as a packaging milestone.

`v14` is earned when the workspace/backend boundary is real enough that a narrower Core-style omission mode would be honest, even though Yazelix still ships as one product.

## Why

The question for `v14` is not “did we package more things?” It is:

- can runtime/distribution commands degrade honestly when Yazelix does not own the install/update path?
- does the pane orchestrator own the important workspace truth instead of scattered side caches?
- is there a real backend-free workspace slice instead of only architecture rhetoric?

Without an explicit gate:

- major-version planning drifts into vibes and backlog size
- packaging work wrongly dominates the release question
- later Rust/Core planning starts before the boundary proof is stable

## Scope

- define the required technical truths for `v14`
- define what is explicitly *not* part of the gate
- define what work becomes valid only after the gate is satisfied

## Gate Criteria

`v14` is earned only when **all** of these are true:

### 1. Explicit Runtime Identity Hygiene

- generic `YAZELIX_DIR`-style root guessing is gone from the maintained runtime path
- config root, runtime root, and state root are explicit and no longer conflated
- runtime-root-only sessions are treated as a real narrower mode instead of as a broken installer state

Satisfied by:

- `yazelix-le0s.6`
- [runtime_root_contract.md](./runtime_root_contract.md)

### 2. Pane-Orchestrator-Owned Workspace Truth

- sidebar identity is owned by the pane orchestrator instead of correctness-critical cache files
- workspace retargeting returns plugin-owned editor/sidebar targeting truth in one response
- sidebar identity no longer has a separate filesystem cache path

Satisfied by:

- `yazelix-0hm3`
- `yazelix-3a0u`
- [workspace_session_contract.md](../workspace_session_contract.md)

### 3. Honest Runtime/Distribution Capability Tiers

- install/update/doctor no longer assume every mode owns a mutable installer-managed runtime
- `yzx update` reports the owning update path instead of promising a Yazelix-owned runtime updater
- doctor no longer warns about missing installer-owned runtime artifacts in narrowed modes that intentionally do not own them

Satisfied by:

- `yazelix-zjyw`
- [runtime_distribution_capability_tiers.md](./runtime_distribution_capability_tiers.md)

### 4. Backend-Free Workspace Proof Slice

- there is a concrete runtime-root-only proof slice showing real workspace UX can run with host-provided tools
- the surviving flows and their prerequisites are explicit
- the remaining mixed seams are named instead of being hand-waved away

Satisfied by:

- `yazelix-5wao`
- [backend_free_workspace_slice.md](./backend_free_workspace_slice.md)

## Explicit Non-gates

These are intentionally **not** required for `v14`:

- nixpkgs submission
- broader packaging polish
- website/marketing cleanup
- Rust implementation beads
- choosing a future Core backend candidate
- solving every cross-language ownership seam in one pass

## Release Decision

Once all four gate criteria are satisfied, the technical proof for `v14` is complete.

At that point:

- the major-version bump becomes a release-planning and messaging decision
- the next architecture branch can move to post-gate Rust/Core sequencing instead of reopening the same boundary question

As of this gate definition, the required technical slices are the completed beads above. The remaining blocked work is downstream planning and implementation, not missing `v14` proof.

## Post-gate Work

Only after this gate is satisfied should Yazelix treat these as on the critical architectural path:

- `yazelix-kt5.1` and the later `kt5*` Rust sequencing/implementation beads
- `yazelix-qgj7.1` and later Core-backend evaluation beads
- `yazelix-2ex.1.11` broader Rust/clap rewrite planning

## Acceptance Cases

1. There is one durable document that says exactly what `v14` means technically.
2. Packaging is explicitly excluded from the gate.
3. The gate criteria point at concrete completed boundary-hardening slices instead of vague aspirations.
4. Later release planning can answer “is `v14` earned?” without re-litigating the architecture from scratch.

## Verification

- manual review against:
  - [architecture_map.md](../architecture_map.md)
  - [yazelix_core_boundary.md](./yazelix_core_boundary.md)
  - [runtime_distribution_capability_tiers.md](./runtime_distribution_capability_tiers.md)
  - [backend_free_workspace_slice.md](./backend_free_workspace_slice.md)
  - [workspace_session_contract.md](../workspace_session_contract.md)
- spec validation:
  - `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-wg3z`
- Defended by:
  - `yzx_repo_validator validate-specs`
