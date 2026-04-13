# Initializer Scripts

Yazelix generates Nushell initializer scripts in `~/.config/yazelix/nushell/initializers/` during environment setup:
- `starship_init.nu`: Runs `starship init nu`.
- `zoxide_init.nu`: Runs `zoxide init nushell --cmd z`.
- `mise_init.nu`: Runs `mise activate nu`.
- `carapace_init.nu`: Runs `carapace _carapace nushell`.

These are sourced in `~/.config/yazelix/nushell/config/config.nu` and regenerated whenever Yazelix refreshes its managed shell-hook/config surface. Do not edit these files manually, as they are generated artifacts. For custom configurations, use `~/.config/nushell/config.nu` or tool-specific configs such as `~/.config/starship.toml`.
