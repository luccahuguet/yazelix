# Helix Managed Config Contract

## Summary

Yazelix makes managed Helix sessions self-contained through a Yazelix-owned Helix config tree under `~/.config/yazelix/helix/`.

The goal is to stop depending on ad hoc edits to the user's personal Helix config while still giving managed Yazelix Helix sessions a curated editor default.

## Why

The Helix reveal path follows the same ownership model as the other managed Yazelix config surfaces:

- Yazi and Zellij already have Yazelix-managed user override surfaces under `~/.config/yazelix/`
- Helix-specific reveal behavior is owned by Yazelix's managed Helix config tree
- `yzx doctor` can verify the Yazelix-owned Helix integration surface instead of the user's unrelated native Helix config

This keeps a core Yazelix integration feature independent from unmanaged personal editor config.

## Delete-First Decisions

To keep the first Helix contract small and honest:

1. Do not parse arbitrary user Helix config looking for custom reveal bindings.
2. Do not automatically edit `~/.config/helix/config.toml`.
3. Do not move, delete, or adopt files from `~/.config/helix/`.
4. Do not keep the old flat `~/.config/yazelix/helix.toml` surface as a second live input.
5. Do not try to apply the same ownership model to Neovim.

## Managed Surface

Managed input surface:

- `~/.config/yazelix/helix/config.toml`
- `~/.config/yazelix/helix/languages.toml`
- `~/.config/yazelix/helix/themes/*.toml`
- `~/.config/yazelix/helix/ignore`
- `~/.config/yazelix/helix/steel_plugins/`

Generated runtime surface:

- `~/.local/share/yazelix/configs/helix/config.toml`
- `~/.local/share/yazelix/configs/helix/helix.scm`
- `~/.local/share/yazelix/configs/helix/init.scm`
- `~/.local/share/yazelix/configs/helix/**` for copied Steel plugin support files

The managed input surface is where the user can add Yazelix-managed Helix settings, keybindings, language config, themes, and custom Steel plugin source files.

The generated runtime surface is the effective Helix config and Steel entrypoint tree used only by Yazelix-managed Helix sessions.

## Scope Boundary

### What Yazelix owns

Yazelix owns the managed Helix config tree needed for Yazelix-specific editor integration, such as:

- the `yzx reveal` binding
- Yazelix-managed Steel plugin loading
- selection and generated materialization of the child-packaged bundled Steel plugin repository
- managed Helix language/theme lookup under `~/.config/yazelix/helix/`
- curated managed-session defaults for Helix visuals, diagnostics, statusline, and editor-local helper keybindings

### What Yazelix does not own

Yazelix does not own:

- `~/.config/helix/config.toml`
- `~/.config/helix/languages.toml`
- arbitrary user keybinding or theme preferences outside the Yazelix-managed tree
- the user's broader Neovim config tree

## Launch Contract

The managed Helix config must apply only to Yazelix-managed Helix sessions.

That means:

- Yazelix-managed Helix launches should go through a Helix-specific launch path that passes `--config-dir ~/.config/yazelix/helix` for core Helix lookup
- Yazelix-managed Helix launches should still pass the generated effective `config.toml` with `-c`
- Yazelix-managed Helix launches should point Steel at the generated Steel entrypoint tree
- plain `hx` launched outside Yazelix should continue to use the user's normal Helix config resolution

The redirection is scoped to the Helix process, not leaked globally into the whole Yazelix environment.

## Bundled Helix Fork Boundary

Yazelix's bundled Helix is `luccahuguet/yazelix-helix`, currently a thin Yazelix-compatible Helix Steel fork.

The fork must remain useful as a standalone editor project. It should not exist only as an implementation detail for this repo.

Thinness is the current implementation shape, not a long-term constraint. The fork may grow when reusable editor behavior or defaults are useful outside Yazelix-managed sessions.

The fork currently tracks Helix Steel and carries Yazelix-compatible config-directory launch support:

- `hx --config-dir <path>`
- loader resolution from that directory for core Helix config files

The fork may own reusable editor behavior, standalone defaults, Steel runtime behavior, and reusable Steel plugin assets when those are useful without the main Yazelix repo. The main repo should not block that standalone product value.

The main Yazelix repo still owns Yazelix-specific workspace policy: settings fields, Home Manager options, generated-state placement, doctor semantics, session integration, and any command visibility or startup policy that only makes sense inside Yazelix.

## Bundled Steel Plugin Pack Boundary

The reusable default Steel plugin repository is owned by `yazelix-helix`.

The child package exposes a Nix passthru contract named
`yazelixHelixPackageContract`:

```nix
{
  schemaVersion = 1;
  packageName = "yazelix-helix";
  steelPluginRoot = "share/yazelix_helix/steel_plugins";
  pluginIds = [ "recentf" "splash" "spacemacs_theme" "keymaps" "labelled_buffers" ];
}
```

Main Yazelix validates that contract during runtime-tree construction and links
the child-owned root into the runtime as `configs/helix/steel_plugins`. The main
repo must not keep a mirrored copy of those reusable plugin files.

The main repo remains the owner for Yazelix-specific policy:

- the child-packaged plugin pack is selected by `config.toml` through `helix.steel_plugins.enabled`
- custom user plugin manifests live beside the same surface in `helix.steel_plugins.extra`
- Yazelix owns command visibility, startup conditions, generated `helix.scm`, generated `init.scm`, and copied plugin placement under generated state

A good extraction deletes main-repo asset ownership and consumes a child-owned artifact with a narrow contract. A bad extraction makes `yazelix-helix` publish main-repo settings semantics, or makes both repos mirror startup policy and generated command metadata.

## Important Constraint

Vanilla Helix and the upstream Steel branch support `-c/--config <file>` for `config.toml`, but they do not offer the full config-directory override surface Yazelix needs for a self-contained managed Helix session.

Because of that, Yazelix's bundled Helix uses the fork boundary above:

- support a Yazelix-managed Helix config directory for Yazelix-managed sessions
- keep personal `~/.config/helix` untouched unless the user explicitly imports it
- keep Yazelix-specific policy in the main repo even when reusable editor assets move to `yazelix-helix`

Yazelix uses `--config-dir ~/.config/yazelix/helix` for core Helix config lookup and `HELIX_STEEL_CONFIG=<generated-state>/configs/helix` for generated Steel entrypoints.

## Import Story

If a user wants to reuse settings from personal Helix config, the adoption path should be explicit:

- `yzx import helix`

In phase 1, that command should only copy:

- `~/.config/helix/config.toml` -> `~/.config/yazelix/helix/config.toml`

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
- the managed Helix config directory is `~/.config/yazelix/helix`
- the generated Steel command surface does not leak internal commands

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
3. The managed Helix source config is `~/.config/yazelix/helix/config.toml`.
4. Core Helix lookup for languages, themes, ignore, and related config-dir surfaces resolves under `~/.config/yazelix/helix`.
5. Neovim is not forced into the same ownership model.

## Verification

- manual review against [editor_configuration.md](../editor_configuration.md)
- manual review against [helix_keybindings.md](../helix_keybindings.md)
- behavior checks for managed Helix launch and reveal
- CI/contract check: `yzx_repo_validator validate-contracts`

## Traceability
- Defended by: `yzx_repo_validator validate-contracts`
