---
name: yazelix
description: All-in-one operating guide for the yazelix runtime — rebuild, terminal identity, config editing + the ratconfig migration contract, the freshness hash/sidecar, the verification ladder, and the common gotchas. Trigger when rebuilding, changing config defaults, touching terminal identity, or diagnosing config drift in the yazelix repo.
---

# /yazelix

The single entry point for working on the yazelix runtime. Consolidates the
knowledge that is otherwise scattered across `CLAUDE.md`, `docs/contracts/*`, and
the Rust/Nix sources. Read the section you need; each is self-contained.

## 0. Canonical source vs. siblings (get this right first)

- **`src/yazelix`** — canonical. `nix profile list` shows `lifeos_foundation_yzx`
  locked to `path:.../src/yazelix`. Edits here ship after a rebuild.
- **`src/yazelix-yazi-assets`** — sibling child repo (flake input): the 5 vendored
  Yazi plugins + config-pack live here, not in `configs/yazi/`.
- **`src/yazelix-terminal-support`** — sibling child repo (flake input): the single
  source of truth for terminal identity (see §2).
- **`src/yazelix-helix`** — sibling child repo (flake input): the Helix fork +
  Steel plugin cogs/manifest.
- `src/yazelix_new_worktree` and `src/yazelix-helix` are NOT the same as the above;
  don't edit a stale worktree expecting it to ship.

## 1. Rebuild the installed runtime — ONE way only

The profile is backed by a local `path:` flake, so rebuild through the frontdoor:

```bash
/home/flexnetos/.nix-profile/bin/yzx update upstream   # fetch + ff a clean checkout, then nix profile upgrade
/home/flexnetos/.nix-profile/bin/yzx doctor --fix
/home/flexnetos/.nix-profile/bin/yzx doctor
```

Verify:
```bash
readlink -f ~/.nix-profile/bin/yzx        # store hash should change after a real upgrade
cat ~/.nix-profile/runtime_variant        # should be "kitty"
/home/flexnetos/.nix-profile/bin/yzx run codedb --version
```

**Do NOT**: rebuild with host-local `FLEXNETOS_*_PATH` inputs or `packaging/*_local_binary.nix`
shims; copy raw `nix profile upgrade` commands with hardcoded store hashes (they go
stale every upgrade and reintroduce profile drift). Runtime tools must come from
published flake inputs or source-owned package defs.

**`yzx doctor` warnings right after a rebuild are usually session carryover**, not
drift: a shell spawned before the mid-session profile swap still has the old store
hash on PATH. Re-launch the desktop entry (or a fresh `env -i HOME=$HOME PATH=$PATH
bash -lc 'type yzx'`) before chasing a startup-file shadow.

## 2. Terminal identity — single source of truth

Kitty is the packaged default terminal; Ghostty is the host backup; Mars was
removed from the launch chain (2026-07-11) but retained as a dormant config
materializer + for `yzx enter` session detection.

**All terminal identity comes from the `yazelix-terminal-support` child crate.** To
change any terminal fact (default terminal, labels, desktop suffix, session
markers, which terminals are supported/packaged), edit
`src/yazelix-terminal-support/config_metadata/terminal_support.toml` — NOT the
Rust or Nix consumers. Then:
1. Push the child repo and pin the new rev in yazelix's `Cargo.toml` (git dep) +
   `flake.nix` input + `packaging/rust_core_helper.nix` (cargoLock outputHash).
2. Rust reads it via `terminal_variant::*` → `yazelix_terminal_support::terminal_support()`.
3. Nix reads the same TOML via `builtins.fromTOML` into `home_manager/module.nix`.
4. The maintainer config-surface validator enforces module-default == TOML.

Never hardcode a terminal name in Rust/Nix — use `default_terminal()`,
`terminal_display_name()`, `supported_terminals()`, `is_supported()`.

## 3. Editing config defaults — the ratconfig contract

`settings_default.jsonc` is the shipped default. Its rebuild-affecting fields are
mirrored in `config_metadata/main_config_contract.toml`. **Changing a user-facing
default is not a one-line edit** — the config-surface validator will reject it
until you also:

1. Edit `settings_default.jsonc` AND the matching `default` in
   `main_config_contract.toml` (keep them in parity).
