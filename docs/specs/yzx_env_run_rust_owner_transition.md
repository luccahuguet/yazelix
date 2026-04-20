# yzx env/run Rust Owner Transition

## Summary

`yzx env` and `yzx run` should become the first Rust-owned public control-plane leaf commands without forcing a broad Rust rewrite of the whole `yzx` command tree. The public `bin/yzx` front door stays in place, but `shells/posix/yzx_cli.sh` should dispatch `env` and `run` directly to an internal Rust control-plane binary before the Nushell command tree loads. Nushell must stop being the public parser/owner for those two commands.

## Why

The current `env` / `run` path leaves too much shipped Nushell ownership in place:

- `nushell/scripts/yzx/env.nu` and `nushell/scripts/yzx/run.nu` are still public command owners
- `nushell/scripts/core/yazelix.nu` still re-exports them as part of the public command tree
- both commands currently call `prepare_environment`, which computes `config_state` even though `env` / `run` only need config and runtime env planning
- the surviving bridge/helpers keep `config_parser.nu`, `environment_bootstrap.nu`, and `runtime_env.nu` larger than they should be

If Rust is going to own more of Yazelix, it needs to delete real Nushell owners, not just sit behind additional wrappers.

## Scope

This spec defines:

- the transition shape for moving `yzx env` / `yzx run` onto a Rust-owned public leaf path
- which surfaces must stop being public Nushell owners
- the preserved behavior contract for both commands
- the compatibility expectations for the remaining v15/v16-hybrid command root and Nushell extern bridge
- the deletion budget required before the slice counts as successful

## Behavior

### Public Front Door

The canonical user-facing entrypoint remains `bin/yzx`.

For this slice, `bin/yzx` continues to route through `shells/posix/yzx_cli.sh`, but that wrapper becomes only a lightweight dispatcher. It should detect `env` and `run` and dispatch them directly to a dedicated internal Rust control-plane binary under `libexec/`.

Use a dedicated internal binary such as `libexec/yzx_control` for these direct leaf commands. Do not overload `yzx_core` with the first public-leaf CLI contract. `yzx_core` should remain the private JSON/helper binary used by Nushell-owned surfaces.

### Ownership Boundary

After this transition:

- `shells/posix/yzx_cli.sh` is the public dispatcher for `env` / `run`
- the Rust control-plane binary is the parser/executor owner for `env` / `run`
- `nushell/scripts/core/yazelix.nu` no longer `export use`s `../yzx/env.nu` or `../yzx/run.nu`
- `nushell/scripts/yzx/env.nu` and `nushell/scripts/yzx/run.nu` are deleted or demoted to clearly internal-only transitional helpers

Not allowed:

- keeping `export def "yzx env"` or `export def "yzx run"` in the public Nushell command tree just for compatibility
- routing `bin/yzx env` or `bin/yzx run` back through `use nushell/scripts/core/yazelix.nu *; yzx ...`
- treating Rust as only an argv shim above the unchanged Nushell wrappers

### Preserved `yzx env` Contract

The Rust-owned `yzx env` path must preserve the live command behavior:

- default mode launches the configured shell as a login shell using the current shell-name mapping behavior
- `--no-shell` stays in the invoking shell family when `$SHELL` is available; otherwise it falls back to the configured default shell family
- the current working directory is preserved
- `SHELL` in the activated runtime env reflects the launched shell executable
- shell launch failure remains a clear user-facing error and keeps the existing actionable tip about rerunning with `yzx env --no-shell`

### Preserved `yzx run` Contract

The Rust-owned `yzx run` path must preserve the live command behavior:

- the first token after `run` is the child command
- every token after that belongs to the child argv
- dash-prefixed child args and child flags such as `--verbose` must remain child argv, not be re-consumed as wrapper flags
- the current working directory is preserved
- the canonical Yazelix runtime env is activated before launching the child command
- a missing child command remains a usage error with non-zero exit

### Compatibility During The Hybrid Stage

This slice does not require a broad Rust rewrite of root help/completion or every public `yzx` command.

Allowed transitional compatibility:

