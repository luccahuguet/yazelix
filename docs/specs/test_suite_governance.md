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

## Behavior

### Test lanes and ownership

| Lane | Entrypoint | Purpose | Notes |
| --- | --- | --- | --- |
| Cheap validator lane | `nu nushell/scripts/dev/validate_syntax.nu`, `nu nushell/scripts/dev/validate_readme_version.nu` | Very fast structural or source-of-truth checks | Good fit for `prek` and direct CI steps |
| Default automated regression lane | `yzx dev test` | The normal non-sweep automated regression suite | Uses explicit membership |
| Internal core regression bundle | `nu nushell/scripts/dev/test_yzx_commands.nu` | High-signal core launch/runtime/workspace/integration contracts | Internal organization detail |
| Internal extra regression bundle | `nu nushell/scripts/dev/test_yzx_extra_regressions.nu` plus `test_reuse_mode.nu` | Small set of extra cheap regressions still worth running by default | Internal organization detail |
| Non-visual sweep lane | `yzx dev test --sweep` | Matrix coverage for config and supported shell/terminal combinations without opening windows | Environment-sensitive but still scriptable |
| Visual sweep lane | `yzx dev test --visual` | Real terminal-window validation | Heavy, manualish, and not the default lane |
| Full lane | `yzx dev test --all` | Default automated suite + non-visual sweep + visual sweep | For broader release confidence |
| Cheap maintainer hook lane | `prek run --all-files` | Fast always-on local hygiene | Should stay cheap enough to run often |
| CI-only or CI-focused lane | `.github/workflows/ci.yml` | Cheap, reliable branch protection checks | Can be narrower than the full local suite when that keeps CI high-signal |
| Manual / exploratory lane | `nushell/scripts/dev/test_fonts.nu`, benchmark and demo helpers | Human-observed or exploratory checks | Not part of the normal regression contract |

### Current suite inventory

The current repo surface should be understood roughly as:

- Cheap validators:
  - `validate_syntax.nu`
  - `validate_readme_version.nu`
  - `validate_specs.nu`
- Default automated lane:
  - `test_yzx_commands.nu` as the spec-backed core bundle
  - `test_yzx_extra_regressions.nu` as a small regression-only allowlist entry with justification
  - the explicit standalone reuse-path regression `test_reuse_mode.nu` as a small regression-only allowlist entry with justification
- Direct maintainer-only or exploratory scripts that are no longer part of the normal default lane:
  - `test_yzx_maintainer.nu`
- Optional sweep coverage:
  - `test_config_sweep.nu`
  - `sweep_verify.nu`
  - helper files under `nushell/scripts/dev/sweep/`
- Manual / exploratory scripts:
  - `test_fonts.nu`
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
- `test_yzx_extra_regressions.nu` and `test_reuse_mode.nu` are currently the only justified regression-only default-suite entrypoints.
- The regression-only allowlist should stay tiny and justified in one validator rather than expanding into a shadow second suite.

### Lane placement rules

- Put cheap structural validators in cheap validator lanes, not in the default `yzx` command bundle.
- Keep the default automated suite small and high-signal even if it includes one tiny extra-regression bundle alongside the core contracts.
- Remove weak, low-level, or packaging/config-sync checks instead of preserving them indefinitely in a public secondary lane.
- Put cross-shell, cross-terminal, or matrix concerns in the sweep lanes.
- Put true windowed or visual checks in the visual sweep lane or manual verification path.
- Keep `prek` for checks maintainers can tolerate on frequent local runs.
- CI may call a narrower set of high-signal commands than the full local suite if the tradeoff is explicit and documented.

### Concrete cleanup in this change

The default lane now uses explicit suite membership instead of implicitly globbing `test_*.nu`. This avoids treating library-like test bundles with no `main` entrypoint as if they were real runnable tests.

The normal non-sweep automated suite was then pruned aggressively: low-signal standalone files for config-parser trivia, terminal-metadata trivia, and weak Nix-detection scenario printing were removed from the maintained suite, and many packaging/config-sync or lower-level helper assertions were dropped from the grouped extra regressions.

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
- integration tests: `nu nushell/scripts/dev/test_yzx_extra_regressions.nu`
- CI checks: `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- CI checks: `nu nushell/scripts/dev/validate_readme_version.nu`
- CI checks: `nu nushell/scripts/dev/validate_specs.nu`
- manual verification: review `.github/workflows/ci.yml` and `.pre-commit-config.yaml` against the lane definitions in this spec

## Traceability

- Bead: `yazelix-leq`
- Defended by: `nu nushell/scripts/dev/test_yzx_commands.nu`
- Defended by: `nu nushell/scripts/dev/validate_default_test_traceability.nu`
- Defended by: `nu nushell/scripts/dev/validate_readme_version.nu`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should `validate_specs.nu` eventually get its own direct CI step instead of being exercised indirectly through the `yzx` command suite?
- Should the tiny internal split between core contracts and extra regressions remain, or should those files eventually collapse into one default-suite entrypoint?
