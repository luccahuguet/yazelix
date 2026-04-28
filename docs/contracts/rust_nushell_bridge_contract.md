# Rust/Nushell Bridge Contract

## Summary

Incremental Rust work in Yazelix should land behind private helper binaries that receive explicit inputs and return machine-readable output.

Nushell remains the startup/profile owner, root resolver, shell/process orchestrator, and user-facing renderer for surviving Nushell-owned command bodies. Rust owns deterministic typed core work for selected slices, public control-plane leaves that have fully moved, and the shared `yzx` command metadata used for help, palette inventory, and generated Nushell externs.

## Why

The Rust rewrite should not force a big-bang replacement of `yzx`, and it should not make each v15.x Rust slice invent a different shim. The first high-value slices are config loading/defaults/normalization, config-state hashing, generated-runtime planning/writes, and startup profile comparability. Those slices need one small bridge contract before implementation starts.

The bridge also limits new Nushell rewrite debt. Complex deterministic work can move to Rust without changing the visible command surface or making v15.0 pretend to be the Rust release.

## Scope

- future Rust core source layout for helper-style code
- the public versus private command boundary
- how Nushell invokes Rust helpers
- success, warning, error, logging, and exit-code contracts
- runtime/config/state root handoff
- build and distribution expectations for the packaged runtime
- profile/report preservation while Rust replaces inner work

## Contract Items

#### BRIDGE-001
- Type: boundary
- Status: live
- Owner: Nushell public surfaces plus private `yzx_core`
- Statement: Rust lands behind private helper commands while Nushell remains the
  user-facing owner for surviving Nu command bodies, shell/process
  orchestration, and prose rendering. A migration only counts when Rust becomes
  the single owner for the moved slice instead of creating a second public CLI
  registry
- Verification: automated `nu nushell/scripts/dev/test_yzx_core_commands.nu`;
  validator `yzx_repo_validator validate-contracts`

#### BRIDGE-002
- Type: boundary
- Status: live
- Owner: wrapper-to-helper invocation seam
- Statement: Wrappers call `yzx_core` by absolute runtime-root path with
  structured argv. The bridge must not assemble inline shell program bodies or
  route helper calls through `bash -lc` or `sh -c`
- Verification: automated
  `nushell/scripts/dev/test_shell_managed_config_contracts.nu`; automated
  `nushell/scripts/dev/test_yzx_workspace_commands.nu`

#### BRIDGE-003
- Type: behavior
- Status: live
- Owner: `yzx_core` machine-mode transport
- Statement: A successful helper call writes exactly one JSON success envelope
  to stdout and no prose. A failed helper call writes exactly one structured
  JSON error envelope to stderr with stable error classes and exit codes
- Verification: automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`; automated
  `nushell/scripts/dev/test_shell_managed_config_contracts.nu`

#### BRIDGE-004
- Type: ownership
- Status: live
- Owner: Rust metadata/extern lifecycle plus shellhook sync boundary
- Statement: Warm startup reuses a current generated `yzx` extern bridge without
  rerendering metadata. When stale, refresh runs
  `yzx_core yzx-command-metadata.sync-externs` and must not probe a second
  Nushell command registry
- Verification: automated
  `nushell/scripts/dev/test_shell_managed_config_contracts.nu`; validator
  `yzx_repo_validator validate-contracts`

#### BRIDGE-005
- Type: ownership
- Status: live
- Owner: Rust-written generated-state owners
- Statement: When Rust writes generated state, it may write only explicit
  Yazelix-managed paths passed by the wrapper, keep writes deterministic for
  identical inputs, and fail loudly instead of silently taking ownership of
  user-managed config
- Verification: automated
  `nushell/scripts/dev/test_yzx_generated_configs.nu`; automated
  `cargo test --manifest-path rust_core/Cargo.toml`

## Behavior

### Ownership Boundary

Nushell owns:

- surviving public `yzx` command bodies that still execute in Nushell
- launch, desktop, Home Manager, doctor, and maintainer UX that has not moved to Rust
- config root, runtime root, and state root resolution
- startup profile schema, report files, and step names
- user-facing prose, remediation text, and final `error make` rendering
- compatibility with existing generated-state and runtime-root contracts

Rust owns:

- shared public `yzx` command metadata for root help, palette inventory, and generated externs
- public control-plane leaf parsing and execution for `yzx env`, `yzx run`, and `yzx update*`
- typed parsing and normalization for selected config/runtime inputs
- deterministic config-state hashes and invalidation decisions
- generated-runtime materialization plans, and later managed writes when that slice is ready
- machine-readable diagnostics that Nushell can translate without re-deriving the same logic
- library-level unit tests for pure behavior and fixture parity

The bridge must not turn Rust into a second public CLI owner for surfaces that are still Nushell-owned. When a surface moves, Rust must become the single owner for that public metadata or leaf parser instead of depending on the old Nushell command tree for discovery.

### Repo Layout

New helper-oriented Rust code should live under `rust_core/`, not under `rust_plugins/`.

Recommended first layout:

```text
rust_core/
  Cargo.toml
  yazelix_core/
    Cargo.toml
    src/
      lib.rs
      bin/
        yzx_core.rs
