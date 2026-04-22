# Maintainer And Validator Nushell Budget

## Summary

This document defines the delete-first budget for the maintainer, `yzx dev`,
and deterministic validator Nushell families.

These surfaces are not product runtime, but they still count toward the hard
under-`5k` Nu floor. The rule here is the same as everywhere else: Nu may stay
only where it is the honest shell/process owner. Deterministic validation,
policy, routing, and test-helper logic must move to Rust or be deleted.

The old governed Nu test files are already gone. What remains here is the shell
orchestration around `yzx dev`, sweep/E2E runners, maintainer tooling, and
deterministic validators that have not been ported yet.

## Scope

In scope:

- `nushell/scripts/maintainer/*.nu`
- `nushell/scripts/yzx/dev.nu`
- non-test, non-validator helpers under `nushell/scripts/dev/`
- deterministic validators under `nushell/scripts/dev/validate*.nu`

Out of scope:

- governed Nu tests, which are budgeted in
  `docs/specs/rust_owned_test_migration_budget.md`
- product/runtime launch, setup, front-door, and integration owners

## Current Measured Surface

Measured on `2026-04-22`:

| Family | Current LOC | Hard target LOC | Main follow-up |
| --- | ---: | ---: | --- |
| Maintainer and `yzx dev` shell orchestration | `3,996` | `1,200` | `yazelix-8ih0` |
| Deterministic validators and contract linters | `837` | `0` | `yazelix-rdn7.4.6` |

## Maintainer And `yzx dev` Budget

`yazelix-8ih0.1` should use this keep-vs-cut table.

### Allowlisted survivors

These are allowed to survive only as fixed argv or shell/process orchestration:

- `maintainer/issue_sync.nu`
- `maintainer/issue_bead_contract.nu`
- `maintainer/version_bump.nu`
- `maintainer/update_workflow.nu`
- `maintainer/repo_checkout.nu`
- `maintainer/plugin_build.nu`
- a much smaller `maintainer/test_runner.nu`
- a much smaller `yzx/dev.nu`
- only the temporary shell-heavy runner scripts named in
  `config_metadata/nushell_budget.toml`

### Forced deletion or migration targets

These should not survive as broad owned Nu surfaces:

- dynamic dispatch and suite-selection policy inside `maintainer/test_runner.nu`
- broad route planning and policy in `yzx/dev.nu`
- deterministic helper libraries that mainly support governed Nu tests or
  validators:
  - `dev/config_normalize_test_helpers.nu`
  - `dev/materialization_dev_helpers.nu`
  - `dev/yzx_test_helpers.nu`
- manual/demo helpers that do not justify permanent governed ownership:
  - `dev/record_demo.nu`
  - `dev/record_demo_fonts.nu`
- thin update helpers that can fold into one canonical maintainer owner:
  - `dev/update_yazi_plugins.nu`

### Maintainer floor rules

1. Keep only shell-, repo-, or external-tool-heavy logic in Nu
2. Do not keep deterministic routing or policy in Nu just because the command
   touches `git`, `gh`, `bd`, or Nix later
3. Route first-party Rust tests through `cargo nextest run` by default
4. Shrink `yzx dev` to a thin public shell router above canonical owners
5. Shrink `test_runner.nu` to fixed suite orchestration and logging only
6. Do not recreate governed Nu test entrypoints inside maintainer tooling after
   the governed Nu suite has been deleted

### `yzx dev` Shell-Floor Split

`yazelix-8ih0.7` should treat the public `yzx dev` surface like this:

| Subsurface | Keep vs cut | Why |
| --- | --- | --- |
| `yzx dev test` public argv parsing and handoff | keep as thin Nu router | it still shells into Nix, Nu, and external tools, but policy should live in the owned runner inventory |
| `yzx dev profile` startup harness entry | keep temporarily | it still executes the real startup shell boundary and records shell-local profile steps |
| `yzx dev update`, `sync_issues`, `build_pane_orchestrator`, `bump` | keep as thin Nu router | these are external-tool-heavy maintainer commands and should not gain extra policy in `yzx/dev.nu` |
| hidden suite membership, lane policy, or test selection logic | cut | this belongs in fixed inventory files or canonical owner modules, not in the public entry router |
| helper logic that only exists to support deleted governed Nu tests | cut | the governed Nu suite is gone, so this helper debt no longer has a justification |

## Validator Budget

`yazelix-rdn7.4.6.1` should use this deletion budget:

### Target

All `837` current lines of deterministic validators should leave Nu. The
target is `0` long-term governed Nu validator LOC.

### Split

| Validator cluster | Current examples | Budget judgment | Owning follow-up |
| --- | --- | --- | --- |
| Spec and test traceability validators | `validate_specs.nu`, `validate_default_test_traceability.nu`, `validate_rust_test_traceability.nu`, `validate_default_test_count_budget.nu` | Rust-port first | `yazelix-rdn7.4.6.2` |
| Config, upgrade, and package validators | `validate_config_surface_contract.nu`, `validate_upgrade_contract.nu`, `validate_flake_interface.nu`, `validate_nixpkgs_package.nu`, `validate_nixpkgs_submission.nu`, `validate_flake_install.nu` | Rust-port first | `yazelix-rdn7.4.6.3` |
| Installed-runtime validator | Rust `yzx_repo_validator validate-installed-runtime-contract`; deleted `validate_installed_runtime_contract.nu` | completed Rust owner cut; fixed external probes remain inside Rust | `yazelix-rdn7.4.6.6` |
| README surface/version validator | Rust `yzx_repo_validator validate-readme-version`; Rust `yzx_repo_maintainer sync-readme-surface`; deleted `validate_readme_version.nu` and `readme_surface.nu` | completed Rust owner cut | `yazelix-8ih0.2` |

