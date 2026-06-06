# Yazelix Docs

Use this page as the docs front door. It points to current user guides first, then maintainer and contract material.

## Start Here

| Goal | Read |
| --- | --- |
| Install Yazelix | [Installation](./installation.md) |
| Understand runtime, config, and update ownership | [README Everyday Model](../README.md#everyday-model) |
| Learn the command surface | [yzx CLI](./yzx_cli.md) |
| Learn keybindings | [Keybindings](./keybindings.md) |
| Customize config, terminals, editors, and shells | [Customization](./customization.md) |
| Fix a broken launch or stale config | [Troubleshooting](./troubleshooting.md) |

## User Guides

- [Terminal emulators](./terminal_emulators.md)
- [Editor configuration](./editor_configuration.md)
- [Yazi configuration](./yazi-configuration.md)
- [Zellij configuration](./zellij-configuration.md)
- [Yazelix Zellij bar](./yazelix_zellij_bar.md)
- [Layouts](./layouts.md)
- [Startup performance](./startup_performance.md)
- [Styling](./styling.md)
- [Editor terminal integration](./editor_terminal_integration.md)

## Current Architecture

- [Architecture map](./architecture_map.md)
- [Documentation architecture](./documentation_architecture.md)
- [Contract inventory](./contracts/contracts_inventory.md)
- [Fork and child-repo maintenance](./contracts/fork_child_repo_maintenance.md)

## Maintainer Fast Path

For a small safe change, read these current-state surfaces before opening older
audits or history:

1. `AGENTS.md` for workflow, Beads, naming, verification, and push policy
2. `br ready` and `br show <id>` for live issue-tracker context
3. [Architecture map](./architecture_map.md) for subsystem ownership
4. [Current trimmed runtime contract](./contracts/v15_trimmed_runtime_contract.md) and [main config contract metadata](../config_metadata/main_config_contract.toml) for runtime/config boundaries
5. [Test suite governance](./contracts/test_suite_governance.md) for validator and test-lane selection

Historical notes, streamlining audits, extraction plans, and roadmap-style
documents are secondary references. Use them for rationale, not as live
behavior contracts.

## Maintainer References

- [Contributing](./contributing.md)
- [Rust maintainer tooling boundary](./rust_maintainer_tooling_boundary.md)
- [Rust code inventory](./rust_code_inventory.md)
- [Subsystem code inventory](./subsystem_code_inventory.md)
- [LOC extraction scorecard](./loc_extraction_scorecard.md)
- [Child repo simplification audit](./child_repo_simplification_audit.md)
- [Fork and child-repo maintenance](./contracts/fork_child_repo_maintenance.md)
- [Zed architecture lessons](./zed_architecture_lessons.md)
- [Package sizes](./package_sizes.md)
- [Upgrade notes data](./upgrade_notes.toml)

Planning status and implementation sequencing belong in Beads, not in docs. Contracts under `docs/contracts/` describe current Yazelix behavior and boundaries.
