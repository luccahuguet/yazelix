# Agent Guidelines

This file is self-contained. Canonical protocol text was rendered into it;
the source repository is needed only to update or verify the import.
Do not edit this generated file directly. Edit `.agent-protocols.local.md`
or `.agent-protocols.exceptions.json`, then render from the pinned source.

## Protocol import record

- Source: `https://github.com/luccahuguet/starcompass`
- Source commit: `83c8fa73744ff4fd15bc2f7c5f1978ca96756e06`
- Profiles: `orchestrator`, `release`
- Manifest: `.agent-protocols.json` (schema 1)

| Protocol | Version | SHA-256 |
| --- | ---: | --- |
| `AP-SCOPE-001` | 1 | `b3f7e012df0708d4baf8957e3c315878a9eb8cd7fddf637dfde1506609d08444` |
| `AP-CONTRACT-001` | 1 | `0aa692f4c52542111149b691b7d0c015a0c16523cd620a67b0cf335f3284da82` |
| `AP-REFERENCE-001` | 1 | `3d72ac864b050af7783493245be59450c39db940d736ea6fb60269cd21f7927b` |
| `AP-MINIMAL-001` | 1 | `256c158cc8b226e4baf96d5590531ea180edc9717338dac0fe51a34dd037791f` |
| `AP-DEPENDENCY-001` | 1 | `a389ff9054708574c52ec5e5dd7fc3e2d13b125d2218c70062f50a86761981ca` |
| `AP-OWNERSHIP-001` | 1 | `bdc09117f79b0d8dbe78e2dd8673a2463398aa31880fe27209fd6cc47f58bbf7` |
| `AP-TEST-001` | 1 | `363da7c542521be22233a4cc3373c0d3c3c5a9a0cf37f633cd5029545f4a3bee` |
| `AP-PROOF-001` | 1 | `1af235a56e9711d55362d869fa4057f1658d0aa7fe766030be0897ee5fd7c02b` |
| `AP-PLAN-001` | 4 | `5bc0426788693717e8af011277d9358c65899e839d7c0c71299003ec0d8acc4b` |
| `AP-CI-001` | 1 | `78f1662259cd83f33d22ff4ddd0859ab0d4f704ba4f38756eef40a8b9b787bec` |
| `AP-ORCHESTRATOR-001` | 1 | `6228280f59c46dae79e45986d3848f071fd57699c8ce548ec2f7e503689e09c8` |
| `AP-FRONTIER-001` | 1 | `60e397fa4862afe902792501ffc6d2a82c94479877196b4278e7283a20111fe0` |
| `AP-PORTABILITY-001` | 1 | `d2800376013bbe1ade3449f835daf8780e60d4c08a79a7f733d4a651c4d6d887` |
| `AP-EXCEPTION-001` | 1 | `f66749229dbbc005e1c3103bfed86cf95169d7b32466c72fd8442e841cadb268` |
| `AP-GIT-001` | 3 | `16d27b0df7ccc94880bb31020e822e32b37503f43c2cf7a69de333300cbfdacf` |
| `AP-DELIVERY-001` | 1 | `5e424f8f1adf9dc86864e3c0e209b47ed577e17baa74e3ac2219e2209a4ff80b` |
| `AP-PROMOTION-001` | 1 | `d75780b1ee1dd658f32c80ff5edda5bff342212569029d0eb3bdc382f471d66e` |

### Local exceptions

No local exceptions.

## Canonical protocols

### AP-SCOPE-001 — User-owned scope

The user decides product and project scope. An agent may inspect, explain, test,
or make the smallest implementation needed for the chosen goal, but it must not
silently create a feature, compatibility promise, public surface, migration,
repository, or planning item outside that direction.

Required practice:

- Separate safe implementation details from choices that change product scope.
- State consequential assumptions; stop when a missing choice would materially
  change the result.
- Treat a terminal instruction such as “finish” as persistence, not broader
  authority.
- Keep useful out-of-scope observations as findings unless the user has chosen
  a durable planning destination for them.

### AP-CONTRACT-001 — Contract-driven changes

State the irreducible externally observable behavior before choosing the code
shape. Give durable contracts stable identifiers when later code, tests, or
repositories need to cite them.

Required practice:

- Name the consumer, trigger, observable result, and important failure behavior.
- Identify the current sources of truth and decide which one owner survives.
- Choose the cheapest check that can falsify the contract.
- Implement the smallest vertical slice that satisfies it.
- Update the contract first when an intentional behavior change is chosen.

