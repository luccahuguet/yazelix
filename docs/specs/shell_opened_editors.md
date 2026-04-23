# Shell-Opened Editors

## Summary

Define how Yazelix should treat editors started manually from a shell pane instead of through the managed Yazi-to-editor flow.

The chosen contract is:

- shell-opened editors are ordinary panes by default
- only Yazelix-managed editor panes count as the managed `editor` pane
- Yazi-driven open/reveal flows target the managed editor pane only

## Why

This boundary is currently easy to misunderstand.

Users can start Helix or another editor manually from a shell pane, but Yazelix also has a separate concept of a managed editor pane tracked through the pane orchestrator. If those are treated as the same thing implicitly, workspace routing becomes ambiguous:

- which pane should Yazi opens target?
- which pane should `Ctrl+y` focus?
- which editor pane should `yzx cwd` synchronize?

The contract needs to be explicit.

## Scope

This spec covers:

- what counts as the managed editor pane
- how shell-opened editors should be treated
- how Yazi-driven opens and cwd sync should behave relative to shell-opened editors
- what users should expect from doctor/docs messaging

## Contract Items

#### SOE-001
- Type: ownership
- Status: live
- Owner: pane orchestrator managed editor identity
- Statement: Only the Yazelix-managed pane titled `editor` counts as the
  managed editor pane. A generic shell pane running an editor process is not
  the managed editor by default
- Verification: automated
  `nu nushell/scripts/dev/test_yzx_yazi_commands.nu`; automated
  `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --lib`

#### SOE-002
- Type: non_goal
- Status: live
- Owner: managed editor boundary
- Statement: Shell-opened editors are ordinary panes. Yazelix does not
  automatically adopt, retitle, or reinterpret them as the managed editor pane
- Verification: automated
  `nu nushell/scripts/dev/test_yzx_yazi_commands.nu`; validator
  `yzx_repo_validator validate-specs`

#### SOE-003
- Type: behavior
- Status: live
- Owner: Yazi open routing plus pane orchestrator editor-open flow
- Statement: Yazi-driven opens target the managed editor pane when it exists,
  and otherwise create a new managed editor pane through the normal Yazelix
  flow. Yazelix does not guess that a shell-opened editor pane should be reused
- Verification: automated
  `nu nushell/scripts/dev/test_yzx_yazi_commands.nu`; automated
  `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --lib`

#### SOE-004
- Type: behavior
- Status: live
- Owner: managed editor sync and explicit integration facts
- Statement: Workspace retargeting and managed editor cwd sync apply only to
  the managed editor pane and its configured managed editor kind. Doctor/docs
  language and helper facts must distinguish "managed editor pane" from
  "editor process exists somewhere"
- Verification: automated
  `nu nushell/scripts/dev/test_yzx_yazi_commands.nu`; automated
  `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`; automated
  `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core --test yzx_core_owned_facts`

## Behavior

### Managed Editor Definition

A pane counts as the managed editor pane only when Yazelix intentionally creates or tracks it as the stable `editor` pane.

Current practical signal:

- the pane orchestrator recognizes the managed editor by the stable pane title `editor`

### Shell-Opened Editors

If a user starts an editor manually from a shell pane:

- that pane remains an ordinary pane
- Yazelix does not automatically adopt it as the managed editor
- Yazelix does not retitle or reinterpret it as the managed editor

### Yazi-Driven Opens

When opening a file from Yazi:

- if a managed editor pane exists in the current tab, Yazelix should target that pane
- if no managed editor pane exists, Yazelix should create a new managed editor pane through the normal Yazelix flow
- Yazelix should not guess that a shell-opened editor pane should be reused

### Workspace and Cwd Sync

Commands that synchronize the managed editor cwd, such as tab workspace retargeting, apply only to the managed editor pane.

They do not try to detect or synchronize arbitrary shell-opened editor processes.

### Discoverability

Doctor and docs should describe this clearly:

- "managed editor pane" means the Yazelix-managed pane
- a manually started editor is not considered managed by default

## Non-goals

- automatic adoption of arbitrary shell-opened editors
- scanning panes for editor processes and guessing intent
- making every editor invocation participate in managed editor routing
- designing a future "adopt this pane as editor" command

Those may be explored later, but they are outside this spec.

## Acceptance Cases

1. When a user starts Helix manually from a normal shell pane, that pane is not treated as the managed editor pane.
2. When Yazi opens a file and no managed editor pane exists, Yazelix creates a new managed editor pane instead of guessing that a shell-opened editor should be reused.
3. When a managed editor pane exists, Yazi-driven opens target that managed pane even if other editor processes are running in ordinary panes.
4. When `yzx cwd` or equivalent workspace retargeting runs, managed editor cwd sync applies only to the managed editor pane.
5. Doctor/docs language distinguishes between "editor process exists somewhere" and "managed editor pane exists in the current tab."

## Verification

- manual verification:
  - open a shell pane and start `hx` manually
  - verify `yzx doctor` still reports no managed editor pane until a managed one is opened
  - trigger a Yazi open and verify Yazelix creates or targets the managed `editor` pane instead of the shell-opened editor
- documentation checks:
  - the editor docs and related diagnostics use "managed editor pane" consistently
- future automated coverage:
  - add pane-state and routing regression coverage once the workspace/session boundary tests are expanded

## Traceability

- Bead: `yazelix-2ex.1.7.3`
- Defended by: `cargo test --manifest-path rust_plugins/zellij_pane_orchestrator/Cargo.toml --lib`
- Defended by: `nu nushell/scripts/dev/test_yzx_doctor_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_yazi_commands.nu`

## Open Questions

- Should Yazelix eventually support an explicit "adopt current pane as managed editor" action?
- Should generic editors ever get a stronger managed-pane story, or should that remain specific to the current Helix/Neovim flow?
