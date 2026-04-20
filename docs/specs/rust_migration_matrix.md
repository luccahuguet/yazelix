# Rust Migration Matrix

## Summary

Yazelix should migrate to Rust by moving typed, deterministic runtime-core decisions behind the Rust/Nushell bridge first, while keeping public command UX, host integration, and text-heavy rendering in Nushell unless a narrower structured subcore proves worth extracting.

Recommended first slice:

1. `yazelix.toml` loading, defaults, and schema-backed normalization
2. config-state hashing and invalidation
3. generated runtime materialization planning and managed writes
4. preserve the current Nushell profile/report boundaries while Rust replaces the inner work

This is a v15.x insertion plan that feeds a Rust-forward v16. It is not a big-bang `yzx` CLI rewrite.

## Why

Rust is valuable where Yazelix needs strong types, deterministic serialization, predictable error classes, and cheap parity tests. It is less valuable where the hard part is shell/terminal host integration, user-facing command prose, Nix packaging, or plain text-template assembly.

The repo has already trimmed old migration and backend clutter. The next step is to avoid freezing transitional Nushell complexity into Rust by mistake. This matrix decides what should move, what should stay, and what should wait.

## Scope

- classify the main surviving Nushell surfaces by Rust fitness, payoff, risk, and timing
- define the first Rust slice order after the bridge contract
- identify surfaces that should stay Nushell or Nix for now
- distinguish v15.x helper-backed insertion from v16-or-later broader rewrites
- connect later beads to one migration sequence

## Decision Rules

Use Rust first when a surface has most of these traits:

- typed records or schema-backed inputs
- deterministic normalization, hashing, or planning
- repeated parity cases that can be fixture-tested
- current Nushell code is doing domain modeling rather than host orchestration
- machine-readable output can replace duplicated parsing or classification

Keep Nushell, Nix, POSIX shell, or shipped data when a surface has most of these traits:

- public command UX and remediation text are the main value
- behavior is mostly process spawning, shell activation, terminal quirks, or desktop integration
- the code is plain template stitching with low defect history
- the source of truth is a Nix/package contract or static shipped file
- the work would create a second public CLI owner before the inner contract is proven

Delete or narrow before porting when a surface still carries old product assumptions, transitional compatibility, or duplicate ownership.

## Rust Dependency Gate

Every Rust implementation bead must start with a crate-vs-in-house decision before code is written.

The decision should record:

- production crates and why each one is worth the dependency cost
- dev-only crates and which tests they unlock
- logic that will be built in-house because it is small, domain-specific, or safer to own
- rejected crates or frameworks when the obvious option is intentionally not used
- Nix packaging impact, including whether the crate set changes the product closure or vendoring/hash work

Default posture:

- Use in-house/std for Yazelix-specific config normalization, state classification, runtime materialization decisions, and bridge envelope shaping.
- Use crates for stable external formats or well-scoped infrastructure, such as TOML parsing, JSON serialization, SHA hashing, explicit error types, and focused test helpers.
- Avoid broad frameworks for the private helper until a public CLI or async/process boundary genuinely needs them.

If the crate list changes mid-bead, update the bead or linked spec before continuing.

## Migration Matrix