Do not turn implementation details into contracts unless another component must
rely on them.

### AP-REFERENCE-001 — Evidence before code shape

Review the relevant sources before deciding architecture or implementation
shape. Memory, summaries, and reputation are discovery aids, not sufficient
evidence for a consequential decision.

Required practice:

- Read the affected local code, contracts, tests, and repository instructions.
- Inspect designated external references at the subsystem named by local rules.
- Record the concrete mechanism adopted, rejected, or left unresolved.
- Distinguish direct source evidence from inference.
- Revisit the evidence when the proposed shape changes materially.

Reference review is a decision gate, not a requirement to copy the reference.

### AP-MINIMAL-001 — Minimum sufficient implementation

Understand the affected flow before choosing the smallest complete solution.
Use the first option that fully satisfies the chosen contract:

1. Make no change when the required behavior already exists.
2. Reuse an existing owner, helper, or pattern in the repository.
3. Use the standard library or a native platform capability.
4. Use an already accepted dependency that owns the behavior.
5. Implement the minimum local code that is correct and maintainable.

Prefer deletion over addition, direct ownership over adapters, and fewer files
over scaffolding. Minimalism must not remove required behavior, trust-boundary
validation, data-loss protection, security, accessibility, or the cheapest
runnable check for non-trivial logic.

Ponytail is the adopted agent-side implementation of this discipline when the
host supports it. Use the upstream project directly rather than copying its
rules or adapters. The reviewed source is
[DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail/tree/16f29800fd2681bdf24f3eb4ccffe38be3baec6b).
If Ponytail is unavailable or disabled, the self-contained requirements above
still apply. Its instruction hooks improve consistency; they do not prove
compliance.

### AP-DEPENDENCY-001 — Dependency gate

Choose dependencies for architectural fit and net system simplicity, not name
recognition or short-term convenience.

Before adding a crate, package, framework, service, or embedded project:

- State the capability and contract it would own.
- Consider the standard library, owned code, and multiple credible candidates.
- Compare maintenance, platform fit, correctness, transitive weight, licensing,
  API stability, and the lines and complexity removed.
- Record the chosen candidate, meaningful rejections, and replacement cost.
- Pin deliberately and add the smallest check that proves the relied-on behavior.

Remove a dependency when it no longer owns enough behavior to justify its cost.

### AP-OWNERSHIP-001 — One owner per invariant

Every invariant, state transition, and user-visible policy must have one clear
owner. Other components may consume its output; they must not independently
reconstruct or reinterpret the same truth.

Required practice:

- Name the owner before adding adapters or synchronization.
- Prefer deleting duplicate owners over reconciling them.
- Keep policy at the highest layer that has the necessary context and mechanism
  at the lowest layer that can enforce it correctly.
- Make cross-boundary data explicit and versioned when independently released
  components depend on it.

### AP-TEST-001 — Strong and few tests

Tests exist to protect contracts, regressions, boundaries, and failure modes
that matter to users or future agents. Prefer one strong test with meaningful
setup and assertions over several thin tests.

Required practice:

- Use TDD for deterministic helpers, parsers, protocol behavior, and regressions
  when the expected behavior can be stated before implementation.
- Choose contract-first integration checks for layout, runtime integration,
  architecture choices, forks, and dogfooding surfaces.
- Delete or merge tests that duplicate another proof, assert implementation
  trivia, or preserve scaffolding.
- Test observable effects rather than mirroring literals, defaults, or source
  structure.
- Add absence guards only when absence is itself a security, licensing, size,
  ownership, or known-regression contract.

### AP-PROOF-001 — Explicit proof lifecycle

Claims and proofs have a lifecycle. A passing check supports only the exact
revision, environment, and surface it exercised.

Required practice:

- Record the command or observation, relevant environment, revision, and result.
- Distinguish proposed, implemented, mechanically verified, manually dogfooded,
  accepted, and promoted states.
- Re-run stale proof after relevant code, dependency, platform, or contract
  changes.
- Never promote a narrower check into a broader claim.
- Preserve important negative results; they constrain the next valid design.

### AP-PLAN-001 — Durable planning state

Keep the outcomes and constraints that later work needs in the project's
durable planning system or canonical documentation. An issue represents a
chosen goal, decision, material defect, or schedulable follow-up. Review and
implementation methods belong to that issue.

Required practice:

- After review, fresh-eyes, simplification, or verification, update the owning
  issue's editable fields to describe the accepted state instead of pass
  chronology.
