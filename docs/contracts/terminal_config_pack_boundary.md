# Terminal Config Pack Boundary

## Summary

Terminal config rendering is separate from terminal-support metadata. Main
Yazelix currently owns deterministic terminal config materialization for
Ghostty, WezTerm, Rio, Foot, and Ratty. A future child config pack is
allowed only when it makes one of those renderers a pure request/output
function and deletes the matching main branch.

The preferred future child name for third-party terminal config rendering is
`yazelix-terminal-config-pack`. yzxterm-specific config/profile rendering may
instead belong in `yazelix-terminal` if the renderer is genuinely tied to
yzxterm package internals.

## Ownership

| Surface | Owner |
| --- | --- |
| Active terminal selection | Main Yazelix |
| Normalized semantic settings and cursor settings | Main Yazelix |
| Runtime root, state root, config root, and write destinations | Main Yazelix |
| Home Manager and package composition | Main Yazelix |
| Terminal launch argv and desktop launch behavior | Main Yazelix |
| Filesystem writes, atomic replacement, repair plans, and doctor output | Main Yazelix |
| Static terminal support metadata | `yazelix-terminal-support` after extraction |
| yzxterm package internals and package-owned profile files | `yazelix-terminal` |
| Cursor shader artifacts and cursor preset registry | `yazelix-cursors` |
| Pure third-party terminal config rendering | `yazelix-terminal-config-pack` only after deletion evidence |

## Renderer Boundary

A config-pack renderer must be a pure function:

```text
render_terminal_config_pack(request) -> output
```

The request may contain only explicit data supplied by main:

- selected terminal id
- normalized terminal settings
- normalized appearance settings
- normalized cursor settings or already-rendered cursor artifact paths
- runtime package paths selected by main
- package profile facts selected by main
- schema version and deterministic fingerprint input

The output may contain:

- rendered config text keyed by target path
- renderer warnings or validation errors
- copied package profile files represented as explicit output actions
- a renderer schema/version field

The renderer must not read:

- `~/.config/yazelix`
- `~/.local/share/yazelix`
- `YAZELIX_RUNTIME_DIR` or other runtime environment variables
- Home Manager ownership state
- live terminal, Zellij, Yazi, or editor state
- main-repo source paths
- adjacent child checkouts

## First Slice

The first implementation slice should move one terminal family only. It should
prefer a renderer with small deterministic output, such as WezTerm, Foot, or
Rio, before moving Ghostty.

Ghostty is larger because its current materialization interacts with cursor
shader paths and runtime paths. Move it only after the request/output shape has
already proved itself.

## Deletion Bar

Extraction counts only when main deletes real ownership:

- terminal-specific render functions
- terminal-specific palette/profile patching code
- renderer fixtures or duplicated output tests
- validator logic whose only job was to guard the moved renderer

It does not count if main keeps an equivalent renderer, fallback templates, or
large adapter branches around the child output.

## Verification

- child output-equivalence tests for the moved terminal family
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core terminal_materialization`
- `yzx_repo_validator validate-config-surface-contract`
- `yzx_repo_validator validate-child-release-transaction` for a consumed child
  commit
- `shells/posix/yazelix_loc_scorecard.sh <base> HEAD` for deletion evidence
