# Standalone Ghostty Cursor Distribution

## Summary

Yazelix should make its Ghostty cursor shaders available to standalone Ghostty
users through an in-repo export first. Do not split a separate repository yet.

The supported first path should be a small flake/package artifact plus docs that
install or expose ready-to-use Ghostty shader files and example config snippets.
The full `yazelix_cursors.toml` sidecar, random reroll behavior, Kitty fallback,
and Yazelix runtime materialization should stay Yazelix-owned.

## Why

The cursor work now has value outside the full Yazelix workspace: standalone
Ghostty users may want a curated cursor trail without adopting Zellij, Yazi,
Yazelix config snapshots, or the Yazelix runtime package.

The delete-first boundary is important. A standalone cursor distribution should
not copy the whole Yazelix terminal-generation system outward. It should expose
the smallest useful asset set for Ghostty users:

- generated or ready-to-generate GLSL shader files
- minimal Ghostty config snippets
- a stable way to choose a named preset
- update instructions that do not mutate user config automatically

## Audience

Supported first audience:

- Ghostty users on Linux or macOS who already manage their own Ghostty config
- Nix users who can consume a flake package or build output
- Yazelix users who want to reuse the same shaders outside a Yazelix window

Not the first audience:

- users who want Yazelix to own their whole Ghostty config
- users who need a graphical installer
- non-Nix users who want a curl installer
- Kitty users, because the Kitty trail fallback is a Yazelix compatibility
  behavior rather than a Ghostty shader asset

## Asset Boundary

Generic enough to publish:

- `configs/terminal_emulators/ghostty/shaders/cursor_trail_common.glsl`
- `configs/terminal_emulators/ghostty/shaders/variants/*.glsl`
- selected upstream effect shaders under
  `configs/terminal_emulators/ghostty/shaders/upstream_effects/`
- generated shader outputs when the export chooses to build them
- example Ghostty config snippets such as:
  - `custom-shader = <installed-path>/cursor_trail_blaze.glsl`
  - `custom-shader-animation = true`

Yazelix-specific and not part of the standalone public contract:

- `yazelix_cursors.toml` as a user config file
- `enabled_cursors`, `random`, and per-window reroll state
- Kitty cursor fallback settings
- runtime materialization and generated-state repair
- `yzx edit cursors`
- Zellij/Yazelix launch integration

## Distribution Options

### Option 1: Docs-Only Copy Path

Pros:

- cheapest to ship
- no new package output
- keeps maintenance inside the current repo

Cons:

- users copy raw files manually
- updates are unclear
- no stable installed path for `custom-shader`
- too easy to copy source fragments that are not complete shaders

Decision: acceptable as a short interim note, not the target supported path.

### Option 2: In-Repo Flake/Package Export

Pros:

- gives users a stable Nix profile path such as
  `~/.nix-profile/share/yazelix-ghostty-cursors/`
- keeps release cadence tied to Yazelix until the boundary proves itself
- avoids a second repository, issue tracker, and release process
- can ship generated shader files plus examples without exposing Yazelix runtime
  state

Cons:

- still couples standalone cursor releases to Yazelix releases
- non-Nix users only get manual copy instructions at first
- the export needs its own validation so broken shaders do not ride along
  invisibly

Decision: supported first path.

### Option 3: Separate Repository

Pros:

- clearer identity for non-Yazelix Ghostty users
- independent releases and README
- easier for other projects to consume without Yazelix context

Cons:

- duplicates maintenance, issue triage, release notes, and versioning
- requires explicit licensing/provenance review for vendored upstream effect
  shaders
- risks splitting the shader source while Yazelix still actively changes its
  cursor settings model
- premature until the in-repo export proves there is real demand

Decision: defer. Re-evaluate after the in-repo export has shipped and the shader
source boundary stays stable for at least one release cycle.

## Supported Install Path

First supported path:

1. Install or build the cursor asset package from the Yazelix flake once it
   exists.
2. Add a `custom-shader` line to the user's own Ghostty config pointing at the
   installed shader file.
3. Leave the user's Ghostty config owner unchanged.

Yazelix should not automatically edit standalone Ghostty config files. Ghostty's
official docs warn that invalid custom shaders can make a window unusable, so
the safer product stance is explicit user opt-in with visible paths and rollback
instructions.

## Maintenance Boundary

Owned by Yazelix until extraction:

- source shader files
- generated shader output shape
- example Ghostty config snippets
- package/export path
- validation that the exported files exist and are complete shaders

Not owned by Yazelix:

- the user's `~/.config/ghostty/config.ghostty` or legacy `config`
- choosing where in a larger personal Ghostty config the user includes the
  shader snippet
- supporting every Ghostty renderer/platform bug
- providing a stable API for `yazelix_cursors.toml` outside Yazelix

Extraction gate:

- the in-repo export has shipped
- at least one release has used the same asset boundary without churn
- shader provenance and license notes are explicit
- a separate release/version policy exists
- update instructions for both Nix and manual users are written
- Yazelix can consume the separate repo without making its runtime generation
  less reliable

## Acceptance Cases

1. A standalone Ghostty user can find which package/export to install.
2. The install path exposes complete shader files, not partial source fragments.
3. The user can add a documented `custom-shader` entry without running Yazelix.
4. Yazelix does not take ownership of external Ghostty config files.
5. The separate-repo path remains blocked until maintenance and release
   ownership are explicit.

## Verification

- local audit:
  - `rg -n "cursor|ghostty_trail|yazelix_cursors|shader|trail" yazelix_cursors_default.toml yazelix_default.toml configs/terminal_emulators configs docs README.md packaging rust_core home_manager`
  - `rg --files configs assets packaging | rg 'ghostty|cursor|shader|trail'`
- external reference:
  - `https://ghostty.org/docs/config`
  - `https://ghostty.org/docs/config/reference#custom-shader`
- CI/spec check:
  - `cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-specs`

## Traceability

- Bead: `yazelix-lo9h`
- Defended by: `docs/specs/standalone_ghostty_cursor_distribution.md`
- Defended by: `yzx_repo_validator validate-specs`