2. Bump `SETTINGS_CONTRACT_CURRENT_VERSION` in
   `rust_core/yazelix_core/src/settings_contract.rs` and add a `ContractChange`
   with a migration transform that upgrades ONLY configs still matching the old
   default (preserve customizations). Add the change id to
   `SETTINGS_CONTRACT_APPLIED_CHANGE_IDS`.
3. Mirror the version + change id in `main_config_contract.toml`
   (`ratconfig_contract_version`, `ratconfig_applied_change_ids`).
4. Update any test that hardcodes the old default (config_ui picker/list tests,
   helix materialization tests) and the golden config hash (§4).

See commit history around `enable-max-feature-defaults` for a worked example
(widget_tray + steel_plugins).

## 4. The freshness hash + the golden

`config_state.rs::compute_config_state` decides when to re-materialize. Inputs:
- `config_hash` = hash of rebuild-required settings.jsonc fields **plus the
  `~/.config/yazelix/zellij.kdl` override sidecar** (folded in only when present).
- `runtime_hash` = hash of `runtime_identity.json` (nix store identity).

Any change to `settings_default.jsonc` changes the **golden config hash** in the
`computes_default_rebuild_hash_without_recorded_state` test. Recompute it: add a
temporary `eprintln!("{}", state.config_hash)`, run the test with `-- --nocapture`,
paste the value into the const, remove the eprintln.

## 5. The zellij.kdl sidecar

`~/.config/yazelix/zellij.kdl` is merged into the generated `config.kdl` at
materialization and now participates in the freshness hash — editing it alone
triggers re-materialization on the next `yzx doctor`. Use it only for native
zellij keys yazelix doesn't already render (from `settings.jsonc`) or enforce
(`enforced_top_level_settings`). E.g. `scrollback_lines_to_serialize` is fine;
`session_serialization` is already enforced.

## 6. Verification ladder (run before claiming done)

```bash
cd rust_core
cargo test -p yazelix_core -p yazelix_maintainer --no-fail-fast   # --no-fail-fast so a flake doesn't mask others
cargo run -q -p yazelix_maintainer --bin yzx_repo_validator -- validate-config-surface-contract
cargo run -q -p yazelix_maintainer --bin yzx_repo_validator -- validate-docs-experience   # after doc edits
cargo fmt --check
```

Other validators (run the relevant one after touching that surface):
`validate-contracts`, `validate-readme-version`, `validate-upgrade-contract`,
`validate-nushell-syntax`, `validate-child-release-transaction`,
`validate-flake-interface`, `validate-installed-runtime-contract`.

Known flake: `agent_commands` tests can hit `ExecutableFileBusy` (ETXTBSY) under
parallel runs (write-then-exec of a fake git); re-run isolated to confirm.

## 7. Landing changes (FlexNetOS git topology)

- Feature branch off freshly-fetched `main` (this repo's PRs target **`main`**, and
  `develop` currently trails main). Short-lived branches may be rebased; never
  rebase/force-push `main`.
- Non-trivial runtime/launch changes wait for the user's **manual kitty-launch
  test** before a `main` push. Trivial changes / explicit "push it" can go straight.
- Merged ⇒ reap the branch. Verify with `gh pr view` → MERGED (or the
  curl+`--resolve` API workaround if the local network hijacks github.com DNS).

## 8. Key file map

| Concern | File |
|---|---|
| Shipped defaults | `settings_default.jsonc` |
| Config contract + ratconfig mirror | `config_metadata/main_config_contract.toml` |
| Migration transforms | `rust_core/yazelix_core/src/settings_contract.rs` |
| Freshness/golden | `rust_core/yazelix_core/src/config_state.rs` |
| Terminal identity (consumers) | `rust_core/yazelix_core/src/terminal_variant.rs` |
| Terminal identity (authority) | `src/yazelix-terminal-support/config_metadata/terminal_support.toml` |
| HM module | `home_manager/module.nix` |
| Runtime tool inventory | `packaging/runtime_tool_registry.nix`, `packaging/flake_outputs.nix` |
| Yazi plugin split | `configs/yazi/README.md` (+ `src/yazelix-yazi-assets`) |
| Repo validators | `rust_core/yazelix_maintainer/src/bin/yzx_repo_validator.rs` |
