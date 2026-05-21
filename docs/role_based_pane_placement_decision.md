# Role-Based Pane Placement Decision

## Status

Accepted for the next workspace architecture pass.

This is a planning decision, not a live runtime contract. Current Yazelix still
uses the existing managed editor/sidebar model, `yzpp` popup/menu/config panes,
and current keybindings until the implementation beads land.

## Core Decision

Yazelix should model workspace surfaces as semantic roles assigned to placement
slots.

Roles answer what the pane is for:

- `file_tree`
- `editor`
- `agent`
- `git_client`
- `config_ui`

Placements answer where and how that role appears:

- `main_stack`
- `left_sidebar`
- `right_sidebar`
- `top_popup`
- `bottom_popup`

Keybindings are a separate layer that invoke role or placement actions. User
configuration shape is also separate and remains the job of the follow-on
role-placement config decision.

## Role Identity And Routing

Roles should be first-class. Routing must target a semantic role, not a physical
pane position.

The pane orchestrator remains the live session-state owner for managed workspace
roles. Existing pane titles such as `editor` and `sidebar` may remain the first
implementation signal, but new routing APIs should name roles explicitly rather
than infer intent from geometry.

Default roles are singleton per tab. Repeatable roles are deferred until there
is a concrete need and an explicit instance id model. Ordinary unmanaged panes
remain outside this role system.

Accepted routing rules:

- `file_tree` routes file opens to the `editor` role
- workspace retargeting may synchronize `editor` and `file_tree` through the
  existing tab-local workspace state
- `agent` and `git_client` do not get rich routing in the first pass
- no route may depend on "the editor is in the main pane" or "the sidebar is on
  the left"

Missing-role behavior should fail fast unless an existing product contract
already owns creation:

- if a Yazi/file-tree open needs `editor` and the managed editor is missing,
  Yazelix may create the managed editor through the existing editor-open flow
- if the configured editor is unsupported or absent, the error should name the
  missing `editor` role and the setting or command the user should fix
- Yazelix must not adopt a shell-opened editor as the managed `editor`
- if `agent` or `git_client` is toggled and its command is missing, fail with a
  role-specific install/configuration message

## Editor Role

`editor` remains a role. Its default placement is `main_stack`.

The main stack is the default placement, not the identity of the editor. The
editor can become configurable later, but the default should preserve durable
editor state and avoid treating the editor as a disposable popup.

Visibility is not required for role routing. If an `editor` role exists and the
adapter can accept an open or cwd-sync command, `file_tree` and Yazi flows may
target it even when focus or placement makes it non-visible. If the adapter
cannot accept the route, Yazelix should fail clearly instead of guessing another
pane.

Rejected defaults:

- defaulting `editor` to `top_popup`
- treating a shell-opened editor process as the managed `editor`
- opening Yazi-selected files in a transient editor popup

## Placement Slots And Lifecycle

The accepted initial placement vocabulary is:

- `main_stack`: durable stack/main workspace area; default placement for
  `editor` and ordinary unmanaged panes
- `left_sidebar`: persistent toggleable singleton side surface; default
  placement for `file_tree`
- `right_sidebar`: persistent toggleable singleton side surface; default
  placement for `agent`
- `top_popup`: transient/floating singleton surface; default placement for
  `config_ui`
- `bottom_popup`: transient/floating singleton surface; default placement for
  `git_client`

`command_pane` stays on the existing command palette/menu path and does not
become a default directional placement slot in this pass.

Sidebar lifecycle:

- sidebars use a single-open policy by default
- opening `left_sidebar` closes `right_sidebar` first
- opening `right_sidebar` closes `left_sidebar` first
- toggling the open sidebar closes it
- do not add an `allow_both_sidebars` config knob yet

Popup lifecycle:

- popups pull focus when opened
- popups are not part of sidebar/editor focus toggles
- repeated popup actions should follow the existing managed-popup pattern:
  absent opens, existing unfocused focuses, focused closes

Placement sharing is intentionally narrow. The initial model should ship one
default role per placement. Extra named popups can be supported later through a
data-driven model, but multiple live default roles in the same placement are
deferred.

Sizing should be placement-specific. The left file-tree sidebar can stay
narrower, while the right agent sidebar should support a wider ceiling, likely
up to the current 40 percent maximum.

## Agent Role

`agent` should be a first-class role, defaulting to `right_sidebar`.

Minimum accepted behavior:

- singleton per tab
- launched or focused through role/placement actions
- cwd starts from the active tab workspace root
- sized for conversation, diffs, tool output, and status
- command configurable
- default command is Codex

Codex is the first-class Yazelix agent integration. OpenCode is a valid
documented alternative command, but Yazelix should not silently fall back from
Codex to OpenCode.

If Codex is missing or unauthenticated, Yazelix should fail with a clear message
telling the user to install/sign in to Codex or configure another agent command.

Deferred agent behavior:

- selected-file routing
- changed-file routing
- editor selection handoff
- background daemon management
- provider-specific status beyond the command and pane lifecycle

## Git Client Role

`git_client` should start as a lightweight role, defaulting to `bottom_popup`.

Minimum accepted behavior:

- singleton per tab
- command defaults to `lazygit`
- launched in the active tab workspace root
- follows the managed-popup focus/close lifecycle

Do not grow a rich Git subsystem yet. The first pass only needs launch, focus,
close, cwd, and command configuration. Diff/rebase/editor handoff behavior
should wait for a real workflow need.

## Follow-On Decisions

The directional keymap decision is recorded in
[`directional_placement_keymap_decision.md`](./directional_placement_keymap_decision.md).

The user-facing config decision is recorded in
[`role_placement_config_decision.md`](./role_placement_config_decision.md).

Implementation details remain separate. The accepted architecture does not yet
choose how much belongs in generated Zellij layouts, `yzpp` specs, pane
orchestrator commands, or Rust adapter code.
