# Ranked Nu Deletion Budget

## Summary

This document turns the completed subsystem audits into the current ranked
budget for remaining Nushell deletion work.

It is retained as historical ranking context, but it is no longer the active
near-term queue.

Use `provable_nushell_floor_budget.md` first for the current hard under-`5k`
family budget and the stricter Rust-first irreducibility proof standard.
Use `second_wave_nushell_deletion_map.md` for the active `13.2k` to `4.2k`
file-level tranche and cut order.

It is not another LOC leaderboard. The question is not "which directory is
large?" The question is "which surviving Nushell owners can still disappear
without losing functionality, and what has to stay because it is still the
honest shell/process owner?"

This budget is written after the current delete-first pass already removed four
meaningful seams:

- `yazelix-f7hz` deletes `nushell/scripts/core/yazelix.nu` and stops test and
  maintainer paths from rebuilding a fake public Nu root
- `yazelix-niec` deletes the tiny Zellij wrapper files whose only job was
  forwarding one pane-orchestrator command
- `yazelix-4xf2` removes full-config reads from `integrations/yazi.nu` and
  `integrations/managed_editor.nu` by introducing a Rust-owned
  `integration-facts.compute` helper
- `yazelix-fg51` removes the dynamic `nu -c` sweep dispatch seam from the
  maintainer test runner
- `yazelix-jkk3`, `yazelix-sq0g.2`, and `yazelix-sq0g.3` remove the remaining
  product-side full-config reads from popup/menu, popup/editor wrappers, and
  startup/launch/setup callers by introducing narrower Rust-owned facts
- `yazelix-sq0g.4` deletes `nushell/scripts/utils/config_parser.nu` and leaves
  only a dev-only normalize probe for the remaining helper-resolution tests
- `yazelix-rdn7.4.5.2` and `yazelix-rdn7.4.5.3` move the first deterministic
  generated-config and public-command coverage clusters onto Rust-owned tests
  and delete the redundant Nu assertions they replaced

The remaining budget has since been hardened again under `yazelix-lj7z`.
The active queue now treats even earlier no-go boundaries as temporary claims
that must be reproved file by file.

## Public-Facing Read

The current repo can still delete more Nushell, but the best remaining work is
selective:

1. keep deleting bridge owners and wrapper owners where Rust or POSIX already
   owns the real work
2. stop promising broad Rust rewrites for shell/process-heavy surfaces that are
   still cleaner in Nu and POSIX
3. keep docs and tests aligned with the real surviving owners so stale planning
   does not recreate deleted seams

The remaining product/runtime Nu is no longer one giant migration lane. It is a
ranked list of smaller cuts with explicit stop conditions.

## Ranking Method

Higher ranks mean:

- more product/runtime Nu disappears
- the surviving owner is already obvious
- existing tests/contracts are strong enough to make the cut safe
- the cut removes an extra owner rather than adding another bridge

Lower ranks mean:

- maintainer or doc cleanup rather than runtime deletion
- heavier contract gaps before deletion is safe
- no clear end-to-end owner deletion yet

## Ranked Remaining Budget

| Rank | Bucket | Follow-up bead | Expected deleted surface | Retained behavior | Surviving owner after cut | Contract state and gaps | Verification and stop condition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `1` | product/runtime bridge collapse | `yazelix-nuj1` | delete roughly `120-220` lines of terminal-materialization and Ghostty request assembly from `nushell/scripts/core/launch_yazelix.nu`; no new wrapper files | launch still filters supported terminals, materializes managed terminal assets, rerolls Ghostty state, and launches the chosen terminal cleanly | Rust `terminal_materialization.rs`, `ghostty_materialization.rs`, and `control_plane.rs` stay the typed owners; Nu keeps terminal selection, prose, and execution | ready now; use `TLAUNCH-*`, `PRE-*`, and the launch/session audit; no new contract batch required before implementation | `test_yzx_generated_configs.nu`, `test_yzx_workspace_commands.nu`, and `yzx_repo_validator validate-flake-profile-install`; stop if the only alternative is a fake Rust launch wrapper that still shells out to the same terminal commands |
| `2` | launch/session bridge collapse | `yazelix-p18h` | delete roughly `40-80` lines of embedded shell-body assembly from `nushell/scripts/utils/terminal_launcher.nu` by moving the fixed detached-launch probe into one checked-in POSIX helper | detached launch probing stays measurable, fast on success, and explicit on early terminal death | POSIX helper under `shells/posix/` plus existing Nu launch orchestration | ready now; `PROF-*` item IDs already exist, and maintainer profile tests are the live executable defense | `test_startup_profile_records_detached_terminal_probe`, `test_detached_launch_probe_success_path_is_fast`, `test_detached_launch_probe_early_failure_is_visible`; stop if terminal-specific argv shapes still require caller-local Nu assembly and only the fixed probe body can move |
| `3` | deterministic Nu test cleanup after replacement coverage | `yazelix-rdn7.4.5.4` | delete or demote the next redundant Nu test clusters now that the first Rust replacements landed | Rust-owned config/materialization and public-command contracts remain defended while the default Nu lane keeps only shell/process behavior | Rust tests in `rust_core/yazelix_core` plus the surviving Nu integration suites | unblocked after the first Rust migration cuts; stop if a candidate deletion would remove the last executable defense of a shell/process boundary |