| Surface | Current owners | Rust fitness | Payoff | Risk | Decision | Timing and beads |
| --- | --- | --- | --- | --- | --- | --- |
| Config loading, defaults, schema-backed normalization | `config_parser.nu`, `config_contract.nu`, `config_schema.nu`, `config_diagnostics.nu`, `config_surfaces.nu`, `config_metadata/main_config_contract.toml`, `yazelix_default.toml` | High | High | Medium | Move behind the bridge first. Rust should output the normalized config record and structured diagnostics while Nushell keeps user-facing rendering. | First v15.x slice: `yazelix-kt5.2.1` |
| Config-state hashing and invalidation | `config_state.nu`, `generated_runtime_state.nu`, `config_contract.nu` rebuild-required metadata | High | High | Low-medium | Move after normalized config output exists. Rust should own rebuild-key extraction, config/runtime hash composition, cached-state comparison, and refresh reasons. | First v15.x slice part 2: `yazelix-kt5.2.2` |
| Generated runtime materialization planning and managed writes | `generated_runtime_state.nu`, `atomic_writes.nu`, generated-state callers in startup/doctor | Medium-high for planning and write ownership; lower for render text | High | Medium | Move the decision layer and managed-write orchestration behind Rust. Do not force Yazi/Zellij/terminal text renderers into Rust as part of the same step. | First v15.x slice part 3: `yazelix-kt5.2.3` |
| Yazi, Zellij, terminal, initializer, and layout renderers | `yazi_config_merger.nu`, `zellij_config_merger.nu`, `terminal_configs.nu`, `terminal_renderers.nu`, `layout_generator.nu`, `initializers.nu` | Mixed | Medium | Medium-high | Keep renderer/template assembly in Nushell. Rust owns the upstream config/state/materialization decisions, but not generated text, plugin/layout copying, per-launch terminal randomness, or shell initializer rendering. Extract only a narrow structured subcore later if repeated defects justify it. | Locked by `yazelix-kt5.5` |
| Runtime dependency checking and doctor/preflight reasoning | `runtime_contract_checker.nu`, `doctor.nu`, `install_ownership_report.nu`, `doctor_helix.nu`, `runtime_distribution_capabilities.nu` | Medium-high | Medium-high | Medium | Move the classification engine later, not the prose-heavy doctor UX first. Rust can return structured findings consumed by launch, doctor, and install smoke. | Later v15.x candidate: `yazelix-kt5.3` |
| Launch, enter, run, env, and environment bootstrap | `launch_yazelix.nu`, `start_yazelix.nu`, `start_yazelix_inner.nu`, `environment_bootstrap.nu`, `runtime_env.nu`, `terminal_launcher.nu`, `yzx/launch.nu`, `yzx/enter.nu`, `yzx/env.nu`, `yzx/run.nu` | Medium | Medium | High | Defer broad porting. Keep Nushell as the process, profile, terminal, and shell-boundary owner until the config/runtime-core bridge has proven itself. | Late v15.x only if seams are clean; otherwise v16: `yazelix-kt5.4` |
| Public `yzx` CLI command surface | `core/yazelix.nu`, `yzx/*.nu` | Low for first slice | Low-medium | High | Keep Nushell as the public CLI owner. Do not start with clap. Revisit broad Rust/clap only after helper-backed slices prove value. | v16-or-later evaluation: `yazelix-2ex.1.11` |
| Extern bridge and completion generation | `nushell_externs.nu`, shellhook sync paths | Medium | Medium | Medium | Treat as a Rust/Nushell bridge seam, not isolated startup caching. Decide whether generation belongs on startup after the helper contract is in use. | During bridge work: `yazelix-4xp1.3` |
| Workspace/session orchestration | `rust_plugins/zellij_pane_orchestrator/`, `integrations/zellij.nu`, `integrations/yazi*.nu`, `zellij_wrappers/` | Already Rust where live state needs it | Medium | Medium | Keep the pane orchestrator as the Rust owner for live Zellij session state. Do not mix it into `rust_core/` or the config/runtime helper. | Continue under pane-orchestrator beads, not `kt5.2` |
| Front-door UX, command palette, popup, tutor, keys, screen, welcome animation | `ascii_art.nu`, `yzx/menu.nu`, `yzx/popup.nu`, `yzx/screen.nu`, `yzx/tutor.nu`, `yzx/keys.nu`, `upgrade_summary.nu` | Low-mixed | Low-medium | Medium | Keep Nushell unless a narrow algorithmic component becomes painful. UX polish and command presentation are not the first Rust migration target. | Not in first Rust sequence |
| Distribution and host integration | `flake.nix`, `packaging/`, `home_manager/`, `shells/`, `yzx/desktop.nu`, update commands | Low for Rust core; Nix/POSIX/Nushell fit remains high | Medium | High | Keep Nix as package owner, POSIX shell as narrow launcher glue, and Nushell as user-facing integration owner. Rust may be a packaged private helper only. | Keep outside first Rust sequence |
| Maintainer workflow, validators, release/update, issue sync | `nushell/scripts/dev/`, `nushell/scripts/maintainer/`, GitHub Actions | Low-mixed | Low | Medium | Keep as maintainer tooling unless a specific validator or parser becomes reusable product logic. Do not port tests just to port tests. | Not a product migration slice |
| Config migration engine revival | Historical/deleted migration surfaces and `yazelix-j498` planning | Low by default | Low unless user pain returns | High product-risk | Do not revive broad automatic migration machinery. Consider only a tiny schema-versioned Rust adjunct if visible diagnostics still prove insufficient. | Backlog/conditional: `yazelix-j498` |
| Shipped runtime data and contracts | `config_metadata/`, `configs/`, `user_configs/`, `yazelix_default.toml`, upgrade notes, assets | Data, not code-owner surface | High as input | Low | Keep as source-of-truth data. Rust should consume contracts and fixtures, not replace them with hardcoded defaults. | Used by `kt5.2.1` and later parity tests |

