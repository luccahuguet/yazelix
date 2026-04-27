# Rust Nix Dependency Layering Evaluation

## Summary

Evaluate whether Yazelix should replace the current `buildRustPackage` runtime
Rust package path with a crane-style dependency-artifact split.

Decision: do not adopt a new Rust Nix builder layer yet.

## Current Shape

`packaging/rust_core_helper.nix` already keeps the user package path narrow:

- builds only `-p yazelix_core`
- uses the workspace `rust_core/Cargo.lock`
- disables package-time tests
- filters the source to `rust_core/`, `config_metadata/`,
  `yazelix_default.toml`, and the zjstatus template needed by runtime
  materialization

The current Nix path still rebuilds the `cargo-vendor-dir` derivation when the
filtered source hash changes, even when `Cargo.lock` does not change. That is
the real possible improvement area for a crane-style split.

## Measurement

Measured on April 27, 2026 from a dirty working tree after the pane-orchestrator
wasm was rebuilt and synced.

Command:

```bash
/usr/bin/time -f 'elapsed=%E maxrss_kb=%M' nix build .#yazelix_ghostty --no-link
```

Results:

| Scenario | Result |
| --- | --- |
| Current invalid-output build after local source changes | `elapsed=0:20.88 maxrss_kb=695604` |
| Current warm no-op package build | `elapsed=0:00.39 maxrss_kb=39472` |

The invalid-output build rebuilt `cargo-vendor-dir`, `yazelix-core`,
`yazelix-runtime`, and `yazelix`. The warm build was effectively evaluation plus
store-path validation.

## Decision Rationale

Do not switch to crane or an equivalent dependency-artifact builder in this
slice.

The current measured source-change package rebuild is about 21 seconds on this
machine and the warm build is sub-second. Adding a second Rust builder stack
would add a new flake input surface and another packaging idiom for a package
path that already builds only the shipped product crate with tests disabled.

The current pain point remains worth tracking, because `cargo-vendor-dir`
rebuilt on a source-only change. That does not justify a builder migration until
we have repeated measurements showing the vendor step or dependency rebuilds are
a meaningful share of Home Manager switch time.

## Follow-Up Threshold

Reopen the builder migration if repeated Home Manager or package timings show
either of these:

- Rust package rebuilds stay above 60 seconds after source-only edits with no
  lockfile changes
- `cargo-vendor-dir` becomes a material share of the rebuild wall time, rather
  than a small fixed cost next to compiling `yazelix_core`

At that point, evaluate crane with a narrow target:

- dependency artifact keyed primarily by `Cargo.lock`
- product package output and public binaries unchanged
- package-time tests still disabled
- no new Rust code or runtime dependency changes

## Traceability

- Bead: `yazelix-r1ax.3`
- Defended by: `nix build .#yazelix_ghostty --no-link`
- Defended by: `nix eval --json .#packages.$(nix eval --raw --impure --expr builtins.currentSystem).yazelix_wezterm.name`