## Buckets With No Honest Large Port Left

These buckets are real code, but they are not honest "big Rust-port targets"
today:

| Bucket | Current owner read | Why it is not the next big port |
| --- | --- | --- |
| public CLI UX command bodies | `yzx/menu.nu`, `yzx/dev.nu` | the public root, metadata, and helper routing already moved to Rust; `yzx edit` and `yzx import` are Rust-owned, so the surviving command bodies are now mostly `fzf` or maintainer-process boundaries and need family-by-family deletion budgets |
| startup/session orchestration | `start_yazelix.nu`, `start_yazelix_inner.nu`, `launch_yazelix.nu`, `setup/environment.nu`, POSIX wrappers | the hard part is shell/process/Zellij handoff, not typed decision logic |
| integration adapters | `yazi.nu`, `managed_editor.nu`, `zellij.nu`, popup wrappers | the surviving work is external-tool execution and user-facing copy after the larger live-state and config-owner cuts already moved |
| issue/release/update automation | maintainer modules and validators | large maintainer surface, but not product/runtime deletion value |

## Likely Nushell Survivors For Now

These are still fair game, but they require a much higher bar than the ranked
cuts above:

- `nushell/scripts/setup/environment.nu`
- `nushell/scripts/core/start_yazelix_inner.nu`
- `nushell/scripts/core/yzx_session.nu`
- `nushell/scripts/integrations/yazi.nu`
- `nushell/scripts/integrations/managed_editor.nu`
- `nushell/scripts/yzx/menu.nu`
- `nushell/scripts/yzx/popup.nu`
- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/utils/front_door_runtime.nu`
- `nushell/scripts/yzx/menu.nu`

The stop condition is consistent across all of them: do not move them to Rust
unless Rust becomes the single honest owner of the retained behavior rather than
just a new layer above the same shell/process code. The current family-by-family
decision record now lives in
`docs/specs/likely_nushell_survivor_owner_cut_decisions.md`.

## Follow-Up Queue Created From This Budget

This earlier queue is complete or superseded. The active follow-up queue is now:

- `yazelix-lj7z.1` for the file-level second-wave map
- `yazelix-lj7z.2` through `yazelix-lj7z.10` for the implementation lanes
- `docs/specs/second_wave_nushell_deletion_map.md` for the current exact file
  targets

## Verification

- `yzx_repo_validator validate-specs`
- manual audit review of:
  - `docs/specs/config_runtime_control_plane_canonicalization_audit.md`
  - `docs/specs/public_yzx_command_surface_canonicalization_audit.md`
  - `docs/specs/integration_glue_canonicalization_audit.md`
  - `docs/specs/launch_startup_session_canonicalization_audit.md`
  - `docs/specs/maintainer_harness_canonicalization_audit.md`
  - `docs/specs/spec_docs_contract_alignment_audit.md`
  - `docs/specs/governed_test_traceability_inventory.md`

## Traceability

- Bead: `yazelix-rdn7.6`
- Bead: `yazelix-lj7z`
- Bead: `yazelix-lj7z.1`
- Defended by: `yzx_repo_validator validate-specs`
- Informed by: `docs/specs/governed_test_traceability_inventory.md`
- Informed by: `docs/specs/second_wave_nushell_deletion_map.md`