### Validator floor rules

1. Deterministic source scanning, spec parsing, and contract linting do not get
   to stay in Nu
2. If a validator still needs to execute a real runtime shell boundary, isolate
   that shell probe and port the rest
3. Delete duplicated validator helper logic after the Rust owners land

### Validator Dependency Decision

The traceability-validator port should add no new crates.

- production crates reused:
  - `serde`
  - `serde_json`
  - `toml`
- in-house logic kept:
  - markdown heading and traceability parsing
  - governed-test metadata scanning
  - repo-relative file discovery
- rejected alternatives:
  - `regex`, because the current validator grammar is narrow enough for direct
    string parsing
  - `walkdir`, because the repo file sets are small and std recursion is
    adequate
- packaging impact:
  - one maintainer-focused Rust validator binary is acceptable
  - the old Nu parser ownership must still be deleted or demoted

The config and upgrade validator port should also add no new crates.

- production crates reused:
  - `serde_json`
  - `toml`
- in-house logic kept:
  - Home Manager parity checks via fixed `nix eval` probes
  - upgrade-notes and changelog contract checks
  - generated-state fixture validation through existing Rust `config_state`
    ownership
- rejected alternatives:
  - a second dedicated validator binary, because the existing
    `yzx_repo_validator` already owns maintainer-facing deterministic repo
    checks
  - keeping `yzx_core_bridge.nu` or config/upgrade parsing in Nu, because that
    would preserve redundant validator ownership after Rust already owns the
    live config-state logic
- packaging impact:
  - the surviving Nu validators may remain only as thin compatibility shims
    that invoke the Rust validator binary
  - any remaining `git` and `nix` probe layer must stay explicit and fixed-argv

The installed-runtime and README surface cuts also add no new crates.

- production crates reused:
  - `serde`
  - `serde_json`
  - `toml`
- in-house logic kept:
  - fixed external command execution for `nix flake show`, `nix build`, and the
    built `yzx why` smoke
  - README title and generated latest-series block rendering
- rejected alternatives:
  - keeping Nu compatibility shims for deleted validator commands, because CI
    and pre-commit can call the Rust binary directly
  - adding a broad command framework, because these are private maintainer
    binaries with small fixed command surfaces
- packaging impact:
  - `yzx_repo_validator` now owns the deterministic installed-runtime and
    README validation surfaces
  - `yzx_repo_maintainer` owns the README surface mutation path used by
    release/update workflows

## Installed-Runtime Boundary

`yazelix-rdn7.4.6.5` should use this owner split for
`validate_installed_runtime_contract.nu`.

### Rust-owned validator work

These checks do not justify a Nu owner:

- repo-tree path existence and path absence assertions
- file-content contract checks against tracked sources such as
  `shells/posix/yzx_cli.sh`, `runtime_env.sh`, `setup/environment.nu`, and
  `packaging/mk_runtime_tree.nix`
- flake package/app surface checks from fixed `nix flake show --json`
- built-output path inspection after fixed `nix build --no-link --print-out-paths`
- the built-CLI smoke command itself when it is launched as fixed argv and its
  stdout contract is checked in Rust

### Explicit live probes that still remain

These are still part of the validator, but they are not a reason to keep the
owner in Nu:

- `nix flake show --json`
- `nix build --no-link --print-out-paths .#runtime`
- `nix build --no-link --print-out-paths .#yazelix`
- `env YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1 <built-yzx> why`

The honest boundary is that these are fixed external build/runtime probes owned
by Rust, not shell-authored logic that needs a surviving Nu seam.

### Decision

No surviving Nu validator owner is justified here.

`yazelix-rdn7.4.6.6` ports the installed-runtime validator end-to-end to Rust,
keeps required external command probes explicit and fixed-argv inside Rust, and
deletes the Nu owner. No compatibility shim is retained.

## Maintainer Decision Cuts From `yazelix-8ih0`

`yazelix-8ih0.2` moved deterministic README mutation and validation to Rust and
deleted `maintainer/readme_surface.nu` plus `dev/validate_readme_version.nu`.

`yazelix-8ih0.4` records a no-go for broad Rust ownership of issue sync and
repo checkout today: they remain external-tool-heavy `gh`, `bd`, `git`, and
repo-shell orchestration. Future cuts should extract deterministic mapping or
selection logic only when it deletes Nu policy, not when it merely hides the
same tools behind Rust.

`yazelix-8ih0.5` records a no-go for broad Rust ownership of plugin build and
update workflow today. These paths are cargo/Nix/git/build orchestration; Rust
should own deterministic request construction and validation, while Nu may
temporarily retain fixed process orchestration until a deletion lane is named.

`yazelix-8ih0.6` deletes `utils/nix_detector.nu`. The retained maintainer Nix
check is a small fixed fail-fast function in `update_workflow.nu`; interactive
repair prompts and broad detector policy are not part of the canonical
maintainer floor.

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- validate-readme-version`
- `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- validate-installed-runtime-contract`
- later Rust validator/test-harness verification should be nextest-first under
  `docs/specs/rust_test_hardening_tools_decision.md`

## Traceability

- Bead: `yazelix-8ih0.1`
- Bead: `yazelix-8ih0.2`
- Bead: `yazelix-8ih0.4`
- Bead: `yazelix-8ih0.5`
- Bead: `yazelix-8ih0.6`
- Bead: `yazelix-rdn7.4.6.1`
- Bead: `yazelix-rdn7.4.6.6`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- validate-readme-version`
- Defended by: `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- validate-installed-runtime-contract`
- Informed by: `docs/specs/maintainer_harness_canonicalization_audit.md`
- Informed by: `docs/specs/provable_nushell_floor_budget.md`
