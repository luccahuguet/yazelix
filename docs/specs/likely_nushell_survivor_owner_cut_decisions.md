# Likely Nushell Survivor Owner-Cut Decisions

## Summary

This decision record re-evaluates the remaining likely Nushell survivors after
the product-side full-config owner cut, the public Rust command-family cuts,
and the recent bridge-collapse work.

It is now background from the earlier, softer survivor pass.

Use `provable_nushell_floor_budget.md` first for the current Rust-first proof
standard and under-`5k` family budget. This file is still useful as design
history for why the earlier pass rejected fake broad Rust rewrites, but it is
no longer the top-level stopping rule for retained Nu.

The result is intentionally strict:

- do not promise a broad Rust rewrite where the surviving value is still shell,
  process, Zellij, XDG, or human-rendering behavior
- only keep a Rust lane when it deletes a real Nushell owner end to end
- record explicit no-go boundaries where a future rewrite would only add a new
  wrapper above the same Nu/POSIX work

The only current family with a live delete-first Rust-adjacent lane is the
launcher/runtime-helper cluster, and even there the honest follow-ups are the
already-created `yazelix-nuj1` and `yazelix-p18h` beads rather than a new broad
Rust wrapper.

## 1. Setup, Initializers, And Welcome

### Reviewed files

- `nushell/scripts/setup/environment.nu`
- `nushell/scripts/setup/initializers.nu`
- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/core/start_yazelix_inner.nu`

### Retained behavior

- shell initializer generation for Bash, Zsh, Fish, and Nushell
- shellhook/runtime-root/log-dir orchestration before entering the live shell
- welcome playback, keypress waiting, and user-facing startup copy
- startup profiling and skip-welcome behavior

### Decision

No new honest Rust owner cut remains here.

`runtime-env.compute` and `startup-facts.compute` already took the deterministic
subcore out of Nushell. What survives now is shell-specific initializer text,
external-tool init probing, welcome playback, and startup UX. A new Rust move
would mostly wrap the same shell behavior without deleting the real owners.

### Viable follow-up

None as a Rust-owner cut.

Future work may still simplify welcome copy or renderer structure, but that is
not the same as a new honest Rust migration lane.

### Stop condition

Reopen only if one future change can delete `environment.nu`,
`initializers.nu`, or `welcome.nu` as an end-to-end owner instead of inserting
another helper layer.

## 2. Front-Door UX Renderers

### Reviewed files

- `nushell/scripts/yzx/menu.nu`
- `nushell/scripts/setup/welcome.nu`
- `nushell/scripts/utils/front_door_runtime.nu`

### Retained behavior

- command-palette rendering and `fzf` interaction
- popup/menu workflow and post-action behavior
- startup-shell welcome gating and logging
- editor/import process handoff after the Rust front-door owner cut

### Decision

No broad Rust owner cut is honest today.

`yzx menu` already consumes Rust-owned command metadata and popup facts; the
surviving work is `fzf` interaction and popup UX. `yzx screen`, `tutor`, and
`whats_new` are text-heavy and product-feel-heavy surfaces. A Rust rewrite
would not delete the real interaction and presentation ownership cleanly.

### Viable follow-up

None as a Rust-owner cut.

The remaining honest work here is delete-first renderer and style cleanup, not
another language migration promise.

### Stop condition

Do not reopen a Rust lane unless one future cut makes Rust the single owner of
the retained renderer behavior without keeping a parallel Nu presentation stack.

## 3. Session, Restart, And Desktop Command Bodies

### Reviewed files

- `nushell/scripts/core/yzx_session.nu`
- `nushell/scripts/yzx/desktop.nu`
- `nushell/scripts/yzx/launch.nu`

### Retained behavior

- Zellij session discovery, restart, and kill/reattach flow
- desktop-entry install/uninstall side effects and cache refresh
- desktop launch env cleanup and leaf launch delegation

### Decision

No broad Rust owner cut is honest today.

`yzx restart` is still Zellij- and process-heavy, and `yzx desktop launch`
stays tied to XDG and launcher execution semantics. The install/uninstall path
already uses Rust-owned install-ownership evaluation where that computation is
typed; what remains is mostly host integration and shell/process choreography.

### Viable follow-up

None as a Rust-owner cut.

Any future work here should target smaller cleanup or documentation seams, not
pretend that restart or desktop launch has become a deterministic Rust domain.

### Stop condition

Reopen only if one future cut can delete `yzx_session.nu` or a substantial
slice of `yzx/desktop.nu` outright instead of wrapping the same host-side
commands in Rust.

## 4. Launcher, Platform Detection, And Runtime Shell Helpers

### Reviewed files

- `nushell/scripts/utils/terminal_launcher.nu`
- deleted `nushell/scripts/utils/nix_detector.nu`
- `nushell/scripts/utils/common.nu`
- adjacent launch/runtime callers in `start_yazelix.nu`, `launch_yazelix.nu`,
  and POSIX launchers

### Retained behavior

- terminal-specific command assembly and detached launch
- runtime helper discovery and failure rendering
- platform/XDG/root-path shaping needed by live shell callers
- maintainer-side Nix availability detection

### Decision

There is no broad new Rust rewrite here, but there are still two honest
delete-first follow-ups already tracked:

- `yazelix-nuj1`
  - collapse launch-time terminal and Ghostty request assembly around the
    existing Rust materialization owners
- `yazelix-p18h`
  - move the fixed detached-launch probe shell body into one checked-in POSIX
    helper and delete the inline Nushell shell-program assembly

Outside those two cuts, the remaining launcher/runtime helper code is still
mostly shell and process ownership. `nix_detector.nu` was maintainer-only and
interactive; moving it to Rust would not delete the real operator-facing logic.
`common.nu` still carries shell-facing path/root helpers that remain coupled to
Nu/POSIX launch code.

### Viable follow-up

- `yazelix-nuj1`
- `yazelix-p18h`

No additional Rust-owner bead is warranted until one of those cuts lands or a
new typed subcore appears that deletes more than helper transport.

### Stop condition

Do not add a new Rust helper layer for terminal launch, Nix detection, or root
path discovery unless it deletes the Nushell owner instead of rewrapping the
same shell behavior.

## Verification

- manual review of the listed Nushell and POSIX files
- `yzx_repo_validator validate-specs`

## Traceability

- Bead: `yazelix-qd6b.1`
- Bead: `yazelix-qd6b.2`
- Bead: `yazelix-qd6b.3`
- Bead: `yazelix-qd6b.4`
- Defended by: `yzx_repo_validator validate-specs`
- Informed by: `docs/specs/setup_shellhook_welcome_terminal_canonicalization_audit.md`
- Informed by: `docs/specs/launch_startup_session_canonicalization_audit.md`
- Informed by: `docs/specs/ranked_nu_deletion_budget.md`