## Renderer Ownership Decision

`yazelix-kt5.5` locks the current renderer boundary: do not port Yazelix's generated text/template surfaces to Rust just to reduce Nushell LOC.

Rust owns the structured decision layer that is already behind `yzx_core`: normalized config, rebuild-state hashing, refresh/no-refresh classification, expected artifact planning, and materialized-state recording. Nushell owns the renderer layer that turns those decisions into product files and host-facing glue.

Current ownership:

| Surface | Decision | Rust-owned inputs/outputs | Nushell-owned renderer/glue |
| --- | --- | --- | --- |
| Runtime materialization | Split ownership | `runtime-materialization.plan` and `runtime-materialization.apply` decide freshness, expected artifacts, and recorded state | `generated_runtime_state.nu` preserves profile labels and invokes the surviving renderers |
| Yazi config generation | Keep Nushell | Normalized config values and materialization plan inputs | TOML merge policy, `init.lua` assembly, bundled plugin/flavor copying, runtime-root placeholder rendering, user override discovery |
| Zellij config and layout generation | Keep Nushell | Normalized config values and materialization plan inputs | Base-source selection, semantic block extraction, owned setting rendering, plugin/load_plugins assembly, layout placeholder expansion, plugin wasm runtime sync |
| Terminal config generation | Keep Nushell | Normalized terminal settings | Ghostty/Kitty/WezTerm/Alacritty/Foot text rendering, per-launch Ghostty random cursor rerolls, shader asset generation/sync |
| Helix managed config generation | Keep Nushell | Normalized managed-editor policy if needed later | TOML merge, reveal binding enforcement, import notice state, generated Helix config write |
| Shell initializers and extern bridge | Keep Nushell/POSIX | None for now beyond runtime/config paths | shell-specific init files, POSIX launch wrappers, Nushell `yzx` extern rendering and fingerprinted refresh |

Revisit triggers:

- A renderer accumulates repeated correctness defects that are hard to defend with focused Nushell tests.
- A surface needs a real parser/AST model, such as a future Zellij KDL merge that cannot be made robust with the existing semantic-block owner split.
- The same structured renderer logic is needed by both product runtime and tests/docs in a way that makes a Rust library cheaper than duplicate Nu helpers.
- The public CLI becomes Rust-owned in a later v16-or-later decision, at which point completion/extern rendering may need a different source of truth.

Non-triggers:

- The file is long.
- The code is mostly string assembly.
- A Rust helper already exists nearby.
- A port would add Rust LOC while leaving the same Nushell renderer and parity glue in place.

## First-Slice Implementation Order

### 1. Config Normalization

Implement `config.normalize` behind the bridge contract.

Inputs should include explicit paths for:

- active user config
- default config
- main config contract
- config root
- runtime root when needed for contract discovery

Output should be the normalized config record plus structured config diagnostics. Nushell should keep the visible startup and doctor rendering.

Parity should defend:

