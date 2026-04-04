# Runtime Activation State Contract

## Summary

Yazelix should treat live session activation state as a first-class runtime layer, separate from dynamic user intent, deterministic runtime code, and materialized/generated state.

Live session activation state is the process-local environment that makes a built Yazelix profile active right now. It includes markers like `DEVENV_PROFILE`, `PATH` entries contributed by profile activation, `IN_NIX_SHELL`, `IN_YAZELIX_SHELL`, `YAZELIX_TERMINAL`, and Zellij session markers. It is not the same thing as persisted runtime truth such as `launch_state.json`, rebuild hashes, or generated runtime assets under `~/.local/share/yazelix`.

## Why

Recent launch/runtime bugs were caused by collapsing two different ideas into one:

- materialized state that Yazelix records and can reuse later
- live shell/session activation that only describes the current process tree

Without naming that split explicitly, helpers keep making the wrong leap:

- treating a stale maintainer shell as if it were the currently launched Yazelix session
- letting ambient `DEVENV_PROFILE` override a fresher recorded launch profile
- persisting shell-local state as if it were durable runtime truth
- forgetting that desktop or other external entrypoints should start from a clean activation surface

## Scope

- define what belongs to live session activation state
- define what does not belong to live session activation state
- define how activation state relates to recorded launch state and built profiles
- define when runtime helpers may trust activation markers
- define how external-launch helpers should treat inherited activation state

## Behavior

- Yazelix has four runtime-relevant layers:
  1. dynamic user intent
  2. deterministic runtime code
  3. materialized/generated state
  4. live session activation state
- Live session activation state is process-local and ephemeral.
  - It is usually created by entering a `devenv` shell, launching Yazelix, or activating a built profile.
  - It ends when the current process tree ends.
- Live session activation state includes values such as:
  - `DEVENV_PROFILE`
  - profile-derived `PATH` activation
  - `IN_NIX_SHELL`
  - `IN_YAZELIX_SHELL`
  - `YAZELIX_TERMINAL`
  - Zellij session markers such as `ZELLIJ`, `ZELLIJ_SESSION_NAME`, `ZELLIJ_PANE_ID`, and related session-local markers
- Live session activation state does not include persisted runtime truth such as:
  - `launch_state.json`
  - recorded rebuild hashes
  - generated configs under `~/.local/share/yazelix/configs/`
  - the runtime project workspace under `~/.local/share/yazelix/runtime/project`
  - the shipped runtime tree itself
- The built `devenv` profile path is a materialized runtime artifact.
  - `DEVENV_PROFILE` is the live activation of that artifact in the current process.
  - `launch_state.json.profile_path` is the persisted recorded profile Yazelix may reuse later.
- A stale maintainer shell can coexist with a correct `launch_state.json`.
  - That is not a contradiction.
  - It means the current shell activation is older than the recorded materialized launch state.
- Runtime helpers should trust live activation markers only when they are explicitly operating inside the current live Yazelix session.
  - Examples: in-session restart, pane helpers, popup helpers, session-local editor detection.
- External launch helpers should sanitize inherited live activation markers before starting a new Yazelix session from outside the current one.
  - Examples: desktop launch, launcher entrypoints, detached launch helpers.
- Install/setup/bootstrap work may materialize generated state, but it must not persist live activation state as durable launch truth.
- Future helper refactors should classify environment variables intentionally:
  - activation-only markers
  - persisted materialized-state recorders
  - explicit config/runtime/state roots

## Non-goals

- redefining the config root, runtime root, or state root themselves
- removing all activation-related environment variables in one step
- redesigning every launch helper in this spec alone
- promising that all current helpers already implement the final clean split

## Acceptance Cases

1. The runtime-facing specs explain why a stale maintainer shell can still have an older `DEVENV_PROFILE` while `launch_state.json` correctly points at the latest built launch profile.
2. The docs state clearly that `launch_state.json`, rebuild hashes, and generated runtime assets belong to materialized/generated state, not live activation state.
3. The docs state clearly that `DEVENV_PROFILE`, profile-derived `PATH`, `IN_YAZELIX_SHELL`, `YAZELIX_TERMINAL`, and Zellij session markers are activation-only markers rather than persisted runtime truth.
4. The docs define that external launch helpers should clear inherited activation markers before starting a new Yazelix session, while in-session helpers may intentionally use the current live activation state.
5. Later refactor beads can target the terms in this contract instead of continuing to treat activation state as an implicit side effect.

## Verification

- spec validation: `nu nushell/scripts/dev/validate_specs.nu`
- regression coverage: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- regression coverage: `nu nushell/scripts/dev/test_yzx_popup_commands.nu`

## Traceability

- Bead: `yazelix-le0s.1`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_workspace_commands.nu`
- Defended by: `nu nushell/scripts/dev/test_yzx_popup_commands.nu`

## Open Questions

- Should Yazelix eventually record a dedicated runtime identity artifact so launch helpers rely less on path- and env-derived heuristics when bridging between materialized state and live activation?
- Which remaining env vars should stay activation-only forever, and which should disappear once helper boundaries are narrower?
