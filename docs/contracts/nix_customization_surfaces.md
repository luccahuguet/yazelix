# Nix customization surfaces

## Summary

Yazelix exposes a complete default package, a narrow Home Manager module, and an advanced Classic package builder

Home Manager installs one complete package and may own sparse files under the canonical Yazelix config root

## Default package

`packages.${system}.yazelix` is the curated complete package and the default Home Manager package

The package owns the Yazelix runtime dependency graph, including Mars, bootstrap tools, generated runtime assets, and tool-source decisions

Host terminal emulators remain outside the package and start Yazelix with `yzx enter`

## Home Manager API

The supported module surface is exactly

- `programs.yazelix.enable`
- `programs.yazelix.package`
- `programs.yazelix.config.settings`
- approved native files under `programs.yazelix.config`

`package` accepts one complete package and installs it unchanged

The module does not expose terminal selection, package component toggles, runtime tool sources, agent helper packages, or per-field semantic options

### Sparse semantic ownership

`config.settings` is nullable TOML data

- `null` creates no `yazelix/config.toml`
- `{}` creates an explicitly owned empty file
- a nonempty value renders exactly the declared TOML tree
- omitted fields inherit `config_default.toml`
- removing a declared field returns it to package inheritance on the next switch

Home Manager must not merge, copy, or freeze packaged defaults into the user file

### Native file ownership

Each approved native file accepts exactly one of `text` or `source`

Classic consumes

- `yazelix/cursors.toml`
- `yazelix/mars/config.toml`
- `yazelix/zellij/config.kdl`
- `yazelix/helix/config.toml`
- `yazelix/helix/languages.toml`
- `yazelix/yazi/yazi.toml`
- `yazelix/yazi/init.lua`
- compatible `yazelix/yazi/keymap.toml`

The final Classic bridge also permits these Nova v1 files to be staged without claiming that Classic consumes them

- `yazelix/nu/env.nu`
- `yazelix/nu/config.nu`
- `yazelix/starship.toml`
- `yazelix/helix/helix.scm`
- `yazelix/helix/init.scm`
- `yazelix/yazi/package.toml`
- `yazelix/yazi/theme.toml`

The module never owns `zellij/plugins.kdl`, plugin or flavor directories, ambient application config, or generated runtime state

Store-backed files are read-only and all remediation must direct users to their Home Manager declaration

## Platform behavior

Linux installations receive the Yazelix icons and Mars desktop entry

Darwin installations receive the package and declared config files without evaluating Linux desktop-entry options

Linux-only package behavior must remain platform-gated rather than failing during shared module evaluation

## Advanced Classic builder

`lib.${system}.mkYazelix` remains a separate Classic package-construction surface while it exists

Its arguments are not Home Manager options and the module must not translate declarations into builder arguments

Users may construct or obtain any complete compatible package and pass it through `programs.yazelix.package`

## Ownership and collisions

Omitted files remain user-owned

Declared files are Home Manager-owned profile links and normal Home Manager collision checks apply

The module must never move, merge, delete, or silently adopt an existing mutable file

## Verification

- `yzx_repo_validator validate-config-surface-contract`
- `yzx_repo_validator validate-nix-customization-api`
- Linux Home Manager activation with absent and explicit `config.settings`
- Darwin module evaluation without desktop-entry options
- representative Home Manager activation package build