- Create a separate issue only for a material finding outside the owning scope
  or one worth scheduling on its own. Name it after the outcome or finding.
- Record the contract, decision boundary, dependencies, acceptance evidence,
  material negative results, and rejected alternatives that constrain later
  work.
- Reserve append-only comments and audit records for chronology needed as
  evidence. Keep raw command logs and build transcripts with their proof. Omit
  baseline hashes, failed attempts, and candidate scoring unless they constrain
  later work.
- Keep issue status honest: planned, active, blocked, and complete are distinct.
- Model real prerequisites as dependencies; do not create decorative graphs.
- Reconcile planning state with the repository before handoff.
- Use the repository-designated issue tool and never edit its storage directly.

Do not erase approvals, contract changes, material failures, or evidence needed
to understand the accepted result.

### AP-CI-001 — Bounded continuous integration

Hosted automation must buy enough confidence to justify its financial,
latency, security, and maintenance cost.

Before enabling CI:

- Name the protected contract and why local verification is insufficient.
- Bound triggers, job count, timeouts, permissions, artifacts, cache growth, and
  concurrency.
- Prefer one cheap deterministic job before matrices or scheduled runs.
- Make fork and secret behavior explicit.
- Record the evidence required to expand, reduce, or remove the workflow.

Private-repository minutes and cache storage are product constraints, not an
invisible externality.

### AP-ORCHESTRATOR-001 — Thin orchestrator ownership

An orchestrator owns composition, lifecycle policy, compatibility selection,
and user-facing defaults. It must not absorb mechanisms or state already owned
by its child components.

Required practice:

- Consume released or pinned child artifacts through explicit contracts.
- Keep child-specific translation at a narrow boundary.
- Reject copies, hidden forks, and duplicate configuration schemas.
- Make startup, shutdown, update, and partial-failure policy observable.
- Validate the orchestrator independently with substitutes or fixtures when a
  child is not yet ready.

### AP-FRONTIER-001 — One active integration frontier

When several greenfield components are intended to work together, keep one
integration boundary under active architectural change whenever practical.

Required practice:

- Stabilize the contracts on the other sides with released artifacts, fixtures,
  adapters, or existing interchangeable projects.
- Dogfood the active frontier through the smallest real vertical slice.
- Move the frontier only after its contract and failure behavior are supported
  by evidence.
- Allow parallel internal work when it does not make multiple integration
  contracts speculative at once.

This is a risk-control default, not a ban on parallel implementation.

### AP-PORTABILITY-001 — Portable core with explicit platform seams

Do not let one operating system's APIs, process model, paths, packaging, or
event facilities become an accidental foundation when supported targets are
broader.

Required practice:

- Keep platform-neutral contracts and state in the core.
- Isolate OS-specific code behind the smallest meaningful seam.
- Evaluate Linux and macOS implications before adopting foundational runtime,
  process, graphics, filesystem, or transport dependencies.
- Prove platform behavior on the platform; compilation alone is narrower
  evidence.
- Record intentionally unsupported platforms rather than implying portability.

### AP-EXCEPTION-001 — Explicit local exceptions

A local rule may narrow, replace, or suspend an imported protocol only through
an explicit exception approved by the user or named project authority.

Each exception records:

- the protocol ID;
- its exact scope;
- the reason the canonical rule does not fit;
- who approved it and when;
- an expiry or review condition when the exception is temporary.

Unrecorded conflicts are drift. A local rule that merely adds detail without
changing the canonical requirement is an overlay, not an exception.

### AP-GIT-001 — Safe repository history

Repository history is shared user state. Preserve unrelated work, follow the
local branch and promotion model, and use the least destructive operation that
achieves the requested result.

Required practice:

- Inspect status and repository instructions before editing.
- Treat existing and concurrent changes as user-owned unless proven otherwise.
- Fold a correction into the current task's unpublished commit when it belongs
  to the same unit of work. Refresh and reverify dependent local commits and
  generated artifacts.
- Use a follow-up commit after a push, promotion, release, external pin, or any
  other point where someone outside the current local work can rely on the
  revision.
- Do not reset, discard, force-push, rewrite published history, or create a
  branch without authority from the user or repository policy.
- Verify the intended diff before committing and the remote state after pushing.
- Make rollbacks additive through a reviewed revert unless policy says otherwise.

### AP-DELIVERY-001 — Installed artifact and fresh-session proof

Source-tree checks do not prove the artifact or runtime experience delivered to
a user. When the changed surface has a package, installation, activation, or
interactive-runtime contract, verify that surface before calling the revision
releasable.

