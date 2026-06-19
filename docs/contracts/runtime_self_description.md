# Runtime Self-Description Contract

## Summary

Yazelix runtimes describe themselves through packaged manifest files under the
runtime root, and external consumers query the active runtime through
`yzx inspect --json`. Consumers must not scrape source checkout files, Nix
expressions, Home Manager modules, or child package internals to discover the
active runtime shape.

## External Query Surface

`yzx inspect --json` is the supported external query surface for the active
runtime. It may aggregate installed-runtime facts, config state, generated-state
freshness, install ownership, session observations, and tool versions.

Stable external consumers may rely on:

- `schema_version`
- `runtime.dir`
- `runtime.exists`
- `runtime.version`
- `runtime.variant`
- `config.dir`
- `config.file`
- `config.status`
- `generated_state.repair_needed`
- `generated_state.materialization_status`
- `generated_state.materialization_reason`
- `generated_state.input_freshness`
- `generated_state.missing_artifacts`
- `install.install_owner`
- `session` presence as an optional live-session observation object

Fields not listed here remain diagnostic detail and may change when their
owning subsystem changes. A future `yzx runtime describe --json` may narrow this
surface further, but it must not duplicate manifest readers or require source
checkout access.

## Packaged Manifests

The runtime-root manifest family is:

| Manifest | Role | Current owner |
| --- | --- | --- |
| `runtime_identity.json` | Runtime identity, release version, source revision, selected runtime variant, and pinned input revisions | main Yazelix package assembly |
| `runtime_components.json` | Enablement state for disableable bundled components | main Yazelix package assembly |
| `runtime_tools.json` | Runtime tool source mode, required commands, and optional host-integration metadata | main Yazelix package assembly |
| `runtime_features/` | Marker files for coarse runtime capabilities such as Zellij Kitty passthrough | main Yazelix package assembly |
| `runtime_variant` | Legacy duplicate terminal-variant file | deprecated compatibility surface |

`runtime_identity.json.schema_version` versions the identity manifest shape.
`yzx inspect --json.schema_version` versions the external aggregate query shape.
Those are separate because `inspect` includes live session and install-owner
facts that are not packaged manifest data.

## Stable Versus Code-Owned Fields

Stable manifest/query fields are data facts with no hidden behavior:

- release version and source/input revisions
- selected runtime variant
- component enablement
- runtime tool source mode and required commands
- install owner and update path
- generated-state materialization status and repair requirement

Code-owned behavior stays in Rust, Nix, or child packages:

- launch command construction
- Home Manager package composition
- config normalization and materialization logic
- terminal config semantics
- doctor repair behavior
- child package metadata validation beyond the stable fields consumed by main

Manifests may describe capabilities and selected package facts. They must not
become a generic rule language for launch or materialization behavior.

## Runtime Modes

Installed package runtimes must ship the manifest family above.

Home Manager-built runtimes are installed package runtimes whose install owner
is detected as `home-manager`. The manifests still live in the active runtime
root and `yzx inspect --json.install.install_owner` reports the owner/update
path.

Source checkout runtimes may lack generated packaged manifests. Maintainer
commands may read source files during repository validation, but user-facing
runtime consumers must use `yzx inspect --json` or typed Rust manifest readers
instead of scraping checkout paths.

## First Deletion Slice

The first duplicated authority to delete is the standalone `runtime_variant`
file. The selected terminal variant already exists in
`runtime_identity.json.runtime_variant`, and `yzx inspect --json.runtime.variant`
is the external aggregate field.

The implementation slice should:

1. Make typed Rust runtime-variant readers prefer `runtime_identity.json`
2. Keep `runtime_variant` only as a temporary compatibility fallback if needed
3. Update tests to prove `runtime_identity.json.runtime_variant` is sufficient
4. Remove the standalone `runtime_variant` file from packaged runtime assembly
   once all consumers have moved

## Consumers

- Rust runtime control plane and `yzx inspect`
- `yzx status` and `yzx doctor`
- maintainer validators
- Home Manager update and ownership diagnostics
- generated documentation and support tooling
- agent-side runtime inspection
- future child repositories that need explicit runtime facts

Child repositories should receive explicit request data or stable manifest
facts. They must not read main-repo source paths or infer behavior from the
runtime tree shape.

## Verification

Once implemented, this contract is defended by:

- focused Rust tests for runtime identity and runtime variant readers
- `yzx status --json` and `yzx inspect --json` runtime-surface tests
- `yzx_repo_validator validate-installed-runtime-contract`
- `yzx_repo_validator validate-contracts`
