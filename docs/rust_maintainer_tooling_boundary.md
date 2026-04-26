# Rust Maintainer Tooling Boundary

Decision date: 2026-04-26

Tracked by: `yazelix-9opk.5`

## Decision

Keep maintainer-only Rust tooling in this repository, but split it out of the product runtime crate.

The target shape is:

- `yazelix_core`: product/runtime crate, shipped helper binaries, user command behavior, config/materialization/runtime contracts, and user-facing doctor logic
- `yazelix_maintainer`: in-repo maintainer crate for repo validators, release/update automation, Beads/GitHub sync, sweep runners, plugin wasm sync, and maintainer test orchestration
- no separate repository for now

This is a separation decision, not a deletion decision. The maintainer code is real and release-critical; the problem is that it currently lives in the same crate as runtime code and therefore inflates product ownership, package-time build surface, and Rust LOC accounting.

## First-Principles Rationale

Runtime users need a small, stable helper surface: `yzx`, `yzx_core`, and `yzx_control`. They do not need to understand or conceptually own release bumping, issue sync, spec validation, CI contract checks, visual sweep orchestration, or pane-orchestrator wasm synchronization.

Maintainers need those tools to live close to the repository contracts they defend. Offloading them to another repository would reduce local LOC on paper, but it would split validators from the files they validate and create version-skew risk between Yazelix, its release process, and its CI checks.

The pragmatic split is therefore an in-repo crate boundary. It makes runtime ownership smaller without weakening reproducible releases or forcing cross-repo coordination for every contract change.

## Runtime Package Impact

Current state:

- `packaging/mk_runtime_tree.nix` exposes only `yzx`, `yzx_core`, and `yzx_control` from the Rust helper package
- `packaging/rust_core_helper.nix` still builds the whole `yazelix_core` crate, including `yzx_repo_validator`, `yzx_repo_maintainer`, and the `repo_*` modules
- package-time Rust checks therefore see maintainer tooling even though users do not call it

Target state after the split:

- runtime/package builds should target only the product crate and shipped helper binaries
- CI and `yzx dev` should invoke maintainer commands through `yazelix_maintainer`
- maintainer commands may depend on `yazelix_core` for product contract APIs, but `yazelix_core` must not depend on `yazelix_maintainer`
- default Rust package-time tests should not require host-only maintainer tools such as Nix, GitHub CLI, Beads, or Home Manager

## Subsystem Decisions

| Subsystem | Current path | Decision | Offload decision | Rationale |
| --- | --- | --- | --- | --- |
| Validator dispatcher | `src/bin/yzx_repo_validator.rs` | move to `yazelix_maintainer` | reject external repo | CI entrypoint is repo-specific and validates local files, specs, packages, and release rules |
| Maintainer dispatcher | `src/bin/yzx_repo_maintainer.rs` | move to `yazelix_maintainer` | reject external repo | Local dev workflow wrapper for Beads/GitHub sync, tests, release bump, updates, and plugin sync |
| Repo contract validators | `repo_contract_validation.rs` | move to `yazelix_maintainer` | reject external repo | Largest maintainer file; all checks are tied to this repo's Nix, README, Home Manager, release, and package contracts |
| Generic repo validation | `repo_validation.rs` | move to `yazelix_maintainer` | reject external repo | Spec/test traceability and package-test-purity are repo policy, not runtime product behavior |
| Issue sync | `repo_issue_sync.rs` | move to `yazelix_maintainer` | reject external repo | Beads/GitHub mapping is local workflow state and should not become a separately versioned tool |
| Nushell lint wrapper | `repo_nu_lint.rs` | move to `yazelix_maintainer` | reject external repo | Thin repo-local maintainer command around checked-in Nu files |
| Pane-orchestrator build/sync | `repo_plugin_build.rs` | move to `yazelix_maintainer` | reject external repo | Sync stamp and tracked wasm are part of this repository; command can depend on `yazelix_core` materialization APIs |
| Sweep runner | `repo_sweep_runner.rs` | move to `yazelix_maintainer` | reject external repo | Runs local runtime/config matrices and visual checks against this checkout |
| Test runner | `repo_test_runner.rs` | move to `yazelix_maintainer` | reject external repo | Maintainer orchestration over local validator/test surfaces, not shipped product behavior |
| Update workflow | `repo_update_workflow.rs` | move to `yazelix_maintainer` | reject external repo | Writes local pins, vendored plugins, README surface, and canary materialization |
| Version bump workflow | `repo_version_bump.rs` | move to `yazelix_maintainer` | reject external repo | Transactional release policy must stay in the repo that owns tags, changelog, and upgrade notes |
| Workspace session validator | `workspace_session_contract.rs` | move to `yazelix_maintainer` unless a runtime caller appears | reject external repo | Validator-only owner; it can call runtime-owned workspace asset checks through `yazelix_core` |
| Workspace asset checks | `workspace_asset_contract.rs` | keep in `yazelix_core` | reject external repo | Used by user-facing `yzx doctor`, so it is product runtime behavior |
| Layout family contract | `layout_family_contract.rs` | keep in `yazelix_core` | reject external repo | Used by runtime workspace asset checks and doctor reporting |
| Profile commands | `profile_commands.rs` | keep in `yazelix_core` for now | reject external repo | `yzx_control profile` is used by live startup/profile instrumentation, not only repo validation |

## Implementation Boundary

The accepted implementation should be a mechanical crate split, not a rewrite:

- add `rust_core/yazelix_maintainer`
- move maintainer-only modules and bins into that crate
- update CI and `nushell/scripts/yzx/dev.nu` to call `cargo run -p yazelix_maintainer --bin yzx_repo_validator` or `yzx_repo_maintainer`
- keep public command names stable: `yzx_repo_validator` and `yzx_repo_maintainer`
- update `packaging/rust_core_helper.nix` so runtime package builds and tests only the product crate/binaries
- keep `yazelix_maintainer -> yazelix_core` as the only dependency direction

Do not split the repository until there is a concrete problem this in-repo crate boundary cannot solve. Current pressure is ownership clarity and package/runtime separation, not independent versioning.

## Follow-Up

Implementation belongs in `yazelix-9opk.5.1`.

The Rust ownership and LOC budget in `yazelix-9opk.6` should wait until the in-repo maintainer crate split lands, otherwise it will codify the current mixed crate shape.
