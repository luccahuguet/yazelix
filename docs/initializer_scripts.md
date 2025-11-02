# Initializer Scripts

Yazelix generates Nushell initializer scripts in `~/.config/yazelix/nushell/initializers/` during the Nix environment setup (`nix develop --impure`):
- `starship_init.nu`: Runs `starship init nu`.
- `zoxide_init.nu`: Runs `zoxide init nushell --cmd z`.
- `mise_init.nu`: Runs `mise activate nu` (only if `recommended_deps = true` in `yazelix.toml`).
- `carapace_init.nu`: Runs `carapace _carapace nushell` (only if `recommended_deps = true` in `yazelix.toml`).
These are sourced in `~/.config/yazelix/nushell/config/config.nu` and **regenerated each time you open WezTerm** to reflect the current tool versions. Do not edit these files manually, as they will be overwritten. For custom configurations, use `~/.config/nushell/config.nu` or tool-specific configs (e.g., `~/.config/starship.toml`).