```

`rust_plugins/` remains the home for Zellij wasm plugin components. The existing pane-orchestrator build/sync workflow is not the pattern for the Rust core helper.

The `yazelix_core` library contains pure typed behavior. The `yzx_core` binary is the private bridge executable invoked by Nushell. The binary may use a small argument parser, but it is not the beginning of a public clap replacement for `yzx`.

### Invocation

Nushell wrappers call the helper by absolute path resolved from the active runtime root:

- installed runtime: `$YAZELIX_RUNTIME_DIR/libexec/yzx_core`
- source checkout: the equivalent helper path produced by the maintainer build flow

The helper must not be discovered from ambient `PATH`, and it must not infer product behavior from ambient `HOME`, `XDG_*`, `YAZELIX_*`, or current working directory. Nushell resolves those values first and passes the exact paths through flags or environment variables owned by the wrapper.

Calls should use structured argv execution. They must not assemble inline quoted shell scripts or route through `bash -lc`, `sh -c`, or similar shell-string seams.

### Command Shape

Helper commands track the Rust-owned slices:

- `config.normalize`
- `config-state.compute`
- `config-state.record`
- `runtime-contract.evaluate`
- `startup-launch-preflight.evaluate`
- `runtime-env.compute`
- `runtime-materialization.plan`
- `runtime-materialization.materialize`
- `runtime-materialization.repair`
- `status.compute`
- `doctor-config.evaluate`
- `doctor-helix.evaluate`
- `doctor-runtime.evaluate`
- `install-ownership.evaluate`
- `zellij-render-plan.compute`
- `yazi-render-plan.compute`
- `yazi-materialization.generate`
- `zellij-materialization.generate`
- `yzx-command-metadata.list`
- `yzx-command-metadata.externs`
- `yzx-command-metadata.sync-externs`
- `yzx-command-metadata.help`

The exact user-facing `yzx` commands do not change when these helper commands land. Nushell maps the public command flow onto helper command ids internally.

Each helper call receives explicit paths for every owned surface it needs, such as:

- user config path
- default config path
- config contract path
- runtime root
- state root
- generated output root
- profile or report context if needed for metrics metadata

### JSON Contract

In machine mode, a successful helper call writes exactly one JSON object to stdout and no prose or ANSI output.

Success envelope:

```json
{
  "schema_version": 1,
  "command": "config.normalize",
  "status": "ok",
  "data": {},
  "warnings": []
}
```

Warning entries are structured objects. They should contain stable codes and data that Nushell can render in the current style without parsing English text.

Failure writes no stdout and writes exactly one JSON object to stderr.

Failure envelope:

```json
{
  "schema_version": 1,
  "command": "config.normalize",
  "status": "error",
  "error": {
    "class": "config",
    "code": "unknown_config_key",
    "message": "Unknown Yazelix config key",
    "remediation": "Remove the key or compare your config with the current template",
    "details": {}
  }
}
```

`class` values should stay small and stable:

- `usage`: bad helper argv or unsupported helper command
- `config`: invalid user config or unsupported config shape
- `io`: missing, unreadable, or unwritable explicit input/output path
- `runtime`: runtime tree or generated-state invariant failure
- `internal`: helper bug or impossible state

Exit codes:

- `0`: success
- `64`: usage error
- `65`: config or user-data validation error
- `66`: explicit input/output path error
- `70`: internal/runtime bug

Nushell wrappers classify the failure from the envelope first and the exit code second. If stderr is not valid JSON in machine mode, the wrapper treats that as an `internal` bridge failure and shows a clear Yazelix-owned error.

### Logging And Human Output

Machine mode is the default path used by Yazelix wrappers. It must keep stdout and stderr reserved for the JSON envelopes above.

If a direct maintainer debugging mode is useful, expose it separately through a flag such as `--human` or through a maintainer-only command. Do not mix human logs with machine-mode JSON.

### Startup Profile Preservation

Nushell remains the startup profile owner. Rust helper calls are wrapped inside the existing `profile_startup_step` boundaries instead of replacing the report schema.

Existing profile step names and meanings should remain comparable before and after each Rust slice. For example, the landed runtime materialization cut still appears under the same high-level `materialization_orchestrator` / `materialize_runtime_state` startup boundary even though Rust now owns the lifecycle itself.

Rust may return optional metrics inside the success `data`, but those metrics are additive and must not become a second startup profile format.

### Extern Bridge Timing

The generated Nushell `yzx` extern bridge is startup-owned glue, not command business logic. It remains inside the existing `shellhook` / `sync_yzx_extern_bridge` profile step so startup reports stay comparable while the bridge implementation changes.

Warm startup must not pay command metadata rendering when the generated extern bridge is already current. The Rust-owned sync command should perform only cheap generated-state checks, such as a Rust helper fingerprint and generated-file hash, before reusing the existing bridge.

When the command metadata is missing or stale, the sync path runs `yzx_core yzx-command-metadata.sync-externs`, which renders generated extern content from Rust-owned metadata and updates the bridge plus fingerprint. It must not spawn Nushell to inspect `core/yazelix.nu` or reconstitute a second command registry. Successful regeneration updates the generated bridge and its fingerprint atomically enough that a later warm startup can skip rendering and writes.

Refresh failure must be non-destructive. If a previous generated bridge exists, keep it instead of replacing it with an empty placeholder. If no bridge exists yet, create a minimal placeholder so managed Nushell config can still source the file and show the generation warning.

### Materialization And Writes

Rust may still expose smaller planning helpers when that is the cleanest seam, but a migration only counts once Rust owns the real writer lifecycle end-to-end. The landed runtime materialization cut proved that the bridge can collapse from a plan/apply split into one Rust-owned `plan` plus full-owner `materialize` and `repair` commands when the owner boundary becomes clear.

When Rust writes files, it must preserve the existing ownership rules:

- only write Yazelix-managed generated-state paths passed explicitly by Nushell
- do not take ownership of user-managed config files
- keep writes deterministic for identical inputs
- avoid rewriting unchanged files when the current generated-state contract depends on stable hashes and cheap startup checks
- fail loudly on ambiguous ownership instead of silently falling back

### Build And Distribution

The packaged runtime should ship a compiled `yzx_core` helper under `libexec/`. It should not expose the helper under `bin/` or `toolbin/`, because the public command remains `yzx`.

The helper build should be wired through Nix packaging as a product artifact, not copied from a local maintainer cache. The maintainer shell should provide the normal Rust check/test tools for the helper in addition to the existing wasm plugin workflow.

Source-checkout development may invoke a locally built helper, but installed runtimes must invoke the helper from their own runtime root so multiple installed revisions do not cross-call each other.

## Non-goals

- replacing the whole `yzx` CLI with clap in the first Rust slice
- moving launch, terminal selection, Home Manager update UX, or desktop-entry UX into Rust as part of this bridge
- moving the Zellij pane orchestrator into `rust_core/`
- reintroducing the deleted config migration engine
- making `yzx_core` a supported user command
- relying on ambient host state or local-only caches for product behavior

## Acceptance Cases

1. A maintainer can tell where helper-style Rust code belongs and why it is separate from Rust Zellij plugins.
2. A future config-normalization slice can land behind Nushell without changing the public `yzx` command surface.
3. Nushell wrappers can call the helper by absolute runtime-root path, parse one JSON success or error envelope, and render existing Yazelix-style diagnostics.
4. Startup profile reports remain comparable because Nushell keeps the same report schema and high-level step names.
5. Packaged runtimes can ship the helper privately under `libexec/` without exposing a second user command.
6. Rust-generated writes stay limited to explicit Yazelix-managed generated-state paths.
7. Warm shell startup reuses a current generated `yzx` extern bridge without rerunning Rust command metadata rendering.

## Verification

- `yzx_repo_validator validate-contracts`
- manual review against `docs/contracts/cross_language_runtime_ownership.md`
- manual review against `docs/contracts/runtime_root_contract.md`
- manual review against `docs/contracts/startup_profile_scenarios.md`
- manual review against `docs/contracts/v15_trimmed_runtime_contract.md`
- future implementation slices should add Rust unit tests plus Nushell wrapper parity tests for their specific bridge commands

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
- Defended by: `nu nushell/scripts/dev/test_shell_managed_config_contracts.nu`

## Open Questions

- Should the helper binary stay named `yzx_core`, or should implementation choose a narrower name once the first slice lands?
- Should `runtime_materialization.apply` write files directly in Rust, or should the first Rust slice emit only a plan that Nushell applies?
- Should a root Cargo workspace eventually include both `rust_core/` and `rust_plugins/`, or should those build flows stay separate because one produces native helpers and the other produces wasm?
