# Yazelix Subsystem Code Inventory

This document is a maintainer-facing snapshot of where Yazelix code currently lives and how large each subsystem family is.

It exists to support architecture and streamlining work. The goal is not to pretend every folder is a separate product boundary. The goal is to make the current code-weight distribution visible enough that deletion and ownership discussions can start from the real repo shape instead of instinct.

## Snapshot

- Snapshot date: `2026-04-09`
- Metric: `tokei` `Code` column only
- Scope: source-owned Yazelix paths
- Excludes: docs prose, Markdown code fences, local scratch state, `.devenv`, build outputs, and non-source binary assets

## Inventory Buckets

| Inventory bucket | Broad owner | Paths counted | Code LOC |
| --- | --- | --- | ---: |
| Runtime control plane and command surface | Runtime | `nushell/scripts/core`, `nushell/scripts/setup`, `nushell/scripts/utils`, `nushell/scripts/yzx` | 17,987 |
| Maintainer workflow and validation | Maintainer Workflow | `nushell/scripts/dev`, `.github` | 13,288 |
| Shipped configs and templates | Runtime + Workspace | `configs`, `config_metadata`, `user_configs` | 5,148 |
| Workspace session orchestration | Workspace | `nushell/scripts/integrations`, `nushell/scripts/zellij_wrappers`, `rust_plugins/zellij_pane_orchestrator`, `rust_plugins/zellij_popup_runner` | 3,223 |
| Distribution and host integration | Integrations | `home_manager`, `packaging`, `shells`, `flake.nix` | 1,160 |
| Ancillary runtime assets and shell config | Runtime + Integrations | `assets`, `nushell/config` | 77 |

Covered by this inventory: `40,883` code lines.

For reference, a raw `tokei .` snapshot currently reports `42,685` code lines for the whole repo. That higher repo-wide number includes code fenced inside Markdown plus other non-subsystem residue that this inventory intentionally does not treat as first-class implementation surface.

## What The Numbers Mean

- The largest shipped logic surface is still the runtime control plane. Most deletion-first work will pay off there before it pays off in Rust, Nix, or shell glue.
- The maintainer workflow is the second-largest bucket. Yazelix has a large amount of test and validation code relative to its shipped runtime.
- The workspace layer is important but comparatively small in code volume. That means many user-visible problems are caused more by ownership seams than by raw workspace implementation size.
- Shipped config and template data is a real subsystem cost. Yazelix is not just Nushell plus Rust; it also carries a meaningful volume of TOML, Lua, GLSL, and generated-surface metadata.

## Reproduce The Snapshot

```bash
tokei nushell/scripts/core nushell/scripts/setup nushell/scripts/utils nushell/scripts/yzx
tokei nushell/scripts/dev .github
tokei configs config_metadata user_configs
tokei nushell/scripts/integrations nushell/scripts/zellij_wrappers rust_plugins/zellij_pane_orchestrator rust_plugins/zellij_popup_runner
tokei home_manager packaging shells flake.nix
tokei assets nushell/config
```

When these numbers are refreshed, prefer updating this document in the same change that materially shifts subsystem ownership or code distribution.

## Traceability

- Bead: `yazelix-dhyi`
