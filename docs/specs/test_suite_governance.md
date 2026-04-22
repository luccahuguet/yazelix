# Test Suite Governance

## Summary

Yazelix should keep a small number of clear testing lanes, and every test should defend a real contract, regression, integration boundary, or maintained source-of-truth invariant. Cheap structural validators belong in cheap validator lanes, heavier cross-environment checks belong in sweep lanes, and the default maintainer suite should stay focused on a small high-signal set of meaningful runtime and workflow behavior. The runner should use explicit suite membership, not an implicit `test_*.nu` glob that hides dead or library-like files. As immediate cleanup, the README version check is no longer duplicated inside the default `yzx` command test bundle because that invariant already has dedicated CI and `prek` ownership.

## Why

Yazelix has accumulated several ways to validate the repo: direct validator scripts, `yzx dev test`, sweep modes, CI-only checks, and cheap maintainer hooks. Without a written policy, it is easy to add tests that are redundant, too weak, or in the wrong lane.

The goal is not maximum test count. The goal is high-signal coverage with clear ownership:

- fast feedback for maintainers
- durable regression protection for user-visible behavior
- explicit boundaries for heavier or environment-sensitive checks
- fewer duplicate assertions across local, CI, and hook lanes

## Scope

This spec defines:

- the main Yazelix test lanes and their entrypoints
- admission criteria for new tests
- retention and pruning rules for existing tests
- when a check belongs in the default automated suite versus an optional or dedicated lane
- a lightweight inventory of the current suite surfaces
- two concrete cleanups: use explicit suite membership instead of implicit globbing, and demote README version validation out of the default `yzx` command bundle

## Contract Items

#### TEST-001
- Type: ownership
- Status: live
- Owner: maintainer test runner and validator lane entrypoints
- Statement: Yazelix keeps a small set of named test lanes with explicit
  entrypoints. Cheap validators, default regressions, sweep coverage, visual
  sweeps, CI-only checks, and manual/exploratory checks stay distinct instead of
  being treated as one undifferentiated test pile
- Verification: automated
  `nu nushell/scripts/dev/validate_default_test_traceability.nu`; automated
  `nu nushell/scripts/dev/validate_specs.nu`

#### TEST-002
- Type: boundary
- Status: live
- Owner: default-lane admission policy
- Statement: A default-lane test must defend a real contract, regression,
  integration boundary, or maintained invariant. Command-discovery noise,
  wording trivia, and checks already better owned by cheap validators do not
  belong in the default suite
- Verification: automated
  `nu nushell/scripts/dev/validate_default_test_traceability.nu`

#### TEST-003
- Type: invariant
- Status: live
- Owner: default suite membership definition
- Statement: The default automated suite uses explicit suite membership instead
  of an implicit `test_*.nu` glob, and default component files must stay small,
  canonical, and traceable
- Verification: automated
  `nu nushell/scripts/dev/validate_default_test_traceability.nu`; automated
  `nu nushell/scripts/dev/test_yzx_commands.nu`

#### TEST-004
- Type: invariant
- Status: live
- Owner: governed test metadata validators
- Statement: Governed Nu and first-party Rust tests must declare a lane, a
  nearby justification marker, and a structured strength score, and they must
  clear the lane minimum mechanically
- Verification: automated
  `nu nushell/scripts/dev/validate_default_test_traceability.nu`; automated
  `nu nushell/scripts/dev/validate_rust_test_traceability.nu`

#### TEST-005
- Type: non_goal
- Status: live
- Owner: test-suite cleanup policy
- Statement: Yazelix does not preserve weak tests by inertia and does not create
  generic `_extended` overflow files. Weak/orphan tests are deleted, demoted, or
  quarantined with an explicit exit path
- Verification: automated
  `nu nushell/scripts/dev/validate_default_test_traceability.nu`; automated
  `nu nushell/scripts/dev/validate_rust_test_traceability.nu`

## Behavior

### Test lanes and ownership

