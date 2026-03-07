# Zellij Plugin Pane Orchestration Plan

As of 2026-03-07.

## Problem

Yazelix currently handles "open from Yazi", "focus editor", and related flows with Nushell heuristics:

- iterate across the next few panes
- guess the editor by running command
- guess the editor by a supposed pane name
- move panes around to recover a preferred stack order

This is fragile for two reasons:

1. The current detection logic does not actually have a reliable pane-identity primitive.
2. Pane stack position is not a stable source of truth.

The result is exactly what the UX feels like today: the integration works often enough, but it still feels like a set of hacks.

## Direction

Do not make "editor is first in the stack" the core invariant.

Instead, make pane identity explicit and stable:

- Yazelix creates managed panes with titles it controls.
- A small Zellij plugin tracks those panes by title and tab.
- Nushell calls the plugin for focus/open targeting instead of scanning neighbors.

Stack pinning can remain an optional UX improvement later, but it should not be the primitive the system depends on.

## Managed Pane Contract

This design depends on a narrow runtime contract.

### Canonical titles

- Managed editor pane title: `editor`
- Managed Yazi pane title: `sidebar`

The exact Yazi title can be changed later, but it must be one canonical title, not multiple ad hoc aliases.

### Ownership rules

- At most one managed editor pane per tab participates in the Yazi open flow.
- The plugin treats the most recent matching `editor` pane in a tab as the managed editor for that tab.
- Managed panes are identified by title first, not by command line.

### Fallback rules

- If no managed editor pane exists in the active tab, Nushell creates one and titles it `editor`.
- After creation, the plugin picks it up from the next pane update.
- If title-based tracking fails, the old heuristic should not silently remain as the main path. It should be a narrow fallback at most, or removed entirely.

## Plugin MVP

The plugin should stay intentionally small.

### Responsibilities

- Subscribe to `PaneUpdate`
- Track pane IDs by managed title per tab
- Focus a tracked pane by ID
- Write to a tracked pane by ID
- Optionally move a tracked pane if stack ordering still matters after targeting is fixed

### Explicit non-goals

- No broad layout management
- No generalized pane manager UI
- No replacement of Nushell orchestration
- No attempt to solve every Zellij workflow in the first version

## Initial Pipe API

The plugin should expose a very small pipe surface.

### `focus_editor`

Behavior:

- find the managed `editor` pane in the current tab
- focus it directly by pane ID

If no managed editor exists, return failure so the caller can create one.

### `open_file`

Behavior:

- find the managed `editor` pane in the current tab
- focus it
- write the editor-specific open command to that pane

For Helix, this means sending the same core command sequence Yazelix already uses, but to a specific pane ID rather than whichever pane happens to be focused after neighbor scanning.

The plugin does not need to invent editor semantics. It only needs to target the correct pane deterministically.

## Nushell Changes

After the plugin exists, Nushell should become simpler.

### Keep

- file path validation
- editor-specific command construction
- tab renaming
- creating a new editor pane when needed
- Yazi synchronization behavior

### Remove

- "check the top 4 panes"
- command-based pane guessing as the main path
- fake pane-name detection via `list-clients`
- stack recovery as a prerequisite for opening files

## Why Titles Are Good Enough

Pane titles are acceptable here because Yazelix itself creates the panes involved in the flow.

That means:

- the title is not guessed from a third-party program
- the title is part of Yazelix's own runtime contract
- the title can be applied consistently whenever Yazelix opens a managed pane

This is a much better basis than inferring identity from whichever command is currently visible in the pane.

## Why A Plugin Is Still Needed

Titles alone are not enough if Nushell still has to act only through the Zellij CLI.

The plugin matters because it can:

- observe real pane updates
- map titles to pane IDs
- focus exact panes by ID
- write to exact panes by ID

That removes the need for positional guessing.

## Migration Plan

### Phase 1

- add plugin scaffold
- track `editor` and `sidebar` pane IDs by tab
- expose `focus_editor`

### Phase 2

- expose `open_file`
- route Yazi open flow through the plugin first
- keep "create new editor pane" fallback in Nushell

### Phase 3

- remove top-N pane scanning
- decide whether stack reordering is still useful
- clean up documentation and logs around the old heuristic path

## Verification

Before considering the migration successful:

- plugin builds cleanly
- Nushell integration scripts pass syntax validation
- opening a file from Yazi reuses the correct editor pane without neighbor scanning
- focusing the editor from Yazi no longer depends on current stack position
- reveal/open flows still work when the editor pane must be created from scratch

## Open Questions

### Should the Yazi pane be titled `sidebar` or `yazi`?

`sidebar` fits the current Yazelix language better, but `yazi` is more explicit. Pick one and keep it canonical.

### Should the plugin send editor commands directly, or only focus the pane?

The smallest useful starting point is:

- plugin owns targeting
- Nushell still owns editor command construction

That keeps responsibilities narrow and reduces migration risk.

### Should stack pinning survive the migration?

Probably not as a hard dependency.

If direct pane targeting makes the workflow feel right, stack pinning should either become optional polish or be removed.

## Recommendation

Proceed with a narrow plugin MVP centered on title-based pane identity.

The primary invariant should be:

> Yazelix can always target the managed editor pane in the current tab directly.

Not:

> Yazelix can usually find the editor by scanning a few nearby panes and repairing the stack afterward.
