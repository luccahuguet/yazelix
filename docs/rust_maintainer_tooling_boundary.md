# Rust Maintainer Tooling Boundary

Decision date: 2026-04-26

Tracked by: `yazelix-9opk.5`; re-evaluated by `yazelix-vyas`

## Decision

Keep maintainer-only Rust tooling in this repository, but split it out of the product runtime crate.

The target shape is:

- `yazelix_core`: product/runtime crate, shipped helper binaries, user command behavior, config/materialization/runtime contracts, and user-facing doctor logic
- `yazelix_maintainer`: in-repo maintainer crate for repo validators, release/update automation, Beads/GitHub sync, sweep runners, plugin wasm sync, and maintainer test orchestration
- no separate repository for now

This is a separation decision, not a deletion decision. The maintainer code is real and release-critical; the problem was that it lived in the same crate as runtime code and therefore inflated product ownership, package-time build surface, and Rust LOC accounting.

## First-Principles Rationale

Runtime users need a small, stable helper surface: `yzx`, `yzx_core`, and `yzx_control`. They do not need to understand or conceptually own release bumping, issue sync, contract validation, CI checks, sweep orchestration, or pane-orchestrator wasm synchronization.

Maintainers need those tools to live close to the repository contracts they defend. Offloading them to another repository would reduce local LOC on paper, but it would split validators from the files they validate and create version-skew risk between Yazelix, its release process, and its CI checks.

The pragmatic split is therefore an in-repo crate boundary. It makes runtime ownership smaller without weakening reproducible releases or forcing cross-repo coordination for every contract change.

## Tool Residency

Use three different homes for three different kinds of speed.

- Personal Home Manager owns frequently used command-line tools that are not part of the shipped Yazelix runtime: `cargo-nextest`, `cargo-udeps`, `tokei`, `gh`, `jq`, `nu-lint`, Beads, and similar maintainer binaries
- The Yazelix maintainer shell owns reproducible repo gates and runtime-adjacent tools that should be available to contributors from the flake
- Cargo compilation outputs, incremental state, and `target/` directories stay project-local; moving those into Home Manager would not make builds cleaner and would make cache ownership harder to reason about

`cargo-udeps` is a manual cleanup audit, not a default gate. It requires a nightly Rust compiler because it uses unstable compiler flags, so it is best run from a loaded maintainer profile that provides both the `cargo-udeps` binary and a nightly toolchain, for example:

```bash
cargo +nightly udeps --manifest-path rust_core/Cargo.toml --workspace --all-targets
cargo +nightly udeps --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --all-targets
```

Do not add `cargo-udeps` to user runtime packages. Runtime users do not need Rust cleanup tools to launch Yazelix.

## Runtime Package Impact

Current state:

- `packaging/mk_runtime_tree.nix` exposes only `yzx`, `yzx_core`, and `yzx_control` from the Rust helper package
- `packaging/rust_core_helper.nix` builds only `-p yazelix_core`
- `yzx_repo_validator`, `yzx_repo_maintainer`, and the `repo_*` modules live in `rust_core/yazelix_maintainer`
- the installed/runtime `yzx dev` surface keeps only runtime diagnostics: `inspect_session` and `profile`
- the maintainer shell provides a repo-local `yzx` wrapper that routes repo-only `yzx dev` commands to `yzx_repo_maintainer`
- package-time Rust tests are disabled for user package builds even though maintainer checks remain available from the workspace

Maintained target state:

- runtime/package builds should target only the product crate and shipped helper binaries
- user package builds should not run Cargo tests; explicit maintainer-shell `yzx dev rust test`, CI, and maintainer validators own Rust verification
- CI and the maintainer shell should invoke repo-only commands through `yazelix_maintainer`
- maintainer commands may depend on `yazelix_core` for product contract APIs, but `yazelix_core` must not depend on `yazelix_maintainer`
- package-time tests should not require host-only maintainer tools such as Nix, GitHub CLI, Beads, or Home Manager because package-time tests should not run on the user install path

## Subsystem Decisions

