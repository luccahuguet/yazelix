# Test Suite Governance

## Summary

Yazelix should keep a small number of clear testing lanes, and every governed
test should defend a real contract, regression, integration boundary, or
maintained source-of-truth invariant. Cheap structural validators belong in
cheap validator lanes, heavier cross-environment checks belong in sweep lanes,
and the default automated suite should stay focused on a small high-signal set
of Rust-owned behavior tests.

The old governed Nushell omnibus files are deleted. The default lane now uses
explicit Rust `cargo nextest` suite membership, not an implicit `test_*.nu`
glob and not a transitional Nu aggregator. The remaining `.nu` files under
`nushell/scripts/dev/` are shell-heavy runners or validators, not governed test
owners.

## Why

Yazelix has accumulated several ways to validate the repo: direct validator scripts, `yzx dev test`, sweep modes, CI-only checks, and cheap maintainer hooks. Without a written policy, it is easy to add tests that are redundant, too weak, or in the wrong lane.

The goal is not maximum test count. The goal is high-signal coverage with clear ownership:

- fast feedback for maintainers
- durable regression protection for user-visible behavior
- explicit boundaries for heavier or environment-sensitive checks
- fewer duplicate assertions across local, CI, and hook lanes

## Scope

This contract defines:

- the main Yazelix test lanes and their entrypoints
- admission criteria for new tests
- retention and pruning rules for existing tests
- when a check belongs in the default automated suite versus an optional or dedicated lane
- a lightweight inventory of the current suite surfaces
- two concrete cleanups: use explicit Rust suite membership instead of implicit
  globbing, and demote README version validation out of the default regression
  lane

## Contract Items

#### TEST-001
- Type: ownership
- Status: live
- Owner: maintainer test runner and validator lane entrypoints
- Statement: Yazelix keeps a small set of named test lanes with explicit
  entrypoints. Cheap validators, default regressions, sweep coverage, CI-only
  checks, and manual/exploratory checks stay distinct instead of
  being treated as one undifferentiated test pile
- Verification: automated
  `yzx_repo_validator validate-contracts`

#### TEST-002
- Type: boundary
- Status: live
- Owner: default-lane admission policy
- Statement: A default-lane test must defend a real contract, regression,
  integration boundary, or maintained invariant. Command-discovery noise,
  wording trivia, and checks already better owned by cheap validators do not
  belong in the default suite
- Verification: automated
  `yzx_repo_validator validate-rust-test-traceability`

#### TEST-003
- Type: invariant
- Status: live
- Owner: default suite membership definition
- Statement: The default automated suite uses explicit suite membership instead
  of an implicit `test_*.nu` glob, and its governed ownership lives in fixed
  Rust `nextest` suites plus explicit `cargo test` exceptions only where
  `nextest` is not the honest fit
- Verification: automated
  `yzx dev test`

#### TEST-004
- Type: invariant
- Status: live
- Owner: governed test metadata validators
- Statement: Governed Nu and first-party Rust tests must declare a lane, a
  nearby justification marker, and any required contract-item traceability.
  Strength scoring is review guidance, not a persisted metadata requirement
- Verification: automated
  `yzx_repo_validator validate-rust-test-traceability`

#### TEST-005
- Type: non_goal
- Status: live
- Owner: test-suite cleanup policy
- Statement: Yazelix does not preserve weak tests by inertia and does not create
  generic `_extended` overflow files. Weak/orphan tests are deleted, demoted, or
  quarantined with an explicit exit path
- Verification: automated
  `yzx_repo_validator validate-rust-test-traceability`

## Behavior

### Test lanes and ownership