| Lane | Entrypoint | Purpose | Notes |
| --- | --- | --- | --- |
| Cheap validator lane | `nu nushell/scripts/dev/validate_syntax.nu`, `nu nushell/scripts/dev/validate_readme_version.nu`, `nu nushell/scripts/dev/validate_config_surface_contract.nu` | Very fast structural or source-of-truth checks | Good fit for `prek` and direct CI steps |
| Default automated regression lane | `yzx dev test` | The normal non-sweep automated regression suite | Uses fixed Rust `nextest` suites plus any explicitly-declared transitional Nu entrypoints |
| Internal core regression bundle | `nu nushell/scripts/dev/test_yzx_commands.nu` | High-signal core launch/runtime/workspace/integration contracts | Internal organization detail |
| Non-visual sweep lane | `yzx dev test --sweep` | Matrix coverage for config and supported shell/terminal combinations without opening windows | Environment-sensitive but still scriptable |
| Visual sweep lane | `yzx dev test --visual` | Real terminal-window validation | Heavy, manualish, and not the default lane |
| Full lane | `yzx dev test --all` | Default automated suite + non-visual sweep + visual sweep | For broader release confidence |
| Cheap maintainer hook lane | `prek run --all-files` | Fast always-on local hygiene | Should stay cheap enough to run often |
| CI-only or CI-focused lane | `.github/workflows/ci.yml` | Cheap, reliable branch protection checks | Can be narrower than the full local suite when that keeps CI high-signal |
| Manual / exploratory lane | `nushell/scripts/dev/record_demo_fonts.nu`, benchmark and demo helpers | Human-observed or exploratory checks | Not part of the normal regression contract |

### Current suite inventory

The current repo surface should be understood roughly as:

- Cheap validators:
  - `validate_syntax.nu`
  - `validate_readme_version.nu`
  - `validate_config_surface_contract.nu`
  - `validate_rust_test_traceability.nu`
  - `validate_specs.nu`
- Default automated lane:
  - `test_yzx_commands.nu` as the spec-backed core bundle
- Direct maintainer-only or exploratory scripts that are no longer part of the normal default lane:
  - `test_yzx_maintainer.nu`
- Optional sweep coverage:
  - `test_config_sweep.nu`
  - `sweep_verify.nu`
  - helper files under `nushell/scripts/dev/sweep/`
- Manual / exploratory scripts:
  - `record_demo_fonts.nu`
  - benchmark and demo helpers

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
3. explicitly grandfather it with a short justification in the related issue, spec, or nearby docs

A test is a strong demotion candidate when it is:

- redundant with a cheaper validator already run in CI or hooks
- mostly command-discovery noise
- tightly coupled to implementation details without protecting supported behavior
- expensive relative to the signal it provides

### Default-suite traceability model

- `test_yzx_commands.nu` should stay tied to one or more real spec `Defended by:` lines.
- The default automated suite should contain only spec-backed entrypoints.
- If a regression matters enough for the default lane, it should be promoted into a spec-backed bundle or accompanied by a dedicated spec instead of living as a policy exception.
- The default lane should also enforce mechanical anti-creep guardrails:
  - no dead noncanonical `def test_...` helpers inside default-lane component files
  - a default-suite test-count budget
  - a default-suite runtime budget
  - explicit `# Test lane:` declarations on all `test_*.nu` files
  - explicit `// Test lane:` declarations on all first-party Rust files that contain `#[test]`
  - universal per-test justification and strength scoring across governed lanes
  - no new generic `_extended` overflow files

### Lane placement rules

Lane placement and per-test quality are separate decisions.

- Use a per-test strength score to judge whether an individual test is worth keeping.
- Use a separate lane-placement model to decide where the surviving test belongs.

For Yazelix, lane placement should use suite-shape thinking similar to the Test Pyramid or Testing Trophy:

- cheap structural checks belong in validator lanes
- core user-visible regressions belong in the default lane
- cross-matrix environment coverage belongs in sweep lanes
- heavy visual or human-observed coverage belongs in visual or manual lanes

Do not use the lane model as a substitute for judging whether a test is good. A badly chosen test can still be dead weight even if it sits in the "right" lane.

- Put cheap structural validators in cheap validator lanes, not in the default `yzx` command bundle.
- Keep the default automated suite small, spec-backed, and high-signal.
- Remove weak, low-level, or packaging/config-sync checks instead of preserving them indefinitely in a public secondary lane.
- Put cross-shell, cross-terminal, or matrix concerns in the sweep lanes.
- Put true windowed or visual checks in the visual sweep lane or manual verification path.
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

Every governed `def test_*` must also carry:

- `# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10`

Every governed Rust `#[test]` must also carry:

- `// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10`

### Governed test strength rubric