| Subsystem | Current path | Decision | Offload decision | Rationale |
| --- | --- | --- | --- | --- |
| Validator dispatcher | `yazelix_maintainer/src/bin/yzx_repo_validator.rs` | moved to `yazelix_maintainer` | reject external repo | CI entrypoint is repo-specific and validates local files, contracts, packages, and release rules |
| Maintainer dispatcher | `yazelix_maintainer/src/bin/yzx_repo_maintainer.rs` | moved to `yazelix_maintainer` | reject external repo | Local dev workflow wrapper for Beads/GitHub sync, tests, release bump, updates, and plugin sync |
| Repo contract validators | `yazelix_maintainer/src/repo_contract_validation.rs` | moved to `yazelix_maintainer` | reject external repo | Largest maintainer file; all checks are tied to this repo's Nix, README, Home Manager, release, and package contracts |
| Generic repo validation | `yazelix_maintainer/src/repo_validation.rs` | moved to `yazelix_maintainer` | reject external repo | Contract/test traceability and package-test-purity are repo policy, not runtime product behavior |
| Issue sync | `yazelix_maintainer/src/repo_issue_sync.rs` | moved to `yazelix_maintainer` | reject external repo | Beads/GitHub mapping is local workflow state and should not become a separately versioned tool |
| Nushell lint wrapper | `yazelix_maintainer/src/repo_nu_lint.rs` | moved to `yazelix_maintainer` | reject external repo | Thin repo-local maintainer command around checked-in Nu files |
| Pane-orchestrator build/sync | `yazelix_maintainer/src/repo_plugin_build.rs` | moved to `yazelix_maintainer` | reject external repo | Sync stamp and tracked wasm are part of this repository; command depends on `yazelix_core` materialization APIs |
| Sweep runner | `yazelix_maintainer/src/repo_sweep_runner.rs` | moved to `yazelix_maintainer` | reject external repo | Runs local runtime/config matrices against this checkout |
| Test runner | `yazelix_maintainer/src/repo_test_runner.rs` | moved to `yazelix_maintainer` | reject external repo | Maintainer orchestration over local validator/test surfaces, not shipped product behavior |
| Update workflow | `yazelix_maintainer/src/repo_update_workflow.rs` | moved to `yazelix_maintainer` | reject external repo | Writes local pins, vendored plugins, README surface, and canary materialization |
| Version bump workflow | `yazelix_maintainer/src/repo_version_bump.rs` | moved to `yazelix_maintainer` | reject external repo | Transactional release policy must stay in the repo that owns tags, changelog, and upgrade notes |
| Workspace session validator | `yazelix_maintainer/src/workspace_session_contract.rs` | moved to `yazelix_maintainer` | reject external repo | Validator-only owner; it can call runtime-owned workspace asset checks through `yazelix_core` |
| Workspace asset checks | `workspace_asset_contract.rs` | keep in `yazelix_core` | reject external repo | Used by user-facing `yzx doctor`, so it is product runtime behavior |
| Layout family contract | `layout_family_contract.rs` | keep in `yazelix_core` | reject external repo | Used by runtime workspace asset checks and doctor reporting |
| Profile commands | `profile_commands.rs` | keep in `yazelix_core` for now | reject external repo | `yzx_control profile` is used by live startup/profile instrumentation, not only repo validation |

## Implementation Boundary

The accepted implementation should be a mechanical crate split, not a rewrite:

- keep `rust_core/yazelix_maintainer`
- keep maintainer-only modules and bins in that crate
- keep installed/runtime `yzx dev` limited to runtime-safe diagnostics
- keep repo-only `yzx dev` commands available through the maintainer-shell wrapper, which dispatches to `yzx_repo_maintainer`
- keep public command names stable: `yzx_repo_validator` and `yzx_repo_maintainer`
- update `packaging/rust_core_helper.nix` so runtime package builds only the product crate/binaries and leaves tests to explicit maintainer/CI gates
- keep `yazelix_maintainer -> yazelix_core` as the only dependency direction

Do not split the repository until there is a concrete problem this in-repo crate boundary cannot solve. Current pressure is ownership clarity and package/runtime separation, not independent versioning.

## Child Repo Re-Evaluation

Decision date: 2026-05-07

Keep `yazelix_maintainer` in this repository for now. An off-repo `yazelix-dev` child repository would make the main Rust LOC inventory look cleaner by removing roughly `11,338` raw Rust lines from this checkout, but it would not reduce total maintenance. Most of that code validates or mutates files in this repository, and moving it elsewhere would add version-skew risk to CI, release bumps, Beads/GitHub sync, runtime asset refreshes, and pane-orchestrator wasm sync.

The better next move is to shrink and split maintainer validation by domain inside this repository. Reconsider an external `yazelix-dev` only if a future pass finds a genuinely generic tool that can work against arbitrary checkouts with a stable machine contract.

### Module Residency Matrix