- defaults from the shipped template and contract
- parser-key shaping used by current consumers
- enum, range, nullable, boolean-to-string, badge-text, and list validations
- unknown and removed key diagnostics
- no automatic migration behavior

### 2. Config-State Hashing

Implement `config_state.compute` after normalized config output is stable.

Rust should own:

- rebuild-required field extraction from the config contract
- deterministic serialization of rebuild-relevant config
- config hash
- runtime hash composition
- cached materialized-state parsing
- refresh/no-refresh classification and reason codes

Parity should defend unchanged inputs, config-only changes, runtime-only changes, combined changes, ignored non-rebuild keys, and missing or malformed cache state.

### 3. Materialization Planning And Managed Writes

Implement `runtime_materialization.plan` and keep `runtime_materialization.apply` limited to materialized-state recording and expected-artifact verification unless a future renderer bead explicitly moves a specific generated file family to Rust.

Rust should own:

- whether generated state needs work
- which managed artifacts are expected
- whether warm paths can skip rewrites
- recording the applied materialized state
- atomic-write coordination for Rust-owned managed files

Nushell keeps Yazi, Zellij, terminal, Helix, shell initializer, layout, and extern text renderers for the current v15.x line. Rust must not become a second writer for those generated files without a narrower renderer-ownership bead.

### 4. Profile And Report Preservation

Every first-slice step must preserve the existing startup profile/report boundaries at the Nushell wrapper layer.

Examples:

- `bootstrap` / `prepare.parse_config`
- `bootstrap` / `prepare.compute_config_state`
- `generated_runtime_state` / `compute_config_state`
- `generated_runtime_state` / `generate_yazi_config`
- `generated_runtime_state` / `generate_zellij_config`

Rust may report additive metrics inside bridge data, but the startup profile schema and high-level labels remain Nushell-owned.

## v15.x Versus v16 Boundary

Safe v15.x Rust insertion:

- private helper binary under `libexec/`
- no public command rename
- no broad clap rewrite
- parity fixtures before removing the Nushell implementation
- one command family using the bridge at a time
- profile reports comparable before and after

Likely v16-or-later work:

- replacing the whole `yzx` CLI with a Rust CLI
- moving launch/bootstrap process orchestration wholesale
- collapsing large text-renderer families into Rust without a smaller proven model
- extracting Rust crates into a separate repository or product
- changing the public runtime/update/install contract because Rust makes it convenient

## Non-goals

- treating Rust as a reason to reintroduce removed migration machinery
- rewriting public command UX before the private helper seams work
- moving Nix packaging ownership into Rust
- replacing the Zellij pane orchestrator workflow with the new core helper workflow
- porting low-risk template assembly just to reduce Nushell LOC

## Acceptance Cases

1. A maintainer can identify the first three Rust implementation slices and the bead that owns each one.
2. A maintainer can tell which major surfaces should stay Nushell, Nix, POSIX shell, or shipped data for now.
3. The matrix explains why config/state/materialization comes before launch, doctor, generators, or a clap rewrite.
4. Later Rust beads can cite this matrix instead of reopening the keep/bridge/rewrite decision from scratch.
5. The matrix remains compatible with the Rust/Nushell bridge contract and v15 trimmed runtime contract.

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual review against `docs/specs/rust_nushell_bridge_contract.md`
- manual review against `docs/specs/cross_language_runtime_ownership.md`
- manual review against `docs/specs/v15_trimmed_runtime_contract.md`
- manual review against `docs/specs/yzx_command_surface_backend_coupling.md`
- manual review against `docs/subsystem_code_inventory.md`

## Traceability

- Bead: `yazelix-kt5.1`
- Bead: `yazelix-kt5.5`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- Should a future Zellij KDL-specific parser/renderer subcore be worth extracting, or did the `yazelix-jheg` Nu-side owner split remove enough risk?
- Should `config.normalize` and `config_state.compute` be separate helper commands from day one, or should the first implementation expose one combined command and split later?
- Which parity fixtures should become stable cross-language golden files versus ordinary Rust unit fixtures?