| Lane | Entrypoint | Purpose | Notes |
| --- | --- | --- | --- |
| Cheap validator lane | `yzx_repo_validator validate-nushell-syntax`, `yzx_repo_validator validate-readme-version`, `yzx_repo_validator validate-config-surface-contract` | Very fast structural or source-of-truth checks | Good fit for `prek` and direct CI steps |
| Default automated regression lane | `yzx dev test` | The normal non-sweep automated regression suite | Uses fixed Rust `nextest` suites plus explicit `cargo test` exceptions only where required |
| Sweep lane | `yzx dev test --sweep` | Matrix coverage for config and supported shell/terminal combinations without opening windows | Environment-sensitive but still scriptable |
| Full lane | `yzx dev test --all` | Default automated suite + config/shell sweep | For broader release confidence |
| Cheap maintainer hook lane | `prek run --all-files` | Fast always-on local hygiene | Should stay cheap enough to run often |
| CI-only or CI-focused lane | `.github/workflows/ci.yml` | Cheap, reliable branch protection checks plus source-level release checks | Always runs source contracts, builds validation helpers once per run, path-gates Nix-heavy child/customization/Darwin lanes by changed surface, and keeps runtime package smoke schedule/manual-only |
| Package cache publishing lane | `.github/workflows/publish_nix_cache.yml` | Build and publish main package outputs for published refs | Owns package-build evidence for `main` cache publication; branch CI owns source, API, child-release, and runtime-behavior evidence before merge |

### Canonical fast gate set

Ordinary Yazelix changes should use a small, purpose-matched gate set instead
of defaulting to every available validator. The canonical fast gates are:

| Change shape | Inner-loop gate | Completion gate |
| --- | --- | --- |
| Rust source changes | `yzx dev rust fmt --check`; `yzx dev rust check`; focused `cargo test` or `yzx dev rust test` filter for touched behavior | `yzx dev test` for nontrivial runtime behavior, shared command routing, or test-governance changes |
| Governed test changes | Focused test binary or filter for edited tests | `yzx_repo_validator validate-rust-test-traceability`; `yzx dev test` when default-suite membership or broad behavior changed |
| Config contract, schema, Home Manager, or generated config metadata | `yzx_repo_validator validate-config-surface-contract` | `yzx dev test` when runtime materialization or config UI behavior changed |
| Zellij layouts, first-party plugin contracts, or workspace assets | Focused Rust/plugin tests for touched behavior | `yzx_repo_validator validate-workspace-session-contract`; `nix build .#runtime` after first-party plugin package or runtime artifact changes |
| Docs front door and current prose | Direct review plus relevant docs validator | `yzx_repo_validator validate-docs-experience` for route/stale-doc changes; `yzx_repo_validator validate-readme-version` for release/version surface changes |
| Nix package, flake API, or child input changes | Cheap `nix eval` or targeted validator for the touched API | `yzx_repo_validator validate-flake-interface`; `yzx_repo_validator validate-nix-customization-api`; `yzx_repo_validator validate-child-release-transaction` for child input changes, first-party child package changes, and Darwin child-package smoke contracts; `yzx_repo_validator validate-runtime-package-smoke` for packaged runtime helper/env/materialization behavior; heavier installed-runtime/profile validators only for release or explicit cold-install checks |

`yzx dev test` is the canonical default regression gate, not the only inner-loop
command. Runtime package smoke is scheduled/manual evidence for packaged
helpers, runtime env setup, and generated materialization because it builds the
full package and can exceed ordinary push-CI budgets on cache-miss hosted
runners. Cache publishing owns package buildability for published `main` refs,
while cold-install remains an explicit manual lane for full profile-install
evidence. Sweep, cold-install, installed-runtime, runtime-smoke, and nixpkgs
submission validators are release or explicit smoke gates, not routine
requirements for small source or test cleanups.

### Current suite inventory

The current repo surface should be understood roughly as:

- Cheap validators:
  - `yzx_repo_validator validate-nushell-syntax`
  - `yzx_repo_validator validate-readme-version`
  - `yzx_repo_validator validate-config-surface-contract`
  - `yzx_repo_validator validate-rust-test-traceability`
  - `yzx_repo_validator validate-contracts`
  - `yzx_repo_validator validate-docs-experience`
- CI-focused release validators:
  - `yzx_repo_validator validate-child-release-transaction`
- Default automated lane:
  - `rust_core/Cargo.toml` `nextest` suite `yazelix_core`
  - `../yazelix-zellij-pane-orchestrator/Cargo.toml` `nextest` suite `zellij_pane_orchestrator`
- Optional sweep coverage:
  - `shells/posix/sweep_verify.sh`
  - `shells/posix/sweep_validate_runtime_tools.sh`

