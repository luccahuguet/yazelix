# Initializer Scripts

Yazelix generates Nushell initializer scripts under `~/.local/share/yazelix/initializers/nushell/` during environment setup:
- `yazelix_init.nu`: Aggregate initializer sourced by the managed Yazelix Nushell config
- `starship_init.nu`: Runs `starship init nu`
- `zoxide_init.nu`: Runs `zoxide init nushell --cmd z`
- `mise_init.nu`: Runs `mise activate nu`
- `carapace_init.nu`: Runs `carapace _carapace nushell`

The shipped managed Yazelix Nushell config sources `yazelix_init.nu` when a shell is running inside Yazelix. These files are regenerated whenever Yazelix refreshes its managed runtime state. Do not edit them manually; use `~/.config/nushell/config.nu` or tool-specific configs such as `~/.config/starship.toml` for host-owned customization.
