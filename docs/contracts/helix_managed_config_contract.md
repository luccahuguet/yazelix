# Helix Managed Config Contract

## Summary

Yazelix should make its Helix reveal integration self-contained by introducing a Yazelix-managed Helix `config.toml` surface for Yazelix-managed Helix sessions.

Phase 1 is intentionally narrow:

- manage Helix `config.toml`
- do not claim full ownership of the user's `~/.config/helix/`
- do not claim `languages.toml` ownership yet

The goal is to stop depending on ad hoc edits to the user's personal Helix config just to make `yzx reveal` work inside Yazelix.

This contract describes the target contract. The current user-facing docs may still describe the existing manual Helix setup until implementation lands.

## Why

Today the Helix reveal path is inconsistent with the rest of Yazelix's config ownership model:

- Yazi and Zellij already have Yazelix-managed user override surfaces under `~/.config/yazelix/user_configs/`
- Helix-specific reveal behavior is still documented as a manual edit to `~/.config/helix/config.toml`
- `yzx doctor` cannot verify a clean contract because there is no Yazelix-owned Helix integration surface to check

This leaves a core Yazelix integration feature depending on unmanaged personal editor config.

## Delete-First Decisions

To keep the first Helix contract small and honest:

1. Do not parse arbitrary user Helix config looking for custom reveal bindings.
2. Do not automatically edit `~/.config/helix/config.toml`.
3. Do not move, delete, or adopt files from `~/.config/helix/`.
4. Do not promise `languages.toml` ownership in phase 1.
5. Do not try to apply the same ownership model to Neovim.

## Managed Surface

Phase 1 managed input surface:

- `~/.config/yazelix/user_configs/helix/config.toml`

Phase 1 generated runtime surface:

- `~/.local/share/yazelix/configs/helix/config.toml`

The managed input surface is where the user can add Yazelix-managed Helix settings and keybindings.

The generated runtime surface is the effective Helix config used only by Yazelix-managed Helix sessions.

## Scope Boundary

### What Yazelix owns

For phase 1, Yazelix owns only the Helix config needed for Yazelix-specific editor integration, such as:

- the `yzx reveal` binding
- future Yazelix-specific Helix-local integration that clearly belongs to Yazelix

### What Yazelix does not own

Yazelix does not own:

- `~/.config/helix/config.toml`
- `~/.config/helix/languages.toml`
- arbitrary user keybinding or theme preferences outside the Yazelix-managed surface
- the user's broader Neovim config tree

## Launch Contract

The managed Helix config must apply only to Yazelix-managed Helix sessions.

That means:

- Yazelix-managed Helix launches should go through a Helix-specific launch path that points Helix at the generated Yazelix-managed `config.toml`
- plain `hx` launched outside Yazelix should continue to use the user's normal Helix config resolution

The redirection should be scoped to Helix itself, not leaked globally into the whole Yazelix environment.

## Important Constraint

Helix supports `-c/--config <file>` for `config.toml`, but it does not offer an equivalent `languages.toml` flag or a full config-dir override surface in the same way.

Because of that, phase 1 must stay narrow:

- support a Yazelix-managed `config.toml`
- do not claim full Helix config-directory ownership yet

If Yazelix later wants a managed `languages.toml` story, that needs a separate design decision and likely a stronger Helix launch wrapper boundary.

## Import Story

If a user wants to reuse settings from personal Helix config, the adoption path should be explicit:

- `yzx import helix`

In phase 1, that command should only copy:

- `~/.config/helix/config.toml` -> `~/.config/yazelix/user_configs/helix/config.toml`

It must not:

- move files
- delete files
- silently rewrite personal Helix config

## Doctor Story

Once the managed Helix surface exists, `yzx doctor` should verify the Yazelix-managed Helix integration contract, not arbitrary personal Helix config.

Examples of valid doctor checks:

- managed Helix is the configured editor surface for the current Yazelix session
- the generated Helix `config.toml` exists when managed Helix is active
- the generated config includes the Yazelix-owned reveal binding

Examples of invalid doctor scope:

- linting the user's unrelated `~/.config/helix/config.toml`
- trying to infer every possible custom reveal binding

## Relationship To Neovim

This contract is intentionally asymmetric.

Helix can reasonably gain a Yazelix-managed config surface because the required integration is small and Yazelix already treats Helix as the default editor.

Neovim should remain user-owned:

- user config stays in the user's Neovim config tree
- Yazelix may provide optional snippets or helper guidance
- Yazelix should not become a Neovim distro

## Acceptance Cases

1. Yazelix-managed Helix sessions no longer depend on manual edits to `~/.config/helix/config.toml` for `yzx reveal`.
2. Personal Helix config remains untouched unless the user runs an explicit import command.
3. The managed Helix surface is limited to `config.toml` in phase 1.
4. The design does not pretend to manage `languages.toml` before the launch/runtime boundary exists to support that honestly.
5. Neovim is not forced into the same ownership model.

## Verification

- manual review against [editor_configuration.md](../editor_configuration.md)
- manual review against [helix_keybindings.md](../helix_keybindings.md)
- future behavior checks for managed Helix launch and reveal
- CI/contract check: `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
