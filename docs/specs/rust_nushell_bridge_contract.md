# Rust/Nushell Bridge Contract

## Summary

Incremental Rust work in Yazelix should land behind a private helper binary that Nushell invokes with explicit inputs and machine-readable output.

Nushell remains the public `yzx` command surface, startup/profile owner, root resolver, and user-facing error renderer. Rust owns deterministic typed core work for selected slices once Nushell has passed explicit runtime, config, state, and command inputs across the bridge.

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

## Behavior

### Ownership Boundary

Nushell owns:

- the public `yzx` CLI and command naming
- launch, desktop, Home Manager, update, doctor, and maintainer UX
- config root, runtime root, and state root resolution
- startup profile schema, report files, and step names
- user-facing prose, remediation text, and final `error make` rendering
- compatibility with existing generated-state and runtime-root contracts

Rust owns:

- typed parsing and normalization for selected config/runtime inputs
- deterministic config-state hashes and invalidation decisions
- generated-runtime materialization plans, and later managed writes when that slice is ready
- machine-readable diagnostics that Nushell can translate without re-deriving the same logic
- library-level unit tests for pure behavior and fixture parity

The bridge must not turn Rust into a second public CLI owner. Rust helper commands are internal implementation details behind Nushell wrappers.

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

The first helper commands should track the planned Rust slices:

- `config.normalize`
- `config_state.compute`
- `runtime_materialization.plan`
- `runtime_materialization.apply`

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

Existing profile step names and meanings should remain comparable before and after each Rust slice. For example, a future Rust implementation of config-state computation should still appear under the same high-level `generated_runtime_state` / `compute_config_state` boundary that current reports use.

Rust may return optional metrics inside the success `data`, but those metrics are additive and must not become a second startup profile format.

### Extern Bridge Timing

The generated Nushell `yzx` extern bridge is startup-owned glue, not command business logic. It remains inside the existing `shellhook` / `sync_yzx_extern_bridge` profile step so startup reports stay comparable while the bridge implementation changes.

Warm startup must not pay the full command metadata probe when the generated extern bridge is already current. The sync path should perform only cheap generated-state checks, such as a command-surface fingerprint and generated-file hash, before reusing the existing bridge.

When the command surface is missing or stale, the sync path may spawn Nushell to inspect the real `yzx` command tree and regenerate the extern file. Successful regeneration updates the generated bridge and its fingerprint atomically enough that a later warm startup can skip the probe.

Refresh failure must be non-destructive. If a previous generated bridge exists, keep it instead of replacing it with an empty placeholder. If no bridge exists yet, create a minimal placeholder so managed Nushell config can still source the file and show the generation warning.

### Materialization And Writes

Rust may plan generated-runtime materialization before it writes generated files. The first implementation should prefer a plan/apply split unless the target slice is small enough that the write contract is obvious.

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
7. Warm shell startup reuses a current generated `yzx` extern bridge without rerunning command metadata introspection.

## Verification

- `nu nushell/scripts/dev/validate_specs.nu`
- manual review against `docs/specs/cross_language_runtime_ownership.md`
- manual review against `docs/specs/runtime_root_contract.md`
- manual review against `docs/specs/startup_profile_scenarios.md`
- manual review against `docs/specs/v15_trimmed_runtime_contract.md`
- future implementation slices should add Rust unit tests plus Nushell wrapper parity tests for their specific bridge commands

## Traceability

- Bead: `yazelix-kt5.1.1`
- Bead: `yazelix-4xp1.3`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: `nu nushell/scripts/dev/test_shell_managed_config_contracts.nu`

## Open Questions

- Should the helper binary stay named `yzx_core`, or should implementation choose a narrower name once the first slice lands?
- Should `runtime_materialization.apply` write files directly in Rust, or should the first Rust slice emit only a plan that Nushell applies?
- Should a root Cargo workspace eventually include both `rust_core/` and `rust_plugins/`, or should those build flows stay separate because one produces native helpers and the other produces wasm?
