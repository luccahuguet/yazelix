# Full-Config Nushell Owner Cut Budget

## Summary

This document defines the delete-first budget for collapsing the remaining
product-side `parse_yazelix_config` owner seam.

The goal is not to delete `config_parser.nu` blindly. The goal is to stop
product entrypoints from reparsing the full Yazelix config when they only need a
small retained fact surface such as popup geometry, managed editor kind,
sidebar enablement, or startup/session flags.

This budget is planning-first. It records what the remaining product callers
actually need, which Rust-owned facts can replace those reads honestly, and
where Nushell must stay the shell/process owner.

## Scope

- product-facing Nushell callers that previously imported
  `parse_yazelix_config`
- front-door command bodies under `nushell/scripts/yzx/`
- integration-adjacent wrappers and launch helpers
- startup/launch/setup entrypoints that only need retained config facts

Out of scope:

- `config_parser.nu` as a config-normalize and diagnostic owner by itself
- dev and test callers that still exercise `parse_yazelix_config`
- shell/process orchestration that still belongs to Nu or POSIX
- public `yzx` family ownership beyond the specific full-config seam

## Caller Inventory

### Front-door callers

| Caller | Previous full-config use | Real retained need | Surviving owner decision |
| --- | --- | --- | --- |
| `nushell/scripts/yzx/menu.nu` | popup program and geometry defaults for the managed menu popup | popup argv/geometry facts only | Rust `transient-pane-facts.compute`; Nu keeps menu UX and Zellij execution |
| `nushell/scripts/yzx/popup.nu` | popup program, geometry, runtime path, and cwd shaping | popup argv/geometry facts only | Rust `transient-pane-facts.compute`; Nu keeps popup UX, cwd choice, and open request rendering |

### Integration-adjacent callers

| Caller | Previous full-config use | Real retained need | Surviving owner decision |
| --- | --- | --- | --- |
| `nushell/scripts/integrations/zellij_runtime_wrappers.nu` | wrapper/runtime env shaping for popup/editor launches | runtime env only | Rust `runtime-env.compute`; Nu keeps wrapper/process execution |
| `nushell/scripts/utils/editor_launch_context.nu` | managed editor command/runtime shaping | runtime env only | Rust `runtime-env.compute`; Nu keeps editor launch orchestration |
| `nushell/scripts/zellij_wrappers/yzx_popup_program.nu` | popup argv/geometry plus runtime env fallback | popup facts plus runtime env | Rust `transient-pane-facts.compute` and `runtime-env.compute`; Nu keeps wrapper mode/env and external command execution |

### Startup, launch, and setup callers

| Caller | Previous full-config use | Real retained need | Surviving owner decision |
| --- | --- | --- | --- |
| `nushell/scripts/core/start_yazelix.nu` | canonical runtime env assembly before entering the live shell | runtime env only | Rust `runtime-env.compute`; Nu keeps shell/bootstrap execution |
| `nushell/scripts/core/start_yazelix_inner.nu` | welcome/session/debug/persistent flags and launch-time config toggles | retained startup/session facts | Rust `startup-facts.compute`; Nu keeps startup profiling, materialization, and Zellij handoff |
| `nushell/scripts/core/launch_yazelix.nu` | startup/session/terminal facts used before launching a terminal | retained startup/session facts | Rust `startup-facts.compute`; Nu keeps terminal selection, launch prose, and process execution |
| `nushell/scripts/yzx/launch.nu` | runtime env shaping for public launch entrypoint | runtime env only | Rust `runtime-env.compute`; Nu keeps public CLI UX and process execution |
| `nushell/scripts/setup/environment.nu` | retained startup/session facts used during setup/welcome/bootstrap | retained startup/session facts | Rust `startup-facts.compute`; Nu keeps shellhook/setup orchestration and file/process ownership |

## Surviving Owner Decision

The surviving typed owners for this seam are:

- Rust `integration-facts.compute`
- Rust `transient-pane-facts.compute`
- Rust `startup-facts.compute`
- Rust `runtime-env.compute`

`config_parser.nu` should stop being a product-side owner when a caller only
needs one of those narrower fact surfaces.

After the cut:

- Rust owns deterministic fact extraction from the managed config
- Nushell keeps public UX, prose, shell execution, Zellij/editor/Yazi process
  execution, and startup/launch orchestration
- tests may still call `parse_yazelix_config` directly until the later
  `yazelix-sq0g.4` demotion/deletion decision

## Deletion Budget

### Lane 1: integration-adjacent wrapper collapse

- bead: `yazelix-jkk3`
- target files:
  - `integrations/zellij_runtime_wrappers.nu`
  - `utils/editor_launch_context.nu`
  - `zellij_wrappers/yzx_popup_program.nu`
- success condition:
  - these files no longer import `parse_yazelix_config`
  - wrapper/process logic stays in Nu
  - Rust facts replace only the deterministic config reads

### Lane 2: front-door config facts

- bead: `yazelix-sq0g.2`
- target files:
  - `yzx/menu.nu`
  - `yzx/popup.nu`
- success condition:
  - front-door popup/menu flows consume one narrower facts surface
  - no fake Rust menu/popup command body is introduced

### Lane 3: startup and launch retained facts

- bead: `yazelix-sq0g.3`
- target files:
  - `core/start_yazelix.nu`
  - `core/start_yazelix_inner.nu`
  - `core/launch_yazelix.nu`
  - `yzx/launch.nu`
  - `setup/environment.nu`
- success condition:
  - these entrypoints stop reparsing full config when they only need retained
    startup or runtime-env facts
  - shell/process orchestration remains Nu/POSIX-owned

### Remaining follow-up after these cuts

- bead: `yazelix-sq0g.4`
- target:
  - decide whether `config_parser.nu` can now be deleted, demoted to a
    dev/test-only owner, or kept as a narrow config-normalize surface

## Explicit Stop Conditions

Stop and record a no-go if any proposed cut would require one of these:

- moving shell/process execution into Rust just to hide a Nu caller
- creating a second generic config bridge after the narrower fact helpers land
- treating dev/test `parse_yazelix_config` callers as if they were still
  product owners
- making Nu parse human-facing command output instead of one structured helper
  surface

If a stop condition is hit, keep the smallest explicit Nu seam and record it.
Do not recreate the old full-config owner by default.

## Acceptance

1. Every live product-side `parse_yazelix_config` caller is classified
2. The budget separates integration, front-door, and startup/launch lanes
3. The surviving Rust owners are named directly
4. The stop conditions make clear what would turn the cut into fake migration
5. Dev/test callers are explicitly excluded from the product-side owner budget

## Traceability

- Bead: `yazelix-sq0g.1`
- Informed by: `docs/specs/rust_migration_matrix.md`
- Informed by: `docs/specs/integration_glue_canonicalization_audit.md`
- Informed by: `docs/specs/launch_startup_session_canonicalization_audit.md`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