Required practice:

- Build through the intended delivery path from the exact candidate revision.
- Record the artifact identity, relevant environment, commands, and result.
- Install or activate the artifact in an isolated destination without silently
  falling back to the development checkout or stale generated state.
- Verify that the installed program or package identifies the intended
  candidate when the product exposes an identity surface.
- Dogfood user-visible interaction changes from a newly started session; an
  already-running session does not prove startup, packaging, or activation.
- Keep claims narrower when a platform, installer, or interaction cannot be
  exercised. Missing proof does not become implied support.

Automate deterministic delivery checks where useful, but do not describe
compilation alone as installation proof or a scripted probe as manual dogfood.

### AP-PROMOTION-001 — Exact-revision channel promotion

Release channels are ordered references over one linear history, not independent
development lines. The consumer declares the channel order, required evidence,
and promotion authority; this protocol does not prescribe branch names or
release cadence.

Required practice:

- Declare the channel ancestry invariant and the development channel where all
  tracked changes originate, including fixes, reverts, documentation, and
  planning state.
- Treat accepted and user-facing channels as promotion-only. Advance each by
  fast-forwarding it to the exact verified revision already present in its
  immediate predecessor.
- Do not create direct channel commits, merge commits, cross-channel
  cherry-picks, published-history rebases, force-pushes, or skipped promotion
  stages.
- Require the checks and delivery evidence defined for the target channel, and
  block a candidate with a known critical or high-severity regression.
- Require explicit user authority before advancing a user-facing channel.
- Roll back through a new revert on the development channel, then verify and
  promote that revision through the normal order.

Promotion changes exposure, not evidence. Moving a reference must never turn an
unproved revision into a proved one.

## Repository-local rules

### Yazelix Nova

Yazelix Nova is a clean architecture track for a Yazelix-like runtime with the
fewest practical lines of code and the simplest ownership model.

### Nova Boundary

Do not mechanically port main Yazelix. Review the current Yazelix sources of
truth and decide explicitly what survives.

Current runtime chain:

```text
yzx -> Mars -> Yazelix Zellij fork
```

The project interface is a Nix/Lix-compatible flake. `yzx` is the installed
command name. Do not broaden Home Manager, layouts, config generation, plugins,
pane policy, or legacy compatibility unless the user chooses that scope.

### Git Channels

Keep one linear history with this ancestry invariant:

```text
stable ⊆ main ⊆ edge
```

All tracked changes originate on `edge`, including fixes, reverts,
documentation, and Beads updates. Work directly on `edge` by default.

`edge` is the active development and experimental dogfood channel. `main` is a
promotion-only accepted-development channel. After an `edge` revision is
accepted and verified for `main`, advance `main` to that exact revision with:

```sh
git push origin <sha>:main
```

`stable` is the promotion-only user channel. Advance it only when the user
explicitly requests promotion. A candidate must be a fast-forward from the
current `stable`, belong to `main`, pass the protected Linux and cache checks,
pass the release checks for its changed surface, and have fresh-session dogfood
for user-visible runtime interaction changes. Do not promote a commit with a
known P0 or P1 regression. Promote the exact verified revision with:

```sh
git push origin <sha>:stable
```

Never delete `stable`.

### Beads

Use `br` for all issue work. Serialize writes and finish with
`br sync --flush-only`.

Use `bv --robot-triage` as the graph-aware planning entry point. Use only
`bv --robot-*` commands; bare `bv` opens an interactive TUI. `bv` decides what
to work on, while `br` creates, updates, and closes issues. Before claiming a
recommendation, verify it with `br show <id> --json` or `br ready --json`.

### LOC and Documentation

Update the README LOC scorecard whenever project files change. Update
`CHANGELOG.md` when user-visible runtime behavior, commands, keymaps, packaged
tools, or runtime contracts change.

If LOC grows, make the added behavior visible in the scorecard and justify it.
Formatting rules outrank LOC pressure; for Rust, keep `rustfmt` output rather
than compressing code manually.

### Nova Verification

For runtime flake changes, normally verify:

```sh
nix flake check
nix flake show --all-systems
nix build .#yazelix --no-link --print-build-logs
nix profile add --refresh /home/lucca/pjs/yazelix-dir/yazelix --profile <tmp>
```

After changing the flake runtime, keep the user's installed runtime current:

```sh
nix profile upgrade --refresh yazelix
```

Do not launch GUI sessions unless the user asks or reports manual dogfooding.