Yazelix uses a small per-test scoring rubric across all governed lanes. This is intentionally closer to Google-style test-quality thinking and Tanzu's "Fast / Clean / Confidence / Freedom" goals than to suite-shape models like the Test Pyramid or Testing Trophy.

Score governed tests out of 10 using five `0-2` dimensions:

1. `Defect signal`
   - `0`: failing would barely matter or would mostly catch noise
   - `1`: catches some real drift
   - `2`: catches a meaningful user-visible or contract regression
2. `Behavior closeness`
   - `0`: mostly implementation trivia
   - `1`: mixed
   - `2`: clearly checks supported behavior or invariant
3. `Refactor resilience`
   - `0`: likely to fail on harmless internal cleanup
   - `1`: somewhat coupled
   - `2`: should fail only when the real contract changes
4. `Cost`
   - `0`: expensive, flaky, or noisy for the value
   - `1`: acceptable
   - `2`: cheap and high-signal
5. `Uniqueness`
   - `0`: redundant with a cheaper check
   - `1`: partially overlapping
   - `2`: distinct useful coverage

Interpretation:

- `0-4`: weak, remove or demote
- `5-6`: borderline, justify explicitly before keeping
- `7-10`: strong enough for the default lane

Lane minimums:

- `default`: `7/10`
- `maintainer`: `6/10`
- `sweep`: `6/10`
- `manual`: `6/10` if a governed `def test_*` exists there at all

The validator enforces these minimums mechanically. It still cannot prove a test is wise, but it can make low-effort junk and generic overflow much harder to land anywhere in the governed suite.

### Concrete cleanup in this change

The default lane now uses explicit suite membership instead of implicitly globbing `test_*.nu`. This avoids treating library-like test bundles with no `main` entrypoint as if they were real runnable tests.

The normal non-sweep automated suite was then pruned aggressively: regression-only sidecar entrypoints were removed from the default lane, and low-signal parser, formatting, metadata, and animation-detail assertions were deleted or demoted so the maintained contract stays centered on supported user behavior.

The README version invariant also belongs to the cheap validator lane, not the default `yzx` command regression bundle.

That invariant is already defended by:

- `.github/workflows/ci.yml` via `nu nushell/scripts/dev/validate_readme_version.nu`
- `.pre-commit-config.yaml` via the `yazelix-validate-readme-version` hook

So the duplicate README-version assertion is removed from `nushell/scripts/dev/test_yzx_dev_commands.nu` instead of being run in yet another lane.

## Non-goals

- Reclassifying every historical test in one pass
- Shrinking the entire suite to the absolute minimum immediately
- Turning maintainer workflow into heavyweight process
- Requiring a separate spec for every tiny test-only cleanup
- Forcing CI to run the full local suite right now

## Acceptance Cases

1. A maintainer can tell which command to run for cheap validators, the default automated regression suite, sweep coverage, and visual coverage.
2. A proposed new test can be accepted or rejected by pointing to a defended contract and a justified lane.
3. At least one redundant or low-value default-suite assertion is removed, demoted, or explicitly grandfathered with justification.
4. The current suite surface is documented at the suite or file-bucket level rather than left implicit.
5. The default runner no longer depends on an implicit `test_*.nu` glob for its core lane definition.

## Verification

- unit tests: n/a
- integration tests: `nu -c 'source nushell/scripts/yzx/dev.nu; yzx dev test'`
- integration tests: `nu nushell/scripts/dev/test_yzx_commands.nu`
- CI checks: `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- CI checks: `nu nushell/scripts/dev/validate_rust_test_traceability.nu`
- CI checks: `nu nushell/scripts/dev/validate_readme_version.nu`
- CI checks: `nu nushell/scripts/dev/validate_config_surface_contract.nu`
- CI checks: `nu nushell/scripts/dev/validate_specs.nu`
- manual verification: review `.github/workflows/ci.yml` and `.pre-commit-config.yaml` against the lane definitions in this spec

## Traceability

- Bead: `yazelix-leq`
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- Defended by: `nu nushell/scripts/dev/validate_rust_test_traceability.nu`
- Defended by: `nu nushell/scripts/dev/validate_readme_version.nu`
- Defended by: `nu nushell/scripts/dev/validate_config_surface_contract.nu`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should `validate_specs.nu` eventually get its own direct CI step instead of being exercised indirectly through the `yzx` command suite?
- Should the tiny internal split between core contracts and extra regressions remain, or should those files eventually collapse into one default-suite entrypoint?
