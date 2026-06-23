# Terminal Support Boundary

## Summary

`yazelix-terminal-support` may become the child owner for static terminal
support metadata. It must not own launch behavior, generated config rendering,
Home Manager option semantics, or child-terminal internals.

The child is useful only if it removes a second hand-maintained authority in
main Yazelix. It is not useful as a mirror of `packaging/runtime_tool_registry.nix`
or `terminal_materialization.rs`.

## Ownership

| Fact or behavior | Owner |
| --- | --- |
| Terminal ids and labels | `yazelix-terminal-support` after extraction |
| Supported command names and package output names | `yazelix-terminal-support` after extraction |
| Static platform gates and unsupported-platform reason strings | `yazelix-terminal-support` after extraction |
| Support/capability hints such as graphics-wrapper needs or known protocol support | `yazelix-terminal-support` after extraction |
| Validation metadata for terminal metadata parity | `yazelix-terminal-support` after extraction |
| Runtime variant selection and package composition | Main Yazelix Nix package builders |
| Home Manager option semantics and defaults | Main Yazelix Home Manager module |
| `runtime_tools.json` projection and runtime source-mode validation | Main Yazelix package assembly until a real deletion slice proves otherwise |
| Terminal launch argv, detach behavior, and platform launch flags | Main Yazelix launch boundary |
| Generated terminal config materialization | Main Yazelix until a separate terminal config-pack boundary deletes it |
| Doctor diagnostics and repair behavior | Main Yazelix doctor aggregation |
| yzxterm wrapper/package internals | `yazelix-terminal` |
| Cursor shader and preset artifacts | `yazelix-cursors` |

## Allowed Child Data

The terminal-support child may expose pure data and validators:

- terminal id
- user-facing label
- executable command names
- package output names
- runtime output names
- host/package source support
- platform support predicates represented as data
- capability/support hints
- package source policy
- schema version

The child must not expose a rule language that decides how to launch a
terminal, compose a Home Manager package, generate terminal config text, or
repair a user runtime.

## Promotion Gate

Main may promote terminal-support metadata into a child only when all gates
pass:

1. The child schema is pure data with a documented version
2. Main consumes the data through typed Rust or Nix readers, not ad hoc string
   scraping
3. The same change deletes or relinquishes a real main-owned metadata table,
   validator, fixture, or duplicated test
4. Generated `runtime_tools.json` and `yzx inspect --json` continue to expose
   the active runtime facts that external consumers need
5. Unsupported terminal ids still fail fast with clear main-owned user errors
6. yzxterm-specific package/profile/shader semantics remain in
   `yazelix-terminal` or `yazelix-cursors`
7. Terminal config rendering remains outside terminal-support unless a separate
   config-pack extraction proves output equivalence and deletes main code

## Verification

- `yzx_repo_validator validate-nix-customization-api`
- `yzx_repo_validator validate-flake-interface`
- focused Rust tests for any typed metadata reader added in main
- child tests for schema and platform-gate parity
- `shells/posix/yazelix_loc_scorecard.sh <base> HEAD` for extraction commits
