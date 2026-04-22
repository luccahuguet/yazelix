# Rust Test Hardening Tools Decision

## Summary

Yazelix should use `cargo-nextest` as the default runner for first-party Rust
tests.

`cargo test` remains required, but only for doctests and any explicit
nextest-unsupported exception surfaces.

`cargo-mutants` and `cargo-fuzz` are still rejected for now. They are useful
tools in the right codebase, but they would add more process weight than signal
to the current Yazelix tree.

## Scope

This decision covers only test-hardening tools for first-party Rust-owned
logic:

- `rust_core/yazelix_core`
- `rust_plugins/zellij_pane_orchestrator`

It does not change the default Nu suite, runtime packaging, or the release
contract by itself.

## External References

- Cargo Book `cargo test`: <https://doc.rust-lang.org/cargo/commands/cargo-test.html>
- cargo-nextest home: <https://nexte.st/>
- cargo-nextest running tests: <https://nexte.st/docs/running/>
- cargo-nextest coverage note on doctests: <https://nexte.st/docs/integrations/test-coverage/>
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
  the canonical ownership model
- rejected alternatives:
  - adding a Cargo crate dependency just to route test execution
  - blanket mutation or fuzz infrastructure before the target buckets are
    hermetic and narrow enough
- packaging impact:
  - no runtime or package dependency changes
  - `cargo-nextest` remains a maintainer-shell and CI tool, but it is now the
    default runner for first-party Rust tests rather than an optional pilot

## Runner Policy

The official nextest docs say to run doctests separately with
`cargo test --doc`. The Cargo Book continues to make `cargo test` the standard
Cargo surface that covers unit, integration, and doc tests.

The resulting Yazelix policy is:

- default first-party Rust test runner: `cargo nextest run`
- required exception runner: `cargo test --doc`
- any other `cargo test` usage must be explicitly allowlisted as a real
  nextest limitation, not habit

This keeps one clear default for owned Rust test buckets while preserving the
official doctest path that nextest does not currently handle.

## Decision Table

| Tool | Decision | Why | Eligible scope | Lane placement | Stop condition |
| --- | --- | --- | --- | --- | --- |
| `cargo-nextest` | adopt as the default Rust test runner | official docs emphasize per-test isolation, parallel execution, retries, and CI-focused reporting, and nextest supports the normal Rust test-binary path while still leaving doctests to Cargo | first-party Rust test buckets in `yazelix_core` and pane-orchestrator crates, especially route planning, metadata, config normalization, report shaping, render plans, and plugin contract tests | default maintainer and CI runner for first-party Rust tests; keep doctests on `cargo test --doc` | stop or narrow only for explicit unsupported surfaces; do not let broad cargo-test-by-habit usage return |
| `cargo-mutants` | reject for now | official docs require hermetic tests and note Cargo-only execution assumptions; mutation runs work on copied or in-place trees and are best when the target suite is already stable and narrow | none for now | none | reopen only after a very small Rust-only bucket is demonstrably hermetic, stable under repeated runs, and worth mutation cost |
| `cargo-fuzz` | reject for now | official docs recommend `cargo-fuzz` for fuzz targets and CI smoke runs, but that implies nightly toolchains, fuzz targets, corpora, and ongoing triage; current Yazelix Rust code is dominated by deterministic config/report/materialization logic rather than parser- or byte-input-heavy bug classes | none for now | none | reopen only when a concrete Rust parser, decoder, or state-machine seam exists that benefits from randomized structured inputs more than from stronger unit or regression tests |

## Why `cargo-nextest` Should Be The Default

The current Rust buckets that benefit most are already deterministic and
self-owned:

- config normalization and runtime/env/control-plane logic
- public route planning and command metadata
- report JSON shaping
- Yazi/Zellij/terminal/Helix render-plan and materialization logic
- pane-orchestrator contract tests

`cargo-nextest` is the only tool in this decision set that improves execution
isolation and feedback without requiring new harness code, new corpora, or tree
mutation.

It should now be the default runner for first-party Rust test buckets:

- nextest does not currently run doctests, so doctests stay on
  `cargo test --doc`
- the earlier pilot already proved useful enough to stop treating nextest as an
  optional side lane
- the runner change does not justify broader or weaker Rust tests

## Why `cargo-mutants` Is Too Early

The official docs are explicit about the main constraints:

- mutation testing is useful only when the target suite is hermetic and
  deterministic
- the tool currently assumes Cargo-driven testing
- it mutates a copied tree by default, and can mutate the live tree with
  `--in-place`

That still does not fit current Yazelix priorities well enough:

- the strongest current Rust defenses are already a mix of unit tests and
  cross-crate contract tests, but the repo still depends heavily on Nu and
  process-driven integration coverage
- the immediate quality win is a clear nextest-first runner policy, not
  mutation triage across a mixed ownership tree
- a premature mutation lane would spend time on filters, skips, and runtime
  cost before the narrow Rust-only buckets are settled

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
deterministic tests and a stricter nextest-first runner policy.

## Command Ownership

The commands should stay explicit:

- default first-party Rust lane:
  - `cargo nextest run --manifest-path rust_core/Cargo.toml`
  - `cargo nextest run --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml`
- required doctest lane:
  - `cargo test --manifest-path rust_core/Cargo.toml --doc`
  - `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --doc`
- explicit exception lane:
  - any non-doctest `cargo test` path must be named and justified by a concrete
    nextest limitation in the bead or linked decision doc

Do not reintroduce a broad cargo-test baseline for first-party Rust tests by
habit. Keep Cargo as the explicit doctest and exception path.

## Follow-Up Beads

- `yazelix-fkgs` is the historical pilot that proved nextest was worth keeping
- `yazelix-rdn7.4.7` defines the current nextest-first runner policy
- `yazelix-rdn7.4.5.15` and `yazelix-8ih0.3` must implement that policy in the
  shared Rust harness and maintainer runner
- no implementation bead is created for `cargo-mutants`
- no implementation bead is created for `cargo-fuzz`

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-rdn7.4.3`
- Bead: `yazelix-rdn7.4.7`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