This inventory is intentionally at the suite or file-bucket level. It is enough to decide lane ownership without pretending every test needs its own long policy entry.

### Admission criteria for new tests

A new test should be added only when it defends at least one of these:

1. a user-visible behavior
2. a regression that already happened
3. an integration boundary
4. a maintained source-of-truth invariant
5. a documented maintainer-workflow contract that would cause real drift if it broke

A new test should also have an obvious answer to both questions:

- What contract does this defend?
- Why does this lane own that contract?

Tests should not be admitted to the default suite when they mostly assert:

- that a command exists in help text
- that a subcommand name appears somewhere in output
- that implementation trivia or incidental wording is present
- that the same invariant already has a cheaper dedicated validator lane

### Retention and pruning rules

Existing tests should be kept only if they still map cleanly to a living contract, regression, supported behavior, or explicit invariant.

When a test no longer clears that bar, Yazelix should do one of three things:

1. remove it
2. move it to a more appropriate lane
3. explicitly grandfather it with a short justification in the related issue, contract, or nearby docs

A test is a strong demotion candidate when it is:

- redundant with a cheaper validator already run in CI or hooks
- mostly command-discovery noise
- tightly coupled to implementation details without protecting supported behavior
- expensive relative to the signal it provides

### Default-suite traceability model

- The default automated suite should contain only contract-backed Rust entrypoints
  declared in `nushell/scripts/maintainer/test_suite_inventory.toml`
- If a regression matters enough for the default lane, it should land as a
  strong Rust test in one of those owned suites instead of reviving a governed
  Nu omnibus file
- The default lane should also enforce mechanical anti-creep guardrails:
  - a zero-governed-Nu-test guard for `nushell/scripts/dev/test_*.nu`
  - a default-suite runtime budget
  - explicit `// Test lane:` declarations on all first-party Rust files that
    contain `#[test]`
  - universal per-test justification across governed lanes
  - no new generic `_extended` overflow files
  - no new governed Nu `test_*.nu` surface without an explicit policy reversal

### Lane placement rules

Lane placement and per-test quality are separate decisions.

- First decide whether the test protects a living behavior, regression, boundary, or invariant.
- Use a separate lane-placement model to decide where the surviving test belongs.

For Yazelix, lane placement should use suite-shape thinking similar to the Test Pyramid or Testing Trophy:

- cheap structural checks belong in validator lanes
- core user-visible regressions belong in the default lane
- cross-matrix environment coverage belongs in sweep lanes
- heavy visual or human-observed coverage belongs in manual lanes

Do not use the lane model as a substitute for judging whether a test is good. A badly chosen test can still be dead weight even if it sits in the "right" lane.

- Put cheap structural validators in cheap validator lanes, not in the default `yzx` command bundle.
- Keep the default automated suite small, contract-backed, and high-signal.
- Remove weak, low-level, or packaging/config-sync checks instead of preserving them indefinitely in a public secondary lane.
- Put cross-shell, cross-terminal, or matrix concerns in the sweep lanes.
- Put true windowed or visual checks in a manual verification path.
- Keep `prek` for checks maintainers can tolerate on frequent local runs.
- CI may call a narrower set of high-signal commands than the full local suite if the tradeoff is explicit and documented.
- Runtime-budget increases should be explicit. If a change needs more default-lane runtime, it should update the runtime validator budget in the same PR with a short justification.
- Test-count increases should be explicit. If a change needs more default-lane tests, it should update the count-budget validator in the same PR with a short justification.
- Do not create generic `_extended` test files as overflow. If a nondefault lane needs more coverage, put it in an explicitly named lane or file that matches its real ownership.

### Enforced test metadata

Every `test_*.nu` file must declare one supported lane with a top-level header:

- `# Test lane: default`
- `# Test lane: maintainer`
- `# Test lane: sweep`
- `# Test lane: manual`

Every first-party Rust file that contains `#[test]` must declare one supported lane with a nearby line comment:

- `// Test lane: default`
- `// Test lane: maintainer`
- `// Test lane: sweep`
- `// Test lane: manual`