| Module or binary | Raw lines | Selected residency | Off-repo LOC effect | Version-skew risk | CI/devShell impact | User-runtime impact |
| --- | ---: | --- | --- | --- | --- | --- |
| `src/bin/yzx_repo_validator.rs` | 255 | in-repo maintainer crate | small main-repo reduction only | high: dispatch must match local validators | CI would need a pinned external tool for every contract edit | none; not packaged for users |
| `src/bin/yzx_repo_maintainer.rs` | 313 | in-repo maintainer crate | small main-repo reduction only | high: command routing tracks local repo workflows | maintainer shell wrapper would depend on external release cadence | none; installed `yzx dev` stays diagnostic-only |
| `src/repo_contract_validation.rs` | 4,149 | in-repo, split by contract domain later | largest apparent LOC reduction | very high: validates README, Nix, config, release, and package contracts in this checkout | every CI contract change would require cross-repo coordination | none; repo-only validator |
| `src/repo_validation.rs` | 1,535 | in-repo, split by validation domain later | large apparent LOC reduction | high: owns local test governance and package-test policy | default gates would need external validator/schema sync | none; repo-only validator |
| `src/repo_docs_validation.rs` | 188 | in-repo | minor | high: docs routes are local file policy | docs CI would require external tool updates for route changes | none |
| `src/repo_issue_sync.rs` | 742 | in-repo | medium | high: Beads/GitHub contract is project-local state | sync workflow would need external release for local policy changes | none |
| `src/repo_nu_lint.rs` | 56 | in-repo wrapper | trivial | low, but too small to justify a repo | no meaningful benefit from extraction | none |
| `src/repo_plugin_build.rs` | 474 | in-repo | medium | high: tracked wasm, sync stamps, runtime plugin paths, and materialization APIs move together | wasm sync failures would become cross-repo failures | none for normal users; maintainer-only |
| `src/repo_rust_budget.rs` | 378 | in-repo | medium | high: budget families and allowed paths are local | scorecard updates would need external release sync | none |
| `src/repo_rust_commands.rs` | 226 | in-repo wrapper | small | medium: command defaults encode local workspace paths and lanes | maintainer shell convenience would depend on external routing | none |
| `src/repo_sweep_runner.rs` | 608 | in-repo, shrink later if weak lanes remain | medium apparent LOC reduction | high: sweeps run local runtime/config matrices | release confidence would depend on external runner matching local layout | none |
| `src/repo_test_runner.rs` | 557 | in-repo | medium | high: test lanes and validators are local policy | default/sweep lane drift likely if externalized | none |
| `src/repo_update_workflow.rs` | 1,417 | in-repo, modularize later | large apparent LOC reduction | very high: mutates pins, vendored assets, canaries, and activation flow | release/update flow would become fragile across repos | none |
| `src/repo_version_bump.rs` | 467 | in-repo | medium | very high: release tags, changelog, version constants, and README must move atomically | release automation must stay coupled to the repo being tagged | none |
| `src/workspace_session_contract.rs` | 203 | in-repo | small | high: validates local layout metadata and runtime assets | workspace CI would require external schema lockstep | none |
| `src/lib.rs` | 13 | in-repo | trivial | none by itself | no independent value | none |
| `tests/repo_upgrade_contract.rs` | 239 | in-repo | small | high: upgrade note fixtures are local release history | release-gate test would track external crate version | none |

### Repo-Only Command Matrix

| Maintainer command | Selected residency | Off-repo LOC effect | Version-skew risk | CI/devShell impact | User-runtime impact |
| --- | --- | --- | --- | --- | --- |
| `yzx dev build_pane_orchestrator [--sync]` | in-repo maintainer crate | medium if moved with plugin build code | very high: sync stamp and tracked wasm live here | must stay aligned with current pane-orchestrator source and runtime asset paths | none; users consume the synced artifact |
| `yzx dev bump VERSION` | in-repo maintainer crate | medium | very high: tags, changelog, README, and version constants must be transactional | release workflow must not wait on external tool release | none |
| `yzx dev lint_nu [paths...]` | in-repo maintainer wrapper | trivial | low | small wrapper, not worth externalizing | none |
| `yzx dev rust <fmt\|check\|test>` | in-repo maintainer wrapper | small | medium: target defaults are local workspace policy | keeps direct maintainer loop stable | none |
| `yzx dev sync_issues [--dry-run]` | in-repo maintainer crate | medium | high: Beads/GitHub mapping is local project policy | sync policy changes land with repo changes | none |
| `yzx dev test [options]` | in-repo maintainer crate | medium | high: test lanes and validators are local contracts | default/sweep lanes stay reproducible from checkout | none |
| `yzx dev update [options]` | in-repo maintainer crate | large | very high: mutates lockfiles, vendored assets, canaries, and activation flow | repo update stays atomic with local package contracts | none |
| `yzx_repo_validator validate-*` | in-repo maintainer crate | large if moved wholesale | very high: every validator is tied to current checked-in contracts | CI uses the validator compiled from the same commit it validates | none |

### Accepted Path

- keep `yazelix_maintainer` in-repo and outside default user packaging
- keep `yzx_repo_validator` and `yzx_repo_maintainer` built from the same commit they validate
- keep installed/runtime `yzx dev` limited to runtime diagnostics
- shrink maintainer LOC by splitting `repo_contract_validation.rs`, `repo_validation.rs`, `repo_update_workflow.rs`, and `repo_sweep_runner.rs` before reconsidering an external child repo
- classify any future reusable dev helper as external only if it has a stable checkout-facing API and does not need to change when Yazelix contracts change

## Follow-Up

The original in-repo crate split implementation belongs in `yazelix-9opk.5.1`.

The Rust ownership and LOC budget in `yazelix-9opk.6` should wait until the in-repo maintainer crate split lands, otherwise it will codify the current mixed crate shape.

The post-re-evaluation shrink pass is `yazelix-vyas.1`.
