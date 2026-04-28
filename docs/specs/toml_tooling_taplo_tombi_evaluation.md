# TOML Tooling Taplo/Tombi Evaluation

## Summary

Yazelix migrated from Taplo to Tombi through a staged track rather than one blind
dependency swap. Tombi is the more active upstream and is now the default runtime
TOML tooling owner, while the repo keeps a bounded formatter corpus to avoid
unrelated vendored/Cargo TOML churn.

Decision: ship Tombi in the runtime, ship `tombi.toml` as the managed TOML
tooling config, and keep Taplo references only as historical migration evidence.

## Why

The product problem was not just "which formatter binary is newer." Yazelix ships
a TOML tool as part of the runtime and copies its support config into the managed
config root so user config editing gets consistent support. Replacing that tool
changed package closure, editor/LSP expectations, managed-support naming,
doctor/onboard diagnostics, and the style of committed TOML.

A migration is attractive because Tombi is actively releasing and explicitly
covers formatting, linting, language-server behavior, JSON Schema use, and
Taplo migration differences. A blind migration is risky because current Yazelix
state is not cleanly formatted by either tool, and Tombi does not implement every
Taplo formatting knob one-for-one.

## Evidence

Upstream status checked on 2026-04-28:

- Taplo's GitHub repository is public and not visibly archived in the fetched
  page, and its latest listed release is `Taplo CLI 0.10.0` from 2025-05-23
- Tombi's GitHub releases page lists `v0.9.25` as latest on 2026-04-27
- Tombi documents itself as a CLI for formatting and linting TOML files
- Tombi's Taplo comparison documents formatter option mappings, schema-driven
  sorting, richer directives, and a stable-formatting policy

Locked nixpkgs input check:

```bash
nix eval --raw --impure --expr 'let flake = builtins.getFlake "path:/home/lucca/pjs/yazelix"; pkgs = import flake.inputs.nixpkgs { system = builtins.currentSystem; }; in pkgs.taplo.version'
# 0.10.0

nix eval --raw --impure --expr 'let flake = builtins.getFlake "path:/home/lucca/pjs/yazelix"; pkgs = import flake.inputs.nixpkgs { system = builtins.currentSystem; }; in if pkgs ? tombi then pkgs.tombi.version else "missing"'
# 0.9.13
```

Package closure check against the locked nixpkgs input:

```text
taplo 0.10.0: 58.8 MiB
tombi 0.9.13: 73.3 MiB
delta: +14.5 MiB
```

Formatter sample:

```bash
taplo fmt --check yazelix_default.toml config_metadata/*.toml docs/upgrade_notes.toml
# failed on config_metadata/vendored_yazi_plugins.toml,
# config_metadata/zellij_layout_families.toml,
# docs/upgrade_notes.toml

tombi format --check yazelix_default.toml config_metadata/*.toml docs/upgrade_notes.toml
# failed on yazelix_default.toml,
# config_metadata/main_config_contract.toml,
# config_metadata/zellij_layout_families.toml,
# docs/upgrade_notes.toml
```

Formatting a temporary copy of the same sample with Tombi changed four files:

```text
4 files changed, 314 insertions(+), 75 deletions(-)
```

Formatting a temporary copy with Taplo also changed the current repo sample:

```text
3 repo files changed, 498 insertions(+), 151 deletions(-)
```

Tombi versus Taplo formatting output for the same temporary sample still differs:

```text
5 repo files changed, 163 insertions(+), 271 deletions(-)
```

## Current Yazelix Tombi Surface

Runtime/package surfaces:

- `packaging/runtime_deps.nix` includes `tombi`
- `packaging/mk_runtime_tree.nix` exports `tombi` in `toolbin`
- `packaging/mk_runtime_tree.nix` symlinks `tombi.toml` into the runtime tree
- `packaging/repo_source.nix` keeps `tombi.toml` in the package source

Runtime/user support surfaces:

- `tombi.toml` is the shipped formatter config
- `active_config_surface.rs` computes runtime and managed TOML tooling config
  paths
- `ensure_managed_toml_tooling_config` copies the runtime TOML tooling support
  file to the managed config root and fails clearly when the runtime support
  file is missing
- doctor/onboard flows call the same managed TOML tooling support path
- tests and fixtures name TOML tooling support directly

Docs:

- `docs/architecture_map.md`, `docs/posix_xdg.md`,
  `docs/package_sizes.md`, and `docs/contributing.md` describe Tombi support

