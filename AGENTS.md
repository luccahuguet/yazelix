# Agent Guidelines

Yazelix Next is a clean architecture track for a Yazelix-like runtime with the
fewest practical lines of code and the simplest ownership model

## Core Rule

The user decides scope. Do not create a feature, compatibility surface, module,
or planning bead until the user has chosen that direction

## Method

Use contract-driven, check-backed development, not mechanical porting from main
Yazelix:

1. State the irreducible user-visible behavior in one paragraph
2. Name the current Yazelix sources of truth and decide what survives
3. Choose one owner in this repo
4. Choose the cheapest check that proves the contract
5. Implement the smallest slice that satisfies the contract
6. Avoid duplicate owners, adapters, generated fixtures, and compatibility shims
7. Record important rejected alternatives in Beads

Start with the smallest usable vertical slice and polish it before expanding

Use TDD where it fits: Rust helpers, parsers, deterministic CLI behavior, and
regression fixes. Do not use classic TDD as the default for layout design,
runtime integration, fork decisions, or dogfooding surfaces; write the contract
and the focused check first.

## Testing Discipline

Keep tests strong and few. A test should prove a contract, regression, boundary,
or failure mode that matters to users or future agents.

Do not keep weak tests. If a test only repeats another check, asserts
implementation trivia, or mostly preserves scaffolding, either merge its useful
assertion into a stronger test or delete it.

Prefer one contract test with clear setup and meaningful assertions over several
thin tests that make refactors harder without increasing confidence.

## Current Runtime

Current chain:

```text
yzn -> Mars -> Yazelix Zellij fork
```

The project interface is a Nix/Lix-compatible flake. `yzn` is the installed
command name so it does not conflict with main Yazelix `yzx`

After changing the flake runtime, keep the user's installed runtime current:

```sh
nix profile upgrade --refresh yazelix-next
```

Do not add Home Manager, layouts, config generation, plugins, pane policy, or
legacy compatibility unless the user explicitly chooses that feature

## Beads

Use `br` for all issue work. Do not edit `.beads/` files directly

Serialize `br` write commands. Keep decisions that matter later in Beads rather
than relying on chat history

## LOC Discipline

Update the README LOC scorecard whenever project files change

Update `CHANGELOG.md` when user-visible runtime behavior, commands, keymaps,
packaged tools, or runtime contracts change

Prefer deleting scope, avoiding abstractions, and reusing existing package
outputs over adding local wrappers. If LOC grows, the added behavior should be
visible in the scorecard and justified by the slice

## Verification

Run the cheapest exact checks for the changed surface. For runtime flake
changes, normally verify:

```sh
nix flake check
nix flake show --all-systems
nix build .#yzn --no-link --print-build-logs
nix profile add --refresh /home/lucca/pjs/yazelix-dir/yazelix-next --profile <tmp>
```

Do not launch GUI sessions unless the user asks or reports manual dogfooding
