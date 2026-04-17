# Backlog Publication Policy

This document defines the publication boundary between GitHub issues and Beads in Yazelix and records the current reviewed set of open beads that intentionally remain internal.

## Policy

GitHub is the public backlog surface. Beads is the full planning surface. The project should not force a one-to-one mapping from every bead to a GitHub issue.

A bead should normally have a public GitHub issue when it is any of the following:

- a user-visible bug, feature, compatibility fix, or contract change
- work where public discussion, release-note visibility, or outside contributor coordination is useful
- a focused item that an external user could reasonably search for, subscribe to, or comment on

A bead should normally stay internal-only when it is any of the following:

- a dotted child bead or other decomposition slice whose public value is already represented by a broader parent or future umbrella issue
- architecture sequencing, migration slicing, or internal implementation planning
- maintainer-only tooling, profiling, policy, or cleanup work with no direct public contract on its own
- experiments, postmortems, or speculative evaluations that are not yet stable enough to present as a public commitment

If a bead becomes public, it must satisfy the GitHub/Beads shared-subset contract in `AGENTS.md`: one matching `external_ref`, matching open or closed lifecycle, and one canonical visible comment of the form `Automated: Tracked in Beads as \`yazelix-...\`.`

## Current Audit

Reviewed on `2026-04-17` from the open backlog.

- Open beads: `79`
- Open beads with `external_ref`: `42`
- Open beads without `external_ref`: `37`

The `37` open beads without a GitHub issue currently split into two buckets:

- `12` public-worthy backlog candidates that could justify a GitHub issue when promoted into active public discussion or implementation
- `25` intentionally internal beads that should remain local planning items for now

## Public-Worthy Open Beads Without GitHub Issues Yet

These are reasonable candidates for future GitHub issues when they become active enough to justify public tracking:

- `yazelix-1et` Improve Helix support for Ghostty cursor effects
- `yazelix-4iej` Bump the Nushell version Yazelix uses
- `yazelix-8h9y` Optimize Yazi config generation on warm startup
- `yazelix-j498` Verifiable config migration: deterministic hashes and fail-proof auto-migration for `yazelix.toml` schema changes
- `yazelix-u7o` Add Ghostty cursor effects for Zellij pane changes
- `yazelix-yho` Evaluate Pretext-inspired text-first welcome screen
- `yazelix-232l` Clarify and verify the zsh/fish managed-shell availability boundary
- `yazelix-3jw` Adopt Nushell-style tutor flow for `yzx tutor`
- `yazelix-c1kw` Surface desktop-entry rebuild progress after terminal switches
- `yazelix-ejqg` Add `VISUAL` to POSIX bootstrap runtime env
- `yazelix-kfs1` Make direct update subcommands describe owner scope neutrally
- `yazelix-z5vf` Align `yazelix` package `meta.platforms` with documented macOS support

## Open Beads Intentionally Kept Internal

### Decomposition And Architecture Slices

- `yazelix-2ex.1.7.7` Fix pane drift by adding pane pinning mechanics
- `yazelix-bl3.1` Decide mission-control tab vs tab-local agent sidebar
- `yazelix-l968.2.1` Sidebar: make the sidebar launcher configurable inside Yazelix
- `yazelix-l968.2.2` Sidebar: isolate the reusable motion and visibility contract
- `yazelix-l968.2.3` Sidebar: separate plugin commands from keybinding policy
- `yazelix-pgdq` Revisit override-layout via owned explicit-run pane creation
- `yazelix-tuj` Replace swap-layout step logic with Zellij 0.44 override-layout transitions
- `yazelix-5yg1.1` Scaffold visible stubs for Yazelix-managed user config surfaces

### Maintainer Tooling, Governance, And Cleanup

- `yazelix-5u8` Define a supply-chain hardening policy for Yazelix tool surfaces
- `yazelix-mriu` Profile desktop and managed-terminal startup as first-class scenarios
- `yazelix-2ex.1.5` Docs experience pass
- `yazelix-865w` Add local startup profile comparison and baseline tooling
- `yazelix-b5u1` Retire legacy shell-block migration checks once the old installer path is dropped
- `yazelix-dg1i` Clarify v15 "non-Rust reboot" wording in docs and specs

### Experiments, Evaluations, And Postmortems

- `yazelix-2ex.1.2.2` Evaluate FlakeHub adoption for Yazelix
- `yazelix-mqb` Evaluate Lenia welcome style later
- `yazelix-qow` Experiment: evaluate a Pixi-backed Yazelix branch
- `yazelix-f0w` Postmortem: session-global Yazi sidebar state caused cross-tab cwd leaks

### Rust Migration Sequencing

- `yazelix-kt5.1` Define the Rust migration matrix and first-slice order after boundaries are proven
- `yazelix-kt5.1.1` Define the Rust/Nushell bridge contract and incremental crate layout
- `yazelix-kt5.2` Port the surviving config contract, parsing, and validation core behind a Rust shim after migration deletion
- `yazelix-kt5.3` Port runtime dependency checking and doctor reasoning into a Rust core
- `yazelix-2ex.1.11` Evaluate a broader Rust/clap rewrite as a v16-or-later path after boundaries are proven
- `yazelix-kt5.4` Port launch/bootstrap state and backend adapter logic into Rust after the seam is extracted
- `yazelix-kt5.5` Decide which config generators should stay Nushell versus move to Rust

## Maintenance Notes

This file is a reviewed policy snapshot, not an auto-generated index. Refresh the counts and lists when backlog grooming materially changes the set of open beads without `external_ref`.

Useful audit commands:

```bash
bd list --status=open --limit 0 --json
nu .github/scripts/validate_issue_bead_contract.nu
nu -c 'source nushell/scripts/yzx/dev.nu; yzx dev sync_issues --dry-run'
```