## Decision

Use a three-step migration track:

1. Prototype Tombi config and formatting on the repo TOML corpus without
   changing the shipped runtime dependency
2. Rename the managed support concept from Taplo-specific support to generic
   TOML tooling support so future tool swaps do not leak into user-visible
   state names unnecessarily
3. Swap the packaged runtime tool from Taplo to Tombi only after the prototype
   proves acceptable formatter churn and the locked nixpkgs input is close
   enough to upstream Tombi for the package to represent the active project

The migration track existed because Tombi's upstream activity and TOML toolkit
scope made it the better candidate once Yazelix could migrate without leaving
Taplo-shaped product seams behind.

## Prototype Outcome

`yazelix-zz0k.1` accepted a configured Tombi corpus instead of the whole tracked
TOML tree:

```toml
[files]
include = [
  "*.toml",
  "config_metadata/**/*.toml",
  "docs/upgrade_notes.toml",
  "user_configs/**/*.toml",
]
exclude = [
  "configs/yazi/flavors/**/*.toml",
  "rust_core/**/*.toml",
  "rust_plugins/**/*.toml",
]
```

Reason:

- vendored Yazi flavor TOML is external theme data, not a Yazelix-authored
  formatting surface
- Cargo manifests already follow Rust tooling expectations
- the runtime/user support target is Yazelix TOML configuration, especially
  `yazelix_default.toml`, `yazelix_cursors_default.toml`,
  `config_metadata/*.toml`, `docs/upgrade_notes.toml`, and
  `user_configs/**/*.toml`

The configured Tombi pass changed five files:

```text
5 files changed, 241 insertions(+), 71 deletions(-)
```

Changed TOML files:

- `.nu-lint.toml`
- `config_metadata/main_config_contract.toml`
- `docs/upgrade_notes.toml`
- `tombi.toml`
- `yazelix_default.toml`

Unsupported or deliberately unmapped Taplo knobs:

- Taplo's `array_auto_expand`, `array_trailing_comma`, `align_entries`,
  `column_width`, `compact_arrays`, `compact_inline_tables`, and
  `indent_string` do not map one-for-one into the accepted Tombi config
- Tombi's accepted replacement is a narrower formatter rule set:
  `indent-width = 2`, `line-width = 200`, and `string-quote-style = "preserve"`

Recommendation after the prototype: the package swap is safe if Yazelix keeps
the configured corpus boundary and does not attempt a full repo-wide TOML
reformat.

## Runtime Swap Outcome

`yazelix-zz0k.3` completed the package and runtime surface swap:

- `taplo` was removed from `packaging/runtime_deps.nix` and `toolbin`
- `tombi` is now the runtime TOML CLI
- `.taplo.toml` was deleted from the shipped source
- `tombi.toml` is now the runtime and managed TOML tooling config
- active config surface code and tests use the generic TOML tooling config
  constant instead of Taplo-specific paths
- docs now describe Tombi/TOML tooling support as the current surface

## Non-Goals

- Do not run broad TOML formatting as incidental cleanup
- Do not add both Taplo and Tombi to the default runtime package just to avoid
  choosing

## Verification

- upstream review:
  - `https://github.com/tamasfe/taplo/releases`
  - `https://github.com/tombi-toml/tombi/releases`
  - `https://tombi-toml.github.io/tombi/docs/cli/`
  - `https://tombi-toml.github.io/tombi/docs/reference/difference-taplo/`
- local package availability:
  - `nix eval --raw --impure --expr 'let flake = builtins.getFlake "path:/home/lucca/pjs/yazelix"; pkgs = import flake.inputs.nixpkgs { system = builtins.currentSystem; }; in pkgs.taplo.version'`
  - `nix eval --raw --impure --expr 'let flake = builtins.getFlake "path:/home/lucca/pjs/yazelix"; pkgs = import flake.inputs.nixpkgs { system = builtins.currentSystem; }; in if pkgs ? tombi then pkgs.tombi.version else "missing"'`
- local formatter sample:
  - `tombi format --check`
  - `tombi lint --offline`
- CI/spec check:
  - `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-specs`

## Traceability

- Bead: `yazelix-zz0k`
- Defended by: `docs/specs/toml_tooling_taplo_tombi_evaluation.md`
- Defended by: `yzx_repo_validator validate-specs`