- `nushell/scripts/core/yazelix.nu` may keep static prose that mentions `env` / `run`
- `nushell/scripts/utils/nushell_externs.nu` may render small explicit compatibility extern blocks for `yzx env` / `yzx run`

Not allowed:

- keeping `env` / `run` in the Nushell scope tree just so `scope commands` and generated extern discovery can still see them

The command palette already excludes `yzx env` and `yzx run`, so this slice should not need palette changes.

### Deletion Budget

The slice does not count as complete unless it deletes a real shipped Nushell owner seam.

Mandatory deletions or demotions:

- remove the two public `export use` lines for `env` / `run` from `nushell/scripts/core/yazelix.nu`
- delete or demote `nushell/scripts/yzx/env.nu`
- delete or demote `nushell/scripts/yzx/run.nu`
- stop using `prepare_environment` on the `env` / `run` path, so `config_state` is no longer computed there as wasted bootstrap work

Post-cutover cleanup must then audit and trim any env/run-only glue left in:

- `nushell/scripts/utils/environment_bootstrap.nu`
- `nushell/scripts/utils/runtime_env.nu`
- `nushell/scripts/utils/config_parser.nu`

The minimum acceptable outcome is a net reduction of at least 100 shipped Nushell LOC by the time `yazelix-ulb2.1` closes. If the wrappers disappear but equivalent bridge complexity is recreated elsewhere, the parent bead should remain open.

## Non-goals

- rewriting the full `yzx` root, help surface, or completion system in this bead
- moving `launch`, `enter`, or `restart`
- moving `status` or `doctor`
- revisiting Yazi/Zellij generator ownership here
- keeping public Nushell wrapper ownership just to minimize short-term implementation work

## Acceptance Cases

1. When a user runs `bin/yzx env`, the command reaches a Rust-owned parser/executor before the Nushell command tree loads.
2. When a user runs `bin/yzx run cargo --verbose check`, the child argv remains `["--verbose", "check"]` and the command does not depend on wrapper-side Nushell parsing.
3. `nushell/scripts/core/yazelix.nu` no longer publicly exports `yzx env` / `yzx run`, and `nushell/scripts/yzx/env.nu` / `run.nu` are no longer public owners.
4. Nushell extern compatibility for `env` / `run`, if kept, exists as a small explicit compatibility surface rather than by reintroducing public Nushell parser ownership.
5. By the time `yazelix-ulb2.1` closes, shipped Nushell has gone down by at least 100 net LOC and the old env/run owner seam has not simply moved sideways into new bridge code.

## Verification

- unit tests: focused Rust tests for shell-selection behavior and run argv parsing
- integration tests:
  - `nu -c 'source nushell/scripts/dev/test_yzx_core_commands.nu; let results = [(test_yzx_run_passes_dash_prefixed_args_through_unchanged) (test_yzx_run_treats_child_verbose_flag_as_child_argv)]; if ($results | all {|result| $result}) { print "ok" } else { error make {msg: "yzx run passthrough tests failed"} }'`
  - `nu nushell/scripts/dev/validate_flake_install.nu all`
- manual verification:
  - `yzx env --no-shell`
  - `yzx run nu -c 'print $env.YAZELIX_RUNTIME_DIR'`
- spec validation:
  - `nu nushell/scripts/dev/validate_specs.nu`

## Traceability

- Bead: `yazelix-ulb2.1.1`
- Defended by: `nu nushell/scripts/dev/test_yzx_core_commands.nu`
- Defended by: `nu nushell/scripts/dev/validate_flake_install.nu all`
- Defended by: `nu nushell/scripts/dev/validate_specs.nu`

## Open Questions

- What exact internal binary name should the implementation use: `yzx_control` or another explicit control-plane name?
- Should the temporary Nushell extern compatibility for `env` / `run` be handwritten in `nushell_externs.nu`, or should it come from a tiny checked-in metadata table?
- Once `env` / `run` are cut over, does `environment_bootstrap.nu` still justify surviving as-is, or should the remaining caller switch to a smaller parse-only helper?
