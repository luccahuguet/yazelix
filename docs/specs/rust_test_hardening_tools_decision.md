# Rust Test Hardening Tools Decision

## Summary

Yazelix should keep `cargo test` as the canonical baseline and adopt only one
additional Rust hardening tool now: `cargo-nextest`, as an optional maintainer
lane for deterministic Rust-owned suites.

`cargo-mutants` and `cargo-fuzz` are rejected for now. They are useful tools in
the right codebase, but they would add more process weight than signal to the
current Yazelix tree.

## Scope

This decision covers only test-hardening tools for first-party Rust-owned logic:

- `rust_core/yazelix_core`
- `rust_plugins/zellij_pane_orchestrator`

It does not change the default Nu suite, runtime packaging, or the release
contract by itself.

## External References

- Cargo Book `cargo test`: <https://doc.rust-lang.org/cargo/commands/cargo-test.html>
- cargo-nextest docs: <https://nexte.st/>
- cargo-mutants docs: <https://mutants.rs/>
- cargo-mutants limitations: <https://mutants.rs/limitations.html>
- cargo-mutants with nextest: <https://mutants.rs/nextest.html>
- Rust Fuzz Book intro: <https://rust-fuzz.github.io/book/>
- Rust Fuzz Book `cargo-fuzz`: <https://rust-fuzz.github.io/book/cargo-fuzz.html>
- Rust Fuzz Book fuzzing in CI: <https://rust-fuzz.github.io/book/cargo-fuzz/ci.html>

## Dependency Gate

Decision recorded per `AGENTS.md`:

- production crates: none
- dev-only Cargo dependencies: none
- external maintainer tools considered: `cargo-nextest`, `cargo-mutants`,
  `cargo-fuzz`
- built in-house: continue using existing Rust tests plus repo validators as
  the default baseline
- rejected alternatives:
  - adding a Cargo crate dependency just to route test execution
  - blanket mutation or fuzz infrastructure before the target buckets are
    hermetic and narrow enough
- packaging impact:
  - no runtime/package dependency changes
  - if `cargo-nextest` is implemented, it should stay a maintainer-shell or CI
    tool only

## Current Baseline

`cargo test` stays the canonical baseline because it is the standard Cargo test
surface, it already executes unit, integration, and doc tests, and the current
Rust tests and maintainer workflows are written around it.

That means:

- all Rust-owned logic must keep passing under `cargo test`
- doctest coverage remains on `cargo test`
- any extra tool is additive and optional until it proves stable

## Decision Table

| Tool | Decision | Why | Eligible scope | Lane placement | Stop condition |
| --- | --- | --- | --- | --- | --- |
| `cargo-nextest` | adopt narrowly | official docs emphasize per-test isolation, parallel execution, retries, and CI-focused reporting; this matches Yazelix’s deterministic Rust-owned suites without changing runtime packaging | deterministic Rust-owned tests in `yazelix_core` and pane-orchestrator crates, especially route planning, metadata, config normalization, report shaping, render plans, and plugin contract tests | optional maintainer/CI lane only; do not replace `cargo test` baseline and keep doctests on `cargo test` | stop or narrow if the pilot is slower, flakes, or creates command-surface duplication instead of a clear optional lane |
| `cargo-mutants` | reject for now | official docs require hermetic tests and note Cargo-only execution assumptions; mutation runs work on copied or in-place trees and are best when the target suite is already stable and narrow | none for now | none | reopen only after a very small Rust-only bucket is demonstrably hermetic, stable under repeated runs, and worth mutation cost |
| `cargo-fuzz` | reject for now | official docs recommend `cargo-fuzz` for fuzz targets and CI smoke runs, but that implies nightly toolchains, fuzz targets, corpora, and ongoing triage; current Yazelix Rust code is dominated by deterministic config/report/materialization logic rather than parser- or byte-input-heavy bug classes | none for now | none | reopen only when a concrete Rust parser/decoder/state-machine seam exists that benefits from randomized structured inputs more than from stronger unit/property-style tests |

## Why `cargo-nextest` Makes Sense

The current Rust buckets that are most likely to benefit are already
deterministic and self-owned:

- config normalization and runtime/env/control-plane logic
- public route planning and command metadata
- report JSON shaping
- Yazi/Zellij/terminal/Helix render-plan and materialization logic
- pane-orchestrator contract tests

`cargo-nextest` is the only tool in this decision set that improves execution
isolation and feedback without requiring new harness code, new corpora, or tree
mutation.

It is still not the default truth source:

- nextest currently does not run doctests, so doctests must stay on
  `cargo test`
- the pilot should stay optional until it proves stable on the chosen buckets
- the tool should not be used as an excuse to broaden the test surface with
  weaker tests

## Why `cargo-mutants` Is Too Early

The official docs are explicit about the main constraints:

- mutation testing is useful only when the target suite is hermetic and
  deterministic
- the tool currently assumes Cargo-driven testing
- it mutates a copied tree by default, and can mutate the live tree with
  `--in-place`

That does not fit current Yazelix priorities well enough:

- the strongest current Rust defenses are already a mix of unit tests and
  cross-crate contract tests, but the repo still depends heavily on Nu and
  process-driven integration coverage
- the immediate quality win is better routeable execution and deterministic
  Rust-suite feedback, not mutation triage across a mixed ownership tree
- a premature mutation lane would spend time on filters, skips, and runtime
  cost before the narrow Rust-only pilot buckets are settled

## Why `cargo-fuzz` Is Too Early

`cargo-fuzz` is the recommended Rust fuzzing tool, but the Rust Fuzz Book also
shows the real cost: nightly toolchains, dedicated fuzz targets, corpus
management, CI smoke runtime, and ongoing artifact triage.

That investment is justified when Yazelix has a concrete seam such as:

- a real parser or decoder that consumes hostile byte or text input
- a state machine whose bug surface is driven by randomized input sequences
- a pure Rust boundary where coverage-guided structured input is clearly better
  than more direct property or regression tests

The current high-value Rust surfaces are mostly configuration shaping, command
routing, and render-plan logic. Those are better served right now by strong
deterministic tests and a narrower nextest pilot.

## Command Ownership

If `cargo-nextest` is implemented, the commands should stay explicit:

- canonical baseline:
  - `cargo test --manifest-path rust_core/Cargo.toml`
  - `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml`
- optional pilot lane:
  - `cargo nextest run --manifest-path rust_core/Cargo.toml`
  - `cargo nextest run --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml`

Do not hide nextest behind a misleading replacement for `cargo test` until the
pilot proves the exact owner buckets, runtime cost, and CI value.

## Follow-Up Beads

- `yazelix-fkgs` pilots `cargo-nextest` on deterministic Rust-owned buckets
- no implementation bead is created for `cargo-mutants`
- no implementation bead is created for `cargo-fuzz`

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-rdn7.4.3`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