Every governed `def test_*` must carry one nearby justification marker:

- `# Defends: ...`
- `# Regression: ...`
- `# Invariant: ...`

Every governed Rust `#[test]` must carry one nearby justification marker:

- `// Defends: ...`
- `// Regression: ...`
- `// Invariant: ...`

Test comments should explain what the test protects, not score the test itself.
Exact palette constants, help-output trivia, command-name discovery,
generated-text implementation details, and one-off color or glyph snapshots are
not sufficient unless they defend a documented product contract or a concrete
regression.

### Concrete cleanup in this change

The default lane now uses explicit Rust suite membership instead of implicitly
globbing `test_*.nu`. This avoids treating library-like Nu bundles as runnable
tests and avoids preserving a second governed test language after the Rust-owned
command surfaces landed.

The normal non-sweep automated suite was then pruned aggressively: the governed
Nu omnibus files were deleted, their strongest deterministic contracts moved
into Rust `nextest` suites, and the remaining shell-heavy `.nu` files were
renamed as runners instead of pretending to be governed tests.

The README version invariant also belongs to the cheap validator lane, not the
default regression lane.

That invariant is already defended by:

- `.github/workflows/ci.yml` via `yzx_repo_validator validate-readme-version`
- `.pre-commit-config.yaml` via the `yazelix-validate-readme-version` hook

So the duplicate README-version assertion is removed from the governed
regression suite instead of being run in yet another lane.

The docs-experience validator owns docs front-door routes and stale markers in
current user docs. It deliberately does not freeze individual command-reference
markers, because command names and help copy are owned by the CLI command
surface and docs review rather than by a second route validator.

## Non-goals

- Reclassifying every historical test in one pass
- Shrinking the entire suite to the absolute minimum immediately
- Turning maintainer workflow into heavyweight process
- Requiring a separate contract for every tiny test-only cleanup
- Forcing CI to run the full local suite right now

## Acceptance Cases

1. A maintainer can tell which command to run for cheap validators, the default automated regression suite, sweep coverage, and manual visual coverage.
2. A proposed new test can be accepted or rejected by pointing to a defended contract and a justified lane.
3. At least one redundant or low-value default-suite assertion is removed, demoted, or explicitly grandfathered with justification.
4. The current suite surface is documented at the suite or file-bucket level rather than left implicit.
5. The default runner no longer depends on an implicit `test_*.nu` glob for its core lane definition.

## Verification

- unit tests: n/a
- integration tests: `yzx dev test`
- integration tests: `nix develop -c cargo nextest run --profile ci --manifest-path rust_core/Cargo.toml -p yazelix_core`
- integration tests: `nix develop -c cargo nextest run --profile ci --manifest-path ../yazelix-zellij-pane-orchestrator/Cargo.toml --lib`
- CI job `source_contracts`: `yzx_repo_validator validate-nushell-syntax`, `validate-readme-version`, `validate-upgrade-contract --ci`, `validate-config-surface-contract`, `validate-contracts`, `validate-docs-experience`, `validate-rust-test-traceability`, and `validate-flake-interface`
- CI job `nix_customization_api`: `yzx_repo_validator validate-nix-customization-api` plus `checks.x86_64-linux.cargo_git_output_hash_contracts`
- CI job `child_release_transaction`: `yzx_repo_validator validate-child-release-transaction` including child publication and Darwin child-package smoke contracts. First-party Zellij plugin Darwin wasm checks consume child-declared package metadata instead of inspecting child build recipe markers; targeted Nix package builds remain authoritative for processed Cargo vendor hashes
- manual verification: review `.github/workflows/ci.yml` and `.pre-commit-config.yaml` against the lane definitions in this contract

## Traceability
- Defended by: `yzx_repo_validator validate-rust-test-traceability`
- Defended by: `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-readme-version`
- Defended by: `yzx_repo_validator validate-config-surface-contract`
- Defended by: `yzx_repo_validator validate-child-release-transaction`
- Defended by: `yzx dev test`
- Defended by: `yzx_repo_validator validate-contracts`

## Open Questions

- Should the surviving Rust default suites collapse further once more plugin and
  control-plane coverage merges land?
